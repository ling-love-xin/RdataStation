/**
 * DuckDB 本地加速/联邦查询引擎
 * 
 * 负责：
 * - 外部数据库 ATTACH/DETACH 管理
 * - 联邦查询执行
 * - 结果集管理（会话级/持久化）
 * - 文件数据源加载（CSV/Excel/Parquet）
 */

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::core::dbi::context::QueryContext;
use crate::core::dbi::engine::ExecutionEngine;
use crate::core::error::CoreError;
use crate::core::error::CommonError;
use crate::core::error::DatabaseError;
use crate::core::models::QueryResult;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// 外部数据库连接信息
#[derive(Debug, Clone)]
pub struct ExternalConnection {
    /// 连接名称
    pub name: String,
    /// 驱动类型（mysql/postgres/sqlite）
    pub driver: String,
    /// 连接字符串（加密存储）
    pub connection_string: String,
    /// 是否只读
    pub read_only: bool,
}

impl ExternalConnection {
    /// 创建加密的连接字符串
    pub fn new_encrypted(
        name: &str,
        driver: &str,
        raw_connection_string: &str,
        read_only: bool,
    ) -> Self {
        let encrypted = BASE64.encode(raw_connection_string.as_bytes());
        Self {
            name: name.to_string(),
            driver: driver.to_string(),
            connection_string: encrypted,
            read_only,
        }
    }

    /// 解密连接字符串
    pub fn decrypt_connection_string(&self) -> Result<String, CoreError> {
        let decoded = BASE64.decode(&self.connection_string)
            .map_err(|e| CoreError::common(CommonError::General(
                format!("Failed to decrypt connection string: {}", e),
            )))?;
        
        String::from_utf8(decoded)
            .map_err(|e| CoreError::common(CommonError::General(
                format!("Invalid UTF-8 in decrypted connection string: {}", e),
            )))
    }
}

/// 结果集信息
#[derive(Debug, Clone)]
pub struct ResultSetInfo {
    /// 结果集名称
    pub name: String,
    /// 存储模式
    pub storage: ResultSetStorage,
    /// 创建时间
    pub created_at: std::time::SystemTime,
    /// SQL 语句（用于持久化时保存）
    pub sql: Option<String>,
}

/// 结果集存储模式
#[derive(Debug, Clone, PartialEq)]
pub enum ResultSetStorage {
    /// 会话级（DuckDB 内存表）
    Session,
    /// 持久化（analytics/data.duckdb）
    Persistent,
}

/// DuckDB 执行引擎
pub struct DuckDBEngine {
    /// DuckDB 连接
    duckdb_conn: Arc<Mutex<Option<duckdb::Connection>>>,
    /// 外部数据库注册表
    external_connections: Arc<Mutex<Vec<ExternalConnection>>>,
    /// 结果集注册表
    result_sets: Arc<Mutex<Vec<ResultSetInfo>>>,
    /// DuckDB 数据库路径（持久化模式）
    persistent_db_path: Option<String>,
}

impl DuckDBEngine {
    /// 创建新的 DuckDB 引擎
    pub fn new() -> Self {
        Self {
            duckdb_conn: Arc::new(Mutex::new(None)),
            external_connections: Arc::new(Mutex::new(Vec::new())),
            result_sets: Arc::new(Mutex::new(Vec::new())),
            persistent_db_path: None,
        }
    }

    /// 创建带持久化路径的 DuckDB 引擎
    pub fn with_persistent_path(path: String) -> Self {
        Self {
            duckdb_conn: Arc::new(Mutex::new(None)),
            external_connections: Arc::new(Mutex::new(Vec::new())),
            result_sets: Arc::new(Mutex::new(Vec::new())),
            persistent_db_path: Some(path),
        }
    }

