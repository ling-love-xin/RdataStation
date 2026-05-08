# 数据流设计

## 概述

本文档描述 RdataStation 后端的数据流，包括请求处理流程、元数据查询流程、DBI 查询流程、DuckDB 联邦查询流程、错误传播机制等。

## 请求处理流程

### 1. SQL 执行流程（传统架构）

```
Frontend (Vue3)
    │
    │ POST /execute_sql
    │ { sql: "SELECT * FROM users", conn_id: "conn_123" }
    ▼
Tauri Runtime
    │
    ▼
Adapter Layer (adapters/tauri/command.rs)
    │ 1. 输入校验
    │    - SQL 非空检查
    │    - conn_id 有效性检查
    │
    │ 2. DTO 转换
    │    - ExecuteSqlInput → 内部类型
    ▼
Core Services (core/services/sql_service.rs)
    │ 1. 获取连接
    │    - ConnectionManager.get_connection(conn_id)
    │
    │ 2. 记录历史
    │    - HistoryStore.save(sql)
    │
    │ 3. 执行 SQL
    │    - Database.query(sql)
    ▼
Database Driver (core/datasource/postgres.rs)
    │ 1. 获取连接池
    │    - Pool.acquire()
    │
    │ 2. 执行查询
    │    - sqlx::query(sql).fetch_all()
    │
    │ 3. 转换结果
    │    - Row → Value → QueryResult
    ▼
PostgreSQL Server
    │
    ▼
Result Propagation
    │ QueryResult
    ▼
Database Driver
    │
    ▼
Core Services
    │
    ▼
Adapter Layer
    │ 1. 结果转换
    │    - QueryResult → ExecuteSqlResponse
    │
    │ 2. 错误处理
    │    - CoreError → String (用户友好)
    ▼
Frontend
    │ { columns: [...], rows: [...], execution_time_ms: 123 }
    ▼
UI 渲染
```

### 2. DBI 查询流程（新架构）🔥

```
Frontend (Vue3)
    │
    │ invoke('dbi_query', { sql: 'SELECT * FROM users', mode: 'auto' })
    ▼
Tauri Command
    │ 1. 输入校验
    │ 2. 创建 DBI 实例
    ▼
DBI::query(sql, mode)
    │ 1. 创建 QueryContext
    │    - connection_id
    │    - execution_mode
    │    - session_config
    ▼
QueryRouter.execute(context)
    │ 2. 智能推荐执行模式（如果 mode == auto）
    │    - 写操作 → Native
    │    - 复杂查询 → DuckDB
    │    - 默认 → UserChoice
    ▼
┌─────────────────────────────────────┐
│  执行引擎选择                        │
├─────────────────────────────────────┤
│  Native Engine                      │
│  └→ Driver::query()                 │
│  └→ Database 实现                   │
│                                     │
│  DuckDB Engine                      │
│  └→ ATTACH 外部数据库               │
│  └→ 联邦查询执行                    │
│  └→ Arrow RecordBatch 返回          │
│                                     │
│  Stream Engine                      │
│  └→ 流式处理                        │
│  └→ 分块返回结果                    │
└─────────────────────────────────────┘
    │
    ▼
Result (Arrow RecordBatch)
    │ 3. 转换为 QueryResult
    │ 4. 返回给前端
    ▼
Frontend
    │ { columns: [...], rows: [...], execution_time_ms: 123 }
    ▼
UI 渲染 (ag-Grid 虚拟滚动)
```

### 3. DuckDB 联邦查询流程 🔥

```
DBI::query(sql, ExecutionMode::DuckDB)
    │
    ▼
DuckDBEngine.execute(sql)
    │ 1. 检查外部数据库注册状态
    │ 2. 执行 ATTACH 命令（如果需要）
    │    - ATTACH 'mysql://user:pass@host/db' AS mysql (READ_ONLY)
    │    - ATTACH 'postgres://user:pass@host/db' AS pg (READ_ONLY)
    ▼
┌─────────────────────────────────────┐
│  DuckDB 实例                         │
│  ├── 本地数据库                      │
│  ├── ATTACH 'mysql://...' AS mysql  │
│  ├── ATTACH 'postgres://...' AS pg  │
│  └── read_csv_auto('data.csv')      │
└─────────────────────────────────────┘
    │
    ▼
联邦查询执行
    │ SELECT * FROM mysql.users u
    │ JOIN pg.orders o ON u.id = o.user_id
    │ WHERE u.created_at > '2024-01-01'
    ▼
DuckDB 查询优化器
    │ 1. 解析 SQL
    │ 2. 生成执行计划
    │ 3. 下推到外部数据源
    ▼
外部数据源并行执行
    │ MySQL: SELECT * FROM users WHERE created_at > '2024-01-01'
    │ PostgreSQL: SELECT * FROM orders
    ▼
结果合并（Arrow RecordBatch）
    │ 零拷贝合并
    │ 内存高效
    ▼
Result (Arrow Format)
```

