# 驱动架构

## 概述

RdataStation 使用 Trait-based 的驱动架构，支持多种数据库类型。驱动层负责数据库连接管理、SQL 执行和元数据查询。

## 核心 Trait

### Database Trait

```rust
#[async_trait]
pub trait Database: Send + Sync {
    /// 执行 SQL 查询（返回结果集）
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError>;
    
    /// 执行 SQL（不返回结果集）
    async fn execute(&self, sql: &str) -> Result<ExecuteResult, CoreError>;
    
    /// 开始事务
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, CoreError>;
    
    /// 获取数据源元数据
    fn meta(&self) -> DataSourceMeta;
    
    /// 列出所有数据库
    async fn list_databases(&self) -> Result<Vec<String>, CoreError>;
    
    /// 列出 Schema
    async fn list_schemas(&self, database: &str) -> Result<Vec<String>, CoreError>;
    
    /// 列出表和视图
    async fn list_tables(
        &self,
        database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<SchemaObject>, CoreError>;
    
    /// 列出列
    async fn list_columns(
        &self,
        database: &str,
        schema: Option<&str>,
        table: &str,
    ) -> Result<Vec<SchemaObject>, CoreError>;
    
    /// 测试连接
    async fn ping(&self) -> Result<Duration, CoreError>;
    
    /// 关闭连接
    async fn close(&self) -> Result<(), CoreError>;
}
```

### Transaction Trait

```rust
#[async_trait]
pub trait Transaction: Send + Sync {
    /// 执行查询
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError>;
    
    /// 执行 SQL
    async fn execute(&self, sql: &str) -> Result<ExecuteResult, CoreError>;
    
    /// 提交事务
    async fn commit(&self) -> Result<(), CoreError>;
    
    /// 回滚事务
    async fn rollback(&self) -> Result<(), CoreError>;
}
```

### DbPool Trait

```rust
#[async_trait]
pub trait DbPool: Send + Sync {
    /// 获取连接
    async fn acquire(&self) -> Result<Box<dyn Database>, CoreError>;
    
    /// 连接池状态
    fn status(&self) -> PoolStatus;
    
    /// 关闭连接池
    async fn close(&self);
}
```

## 驱动注册

### DriverRegistry

```rust
pub struct DriverRegistry {
    factories: RwLock<HashMap<String, Box<dyn DriverFactory>>>,
}

impl DriverRegistry {
    /// 全局单例
    pub fn global() -> &'static Self {
        static INSTANCE: OnceCell<DriverRegistry> = OnceCell::new();
        INSTANCE.get_or_init(|| Self::new())
    }
    
    /// 注册驱动工厂
    pub fn register<F: DriverFactory + 'static>(factory: F) {
        let registry = Self::global();
        let mut factories = registry.factories.write();
        factories.insert(factory.id().to_string(), Box::new(factory));
    }
    
    /// 获取驱动工厂
    pub fn get(id: &str) -> Option<Box<dyn DriverFactory>> {
        let registry = Self::global();
        let factories = registry.factories.read();
        factories.get(id).map(|f| f.box_clone())
    }
    
    /// 列出所有驱动
    pub fn list_all() -> Vec<DriverDescriptor> {
        let registry = Self::global();
        let factories = registry.factories.read();
        factories.values().map(|f| f.descriptor()).collect()
    }
}
```

### DriverFactory Trait

```rust
pub trait DriverFactory: Send + Sync {
    /// 驱动 ID
    fn id(&self) -> &'static str;
    
    /// 驱动描述
    fn descriptor(&self) -> DriverDescriptor;
    
    /// 创建连接
    async fn create(&self, config: ConnectionConfig) -> Result<Box<dyn Database>, CoreError>;
    
    /// 克隆（用于 Registry）
    fn box_clone(&self) -> Box<dyn DriverFactory>;
}
```

## 驱动实现

### PostgreSQL 驱动

