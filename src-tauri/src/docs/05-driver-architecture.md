# 驱动架构

> 版本：v2.0
> 最后更新：2026-05-09
> 状态：✅ 实际代码对齐

## 概述

RdataStation 使用 **Trait-based + Registry** 的驱动架构，支持多种数据库类型。驱动层负责数据库连接创建、SQL 执行、元数据查询和 Schema 浏览。

当前阶段锁定 4 种内置数据库，架构稳定后通过插件机制（JDBC/ODBC/WASM/ADBC）扩展。

## 核心架构

```
┌──────────────────────────────────────────────────────┐
│                   Tauri Commands                      │
│         (commands/connection_commands.rs)             │
└───────────────────────┬──────────────────────────────┘
                        │
┌───────────────────────▼──────────────────────────────┐
│              ConnectionService                        │
│         (core/services/connection_service.rs)         │
│  - create_database() [P0: 硬编码 4 种匹配]            │
│  - connect_by_url()                                   │
└───────────────────────┬──────────────────────────────┘
                        │
┌───────────────────────▼──────────────────────────────┐
│              DataSourceRouter                         │
│           (core/datasource/router.rs)                 │
│  - route(config) → DriverRegistry::get(id)           │
│  - list_registered_drivers()                         │
└───────────────────────┬──────────────────────────────┘
                        │
┌───────────────────────▼──────────────────────────────┐
│              DriverRegistry                           │
│          (core/driver/registry.rs)                    │
│  OnceLock<RwLock<HashMap<id, Arc<dyn DriverFactory>>>> │
│  - register(factory)                                  │
│  - get(id) → Option<Arc<dyn DriverFactory>>          │
│  - unregister(id)                                     │
└───────────────────────┬──────────────────────────────┘
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
   ┌──────────┐  ┌──────────┐  ┌──────────┐
   │  MySQL   │  │PostgreSQL│  │  SQLite  │  ...
   │ Factory  │  │ Factory  │  │ Factory  │
   └────┬─────┘  └────┬─────┘  └────┬─────┘
        │             │             │
        ▼             ▼             ▼
   ┌──────────┐  ┌──────────┐  ┌──────────┐
   │MySqlDB   │  │PostgresDB│  │SqliteDB  │
   │(sqlx)    │  │(sqlx)    │  │(rusqlite)│
   └──────────┘  └──────────┘  └──────────┘
        │             │             │
        └─────────────┼─────────────┘
                      │
               impl Database
              (driver/traits.rs)
```

## 核心 Trait

### Database Trait（实际代码）

**路径**: [traits.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/traits.rs)

```rust
#[async_trait::async_trait]
pub trait Database: Send + Sync {
    /* ===== 核心查询能力 ===== */

    /// 执行查询
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError>;

    /// 执行参数化查询（防止 SQL 注入）
    async fn query_with_params(&self, sql: &str, params: Vec<Value>) -> Result<QueryResult, CoreError>;

    /// 执行可取消的查询
    async fn query_with_cancel(
        &self, sql: &str, cancel_token: CancellationToken,
    ) -> Result<QueryResult, CoreError>;

    /// 开始事务
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, CoreError>;

    /// 获取数据源元数据
    fn meta(&self) -> DataSourceMeta;

    /* ===== 对象树能力（Schema 浏览） ===== */

    /// 列举数据库 / catalog
    async fn list_databases(&self) -> Result<Vec<String>, CoreError> { Ok(vec![]) }

    /// 列举 schema
    async fn list_schemas(&self, db: &str) -> Result<Vec<String>, CoreError> { Ok(vec![]) }

    /// 列举表 / 视图
    async fn list_tables(&self, db: &str, schema: Option<&str>) -> Result<Vec<SchemaObject>, CoreError> { Ok(vec![]) }

    /// 列举列
    async fn list_columns(&self, db: &str, schema: Option<&str>, table: &str) -> Result<Vec<SchemaObject>, CoreError> { Ok(vec![]) }

    /* ===== 联邦查询能力 ===== */

    async fn register_external_database(&self, name: &str, driver: &str, conn_str: &str) -> Result<(), CoreError>;
    async fn create_external_table(&self, ...) -> Result<(), CoreError>;
}
```

**关键设计**：metadata 方法有默认空实现，每个驱动按需覆盖。

### Transaction Trait

```rust
#[async_trait::async_trait]
pub trait Transaction: Send + Sync {
    async fn query(&mut self, sql: &str) -> Result<QueryResult, CoreError>;
    async fn commit(&mut self) -> Result<(), CoreError>;
    async fn rollback(&mut self) -> Result<(), CoreError>;
}
```

### DbPool Trait