    /// 初始化 DuckDB 连接
    pub async fn initialize(&self) -> Result<(), CoreError> {
        let mut conn_lock = self.duckdb_conn.lock().await;
        
        if conn_lock.is_some() {
            return Ok(());
        }

        let conn = match &self.persistent_db_path {
            Some(path) => duckdb::Connection::open(path)
                .map_err(|e| CoreError::database(DatabaseError::Driver {
                    db_type: "duckdb".to_string(),
                    operation: "open".to_string(),
                    source: e.to_string(),
                }))?,
            None => duckdb::Connection::open(":memory:")
                .map_err(|e| CoreError::database(DatabaseError::Driver {
                    db_type: "duckdb".to_string(),
                    operation: "open".to_string(),
                    source: e.to_string(),
                }))?,
        };

        *conn_lock = Some(conn);
        Ok(())
    }

    /// 获取 DuckDB 连接
    async fn get_connection(&self) -> Result<tokio::sync::MutexGuard<'_, Option<duckdb::Connection>>, CoreError> {
        let conn_lock = self.duckdb_conn.lock().await;
        
        if conn_lock.is_none() {
            drop(conn_lock);
            self.initialize().await?;
            return Ok(self.duckdb_conn.lock().await);
        }

        Ok(conn_lock)
    }

    /// 注册外部数据库
    pub async fn register_external_database(
        &self,
        name: &str,
        driver: &str,
        connection_string: &str,
    ) -> Result<(), CoreError> {
        let mut connections = self.external_connections.lock().await;
        
        if connections.iter().any(|c| c.name == name) {
            return Err(CoreError::common(CommonError::General(
                format!("External database '{}' already registered", name),
            )));
        }

        let external_conn = ExternalConnection::new_encrypted(
            name,
            driver,
            connection_string,
            true,
        );

        let conn = self.get_connection().await?;
        let conn_ref = conn.as_ref().ok_or_else(|| CoreError::common(CommonError::General(
            "DuckDB connection not initialized".to_string(),
        )))?;

        let decrypted = external_conn.decrypt_connection_string()?;
        let sql = format!("ATTACH '{}' AS {} (READ_ONLY)", decrypted, name);
        conn_ref.execute(&sql, [])
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "attach".to_string(),
                source: e.to_string(),
            }))?;

        connections.push(external_conn);
        Ok(())
    }

    /// 注销外部数据库
    pub async fn unregister_external_database(&self, name: &str) -> Result<(), CoreError> {
        let mut connections = self.external_connections.lock().await;
        connections.retain(|c| c.name != name);

        let conn = self.get_connection().await?;
        let conn_ref = conn.as_ref().ok_or_else(|| CoreError::common(CommonError::General(
            "DuckDB connection not initialized".to_string(),
        )))?;

        let sql = format!("DETACH {}", name);
        conn_ref.execute(&sql, [])
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "detach".to_string(),
                source: e.to_string(),
            }))?;

        Ok(())
    }

    /// 注册结果集
    pub async fn register_result_set(
        &self,
        name: &str,
        sql: &str,
        storage: ResultSetStorage,
    ) -> Result<(), CoreError> {
        let mut sets = self.result_sets.lock().await;
        
        if sets.iter().any(|s| s.name == name) {
            return Err(CoreError::common(CommonError::General(
                format!("Result set '{}' already exists", name),
            )));
        }

        let conn = self.get_connection().await?;
        let conn_ref = conn.as_ref().ok_or_else(|| CoreError::common(CommonError::General(
            "DuckDB connection not initialized".to_string(),
        )))?;

        let create_sql = match storage {
            ResultSetStorage::Session => {
                format!("CREATE TEMP TABLE IF NOT EXISTS {} AS {}", name, sql)
            }
            ResultSetStorage::Persistent => {
                format!("CREATE TABLE IF NOT EXISTS {} AS {}", name, sql)
            }
        };

        conn_ref.execute(&create_sql, [])
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "create_result_set".to_string(),
                source: e.to_string(),
            }))?;

        sets.push(ResultSetInfo {
            name: name.to_string(),
            storage,
            created_at: std::time::SystemTime::now(),
            sql: Some(sql.to_string()),
        });

        Ok(())
    }

    /// 注销结果集
    pub async fn unregister_result_set(&self, name: &str) -> Result<(), CoreError> {
        let mut sets = self.result_sets.lock().await;
        sets.retain(|s| s.name != name);

        let conn = self.get_connection().await?;
        let conn_ref = conn.as_ref().ok_or_else(|| CoreError::common(CommonError::General(
            "DuckDB connection not initialized".to_string(),
        )))?;

        let sql = format!("DROP TABLE IF EXISTS {}", name);
        conn_ref.execute(&sql, [])
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "drop_result_set".to_string(),
                source: e.to_string(),
            }))?;

        Ok(())
    }

    /// 获取所有结果集
    pub async fn list_result_sets(&self) -> Vec<ResultSetInfo> {
        let sets = self.result_sets.lock().await;
        sets.clone()
    }

    /// 加载文件数据源
    pub async fn load_file_source(&self, path: &str, table_name: &str) -> Result<(), CoreError> {
        let conn = self.get_connection().await?;
        let conn_ref = conn.as_ref().ok_or_else(|| CoreError::common(CommonError::General(
            "DuckDB connection not initialized".to_string(),
        )))?;

        let sql = if path.ends_with(".csv") {
            format!("CREATE TEMP TABLE IF NOT EXISTS {} AS SELECT * FROM read_csv_auto('{}')", table_name, path)
        } else if path.ends_with(".parquet") {
            format!("CREATE TEMP TABLE IF NOT EXISTS {} AS SELECT * FROM read_parquet('{}')", table_name, path)
        } else if path.ends_with(".xlsx") || path.ends_with(".xls") {
            format!("CREATE TEMP TABLE IF NOT EXISTS {} AS SELECT * FROM read_excel_auto('{}')", table_name, path)
        } else {
            return Err(CoreError::common(CommonError::NotSupported(
                format!("Unsupported file type: {}", path),
            )));
        };

        conn_ref.execute(&sql, [])
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "load_file".to_string(),
                source: e.to_string(),
            }))?;

        Ok(())
    }

    /// 执行 DuckDB 查询
    async fn execute_query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        let conn = self.get_connection().await?;
        let conn_ref = conn.as_ref().ok_or_else(|| CoreError::common(CommonError::General(
            "DuckDB connection not initialized".to_string(),
        )))?;

        let mut stmt = conn_ref.prepare(sql)
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let columns: Vec<String> = stmt.column_names()
            .iter()
            .map(|name| name.to_string())
            .collect();

        let mut rows = stmt.query([])
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let mut row_data: Vec<Vec<duckdb::types::Value>> = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let values: Vec<duckdb::types::Value> = columns.iter().enumerate()
                .map(|(i, _)| {
                    row.get::<usize, duckdb::types::Value>(i).unwrap_or(duckdb::types::Value::Null)
                })
                .collect();
            row_data.push(values);
        }

        let sql_upper = sql.trim_start().to_uppercase();
        let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("SHOW") || sql_upper.starts_with("DESCRIBE");
        let row_count = row_data.len();

        let batch = if row_count > 0 {
            crate::core::driver::native::duckdb::duckdb_rows_to_arrow(&columns, &row_data)?
        } else {
            return Ok(QueryResult::empty());
        };

        Ok(QueryResult {
            columns,
            batches: vec![batch],
            affected_rows: if is_read_only { Some(row_count) } else { None },
            is_read_only: Some(is_read_only),
        })
    }

    /// 执行联邦查询（跨多个数据源）
    pub async fn execute_federated_query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        let connections = self.external_connections.lock().await;
        if connections.is_empty() {
            return Err(CoreError::common(CommonError::General(
                "No external databases registered for federated query".to_string(),
            )));
        }

        let involved_sources: Vec<&str> = connections.iter()
            .filter(|c| sql.contains(&c.name))
            .map(|c| c.name.as_str())
            .collect();

        if involved_sources.is_empty() {
            return Err(CoreError::common(CommonError::General(
                "Query does not reference any registered external databases".to_string(),
            )));
        }

        drop(connections);

        self.execute_query(sql).await
    }

    /// 获取所有已注册的外部数据库
    pub async fn list_external_connections(&self) -> Vec<ExternalConnection> {
        let connections = self.external_connections.lock().await;
        connections.clone()
    }

    /// 安装 DuckDB 扩展
    pub async fn install_extension(&self, extension_name: &str) -> Result<(), CoreError> {
        let conn = self.get_connection().await?;
        let conn_ref = conn.as_ref().ok_or_else(|| CoreError::common(CommonError::General(
            "DuckDB connection not initialized".to_string(),
        )))?;

        let install_sql = format!("INSTALL {}", extension_name);
        conn_ref.execute(&install_sql, [])
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "install_extension".to_string(),
                source: e.to_string(),
            }))?;

        let load_sql = format!("LOAD {}", extension_name);
        conn_ref.execute(&load_sql, [])
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "load_extension".to_string(),
                source: e.to_string(),
            }))?;

        Ok(())
    }

    /// 安装并配置 MySQL 扩展（用于联邦查询）
    pub async fn setup_mysql_extension(&self) -> Result<(), CoreError> {
        self.install_extension("mysql").await
    }

    /// 安装并配置 PostgreSQL 扩展（用于联邦查询）
    pub async fn setup_postgres_extension(&self) -> Result<(), CoreError> {
        self.install_extension("postgres_scanner").await
    }

    /// 安装并配置 SQLite 扩展（用于联邦查询）
    pub async fn setup_sqlite_extension(&self) -> Result<(), CoreError> {
        self.install_extension("sqlite_scanner").await
    }

    /// 安装并配置 Excel 扩展
    pub async fn setup_excel_extension(&self) -> Result<(), CoreError> {
        self.install_extension("excel").await
    }

    /// 安装并配置 Parquet 扩展
    pub async fn setup_parquet_extension(&self) -> Result<(), CoreError> {
        self.install_extension("parquet").await
    }

    /// 安装所有常用扩展
    pub async fn setup_all_extensions(&self) -> Result<(), CoreError> {
        let extensions = vec![
            "mysql",
            "postgres_scanner",
            "sqlite_scanner",
            "excel",
            "parquet",
        ];

        for ext in extensions {
            if let Err(e) = self.install_extension(ext).await {
                eprintln!("Warning: Failed to install extension {}: {}", ext, e);
            }
        }

        Ok(())
    }
}

