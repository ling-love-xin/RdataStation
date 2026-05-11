//! SQLite 数据库驱动实现
//!
//! 使用 `rusqlite`（官方 Rust 绑定）实现 `Database` trait。
//! SQLite 是嵌入式文件数据库，连接以 `Mutex<Connection>` 管理。
//!
//! ## 关键约束
//! - SQLite 不支持 schema 层级 — list_schemas 返回空 vec
//! - 查询结果通过 `RecordBatch` (Arrow) 返回，实现零拷贝传输
//! - 写操作使用 `query_row` 标记为只读检查（`is_read_only_sql`）

use std::sync::Arc;
use std::sync::Mutex;

use arrow::array::{ArrayRef, BinaryArray, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use rusqlite::Connection;

use crate::core::driver::traits::MetadataBrowser;
use crate::core::driver::utils::quote_identifier;
use crate::core::driver::{ColumnDetail, DataSourceMeta, Database, Transaction};
use crate::core::error::{CoreError, DatabaseError};
use crate::core::models::{ArrowBatch, QueryResult, Value};

/// SQLite 数据库连接
///
/// 封装 `rusqlite::Connection`，以 `Arc<Mutex<Connection>>` 管理线程安全访问。
/// SQLite 是嵌入式文件数据库，不支持 schema 层级和网络连接。
///
/// # 字段
/// * `conn` - 由 Arc + Mutex 保护的 rusqlite 连接
/// * `server_version` - SQLite 版本号
pub struct SqliteDatabase {
    conn: Arc<Mutex<Connection>>,
    server_version: Option<String>,
}

impl SqliteDatabase {
    pub fn new(url: &str) -> Result<Self, CoreError> {
        let path = if url.starts_with("sqlite://") {
            url.trim_start_matches("sqlite://")
        } else {
            url
        };

        // 确保父目录存在
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    CoreError::database(DatabaseError::Driver {
                        db_type: "sqlite".to_string(),
                        operation: "create_directory".to_string(),
                        source: e.to_string(),
                    })
                })?;
            }
        }

        let conn = Connection::open(path).map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "sqlite".to_string(),
                operation: "connect".to_string(),
                source: e.to_string(),
            })
        })?;
        let server_version = conn
            .query_row("SELECT sqlite_version()", [], |row| row.get::<_, String>(0))
            .ok();
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            server_version,
        })
    }

    pub fn from_connection(conn: Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            server_version: None,
        }
    }
}

fn is_read_only_sql(sql: &str) -> bool {
    let sql_upper = sql.trim_start().to_uppercase();
    sql_upper.starts_with("SELECT")
        || sql_upper.starts_with("PRAGMA")
        || sql_upper.starts_with("EXPLAIN")
}