**时序图**：

```mermaid
sequenceDiagram
    participant F as Frontend
    participant A as Adapter
    participant S as SqlService
    participant M as ConnectionManager
    participant D as PostgresDriver
    participant DB as PostgreSQL

    F->>A: invoke('execute_sql', input)
    A->>A: validate input
    A->>S: execute(conn_id, sql)
    S->>M: get_connection(conn_id)
    M-->>S: db: Database
    S->>S: record_history(sql)
    S->>D: query(sql)
    D->>DB: SELECT * FROM users
    DB-->>D: rows
    D-->>S: QueryResult
    S-->>A: QueryResult
    A->>A: convert to DTO
    A-->>F: ExecuteSqlResponse
```

### 2. 连接建立流程

```
Frontend
    │
    │ POST /connect_database
    │ { db_type: "postgresql", url: "postgres://...", name: "MyDB" }
    ▼
Adapter Layer
    │ 1. 参数校验
    │    - URL 非空
    │    - db_type 有效性
    ▼
ConnectionService
    │ 1. 创建连接配置
    │    - ConnectionConfig::from_url(url)
    │
    │ 2. 获取驱动工厂
    │    - DriverRegistry.get_factory(db_type)
    │
    │ 3. 创建连接
    │    - factory.create(config)
    ▼
PostgresDriver
    │ 1. 解析 URL
    │    - host, port, database, username, password
    │
    │ 2. 创建连接池
    │    - PgPool::connect(url)
    │
    │ 3. 测试连接
    │    - pool.acquire().await?
    ▼
PostgreSQL
    │
    ▼
ConnectionManager
    │ 1. 存储连接
    │    - connections.insert(id, db)
    │
    │ 2. 保存最近连接
    │    - ConnectionStore.save(config)
    ▼
Adapter Layer
    │ 1. 构建响应
    │    - conn_id, name, db_type, meta
    ▼
Frontend
    │ { conn_id: "conn_123", name: "MyDB", ... }
```

### 3. 元数据查询流程

#### 3.1 获取数据库列表

```
Navigator Panel (Frontend)
    │
    │ 展开连接节点
    │ invoke('get_databases', { conn_id: "conn_123" })
    ▼
Adapter Layer (get_databases)
    │
    ▼
ConnectionManager
    │ get_connection(conn_id)
    ▼
PostgresDriver
    │ list_databases()
    │
    │ SQL: SELECT datname FROM pg_database WHERE datistemplate = false
    ▼
PostgreSQL
    │
    ▼
Result
    │ ["postgres", "myapp", "test"]
    ▼
Adapter Layer
    │ map to DatabaseInfoResponse
    ▼
Frontend
    │ [{ name: "postgres" }, { name: "myapp" }, ...]
```

#### 3.2 获取表列表（带缓存）

```
Navigator Panel
    │
    │ 展开 schema 节点
    │ invoke('get_tables', { conn_id, database, schema })
    ▼
MetaDataCache (L1 - Memory)
    │ check cache: metadata:conn_123:tables:public
    │ Cache MISS
    ▼
MetaDataCache (L2 - IndexedDB)
    │ check cache
    │ Cache MISS (或已过期)
    ▼
Adapter Layer (get_tables)
    │
    ▼
Database.list_tables(database, schema)
    │
    ▼
PostgreSQL
    │
    ▼
Result
    │ ["users", "orders", "products"]
    ▼
MetaDataCache
    │ 1. L1.set(key, value, TTL=5min)
    │ 2. L2.set(key, value, TTL=1hour)
    ▼
Frontend
```

## 错误传播流程

### 错误类型层级

