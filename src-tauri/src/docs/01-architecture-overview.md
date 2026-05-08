# 架构概述

## 设计目标

RdataStation 后端架构设计目标：

1. **可维护性**：代码结构清晰，易于理解和维护
2. **可测试性**：业务逻辑可独立单元测试
3. **可扩展性**：轻松添加新的数据库支持
4. **可移植性**：Core 层无框架依赖，可复用

## 架构风格

采用**分层架构（Layered Architecture）** + **六边形架构（Hexagonal Architecture）**混合风格：

### 分层架构

```
┌─────────────────────────────────────────┐
│  Presentation Layer (Adapters)          │
│  - Tauri Commands                        │
│  - CLI Commands (未来)                   │
│  - HTTP API (未来)                       │
├─────────────────────────────────────────┤
│  Application Layer (Services)           │
│  - 业务服务编排                           │
│  - 连接管理                              │
│  - SQL 执行                              │
├─────────────────────────────────────────┤
│  DBI Layer (统一数据访问) 🔥             │
│  - DBI::query() / execute()             │
│  - QueryRouter (智能路由)                │
│  - Session / Context 管理                │
│  - 执行引擎路由                          │
├─────────────────────────────────────────┤
│  Domain Layer (Core)                    │
│  - 领域模型                              │
│  - 业务规则                              │
│  - 驱动抽象                              │
├─────────────────────────────────────────┤
│  Infrastructure Layer                   │
│  ├── driver/native/    - 原生驱动实现    │
│  ├── driver/jdbc/      - JDBC 桥接驱动   │
│  ├── driver/wasm/      - WASM 驱动       │
│  ├── dbi/engine/       - 执行引擎        │
│  │   ├── driver_engine   - 原生引擎      │
│  │   ├── duckdb_engine   - DuckDB加速    │
│  │   └── stream_engine   - 流处理引擎    │
│  └── datasource/       - 数据源路由      │
└─────────────────────────────────────────┘
```

### 六边形架构视角

```
                    ┌─────────────┐
                    │  Frontend   │
                    │   (Vue3)    │
                    └──────┬──────┘
                           │
┌──────────────┐    ┌──────▼──────┐    ┌──────────────┐
│   CLI        │◄──►│             │◄──►│   HTTP API   │
│  Adapter     │    │    Core     │    │   Adapter    │
└──────────────┘    │   (Domain)  │    └──────────────┘
                    │             │
┌──────────────┐    │             │    ┌──────────────┐
│   Tauri      │◄──►│             │◄──►│   Extism     │
│  Adapter     │    └──────┬──────┘    │   Plugins    │
└──────────────┘           │           └──────────────┘
                           │
                    ┌──────▼──────┐
                    │  Database   │
                    │  Drivers    │
                    └─────────────┘
```

## 核心模块

### 1. API 层 (`api/`)

**职责**：定义前后端共享的数据类型

```rust
// api/dto.rs
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub execution_time_ms: u64,
}

pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}
```

**设计原则**：

- DTO 与内部模型分离
- 使用 serde 支持序列化
- 版本化管理（未来）

### 2. Core 层 (`core/`)

**职责**：纯业务逻辑，无外部依赖

#### 2.1 Driver 模块 (`driver/`)

数据库驱动抽象：

```rust
// driver/traits.rs
#[async_trait]
pub trait Database: Send + Sync {
    /// 执行 SQL 查询
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError>;

    /// 获取数据源元数据
    fn meta(&self) -> DataSourceMeta;

    /// 列出数据库
    async fn list_databases(&self) -> Result<Vec<String>, CoreError>;

    /// 列出表
    async fn list_tables(&self, database: &str, schema: Option<&str>)
        -> Result<Vec<SchemaObject>, CoreError>;
}

/// 事务支持
trait Transaction {
    async fn commit(&self) -> Result<(), CoreError>;
    async fn rollback(&self) -> Result<(), CoreError>;
}
```

#### 2.2 DBI 模块 (`dbi/`) 🔥

统一数据访问层：

```
dbi/
├── mod.rs              # 模块导出
├── dbi.rs              # DBI 主接口
├── session.rs          # 会话管理
├── context.rs          # 查询上下文
└── engine/             # 执行引擎
    ├── mod.rs          # 引擎模块导出
    ├── driver_engine.rs # 原生驱动引擎
    ├── duckdb_engine.rs # DuckDB 加速引擎
    └── stream_engine.rs # 流处理引擎
```

DBI 接口：

```rust
// dbi/dbi.rs
pub struct DBI {
    router: Arc<QueryRouter>,
    session: Arc<Session>,
}

impl DBI {
    /// 执行查询（只读）
    pub async fn query(&self, sql: &str, mode: ExecutionMode)
        -> Result<QueryResult, CoreError>;

    /// 执行更新（写操作）
    pub async fn execute(&self, sql: &str)
        -> Result<QueryResult, CoreError>;
}
```