```rust
pub struct PostgresDriver {
    pool: PgPool,
    meta: DataSourceMeta,
}

#[async_trait]
impl Database for PostgresDriver {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        let start = Instant::now();
        
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| self.convert_error(e, sql))?;
        
        let columns = rows.first()
            .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect())
            .unwrap_or_default();
        
        let values: Vec<Vec<Value>> = rows.iter()
            .map(|r| self.convert_row(r))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(QueryResult {
            columns,
            rows: values,
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    async fn list_databases(&self) -> Result<Vec<String>, CoreError> {
        let sql = "SELECT datname FROM pg_database WHERE datistemplate = false";
        let result = self.query(sql).await?;
        
        Ok(result.rows.iter()
            .map(|row| row[0].as_str().unwrap_or_default().to_string())
            .collect())
    }
    
    async fn list_schemas(&self, _database: &str) -> Result<Vec<String>, CoreError> {
        let sql = r#"
            SELECT schema_name 
            FROM information_schema.schemata 
            WHERE schema_name NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
        "#;
        let result = self.query(sql).await?;
        
        Ok(result.rows.iter()
            .map(|row| row[0].as_str().unwrap_or_default().to_string())
            .collect())
    }
    
    async fn list_tables(
        &self,
        database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<SchemaObject>, CoreError> {
        let schema = schema.unwrap_or("public");
        let sql = format!(
            r#"
            SELECT 
                table_name,
                CASE table_type 
                    WHEN 'BASE TABLE' THEN 'table'
                    WHEN 'VIEW' THEN 'view'
                END as type
            FROM information_schema.tables
            WHERE table_schema = '{}'
            "#,
            schema
        );
        
        let result = self.query(&sql).await?;
        
        Ok(result.rows.iter()
            .map(|row| SchemaObject {
                name: row[0].as_str().unwrap_or_default().to_string(),
                kind: match row[1].as_str() {
                    Some("view") => SchemaObjectKind::View,
                    _ => SchemaObjectKind::Table,
                },
                children: None,
            })
            .collect())
    }
    
    fn meta(&self) -> DataSourceMeta {
        self.meta.clone()
    }
}

impl PostgresDriver {
    fn convert_error(&self, err: sqlx::Error, sql: &str) -> CoreError {
        match err {
            sqlx::Error::Database(db_err) => {
                let code = db_err.code().unwrap_or_default();
                match code.as_ref() {
                    "23505" => CoreError::database_constraint_violation(
                        db_err.constraint()
                    ),
                    "28P01" => CoreError::connection_auth_failed(
                        "authentication failed"
                    ),
                    _ => CoreError::database_query_error(sql, db_err.message()),
                }
            }
            sqlx::Error::PoolTimedOut => {
                CoreError::connection_timeout(Duration::from_secs(30))
            }
            _ => CoreError::database_error(err.to_string()),
        }
    }
    
    fn convert_row(&self, row: &PgRow) -> Result<Vec<Value>, CoreError> {
        let mut values = Vec::new();
        
        for i in 0..row.len() {
            let value: Value = if let Ok(v) = row.try_get::<Option<String>, _>(i) {
                v.map(Value::String).unwrap_or(Value::Null)
            } else if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
                v.map(Value::Int64).unwrap_or(Value::Null)
            } else if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
                v.map(Value::Float64).unwrap_or(Value::Null)
            } else if let Ok(v) = row.try_get::<Option<bool>, _>(i) {
                v.map(Value::Bool).unwrap_or(Value::Null)
            } else {
                Value::Null
            };
            values.push(value);
        }
        
        Ok(values)
    }
}

// 工厂实现
pub struct PostgresDriverFactory;

impl DriverFactory for PostgresDriverFactory {
    fn id(&self) -> &'static str {
        "postgresql"
    }
    
    fn descriptor(&self) -> DriverDescriptor {
        DriverDescriptor {
            id: "postgresql".to_string(),
            name: "PostgreSQL".to_string(),
            description: "PostgreSQL database driver".to_string(),
            version: "15.0".to_string(),
            icon: Some("postgresql.svg".to_string()),
            default_port: 5432,
            connection_fields: vec![
                DriverField {
                    name: "host".to_string(),
                    label: "Host".to_string(),
                    field_type: DriverFieldType::String,
                    required: true,
                    default_value: Some("localhost".to_string()),
                },
                DriverField {
                    name: "port".to_string(),
                    label: "Port".to_string(),
                    field_type: DriverFieldType::Number,
                    required: true,
                    default_value: Some("5432".to_string()),
                },
                DriverField {
                    name: "database".to_string(),
                    label: "Database".to_string(),
                    field_type: DriverFieldType::String,
                    required: true,
                    default_value: None,
                },
                DriverField {
                    name: "username".to_string(),
                    label: "Username".to_string(),
                    field_type: DriverFieldType::String,
                    required: true,
                    default_value: None,
                },
                DriverField {
                    name: "password".to_string(),
                    label: "Password".to_string(),
                    field_type: DriverFieldType::Password,
                    required: true,
                    default_value: None,
                },
            ],
            features: DriverFeatures {
                supports_transactions: true,
                supports_ssl: true,
                supports_ssh_tunnel: true,
                supports_multiple_databases: true,
                supports_schemas: true,
                supports_views: true,
                supports_stored_procedures: true,
                supports_functions: true,
                supports_triggers: true,
            },
        }
    }
    
    async fn create(&self, config: ConnectionConfig) -> Result<Box<dyn Database>, CoreError> {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.username,
            config.password,
            config.host,
            config.port,
            config.database
        );
        
        let pool = PgPool::connect(&url).await
            .map_err(|e| CoreError::connection_failed(e.to_string()))?;
        
        let meta = DataSourceMeta {
            supports_transaction: true,
            supports_streaming: true,
            supports_arrow: false,
            supports_federated: false,
            supports_concurrent_write: true,
            is_in_memory: false,
        };
        
        Ok(Box::new(PostgresDriver { pool, meta }))
    }
    
    fn box_clone(&self) -> Box<dyn DriverFactory> {
        Box::new(PostgresDriverFactory)
    }
}
```