#[async_trait::async_trait]
/// Database trait 实现：SQLite
///
/// 核心查询方法使用 `rusqlite::Connection::prepare()` 逐行读取，
/// 然后转换为 Arrow `RecordBatch` 返回。
impl Database for SqliteDatabase {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        self.query_with_params(sql, vec![]).await
    }

    async fn query_with_params(
        &self,
        sql: &str,
        params: Vec<Value>,
    ) -> Result<QueryResult, CoreError> {
        let conn = self.conn.lock().map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "sqlite".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            })
        })?;

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let columns: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|name| name.to_string())
            .collect();

        let params_slice: Vec<rusqlite::types::Value> = params
            .iter()
            .map(|v| match v {
                Value::Null => rusqlite::types::Value::Null,
                Value::Bool(b) => rusqlite::types::Value::Integer(*b as i64),
                Value::Int(i) => rusqlite::types::Value::Integer(*i),
                Value::Float(f) => rusqlite::types::Value::Real(*f),
                Value::Text(s) => rusqlite::types::Value::Text(s.clone()),
                Value::Bytes(b) => rusqlite::types::Value::Blob(b.clone()),
            })
            .collect();

        let mut rows = match params_slice.len() {
            0 => stmt.query([]),
            1 => stmt.query([&params_slice[0]]),
            2 => stmt.query([&params_slice[0], &params_slice[1]]),
            3 => stmt.query([&params_slice[0], &params_slice[1], &params_slice[2]]),
            4 => stmt.query([
                &params_slice[0],
                &params_slice[1],
                &params_slice[2],
                &params_slice[3],
            ]),
            5 => stmt.query([
                &params_slice[0],
                &params_slice[1],
                &params_slice[2],
                &params_slice[3],
                &params_slice[4],
            ]),
            _ => {
                let params_refs: Vec<&dyn rusqlite::ToSql> = params_slice
                    .iter()
                    .take(16)
                    .map(|v| v as &dyn rusqlite::ToSql)
                    .collect();
                stmt.query(rusqlite::params_from_iter(params_refs))
            }
        }
        .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let mut row_data: Vec<Vec<rusqlite::types::Value>> = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let values: Vec<rusqlite::types::Value> = columns
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    row.get::<usize, rusqlite::types::Value>(i)
                        .unwrap_or(rusqlite::types::Value::Null)
                })
                .collect();
            row_data.push(values);
        }

        let is_read_only = is_read_only_sql(sql);
        let row_count = row_data.len();

        let batch = if row_count > 0 {
            sqlite_rows_to_arrow(&columns, &row_data)?
        } else {
            return Ok(QueryResult {
                columns,
                batches: vec![],
                affected_rows: if is_read_only { None } else { Some(0) },
                is_read_only: Some(is_read_only),
            });
        };

        Ok(QueryResult {
            columns,
            batches: vec![batch],
            affected_rows: if is_read_only { None } else { Some(row_count) },
            is_read_only: Some(is_read_only),
        })
    }

    async fn query_with_cancel(
        &self,
        sql: &str,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<QueryResult, CoreError> {
        let conn = Arc::clone(&self.conn);
        let sql_owned = sql.to_string();
        let sql_for_error = sql.to_string();

        tokio::select! {
            result = tokio::task::spawn_blocking(move || {
                let conn = conn.lock().map_err(|e| CoreError::database(DatabaseError::Driver {
                    db_type: "sqlite".to_string(),
                    operation: "lock".to_string(),
                    source: e.to_string(),
                }))?;

                let mut stmt = conn.prepare(&sql_owned)
                    .map_err(|e| CoreError::database(DatabaseError::query(&sql_owned, e.to_string())))?;

                let columns: Vec<String> = stmt.column_names()
                    .iter()
                    .map(|name| name.to_string())
                    .collect();

                let mut rows = stmt.query([])
                    .map_err(|e| CoreError::database(DatabaseError::query(&sql_owned, e.to_string())))?;

                let mut row_data: Vec<Vec<rusqlite::types::Value>> = Vec::new();
                while let Ok(Some(row)) = rows.next() {
                    let values: Vec<rusqlite::types::Value> = columns.iter().enumerate()
                        .map(|(i, _)| {
                            row.get::<usize, rusqlite::types::Value>(i).unwrap_or(rusqlite::types::Value::Null)
                        })
                        .collect();
                    row_data.push(values);
                }

                let is_read_only = is_read_only_sql(&sql_owned);
                let row_count = row_data.len();

                let batch = if row_count > 0 {
                    sqlite_rows_to_arrow(&columns, &row_data)?
                } else {
                    return Ok(QueryResult {
                        columns,
                        batches: vec![],
                        affected_rows: if is_read_only { None } else { Some(0) },
                        is_read_only: Some(is_read_only),
                    });
                };

                Ok(QueryResult {
                    columns,
                    batches: vec![batch],
                    affected_rows: if is_read_only { None } else { Some(row_count) },
                    is_read_only: Some(is_read_only),
                })
            }) => {
                result.map_err(|e| CoreError::database(DatabaseError::query(
                    &sql_for_error,
                    format!("Task panicked: {}", e),
                )))?
            }
            _ = cancel_token.cancelled() => {
                Err(CoreError::database(DatabaseError::Query {
                    sql: sql_for_error,
                    reason: "Query cancelled".to_string(),
                    position: None,
                }))
            }
        }
    }

    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, CoreError> {
        let conn = self.conn.lock().map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "sqlite".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            })
        })?;

        conn.execute("BEGIN TRANSACTION", []).map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "sqlite".to_string(),
                operation: "begin_transaction".to_string(),
                source: e.to_string(),
            })
        })?;

        Ok(Box::new(SqliteTransaction::new(Arc::clone(&self.conn))))
    }

    fn meta(&self) -> DataSourceMeta {
        DataSourceMeta {
            server_version: self.server_version.clone(),
            ..DataSourceMeta::sqlite()
        }
    }

    async fn ping(&self) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "sqlite".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            })
        })?;
        conn.query_row("SELECT 1", [], |_| Ok(()))
            .map_err(|e| CoreError::database(DatabaseError::query("SELECT 1", e.to_string())))?;
        Ok(())
    }

    async fn list_databases(&self) -> Result<Vec<String>, CoreError> {
        Ok(vec!["main".to_string()])
    }

    async fn list_tables(
        &self,
        _db: &str,
        _schema: Option<&str>,
    ) -> Result<Vec<crate::core::driver::SchemaObject>, CoreError> {
        let nodes = self.get_tables("main", "main").await?;
        Ok(nodes
            .into_iter()
            .map(|n| crate::core::driver::SchemaObject {
                name: n.name,
                kind: n.kind,
                children: None,
                comment: n.comment,
            })
            .collect())
    }

    async fn list_columns(
        &self,
        _db: &str,
        _schema: Option<&str>,
        table: &str,
    ) -> Result<Vec<ColumnDetail>, CoreError> {
        let detail = self.get_table_detail("main", "main", table).await?;
        Ok(detail.columns)
    }
}