fn duckdb_value_to_value(row: &duckdb::Row, index: usize) -> crate::core::models::Value {
    if let Ok(value) = row.get::<_, Option<i64>>(index) {
        if let Some(v) = value {
            return crate::core::models::Value::Int(v);
        }
    }
    
    if let Ok(value) = row.get::<_, Option<f64>>(index) {
        if let Some(v) = value {
            return crate::core::models::Value::Float(v);
        }
    }
    
    if let Ok(value) = row.get::<_, Option<String>>(index) {
        if let Some(v) = value {
            return crate::core::models::Value::Text(v);
        }
    }
    
    if let Ok(value) = row.get::<_, Option<Vec<u8>>>(index) {
        if let Some(v) = value {
            return crate::core::models::Value::Bytes(v);
        }
    }
    
    crate::core::models::Value::Null
}

#[async_trait::async_trait]
impl ExecutionEngine for DuckDBEngine {
    async fn execute(&self, sql: &str, _context: &QueryContext) -> Result<QueryResult, CoreError> {
        self.execute_query(sql).await
    }

    fn name(&self) -> &str {
        "duckdb"
    }

    fn supports(&self, sql: &str) -> bool {
        let sql_upper = sql.trim_start().to_uppercase();
        
        // DuckDB 加速模式不支持写操作
        if sql_upper.starts_with("INSERT")
            || sql_upper.starts_with("UPDATE")
            || sql_upper.starts_with("DELETE")
        {
            return false;
        }

        true
    }
}