### MySQL 驱动

```rust
pub struct MySqlDriver {
    pool: MySqlPool,
    meta: DataSourceMeta,
}

#[async_trait]
impl Database for MySqlDriver {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        // 类似 PostgreSQL 实现
        // ...
    }
    
    async fn list_databases(&self) -> Result<Vec<String>, CoreError> {
        let sql = r#"
            SELECT schema_name 
            FROM information_schema.schemata 
            WHERE schema_name NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys')
        "#;
        let result = self.query(sql).await?;
        
        Ok(result.rows.iter()
            .map(|row| row[0].as_str().unwrap_or_default().to_string())
            .collect())
    }
    
    async fn list_schemas(&self, _database: &str) -> Result<Vec<String>, CoreError> {
        // MySQL 中 schema = database，返回自身
        Ok(vec![_database.to_string()])
    }
    
    // ... 其他方法
}
```

### SQLite 驱动

```rust
pub struct SqliteDriver {
    pool: SqlitePool,
    meta: DataSourceMeta,
}

#[async_trait]
impl Database for SqliteDriver {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        // SQLite 实现
        // ...
    }
    
    async fn list_databases(&self) -> Result<Vec<String>, CoreError> {
        // SQLite 是文件数据库，返回主数据库名
        Ok(vec!["main".to_string()])
    }
    
    async fn list_schemas(&self, _database: &str) -> Result<Vec<String>, CoreError> {
        Ok(vec!["main".to_string()])
    }
    
    async fn list_tables(
        &self,
        _database: &str,
        _schema: Option<&str>,
    ) -> Result<Vec<SchemaObject>, CoreError> {
        let sql = r#"
            SELECT name, type FROM sqlite_master 
            WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%'
        "#;
        let result = self.query(sql).await?;
        
        Ok(result.rows.iter()
            .map(|row| SchemaObject {
                name: row[0].as_str().unwrap_or_default().to_string(),
                kind: match row[1].as_str() {
                    Some("view") => SchemaObjectKind::View,
                    _ => SchemaObjectKind::Table,
                },
                children: None,
            })
            .collect())
    }
    
    // ... 其他方法
}
```

### DuckDB 驱动