/// SQLite 事务句柄
///
/// 通过 `Arc<Mutex<Connection>>` 共享连接，支持 begin/commit/rollback。
/// `committed` 标记用于 Drop 时判断是否需要自动回滚。
pub struct SqliteTransaction {
    conn: Arc<Mutex<Connection>>,
    committed: bool,
}

impl SqliteTransaction {
    fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            conn,
            committed: false,
        }
    }
}

#[async_trait::async_trait]
impl Transaction for SqliteTransaction {
    async fn query(&mut self, sql: &str) -> Result<QueryResult, CoreError> {
        let conn = self.conn.lock().map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "sqlite".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            })
        })?;

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let columns: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|name| name.to_string())
            .collect();

        let mut rows = stmt
            .query([])
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let mut row_data: Vec<Vec<rusqlite::types::Value>> = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let mut values: Vec<rusqlite::types::Value> = Vec::with_capacity(columns.len());
            for (i, _) in columns.iter().enumerate() {
                let v = row.get::<usize, rusqlite::types::Value>(i).map_err(|e| {
                    CoreError::database(DatabaseError::Driver {
                        db_type: "sqlite".to_string(),
                        operation: "row_parsing".to_string(),
                        source: e.to_string(),
                    })
                })?;
                values.push(v);
            }
            row_data.push(values);
        }

        let sql_upper = sql.trim_start().to_uppercase();
        let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("PRAGMA");
        let row_count = row_data.len();

        let batch = if row_count > 0 {
            sqlite_rows_to_arrow(&columns, &row_data)?
        } else {
            return Ok(QueryResult {
                columns,
                batches: vec![],
                affected_rows: if is_read_only { None } else { Some(0) },
                is_read_only: Some(is_read_only),
            });
        };

        Ok(QueryResult {
            columns,
            batches: vec![batch],
            affected_rows: if is_read_only { None } else { Some(row_count) },
            is_read_only: Some(is_read_only),
        })
    }

    async fn commit(&mut self) -> Result<(), CoreError> {
        if !self.committed {
            let conn = self.conn.lock().map_err(|e| {
                CoreError::database(DatabaseError::Driver {
                    db_type: "sqlite".to_string(),
                    operation: "lock".to_string(),
                    source: e.to_string(),
                })
            })?;

            conn.execute("COMMIT", []).map_err(|e| {
                CoreError::database(DatabaseError::Driver {
                    db_type: "sqlite".to_string(),
                    operation: "commit".to_string(),
                    source: e.to_string(),
                })
            })?;

            self.committed = true;
        }
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), CoreError> {
        if !self.committed {
            let conn = self.conn.lock().map_err(|e| {
                CoreError::database(DatabaseError::Driver {
                    db_type: "sqlite".to_string(),
                    operation: "lock".to_string(),
                    source: e.to_string(),
                })
            })?;

            if let Err(e) = conn.execute("ROLLBACK", []) {
                tracing::warn!("SQLite transaction rollback error: {}", e);
            }
            self.committed = true;
        }
        Ok(())
    }
}