执行模式：

```rust
// dbi/engine/mod.rs
pub enum ExecutionMode {
    Native,      // 原生驱动执行
    DuckDB,      // DuckDB 加速执行
    Stream,      // 流式执行
    UserChoice,  // 用户选择
}
```

智能路由：

```rust
// dbi/engine/mod.rs
pub struct QueryRouter {
    driver_engine: Arc<DriverEngine>,
    duckdb_engine: Arc<DuckDBEngine>,
    stream_engine: Arc<StreamEngine>,
}

impl QueryRouter {
    /// 智能推荐执行模式
    pub fn recommend_mode(&self, sql: &str) -> ExecutionMode {
        // 写操作 → Native
        // 复杂查询 → DuckDB
        // 默认 → UserChoice
    }
}
```

#### 2.3 Datasource 模块 (`datasource/`)

数据源路由层（注册/路由，不存放具体驱动实现）：

```
datasource/
├── mod.rs          # 模块导出
└── router.rs       # 数据源路由器
```

具体驱动实现在 `driver/native/`、`driver/jdbc/`、`driver/wasm/` 中。

#### 2.4 Services 模块 (`services/`)

业务服务：

```rust
// services/connection_service.rs
pub struct ConnectionService {
    manager: Arc<ConnectionManager>,
}

impl ConnectionService {
    /// 创建连接
    pub async fn connect(&self, config: ConnectionConfig)
        -> Result<String, CoreError>;

    /// 关闭连接
    pub async fn close_connection(&self, conn_id: &str)
        -> Result<(), CoreError>;

    /// 执行 SQL
    pub async fn execute_sql(&self, conn_id: &str, sql: &str)
        -> Result<QueryResult, CoreError>;
}
```

#### 2.5 Persistence 模块 (`persistence/`)

数据持久化：

```rust
// persistence/connection_store.rs
pub struct ConnectionStore;

impl ConnectionStore {
    /// 保存连接配置
    pub fn save(config: &ConnectionConfig) -> Result<(), StorageError>;

    /// 加载最近连接
    pub fn load_recent() -> Result<Vec<RecentConnection>, StorageError>;
}
```

### 3. Adapters 层 (`adapters/`)

**职责**：框架适配，输入输出转换

```rust
// adapters/tauri/command.rs
#[tauri::command]
pub async fn execute_sql(
    input: ExecuteSqlInput,
) -> Result<ExecuteSqlResponse, String> {
    // 1. 输入校验
    if input.sql.trim().is_empty() {
        return Err("SQL cannot be empty".into());
    }

    // 2. 调用 Core 服务
    let service = SqlService::new(get_connection_manager());
    let result = service.execute(&input.sql).await
        .map_err(|e| e.to_string())?;

    // 3. 转换为响应 DTO
    Ok(result.into())
}
```

## 数据流

### 请求处理流程

```
Frontend (Vue3)
    │
    │ invoke('execute_sql', { sql: 'SELECT * FROM users' })
    ▼
Tauri Runtime
    │
    ▼
Adapter Layer (command.rs)
    │ 1. 输入校验
    │ 2. DTO 转换
    ▼
Core Services
    │ 1. 业务逻辑处理
    │ 2. 调用 Driver
    ▼
Database Driver
    │ 1. 执行 SQL
    │ 2. 返回结果
    ▼
Result Propagation
    │ (逐层返回)
    ▼
Frontend
```

### 元数据查询流程

```
Navigator Panel (Frontend)
    │
    │ 1. 展开连接节点
    ▼
MetaNavigatorService
    │ 2. 检查缓存
    │ 3. 调用后端 API
    ▼
Tauri Command (get_databases)
    │ 4. 获取连接
    ▼
Database.list_databases()
    │ 5. 执行元数据查询
    ▼
PostgreSQL / MySQL / etc.
    │
    ▼
Result ←── (逐层返回) ←──
```

### DBI 查询流程（新架构）🔥

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
    │ 3. 创建 QueryContext
    │ 4. 调用 QueryRouter
    ▼
QueryRouter.execute(context)
    │ 5. 智能推荐执行模式
    │    - 写操作 → Native
    │    - 复杂查询 → DuckDB
    │    - 默认 → UserChoice
    ▼
┌─────────────────────────────────────┐
│  执行引擎选择                        │
├─────────────────────────────────────┤
│  Native Engine → Driver → Database  │
│  DuckDB Engine → ATTACH → 联邦查询   │
│  Stream Engine → 流式处理            │
└─────────────────────────────────────┘
    │
    ▼