```rust
pub struct DuckDbDriver {
    conn: Arc<Mutex<duckdb::Connection>>,
    meta: DataSourceMeta,
}

#[async_trait]
impl Database for DuckDbDriver {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        let conn = self.conn.lock().await;
        
        let start = Instant::now();
        
        let mut stmt = conn.prepare(sql)
            .map_err(|e| CoreError::database_error(e.to_string()))?;
        
        let column_names: Vec<String> = stmt.column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        let rows = stmt.query_map([], |row| {
            let mut values = Vec::new();
            for i in 0..column_names.len() {
                let value: Value = if let Ok(v) = row.get::<_, Option<String>>(i) {
                    v.map(Value::String).unwrap_or(Value::Null)
                } else if let Ok(v) = row.get::<_, Option<i64>>(i) {
                    v.map(Value::Int64).unwrap_or(Value::Null)
                } else if let Ok(v) = row.get::<_, Option<f64>>(i) {
                    v.map(Value::Float64).unwrap_or(Value::Null)
                } else {
                    Value::Null
                };
                values.push(value);
            }
            Ok(values)
        }).map_err(|e| CoreError::database_error(e.to_string()))?;
        
        let values: Vec<Vec<Value>> = rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::database_error(e.to_string()))?;
        
        Ok(QueryResult {
            columns: column_names,
            rows: values,
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    async fn list_databases(&self) -> Result<Vec<String>, CoreError> {
        let sql = "SELECT database_name FROM information_schema.schemata GROUP BY database_name";
        let result = self.query(sql).await?;
        
        Ok(result.rows.iter()
            .map(|row| row[0].as_str().unwrap_or_default().to_string())
            .collect())
    }
    
    // ... 其他方法
}
```

## 驱动特性支持

| 特性 | PostgreSQL | MySQL | SQLite | DuckDB |
|------|------------|-------|--------|--------|
| 事务 | ✅ | ✅ | ✅ | ✅ |
| SSL | ✅ | ✅ | ❌ | ❌ |
| SSH 隧道 | ✅ | ✅ | ❌ | ❌ |
| 多数据库 | ✅ | ✅ | ❌ | ✅ |
| Schema | ✅ | ❌ | ❌ | ✅ |
| 视图 | ✅ | ✅ | ✅ | ✅ |
| 存储过程 | ✅ | ✅ | ❌ | ❌ |
| 函数 | ✅ | ✅ | ❌ | ✅ (宏) |
| 触发器 | ✅ | ✅ | ✅ | ❌ |

## 连接池管理

### 连接池配置

```rust
pub struct PoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub test_on_checkout: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            test_on_checkout: true,
        }
    }
}
```

### 连接池实现

```rust
pub struct PgPoolWrapper {
    inner: PgPool,
    config: PoolConfig,
}

#[async_trait]
impl DbPool for PgPoolWrapper {
    async fn acquire(&self) -> Result<Box<dyn Database>, CoreError> {
        let conn = self.inner.acquire().await
            .map_err(|e| CoreError::connection_pool_exhausted(e.to_string()))?;
        
        Ok(Box::new(PgConnection::new(conn)))
    }
    
    fn status(&self) -> PoolStatus {
        PoolStatus {
            size: self.inner.size(),
            idle: self.inner.num_idle(),
            active: self.inner.size() - self.inner.num_idle(),
        }
    }
    
    async fn close(&self) {
        self.inner.close().await;
    }
}
```

## 错误处理

### 错误转换

```rust
impl PostgresDriver {
    fn convert_error(&self, err: sqlx::Error, sql: &str) -> CoreError {
        match err {
            sqlx::Error::Database(db_err) => {
                let code = db_err.code().unwrap_or_default();
                match code.as_ref() {
                    // 唯一约束冲突
                    "23505" => CoreError::database_constraint_violation(
                        db_err.constraint()
                    ),
                    // 外键约束冲突
                    "23503" => CoreError::database_foreign_key_violation(
                        db_err.constraint()
                    ),
                    // 检查约束冲突
                    "23514" => CoreError::database_check_violation(
                        db_err.constraint()
                    ),
                    // 认证失败
                    "28P01" => CoreError::connection_auth_failed(
                        "authentication failed"
                    ),
                    // 数据库不存在
                    "3D000" => CoreError::connection_database_not_found(
                        db_err.message()
                    ),
                    // 语法错误
                    "42601" => CoreError::database_syntax_error(
                        sql, db_err.message()
                    ),
                    _ => CoreError::database_query_error(sql, db_err.message()),
                }
            }
            sqlx::Error::PoolTimedOut => {
                CoreError::connection_timeout(Duration::from_secs(30))
            }
            sqlx::Error::PoolClosed => {
                CoreError::connection_pool_closed()
            }
            _ => CoreError::database_error(err.to_string()),
        }
    }
}
```

## 智能连接池 🔥

### SmartPoolWrapper

智能连接池在原有 DbPool 基础上增加了动态扩缩容、健康检查和负载均衡能力：