/// 将 SQLite 行转换为 Arrow 批处理
fn sqlite_rows_to_arrow(
    columns: &[String],
    rows: &[Vec<rusqlite::types::Value>],
) -> Result<ArrowBatch, CoreError> {
    let num_rows = rows.len();
    let num_cols = columns.len();

    let mut arrays: Vec<ArrayRef> = Vec::with_capacity(num_cols);

    for col_idx in 0..num_cols {
        let mut string_values: Vec<Option<String>> = Vec::with_capacity(num_rows);
        let mut int_values: Vec<Option<i64>> = Vec::with_capacity(num_rows);
        let mut float_values: Vec<Option<f64>> = Vec::with_capacity(num_rows);
        let mut bool_values: Vec<Option<bool>> = Vec::with_capacity(num_rows);
        let mut binary_values: Vec<Option<Vec<u8>>> = Vec::with_capacity(num_rows);

        let mut detected_type: Option<DataType> = None;

        for row in rows {
            // 每一行必须向所有 vector 插入，否则各列长度不一致
            if let Some(value) = row.get(col_idx) {
                match value {
                    rusqlite::types::Value::Null => {
                        string_values.push(None);
                        int_values.push(None);
                        float_values.push(None);
                        bool_values.push(None);
                        binary_values.push(None);
                    }
                    rusqlite::types::Value::Integer(i) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Int64);
                        }
                        string_values.push(None);
                        int_values.push(Some(*i));
                        float_values.push(None);
                        bool_values.push(None);
                        binary_values.push(None);
                    }
                    rusqlite::types::Value::Real(f) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Float64);
                        }
                        string_values.push(None);
                        int_values.push(None);
                        float_values.push(Some(*f));
                        bool_values.push(None);
                        binary_values.push(None);
                    }
                    rusqlite::types::Value::Text(s) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Utf8);
                        }
                        string_values.push(Some(s.clone()));
                        int_values.push(None);
                        float_values.push(None);
                        bool_values.push(None);
                        binary_values.push(None);
                    }
                    rusqlite::types::Value::Blob(b) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Binary);
                        }
                        string_values.push(None);
                        int_values.push(None);
                        float_values.push(None);
                        bool_values.push(None);
                        binary_values.push(Some(b.clone()));
                    }
                }
            } else {
                // row.get() 返回 None（列不存在于该行）
                string_values.push(None);
                int_values.push(None);
                float_values.push(None);
                bool_values.push(None);
                binary_values.push(None);
            }
        }

        let array: ArrayRef = match detected_type.unwrap_or(DataType::Utf8) {
            DataType::Boolean => Arc::new(BooleanArray::from(bool_values)),
            DataType::Int64 => Arc::new(Int64Array::from(int_values)),
            DataType::Float64 => Arc::new(Float64Array::from(float_values)),
            DataType::Binary => {
                let refs: Vec<Option<&[u8]>> = binary_values
                    .iter()
                    .map(|opt| opt.as_ref().map(|v| v.as_slice()))
                    .collect();
                Arc::new(BinaryArray::from(refs))
            }
            _ => Arc::new(StringArray::from(string_values)),
        };

        arrays.push(array);
    }

    let fields: Vec<Field> = columns
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let data_type = arrays[i].data_type().clone();
            Field::new(name, data_type, true)
        })
        .collect();

    let schema = Arc::new(Schema::new(fields));

    RecordBatch::try_new(schema, arrays).map_err(|e| {
        CoreError::database(DatabaseError::Driver {
            db_type: "sqlite".to_string(),
            operation: "arrow_conversion".to_string(),
            source: e.to_string(),
        })
    })
}

#[async_trait::async_trait]
impl crate::core::driver::MetadataBrowser for SqliteDatabase {
    async fn get_databases(&self) -> Result<Vec<crate::core::driver::NodeInfo>, CoreError> {
        Ok(vec![crate::core::driver::NodeInfo {
            name: "main".to_string(),
            kind: crate::core::driver::SchemaObjectKind::Database,
            icon: Some("database".to_string()),
            comment: None,
        }])
    }

    async fn get_schemas(
        &self,
        _db: &str,
    ) -> Result<Vec<crate::core::driver::NodeInfo>, CoreError> {
        Ok(vec![crate::core::driver::NodeInfo {
            name: "main".to_string(),
            kind: crate::core::driver::SchemaObjectKind::Schema,
            icon: Some("schema".to_string()),
            comment: None,
        }])
    }