Result (Arrow Format) ←── (逐层返回) ←──
```

### DuckDB 联邦查询流程 🔥

```
DBI::query(sql, ExecutionMode::DuckDB)
    │
    ▼
DuckDBEngine.execute(sql)
    │ 1. 检查外部数据库注册
    │ 2. 执行 ATTACH 命令
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
    ▼
Result (Arrow RecordBatch)
```

## 扩展点

### 1. 添加新数据库支持

步骤：

1. 在 `driver/native/` 添加实现文件
2. 实现 `Database` trait
3. 在 `lib.rs` 注册驱动

```rust
// driver/native/mongodb.rs
pub struct MongoDbDriver;

#[async_trait]
impl Database for MongoDbDriver {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        // 实现查询逻辑
    }
}

// lib.rs
fn register_drivers() {
    DriverRegistry::register(MongoDbDriverFactory);
}
```

### 2. 添加 WASM 插件支持

RdataStation 使用 Extism 1.21 作为 WASM 插件运行时，支持热插拔插件扩展。

插件架构：

```
┌─────────────────────────────────────────┐
│  Rust Core                              │
│  ┌───────────────────────────────────┐  │
│  │  ExtismPluginManager              │  │
│  │  ├── load_plugin()                │  │
│  │  ├── call_plugin()                │  │
│  │  ├── unload_plugin()              │  │
│  │  └── list_plugins()               │  │
│  └───────────────────────────────────┘  │
└──────────────┬──────────────────────────┘
               │ WASM 调用
┌──────────────▼──────────────────────────┐
│  WASM Plugin (.wasm)                    │
│  ├── Python 插件 (wasi-python)          │
│  ├── 自定义数据库驱动                   │
│  └── 数据分析工具                       │
└─────────────────────────────────────────┘
```

插件开发步骤：

1. 编写 WASM 插件（支持 Rust/Python/C 等）
2. 编译为 `.wasm` 文件
3. 通过 `ExtismPluginManager::load_plugin()` 加载
4. 通过 `call_plugin()` 调用插件函数

### 3. 添加 JDBC 数据库支持

JDBC 驱动通过 JVM 桥接支持 JDBC 兼容的数据库：

```
Rust Core → JDBC Driver → JVM → JDBC Driver (.jar)
```

支持的数据库：

- Oracle
- SQL Server
- DB2
- 其他 JDBC 兼容数据库

### 4. 添加新命令

步骤：

1. 在 `adapters/tauri/command.rs` 添加命令函数
2. 在 `lib.rs` 注册命令

```rust
// command.rs
#[tauri::command]
pub async fn my_command(input: MyInput) -> Result<MyOutput, String> {
    // 实现
}

// lib.rs
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    my_command,
])
```

### 5. 添加新服务

步骤：

1. 在 `core/services/` 创建服务文件
2. 在 `core/mod.rs` 导出

```rust
// services/my_service.rs
pub struct MyService;

impl MyService {
    pub async fn do_something(&self) -> Result<(), CoreError> {
        // 实现
    }
}

// core/mod.rs
pub use services::MyService;
```

## 性能考虑

### 1. 连接池

```rust
// driver/native/pool.rs
pub struct DbPool {
    inner: Pool<Postgres>, // sqlx Pool
}

impl DbPool {
    /// 获取连接
    pub async fn acquire(&self) -> Result<PoolConnection, CoreError>;

    /// 连接池状态
    pub fn status(&self) -> PoolStatus;
}
```

### 2. 缓存策略

```rust
// 元数据缓存
pub struct MetadataCache {
    l1: Arc<RwLock<HashMap<String, CachedValue>>>, // 内存缓存
    l2: Arc<dyn CacheStore>, // IndexedDB / SQLite
}

impl MetadataCache {
    pub async fn get(&self, key: &str) -> Option<CachedValue>;
    pub async fn set(&self, key: &str, value: CachedValue, ttl: Duration);
}
```

### 3. 异步处理

- 使用 `tokio` 作为异步运行时
- 所有 I/O 操作均为异步
- 使用 `Arc` 共享状态，避免克隆

## 安全考虑

### 1. SQL 注入防护

- 使用参数化查询（sqlx）
- 禁止字符串拼接 SQL
- 输入校验在 Adapter 层

### 2. 连接安全

- 支持 SSL/TLS 连接
- 密码加密存储
- SSH 隧道支持

### 3. 错误处理

- 不向客户端暴露内部错误细节
- 错误日志记录
- 敏感信息脱敏

## 监控与日志

### 日志级别

```rust
// error! - 系统错误，需要处理
// warn!  - 警告，可能影响功能
// info!  - 重要操作记录
// debug! - 调试信息
// trace! - 详细跟踪
```

### 性能指标

- SQL 执行时间
- 连接池使用率
- 缓存命中率
- 内存使用量