```rust
pub struct SmartPoolWrapper {
    inner: Arc<dyn DbPool>,
    config: SmartPoolConfig,
    metrics: Arc<PoolMetrics>,
    closed: Arc<AtomicBool>,
}

impl SmartPoolWrapper {
    /// 动态调整连接池大小
    pub async fn adjust_pool_size(&self) -> Result<(), CoreError> {
        let current_load = self.metrics.get_active_connections();
        let max_connections = self.config.max_connections;
        
        if current_load > (max_connections as f64 * 0.8) as u32 {
            // 高负载：扩容
            self.expand_pool().await?;
        } else if current_load < (max_connections as f64 * 0.2) as u32 {
            // 低负载：缩容
            self.shrink_pool().await?;
        }
        
        Ok(())
    }
    
    /// 健康检查
    pub async fn health_check(&self) -> PoolHealthStatus {
        let status = self.inner.status();
        let error_rate = self.metrics.get_error_rate();
        
        if error_rate > 0.5 {
            PoolHealthStatus::Unhealthy
        } else if status.active > (status.size as f64 * 0.9) as u32 {
            PoolHealthStatus::Degraded
        } else {
            PoolHealthStatus::Healthy
        }
    }
}

#[async_trait]
impl DbPool for SmartPoolWrapper {
    async fn acquire(&self) -> Result<Box<dyn Database>, CoreError> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(CoreError::connection_pool_closed());
        }
        
        let db = self.inner.acquire().await?;
        self.metrics.record_connection_acquired();
        Ok(db)
    }
    
    fn status(&self) -> PoolStatus {
        self.inner.status()
    }
    
    async fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.inner.close().await;
    }
}
```

### 连接池指标监控

```rust
pub struct PoolMetrics {
    active_connections: AtomicU32,
    total_acquisitions: AtomicU64,
    total_releases: AtomicU64,
    errors: AtomicU64,
    avg_wait_time_ms: AtomicU64,
}

impl PoolMetrics {
    pub fn get_active_connections(&self) -> u32 {
        self.active_connections.load(Ordering::SeqCst)
    }
    
    pub fn get_error_rate(&self) -> f64 {
        let total = self.total_acquisitions.load(Ordering::SeqCst);
        if total == 0 {
            return 0.0;
        }
        self.errors.load(Ordering::SeqCst) as f64 / total as f64
    }
    
    pub fn record_connection_acquired(&self) {
        self.active_connections.fetch_add(1, Ordering::SeqCst);
        self.total_acquisitions.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn record_connection_released(&self) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
        self.total_releases.fetch_add(1, Ordering::SeqCst);
    }
}
```

## DuckDB 联邦查询 🔥

### DuckDBEngine 架构

DuckDB 作为分析引擎，支持外部数据库 ATTACH 和联邦查询：