```
CoreError (核心错误)
    ├── ConnectionError (连接错误)
    │       ├── NotFound { conn_id }
    │       ├── InvalidUrl { url }
    │       ├── AuthenticationFailed { username }
    │       └── Timeout { duration }
    │
    ├── DatabaseError (数据库错误)
    │       ├── QueryError { sql, message }
    │       ├── ConstraintViolation { constraint }
    │       └── ConnectionLost
    │
    ├── StorageError (存储错误)
    │       ├── ReadError { path }
    │       ├── WriteError { path }
    │       └── SerializationError
    │
    └── CommonError (通用错误)
            ├── InvalidArgument { name, value }
            ├── NotSupported { feature }
            └── Internal { message }
```

### 错误传播路径

```
PostgreSQL (底层错误)
    │ sqlx::Error::Database(db_err)
    ▼
PostgresDriver
    │ 1. 转换错误类型
    │    - 唯一约束冲突 → ConstraintViolation
    │    - 连接断开 → ConnectionLost
    │    - 语法错误 → QueryError
    │
    │ 2. 添加上下文
    │    - sql: "INSERT INTO ..."
    ▼
CoreError::DatabaseError
    │
    ▼
SqlService
    │ 1. 记录错误日志
    │    - error!("SQL execution failed: {}", err)
    │
    │ 2. 返回错误
    ▼
Adapter Layer
    │ 1. 错误转换
    │    - CoreError → 用户友好消息
    │
    │ 2. 脱敏处理
    │    - 移除敏感信息（密码等）
    │
    │ 3. 构建响应
    │    - Err("查询失败: 表不存在".to_string())
    ▼
Frontend
    │ 显示错误提示
```

### 错误处理代码示例

```rust
// 底层：PostgresDriver
async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
    let rows = sqlx::query(sql)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) => {
                if db_err.code() == Some("23505".into()) {
                    CoreError::database_constraint_violation(
                        db_err.constraint()
                    )
                } else {
                    CoreError::database_query_error(sql, db_err.message())
                }
            }
            sqlx::Error::PoolTimedOut => {
                CoreError::connection_timeout(Duration::from_secs(30))
            }
            _ => CoreError::database_error(e.to_string()),
        })?;

    Ok(convert_rows(rows))
}

// 服务层：SqlService
pub async fn execute(&self, sql: &str) -> Result<QueryResult, CoreError> {
    let start = Instant::now();

    let result = self.db.query(sql).await;

    if let Err(ref e) = result {
        error!(
            sql = sql,
            error = ?e,
            duration_ms = start.elapsed().as_millis(),
            "SQL execution failed"
        );
    }

    result
}

// 适配层：Tauri Command
#[tauri::command]
pub async fn execute_sql(input: ExecuteSqlInput) -> Result<ExecuteSqlResponse, String> {
    let service = SqlService::new();

    service.execute(&input.sql).await
        .map(|r| r.into())
        .map_err(|e| {
            // 转换为用户友好的错误消息
            let msg = match e {
                CoreError::Database(DatabaseError::QueryError { sql, message }) => {
                    format!("查询失败: {}", message)
                }
                CoreError::Connection(ConnectionError::NotFound { conn_id }) => {
                    format!("连接不存在: {}", conn_id)
                }
                _ => "执行失败，请稍后重试".to_string(),
            };

            // 脱敏：移除 SQL 中的敏感信息
            sanitize_error_message(msg)
        })
}
```

## 数据转换流程

### DTO 转换

```rust
// 内部模型 → API DTO

// core/models.rs
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub execution_time_ms: u64,
}

// api/dto.rs
#[derive(Serialize)]
pub struct QueryResultDto {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<ValueDto>>,
    pub execution_time_ms: u64,
}

// 转换实现
impl From<QueryResult> for QueryResultDto {
    fn from(result: QueryResult) -> Self {
        Self {
            columns: result.columns,
            rows: result.rows.into_iter()
                .map(|row| row.into_iter().map(|v| v.into()).collect())
                .collect(),
            execution_time_ms: result.execution_time_ms,
        }
    }
}
```

### 数据库行转换

```rust
// PostgreSQL Row → Value

use sqlx::Row;

fn convert_row(row: &PgRow, columns: &[String]) -> Result<Vec<Value>, CoreError> {
    let mut values = Vec::with_capacity(columns.len());

    for (i, col) in columns.iter().enumerate() {
        let value: Value = match row.try_get::<Option<String>, _>(i)? {
            Some(s) => Value::String(s),
            None => Value::Null,
        };
        values.push(value);
    }

    Ok(values)
}
```