    async fn get_tables(
        &self,
        _db: &str,
        _schema: &str,
    ) -> Result<Vec<crate::core::driver::NodeInfo>, CoreError> {
        let result = self.query("SELECT name, type FROM sqlite_master WHERE type IN ('table', 'view') ORDER BY name").await?;
        let nodes: Vec<crate::core::driver::NodeInfo> = (0..result.total_rows())
            .filter_map(|row_idx| {
                result.batches.iter().find_map(|batch| {
                    if row_idx < batch.num_rows() {
                        if let (Some(name_arr), Some(type_arr)) = (
                            batch.column(0).as_any().downcast_ref::<StringArray>(),
                            batch.column(1).as_any().downcast_ref::<StringArray>(),
                        ) {
                            let table_type = type_arr.value(row_idx);
                            let kind = if table_type == "view" {
                                crate::core::driver::SchemaObjectKind::View
                            } else {
                                crate::core::driver::SchemaObjectKind::Table
                            };
                            Some(crate::core::driver::NodeInfo {
                                name: name_arr.value(row_idx).to_string(),
                                kind,
                                icon: Some(if table_type == "view" {
                                    "view".to_string()
                                } else {
                                    "table".to_string()
                                }),
                                comment: None,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .collect();
        Ok(nodes)
    }

    async fn get_table_detail(
        &self,
        _db: &str,
        _schema: &str,
        table: &str,
    ) -> Result<crate::core::driver::NodeDetail, CoreError> {
        let sql = format!("PRAGMA table_info({})", quote_identifier(table, '"'));
        let result = self.query(&sql).await?;
        let columns: Vec<crate::core::driver::ColumnDetail> = (0..result.total_rows())
            .filter_map(|row_idx| {
                result.batches.iter().find_map(|batch| {
                    if row_idx < batch.num_rows() {
                        let col_name = batch
                            .column(1)
                            .as_any()
                            .downcast_ref::<StringArray>()?
                            .value(row_idx);
                        let data_type = batch
                            .column(2)
                            .as_any()
                            .downcast_ref::<StringArray>()?
                            .value(row_idx);
                        let nullable = batch
                            .column(3)
                            .as_any()
                            .downcast_ref::<StringArray>()?
                            .value(row_idx)
                            == "0";
                        let default = batch
                            .column(4)
                            .as_any()
                            .downcast_ref::<StringArray>()?
                            .value(row_idx);
                        let pk = batch
                            .column(5)
                            .as_any()
                            .downcast_ref::<StringArray>()?
                            .value(row_idx);
                        Some(crate::core::driver::ColumnDetail {
                            name: col_name.to_string(),
                            data_type: data_type.to_string(),
                            nullable,
                            is_primary_key: pk == "1",
                            is_foreign_key: false,
                            default_value: if default.is_empty() {
                                None
                            } else {
                                Some(default.to_string())
                            },
                            comment: None,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(crate::core::driver::NodeDetail {
            node: crate::core::driver::NodeInfo {
                name: table.to_string(),
                kind: crate::core::driver::SchemaObjectKind::Table,
                icon: Some("table".to_string()),
                comment: None,
            },
            columns,
            index_count: None,
            row_count_estimate: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::driver::Database;

    const SQLITE_PATH: &str = r"E:\Documents\new.db";

    #[test]
    fn test_connect() {
        let db = SqliteDatabase::new(SQLITE_PATH);
        assert!(db.is_ok(), "Failed to connect to SQLite: {:?}", db.err());
    }

    #[tokio::test]
    async fn test_query_select_one() {
        let db = SqliteDatabase::new(SQLITE_PATH).expect("Failed to connect");
        let result = db.query("SELECT 1 AS val").await.expect("Query failed");
        assert_eq!(result.columns, vec!["val"]);
    }

    #[tokio::test]
    async fn test_crud_roundtrip() {
        let db = SqliteDatabase::new(SQLITE_PATH).expect("Failed to connect");

        db.query(
            "CREATE TABLE IF NOT EXISTS _rd_test (id INTEGER PRIMARY KEY, name TEXT, value REAL)",
        )
        .await
        .expect("CREATE TABLE failed");

        db.query("INSERT INTO _rd_test (id, name, value) VALUES (1, 'hello', 3.14)")
            .await
            .expect("INSERT failed");

        let result = db
            .query("SELECT id, name, value FROM _rd_test WHERE id = 1")
            .await
            .expect("SELECT failed");
        assert_eq!(result.columns, vec!["id", "name", "value"]);

        db.query("DROP TABLE IF EXISTS _rd_test")
            .await
            .expect("DROP TABLE failed");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let db = SqliteDatabase::new(SQLITE_PATH).expect("Failed to connect");
        let result = db.query("SELECT * FROM _non_existent_table_rd").await;
        assert!(result.is_err(), "Expected error for non-existent table");
    }

    #[tokio::test]
    async fn test_list_tables() {
        let db = SqliteDatabase::new(SQLITE_PATH).expect("Failed to connect");
        let tables = db.list_tables("main", None).await;
        assert!(tables.is_ok(), "list_tables failed: {:?}", tables.err());
    }

    #[tokio::test]
    async fn test_meta() {
        let db = SqliteDatabase::new(SQLITE_PATH).expect("Failed to connect");
        let meta = db.meta();
        assert!(!meta.supports_streaming);
        assert!(!meta.supports_concurrent_write);
    }

    #[tokio::test]
    async fn test_is_read_only_flag() {
        let db = SqliteDatabase::new(SQLITE_PATH).expect("Failed to connect");
        let result = db.query("SELECT 1").await.expect("Query failed");
        assert_eq!(result.is_read_only, Some(true));
    }
}