```rust
pub struct DuckDBEngine {
    duckdb_conn: Arc<Mutex<Option<duckdb::Connection>>>,
    external_connections: Arc<Mutex<Vec<ExternalConnection>>>,
    result_sets: Arc<Mutex<Vec<String>>>,
}

impl DuckDBEngine {
    /// 注册外部数据库
    pub async fn register_external_database(
        &self,
        name: &str,
        driver: &str,
        connection_string: &str,
    ) -> Result<(), CoreError> {
        let mut connections = self.external_connections.lock().await;
        
        // 检查是否已存在
        if connections.iter().any(|c| c.name == name) {
            return Err(CoreError::common(CommonError::General(
                format!("External database '{}' already registered", name),
            )));
        }
        
        connections.push(ExternalConnection {
            name: name.to_string(),
            driver: driver.to_string(),
            connection_string: connection_string.to_string(),
            read_only: true, // DuckDB 加速模式只读
        });
        
        // 执行 ATTACH 命令
        let attach_sql = format!(
            "ATTACH '{}' AS {} (READ_ONLY)",
            connection_string, name
        );
        
        self.execute_internal(&attach_sql).await?;
        
        Ok(())
    }
    
    /// 卸载外部数据库
    pub async fn detach_external_database(&self, name: &str) -> Result<(), CoreError> {
        let detach_sql = format!("DETACH {}", name);
        self.execute_internal(&detach_sql).await?;
        
        let mut connections = self.external_connections.lock().await;
        connections.retain(|c| c.name != name);
        
        Ok(())
    }
    
    /// 加载文件数据源
    pub async fn load_file_source(
        &self,
        path: &str,
        table_name: &str,
    ) -> Result<(), CoreError> {
        let sql = if path.ends_with(".csv") {
            format!(
                "CREATE TEMP TABLE {} AS SELECT * FROM read_csv_auto('{}')",
                table_name, path
            )
        } else if path.ends_with(".parquet") {
            format!(
                "CREATE TEMP TABLE {} AS SELECT * FROM read_parquet('{}')",
                table_name, path
            )
        } else if path.ends_with(".xlsx") || path.ends_with(".xls") {
            format!(
                "CREATE TEMP TABLE {} AS SELECT * FROM read_excel_auto('{}')",
                table_name, path
            )
        } else {
            return Err(CoreError::common(CommonError::NotSupported(
                format!("Unsupported file type: {}", path),
            )));
        };
        
        self.execute_internal(&sql).await
    }
    
    /// 创建持久化结果集
    pub async fn persist_result_set(
        &self,
        result_name: &str,
        sql: &str,
    ) -> Result<(), CoreError> {
        let persist_sql = format!(
            "CREATE TABLE {} AS {}",
            result_name, sql
        );
        
        self.execute_internal(&persist_sql).await?;
        
        let mut result_sets = self.result_sets.lock().await;
        result_sets.push(result_name.to_string());
        
        Ok(())
    }
    
    /// 执行联邦查询
    pub async fn execute_federated_query(
        &self,
        sql: &str,
    ) -> Result<QueryResult, CoreError> {
        let start = Instant::now();
        
        // DuckDB 会自动优化查询并下推到外部数据源
        let result = self.execute_internal(sql).await?;
        
        Ok(QueryResult {
            columns: result.columns,
            rows: result.rows,
            execution_time_ms: start.elapsed().as_millis() as u64,
            affected_rows: result.affected_rows,
            is_read_only: true,
        })
    }
}
```

### 外部连接管理

```rust
pub struct ExternalConnection {
    pub name: String,
    pub driver: String,
    pub connection_string: String,
    pub read_only: bool,
}

impl ExternalConnection {
    /// 构建 ATTACH SQL
    pub fn build_attach_sql(&self) -> String {
        if self.read_only {
            format!(
                "ATTACH '{}' AS {} (READ_ONLY)",
                self.connection_string, self.name
            )
        } else {
            format!(
                "ATTACH '{}' AS {}",
                self.connection_string, self.name
            )
        }
    }
}
```

### DuckDB 扩展管理

```rust
pub struct DuckDBExtensionManager {
    loaded_extensions: Arc<Mutex<Vec<String>>>,
}

impl DuckDBExtensionManager {
    /// 加载扩展
    pub async fn load_extension(&self, extension: &str) -> Result<(), CoreError> {
        let sql = format!("LOAD {}", extension);
        
        // 执行加载
        // TODO: 实际执行
        
        let mut extensions = self.loaded_extensions.lock().await;
        extensions.push(extension.to_string());
        
        Ok(())
    }
    
    /// 列出已加载扩展
    pub async fn list_extensions(&self) -> Vec<String> {
        self.loaded_extensions.lock().await.clone()
    }
}
```

## 测试

### Mock 驱动

```rust
#[cfg(test)]
pub struct MockDriver {
    queries: Arc<Mutex<HashMap<String, QueryResult>>>,
}

#[cfg(test)]
#[async_trait]
impl Database for MockDriver {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        let queries = self.queries.lock().await;
        queries.get(sql).cloned()
            .ok_or_else(|| CoreError::database_error("Query not mocked"))
    }
    
    // ... 其他方法
}

#[cfg(test)]
impl MockDriver {
    pub fn new() -> Self {
        Self {
            queries: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn mock_query(&self, sql: &str, result: QueryResult) {
        let mut queries = self.queries.lock().await;
        queries.insert(sql.to_string(), result);
    }
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_postgres_driver() {
    let config = ConnectionConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: "test".to_string(),
        username: "postgres".to_string(),
        password: "password".to_string(),
        ..Default::default()
    };
    
    let factory = PostgresDriverFactory;
    let db = factory.create(config).await.unwrap();
    
    // 测试 ping
    let latency = db.ping().await.unwrap();
    assert!(latency < Duration::from_secs(1));
    
    // 测试查询
    let result = db.query("SELECT 1 as one").await.unwrap();
    assert_eq!(result.columns, vec!["one"]);
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Int64(1));
}
```