## 异步流程

### 并发查询

```rust
// 同时查询多个表的信息

async fn get_tables_info(
    db: &dyn Database,
    tables: &[String],
) -> Result<Vec<TableInfo>, CoreError> {
    let futures = tables.iter().map(|table| {
        async move {
            let row_count = db.query(&format!("SELECT COUNT(*) FROM {}", table)).await?;
            let size = db.query(&format!(
                "SELECT pg_total_relation_size('{table}')"
            )).await?;

            Ok(TableInfo {
                name: table.clone(),
                row_count: row_count.get::<i64, _>(0) as usize,
                size_bytes: size.get::<i64, _>(0) as usize,
            })
        }
    });

    // 并发执行
    let results = futures::future::join_all(futures).await;

    // 收集结果
    results.into_iter().collect::<Result<Vec<_>, _>>()
}
```

### 流式查询

```rust
// 大数据量流式处理

pub async fn stream_query<F>(
    &self,
    sql: &str,
    mut callback: F,
) -> Result<(), CoreError>
where
    F: FnMut(Vec<Value>) -> Result<(), CoreError>,
{
    let mut stream = sqlx::query(sql)
        .fetch(&self.pool);

    while let Some(row) = stream.try_next().await? {
        let values = convert_row(&row)?;
        callback(values)?;
    }

    Ok(())
}
```

## 缓存策略

### 多级缓存

```rust
pub struct MetadataCache {
    // L1: 内存缓存（当前会话）
    l1: Arc<RwLock<HashMap<String, CachedValue>>>,

    // L2: IndexedDB（跨会话）
    l2: Arc<dyn CacheStore>,

    // L3: 后端 SQLite（项目级）
    l3: Arc<dyn CacheStore>,
}

impl MetadataCache {
    pub async fn get(&self, key: &str) -> Option<CachedValue> {
        // 1. 检查 L1
        if let Some(value) = self.l1.read().await.get(key) {
            if !value.is_expired() {
                return Some(value.clone());
            }
        }

        // 2. 检查 L2
        if let Some(value) = self.l2.get(key).await.ok().flatten() {
            // 回填 L1
            self.l1.write().await.insert(key.to_string(), value.clone());
            return Some(value);
        }

        // 3. 检查 L3
        if let Some(value) = self.l3.get(key).await.ok().flatten() {
            // 回填 L1 和 L2
            self.l1.write().await.insert(key.to_string(), value.clone());
            self.l2.set(key, &value).await.ok()?;
            return Some(value);
        }

        None
    }

    pub async fn set(
        &self,
        key: &str,
        value: CachedValue,
        ttl: Duration,
    ) {
        let value = value.with_ttl(ttl);

        // 写入 L1
        self.l1.write().await.insert(key.to_string(), value.clone());

        // 异步写入 L2 和 L3
        let l2 = self.l2.clone();
        let l3 = self.l3.clone();
        let key = key.to_string();

        tokio::spawn(async move {
            l2.set(&key, &value).await.ok();
            l3.set(&key, &value).await.ok();
        });
    }
}
```

## 性能优化

### 批量操作

```rust
// 批量插入

pub async fn batch_insert(
    &self,
    table: &str,
    columns: &[String],
    rows: Vec<Vec<Value>>,
) -> Result<(), CoreError> {
    if rows.is_empty() {
        return Ok(());
    }

    // 构建批量插入 SQL
    let placeholders: Vec<String> = (0..rows.len())
        .map(|i| {
            let start = i * columns.len() + 1;
            let end = start + columns.len();
            format!("({})", (start..end).map(|n| format!("${}", n)).join(", "))
        })
        .collect();

    let sql = format!(
        "INSERT INTO {} ({}) VALUES {}",
        table,
        columns.join(", "),
        placeholders.join(", ")
    );

    // 扁平化参数
    let params: Vec<&(dyn ToSql + Sync)> = rows
        .iter()
        .flat_map(|row| row.iter().map(|v| v as &(dyn ToSql + Sync)))
        .collect();

    self.execute(&sql, &params).await
}
```

### 连接池管理

```rust
// 连接池配置

pub struct PoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}
```