```rust
#[async_trait::async_trait]
pub trait DbPool: Send + Sync {
    async fn acquire(&self) -> Result<Box<dyn Database + Send + Sync>, CoreError>;
    async fn close(&self) -> Result<(), CoreError>;
    fn is_closed(&self) -> bool;
    fn status(&self) -> PoolStatus;
}
```

### DriverFactory Trait（实际代码）

**路径**: [registry.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/registry.rs#L637)

```rust
pub trait DriverFactory: Send + Sync {
    /// 获取驱动描述符（用于前端渲染表单）
    fn descriptor(&self) -> DriverDescriptor;

    /// 创建数据库连接
    fn create(
        &self,
        config: ConnectionConfig,
    ) -> Pin<Box<dyn Future<Output = Result<DynDatabase, CoreError>> + Send>>;
}
```

> ⚠️ **与旧版文档的差异**：当前 `DriverFactory` 没有 `id()` 和 `box_clone()` 方法。ID 从 `descriptor().id` 获取。

## Schema 对象模型

### SchemaObject（实际代码）

**路径**: [traits.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/traits.rs#L22)

```rust
pub struct SchemaObject {
    pub name: String,
    pub kind: SchemaObjectKind,
    pub children: Option<Vec<SchemaObject>>,  // None = 未加载（懒加载）
}

pub enum SchemaObjectKind {
    Database,
    Schema,
    Table,
    View,
    Column,
}
```

> ⚠️ **已知局限（P0-4）**：`SchemaObject` 仅包含 `name` / `kind` / `children`，缺少：
> - 列注释（comment）
> - 列类型（data_type）
> - 是否可空（nullable）
> - 主键信息（is_primary_key）
> - 表行数估算（row_count）
>
> **改进方向**：引入 `NodeDetail` 结构体，在 `list_columns` 中返回完整元数据。

## DataSourceMeta（数据源能力描述）

```rust
pub struct DataSourceMeta {
    pub supports_transaction: bool,
    pub supports_streaming: bool,
    pub supports_arrow: bool,          // Arrow 格式（用于插件通信）
    pub supports_federated: bool,      // 联邦查询
    pub supports_concurrent_write: bool,
    pub is_in_memory: bool,
}
```

| 能力         | MySQL | PostgreSQL | SQLite | DuckDB |
| ------------ | ----- | ---------- | ------ | ------ |
| 事务         | ✅    | ✅         | ✅     | ✅     |
| 流式查询     | ✅    | ✅         | ❌     | ✅     |
| Arrow 格式   | ❌    | ❌         | ❌     | ✅     |
| 联邦查询     | ❌    | ❌         | ❌     | ✅     |
| 并发写入     | ✅    | ✅         | ❌     | ✅     |
| 内存数据库   | ❌    | ❌         | ❌     | ❌     |

## DriverRegistry（驱动注册表）

**路径**: [registry.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/registry.rs#L656)

```rust
// 全局存储：OnceLock + RwLock + HashMap
static DRIVER_REGISTRY: OnceLock<RwLock<HashMap<String, Arc<dyn DriverFactory>>>> = OnceLock::new();

impl DriverRegistry {
    /// 注册驱动工厂
    pub fn register<F: DriverFactory + 'static>(factory: F);

    /// 根据 ID 获取驱动工厂
    pub fn get(id: &str) -> Option<Arc<dyn DriverFactory>>;

    /// 获取所有已注册驱动的描述符
    pub fn all_descriptors() -> Vec<DriverDescriptor>;

    /// 获取所有已注册的驱动 ID
    pub fn all_driver_ids() -> Vec<String>;

    /// 检查驱动是否已注册
    pub fn is_registered(id: &str) -> bool;

    /// 注销驱动（支持热卸载）
    pub fn unregister(id: &str) -> bool;

    /// 清空所有注册的驱动
    pub fn clear();
}
```

**设计要点**：
- 使用 `OnceLock` 保证全局单例
- `RwLock` 支持并发读（获取工厂）、排他写（注册/注销）
- `Arc<dyn DriverFactory>` 允许工厂在多个位置共享引用
- `unregister()` 支持热卸载（未来插件系统基础）

## 驱动注册流程

### 启动注册

**路径**: [lib.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/lib.rs#L35)

```rust
fn register_drivers() {
    use core::driver::auto_register::AutoDriverRegistrar;
    AutoDriverRegistrar::auto_register();
}
```

**路径**: [auto_register.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/auto_register.rs#L22)

```rust
impl AutoDriverRegistrar {
    pub fn register_builtin_drivers() {
        DriverRegistry::register(MySqlDriverFactory);
        DriverRegistry::register(PostgresDriverFactory);
        DriverRegistry::register(SqliteDriverFactory);
        DriverRegistry::register(DuckDbDriverFactory);
    }

    pub fn auto_register() {
        Self::register_builtin_drivers();
        // 预留：配置文件驱动注册
        // 预留：自动扫描驱动注册（WASM 插件等）
    }
}
```

### ⚠️ 已知问题：双重注册（P0-1）

存在两套并行的驱动注册机制：

| 注册方式                         | 存储位置                     | 文件             |
| -------------------------------- | ---------------------------- | ---------------- |
| `DriverRegistry::register()`     | `OnceLock<RwLock<HashMap>>`  | registry.rs      |
| `DRIVER_FACTORY_MANAGER` (Lazy)  | `DriverFactoryManager`       | factory.rs       |

**影响**：`connection_service.rs:create_database()` 硬编码匹配，绕过了两套注册机制。

**改进方向**：统一到 `DriverRegistry`，移除 `DRIVER_FACTORY_MANAGER`，`create_database()` 通过 `DataSourceRouter::route()` 或 `DriverRegistry::get()` 动态创建。

## 连接配置

### ConnectionConfig（实际代码）

**路径**: [registry.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/registry.rs#L18)

```rust
pub struct ConnectionConfig {
    pub driver: String,              // "mysql" | "postgres" | "sqlite" | "duckdb"
    pub name: Option<String>,        // 连接显示名称
    pub host: Option<String>,        // 主机地址
    pub port: Option<u16>,           // 端口
    pub database: Option<String>,    // 数据库名
    pub username: Option<String>,    // 用户名
    pub password: Option<String>,    // 密码
    pub file_path: Option<String>,   // 文件路径（SQLite/DuckDB）
    pub connection_method: ConnectionMethod,  // Direct/SSL/SSH/HTTP/SOCKS
    pub options: HashMap<String, String>,     // 额外连接选项
}
```

### to_url() 硬编码问题（P0-3）

**路径**: [registry.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/registry.rs#L117)

```rust
pub fn to_url(&self) -> Result<String, String> {
    match self.driver.as_str() {
        "mysql" => self.build_mysql_url(),
        "postgres" => self.build_postgres_url(),
        "sqlite" => self.build_sqlite_url(),
        "duckdb" => self.build_duckdb_url(),
        _ => Err(format!("Unsupported driver: {}", self.driver)),
    }
}
```

**改进方向**：`to_url()` 应该通过 `DriverRegistry::get(id)` 获取工厂后，由工厂提供 URL 构建逻辑，或由 `DriverDescriptor` 携带 `url_template`。

## DriverDescriptor（驱动描述符）

```rust
pub struct DriverDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_port: Option<u16>,
    pub require_database: bool,
    pub require_file: bool,
    pub supports_ssl: bool,
    pub supports_ssh_tunnel: bool,
    pub supports_http_proxy: bool,
    pub supports_socks_proxy: bool,
    pub fields: Vec<DriverField>,           // 前端表单字段
    pub extra_options: Vec<DriverOption>,    // 额外配置选项
}
```

> ⚠️ **待增强（P3）**：缺少 `target_database`、`driver_kind`（native/jdbc/odbc/wasm）、`url_template` 字段。

## 四种内置驱动

### MySQL (sqlx)

**路径**: [native/mysql.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/native/mysql.rs)

- 驱动: `sqlx::mysql::MySqlPool`
- 默认端口: 3306
- 支持: SSL、SSH 隧道、HTTP/SOCKS 代理
- 连接方式: `mysql://user:pass@host:port/db`

### PostgreSQL (sqlx)

**路径**: [native/postgres.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/native/postgres.rs)

- 驱动: `sqlx::postgres::PgPool`
- 默认端口: 5432
- 支持: SSL、SSH 隧道（完整实现）、HTTP/SOCKS 代理
- 连接方式: `postgres://user:pass@host:port/db`

### SQLite (rusqlite)

**路径**: [native/sqlite.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/native/sqlite.rs)

- 驱动: `rusqlite::Connection`
- 文件型数据库
- 支持 WAL 模式

### DuckDB (duckdb-rs)

**路径**: [native/duckdb.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/native/duckdb.rs)

- 驱动: `duckdb::Connection`
- 文件型 / 内存型数据库
- 支持: Arrow 格式、联邦查询

## 数据源路由层

**路径**: [router.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/datasource/router.rs)

```rust
pub struct DataSourceRouter;

impl DataSourceRouter {
    /// 根据驱动配置创建数据库连接
    pub async fn route(config: ConnectionConfig) -> Result<DynDatabase, CoreError> {
        let factory = DriverRegistry::get(&config.driver)?;
        factory.create(config).await
    }
}
```

**职责**：
- 根据 `config.driver` 从 `DriverRegistry` 查找工厂
- 调用工厂创建连接
- **不直接实例化任何数据库驱动**

> ⚠️ 当前 `ConnectionService::create_database()` 绕过了 Router，直接硬编码匹配。应统一走 Router。

## 驱动目录结构

```
core/driver/
├── mod.rs              # 模块导出
├── traits.rs           # Database / Transaction / DbPool trait 定义
├── registry.rs         # DriverRegistry + ConnectionConfig + DriverFactory trait
├── factory.rs          # DriverFactoryManager + 4 个工厂实现 (⚠️ 重复注册)
├── auto_register.rs    # AutoDriverRegistrar（启动时注册 4 个驱动）
├── manager.rs          # DriverManager（全局驱动状态管理）
├── metadata.rs         # DriverMetadata + DriverType + DriverIcon
├── loader.rs           # DriverLoader + 发现机制（Builtin/Wasm/JDBC）
├── utils.rs            # 工具函数
├── smart_pool.rs       # SmartPool 智能连接池
├── native/             # 原生驱动实现
│   ├── mod.rs
│   ├── mysql.rs        # MySqlDatabase
│   ├── mysql_pool.rs
│   ├── postgres.rs     # PostgresDatabase
│   ├── sqlite.rs       # SqliteDatabase
│   ├── sqlite_pool.rs
│   ├── duckdb.rs       # DuckDbDatabase
│   └── duckdb_pool.rs
├── jdbc/               # JDBC 桥接（骨架，待 Go Sidecar 实现）
├── wasm/               # WASM 插件驱动（骨架）
└── tests/              # 驱动测试
```

## 连接池架构

### 用户连接池（业务库）

每种数据库有独立的 Pool 实现：

| 数据库     | Pool 实现     | 文件                    |
| ---------- | ------------- | ----------------------- |
| MySQL      | sqlx MySqlPool | native/mysql_pool.rs    |
| PostgreSQL | sqlx PgPool    | native/sqlite_pool.rs   |
| SQLite     | 自定义连接池  | native/sqlite_pool.rs   |
| DuckDB     | 自定义连接池  | native/duckdb_pool.rs   |

所有 Pool 实现 `DbPool` trait，通过 `acquire()` 返回 `Box<dyn Database>`。

### 系统连接池（SQLite + DuckDB 双层）

参见 [06-存储架构](./06-storage-architecture.md)。

### SmartPool（智能连接池）

**路径**: [smart_pool.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/smart_pool.rs)

包装标准 `DbPool`，提供：
- 动态扩缩容
- 健康检查
- 负载均衡

## P0 问题总结

| 编号 | 问题                             | 影响                   | 改进方向                           |
| ---- | -------------------------------- | ---------------------- | ---------------------------------- |
| P0-1 | `DRIVER_FACTORY_MANAGER` 重复注册 | 维护两套注册表         | 移除，统一到 DriverRegistry        |
| P0-2 | `create_database()` 硬编码匹配   | 新增数据库需改多处代码 | 通过 DriverRegistry 动态创建       |
| P0-3 | `to_url()` 硬编码匹配            | 同上                   | 由 DriverFactory/Descriptor 提供   |
| P0-4 | `SchemaObject` 缺少列详情        | 无法展示列注释/类型    | 引入 NodeDetail 结构体             |

## 后续演进

### Phase 1: 架构归一化

- 移除 `DRIVER_FACTORY_MANAGER`，统一到 `DriverRegistry`
- `create_database()` 改为通过 `DataSourceRouter::route()` 创建
- `to_url()` 改为通过 `DriverDescriptor` 的 `url_template` 构建

### Phase 2: MetadataBrowser Trait

```
trait MetadataBrowser {
    fn get_databases(&self) -> Vec<NodeInfo>;
    fn get_schemas(&self, db: &str) -> Vec<NodeInfo>;
    fn get_tables(&self, db: &str, schema: &str) -> Vec<NodeInfo>;
    fn get_columns(&self, db: &str, schema: &str, table: &str) -> Vec<NodeDetail>;
}
```

### Phase 3: 多驱动支持

- 同一数据库可注册多个驱动（如 MySQL via JDBC / ADBC）
- `DriverDescriptor` 增加 `driver_kind` 和 `target_database` 字段
- 前端展示驱动选择器

### Phase 4: 插件热插拔

- `DriverRegistry::unregister()` 已就绪
- WASM 插件通过 `wasmtime` 加载
- JDBC 通过 Go Sidecar + gRPC 桥接