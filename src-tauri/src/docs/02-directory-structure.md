# 目录结构

## 整体结构

```
src-tauri/src/
├── lib.rs                    # 主入口：模块导出、命令注册
├── main.rs                   # 程序启动入口
│
├── api/                      # API 层（前后端共享）
│   ├── mod.rs               # 模块导出
│   └── dto.rs               # 数据传输对象
│
├── core/                     # 核心业务层（无框架依赖）
│   ├── mod.rs               # 模块导出、依赖规则
│   ├── error.rs             # 错误定义
│   ├── macros.rs            # 宏定义
│   ├── models.rs            # 核心数据模型
│   ├── port_negotiation.rs  # 端口协商
│   ├── arrow.rs             # Arrow 格式支持
│   ├── stream.rs            # 流处理
│   │
│   ├── dbi/                 # 🔥 统一数据访问层（总调度）
│   │   ├── mod.rs           # 模块导出
│   │   ├── dbi.rs           # DBI 主接口
│   │   ├── session.rs       # 会话管理
│   │   ├── context.rs       # 查询上下文
│   │   └── engine/          # 执行引擎
│   │       ├── mod.rs       # 引擎模块导出
│   │       ├── driver_engine.rs  # 原生驱动引擎
│   │       ├── duckdb_engine.rs  # DuckDB 加速引擎
│   │       └── stream_engine.rs  # 流处理引擎
│   │
│   ├── connection/          # 连接管理
│   │   ├── mod.rs
│   │   ├── config.rs        # 连接配置
│   │   ├── connector.rs     # 连接器
│   │   ├── factory.rs       # 连接工厂
│   │   └── stream.rs        # 连接流
│   │
│   ├── datasource/          # 数据源路由层
│   │   ├── mod.rs           # 模块导出
│   │   └── router.rs        # 数据源路由器
│   │
│   ├── driver/              # 驱动层（统一）
│   │   ├── mod.rs
│   │   ├── traits.rs        # Database/Transaction trait
│   │   ├── registry.rs      # 驱动注册表
│   │   ├── factory.rs       # 驱动工厂
│   │   ├── manager.rs       # 驱动管理器
│   │   ├── metadata.rs      # 驱动元数据
│   │   ├── loader.rs        # 驱动加载器
│   │   ├── auto_register.rs # 自动注册
│   │   ├── utils.rs         # 工具函数
│   │   │
│   │   ├── native/          # 原生驱动实现
│   │   │   ├── mod.rs
│   │   │   ├── mysql.rs     # MySQL 驱动
│   │   │   ├── postgres.rs  # PostgreSQL 驱动
│   │   │   ├── sqlite.rs    # SQLite 驱动
│   │   │   └── duckdb.rs    # DuckDB 驱动
│   │   │
│   │   ├── jdbc/            # JDBC 桥接驱动
│   │   │   ├── mod.rs
│   │   │   ├── driver.rs    # JDBC 驱动
│   │   │   ├── executor.rs  # SQL 执行器
│   │   │   ├── jvm_manager.rs # JVM 管理
│   │   │   └── connection.rs # JDBC 连接
│   │   │
│   │   └── wasm/            # WASM 驱动
│   │       ├── mod.rs
│   │       ├── driver.rs    # WASM 驱动
│   │       ├── adapter.rs   # WASM 适配器
│   │       └── pool.rs      # WASM 连接池
│   │
│   ├── persistence/         # 持久化层
│   │   ├── mod.rs
│   │   ├── connection_store.rs
│   │   ├── history_store.rs
│   │   ├── project_db.rs
│   │   ├── project_store.rs
│   │   └── project_connection_store.rs
│   │
│   ├── project/             # 项目管理
│   │   ├── mod.rs
│   │   ├── models.rs
│   │   └── store.rs
│   │
│   ├── services/            # 业务服务
│   │   ├── mod.rs
│   │   ├── connection_manager.rs
│   │   ├── connection_service.rs
│   │   └── sql_service.rs
│   │
│   └── cache/               # 缓存层
│       ├── mod.rs
│       ├── cache_manager.rs
│       ├── lru_cache.rs
│       ├── metadata_cache.rs
│       └── query_cache.rs
│
├── adapters/                 # 适配器层
│   ├── mod.rs
│   ├── tauri/               # Tauri 适配器
│   │   ├── mod.rs
│   │   ├── command.rs       # Tauri 命令实现
│   │   ├── event.rs         # 事件系统
│   │   ├── state.rs         # 状态管理
│   │   └── stream.rs        # 流处理
│   │
│   └── wasm/                # WASM 插件适配器
│       ├── mod.rs
│       ├── extism.rs        # Extism 集成
│       ├── plugin_manager.rs # 插件管理器
│       └── api.rs           # 插件 API
│
└── commands/                 # 命令模块（按功能组织）
    ├── mod.rs
    ├── connection_commands.rs
    ├── driver_commands.rs
    ├── navigator_commands.rs
    ├── port_commands.rs
    ├── project_commands.rs
    ├── project_store_commands.rs
    └── sql_commands.rs
```

## 详细说明

### 1. lib.rs

**职责**：
- 模块导出
- Tauri 命令注册
- 驱动注册

**关键代码**：
```rust
pub mod api;
pub mod core;
pub mod adapters;
pub mod commands;

// 驱动注册
fn register_drivers() {
    use core::driver::DriverRegistry;
    DriverRegistry::register(MySqlDriverFactory);
    DriverRegistry::register(PostgresDriverFactory);
}

// Tauri 命令注册
.invoke_handler(tauri::generate_handler![
    connect_database,
    execute_sql,
    // ... 其他命令
])
```

### 2. api/ 目录

**职责**：定义前后端共享的数据类型

**文件说明**：

| 文件 | 职责 | 示例 |
|------|------|------|
| `mod.rs` | 模块导出 | `pub use dto::*;` |
| `dto.rs` | DTO 定义 | `QueryResult`, `Row`, `Value` |

**设计原则**：
- DTO 与内部模型分离
- 使用 serde 支持序列化
- 保持向后兼容

### 3. core/ 目录

核心业务逻辑，**无外部框架依赖**。

#### 3.1 dbi/ 目录 🔥

**职责**：统一数据访问层（总调度）

| 文件 | 职责 | 关键类型 |
|------|------|----------|
| `mod.rs` | 模块导出 | `DBI`, `Session`, `QueryContext` |
| `dbi.rs` | DBI 主接口 | `DBI` |
| `session.rs` | 会话管理 | `Session`, `SessionConfig`, `SessionMode` |
| `context.rs` | 查询上下文 | `QueryContext`, `ExecutionContext` |
| `engine/mod.rs` | 引擎模块导出 | `ExecutionMode`, `QueryRouter` |
| `engine/driver_engine.rs` | 原生驱动引擎 | `DriverEngine` |
| `engine/duckdb_engine.rs` | DuckDB 加速引擎 | `DuckDBEngine`, `ExternalConnection` |
| `engine/stream_engine.rs` | 流处理引擎 | `StreamEngine` |

**设计原则**：
- 所有数据操作通过 DBI::query() / execute() 统一入口
- QueryRouter 智能推荐执行模式
- 写操作必须走原生驱动，读操作可路由到 DuckDB 加速
- 支持会话级和持久化结果集管理

#### 3.2 connection/ 目录

**职责**：数据库连接管理

| 文件 | 职责 | 关键类型 |
|------|------|----------|
| `config.rs` | 连接配置 | `ConnectionConfig`, `SslConfig`, `SshConfig` |
| `connector.rs` | 连接器 | `Connector` trait |
| `factory.rs` | 连接工厂 | `ConnectionFactory` |
| `stream.rs` | 连接流 | `ConnectionStream` |

#### 3.3 datasource/ 目录

**职责**：数据源路由层（注册/路由，不存放具体驱动实现）

| 文件 | 职责 | 关键类型 |
|------|------|----------|
| `mod.rs` | 模块导出 | `DataSourceRouter` |
| `router.rs` | 数据源路由 | `DataSourceRouter` |

**设计原则**：
- 只负责路由和注册，不存放具体驱动实现
- 具体驱动实现在 `driver/native/`、`driver/jdbc/`、`driver/wasm/` 中

#### 3.4 driver/ 目录

**职责**：驱动层（统一）

| 文件 | 职责 | 关键类型 |
|------|------|----------|
| `traits.rs` | Trait 定义 | `Database`, `Transaction`, `DbPool` |
| `registry.rs` | 驱动注册 | `DriverRegistry`, `DriverFactory` |
| `factory.rs` | 驱动工厂 | `DriverFactoryManager` |
| `manager.rs` | 驱动管理 | `DriverManager` |
| `metadata.rs` | 驱动元数据 | `DriverMetadata`, `DriverType` |
| `loader.rs` | 驱动加载 | `DriverLoader` |
| `auto_register.rs` | 自动注册 | `AutoDriverRegistrar` |
| `utils.rs` | 工具函数 | `build_connection_url`, `validate_driver_config` |

**native/ 子目录**：

| 文件 | 职责 | 关键类型 |
|------|------|----------|
| `mysql.rs` | MySQL 实现 | `MySqlDriver`, `MySqlPool` |
| `postgres.rs` | PostgreSQL 实现 | `PostgresDriver`, `PostgresPool` |
| `sqlite.rs` | SQLite 实现 | `SqliteDriver` |
| `duckdb.rs` | DuckDB 实现 | `DuckDbDriver` |

**jdbc/ 子目录**：

| 文件 | 职责 | 关键类型 |
|------|------|----------|
| `driver.rs` | JDBC 驱动 | `JdbcDriver` |
| `executor.rs` | SQL 执行器 | `JdbcExecutor` |
| `jvm_manager.rs` | JVM 管理 | `JvmManager` |
| `connection.rs` | JDBC 连接 | `JdbcConnection` |

**wasm/ 子目录**：

| 文件 | 职责 | 关键类型 |
|------|------|----------|
| `driver.rs` | WASM 驱动 | `WasmDriver` |
| `adapter.rs` | WASM 适配器 | `WasmAdapter` |
| `pool.rs` | WASM 连接池 | `WasmPool` |

**关键 trait**：
```rust
#[async_trait]
pub trait Database: Send + Sync {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError>;
    async fn execute(&self, sql: &str) -> Result<ExecuteResult, CoreError>;
    fn meta(&self) -> DataSourceMeta;
}
```

#### 3.5 persistence/ 目录

**职责**：数据持久化

| 文件 | 职责 | 存储内容 |
|------|------|----------|
| `connection_store.rs` | 连接存储 | 最近连接、连接配置 |
| `history_store.rs` | 历史存储 | SQL 执行历史 |
| `project_db.rs` | 项目数据库管理器 | `.RSMETA/project.db` (SQLite)、`.RSMETA/analytics.duckdb` |
| `project_store.rs` | 项目存储 | 项目连接、SQL历史、设置 |
| `project_connection_store.rs` | 项目连接存储 | 使用 ProjectDatabaseManager 的 SQLite 池 |

**项目目录结构**：

```
E:\RDPRJ\C5\.RSMETA\
├── analytics.duckdb          # DuckDB 分析数据库
├── project.db                # SQLite 主数据库（连接信息、SQL历史、项目设置等）
├── project.json              # 项目基本信息
├── config\
│   └── settings.json         # 项目配置
└── project_metadata\         # 各连接的元数据 SQLite 文件
    ├── conn_mysql_01.db      # MySQL连接01的元数据
    ├── conn_pg_02.db         # PostgreSQL连接02的元数据
    └── ...
```

**设计原则**：
- 所有项目数据统一存储在 `.RSMETA/` 目录下
- `project.db` 是唯一的 SQLite 主数据库
- `ProjectConnectionStore` 使用 `ProjectDatabaseManager` 的 SQLite 池，不独立创建数据库
- `project_metadata/` 为每个连接存储独立的元数据 SQLite 文件

#### 3.6 project/ 目录

**职责**：项目管理

| 文件 | 职责 | 关键类型 |
|------|------|----------|
| `models.rs` | 项目模型 | `Project`, `ProjectConfig` |
| `store.rs` | 项目存储 | `ProjectStore` |

#### 3.7 services/ 目录

**职责**：业务服务

| 文件 | 职责 | 关键类型 |
|------|------|----------|
| `connection_manager.rs` | 连接管理器 | `ConnectionManager` |
| `connection_service.rs` | 连接服务 | `ConnectionService` |
| `sql_service.rs` | SQL 服务 | `SqlService` |

**服务职责**：
- `ConnectionManager`：连接生命周期管理（创建、复用、关闭）
- `ConnectionService`：连接业务逻辑（连接、断开、测试）
- `SqlService`：SQL 执行业务（查询、事务、历史）

#### 3.8 cache/ 目录

**职责**：缓存管理

| 文件 | 职责 | 关键类型 |
|------|------|----------|
| `cache_manager.rs` | 缓存管理器 | `CacheManager` |
| `lru_cache.rs` | LRU 缓存 | `LruCache` |
| `metadata_cache.rs` | 元数据缓存 | `MetadataCache` |
| `query_cache.rs` | 查询缓存 | `QueryCache` |

### 4. adapters/ 目录

**职责**：框架适配

#### 4.1 tauri/ 目录

| 文件 | 职责 | 关键函数 |
|------|------|----------|
| `command.rs` | Tauri 命令 | `execute_sql`, `connect_database` |
| `event.rs` | 事件系统 | `emit_event` |
| `state.rs` | 状态管理 | `AppState` |
| `stream.rs` | 流处理 | `StreamHandler` |

**命令分类**：
- 连接命令：`connect_database`, `close_connection`
- SQL 命令：`execute_sql`, `execute_transaction`
- 元数据命令：`list_databases`, `list_tables`, `list_columns`
- 项目命令：`create_project`, `open_project`

#### 4.2 wasm/ 目录

| 文件 | 职责 | 关键类型 |
|------|------|----------|
| `extism.rs` | Extism 集成 | `ExtismPluginManager` |
| `plugin_manager.rs` | 插件管理器 | `PluginManager` |
| `api.rs` | 插件 API | `PluginApi` |

### 5. commands/ 目录

**职责**：按功能组织的命令模块

| 文件 | 职责 | 说明 |
|------|------|------|
| `connection_commands.rs` | 连接相关命令 | 连接、断开、测试 |
| `driver_commands.rs` | 驱动相关命令 | 驱动列表、注册 |
| `navigator_commands.rs` | 导航器命令 | 元数据查询 |
| `port_commands.rs` | 端口命令 | 端口协商 |
| `project_commands.rs` | 项目命令 | 项目创建、打开 |
| `project_store_commands.rs` | 项目存储命令 | 项目存储操作 |
| `sql_commands.rs` | SQL 命令 | SQL 执行、事务 |

**与 adapters/tauri/command.rs 的区别**：
- `adapters/tauri/command.rs`：Tauri 特定的命令实现
- `commands/`：通用的命令逻辑，可复用

## 文件组织原则

### 1. 单一职责

每个文件只负责一个明确的职责：
- ✅ `mysql.rs` 只负责 MySQL 实现
- ❌ 不要把 MySQL 和 PostgreSQL 放在同一个文件

### 2. 模块深度

控制模块嵌套深度：
- ✅ `core::services::connection_service`
- ❌ `core::services::connection::manager::service`

### 3. 测试组织

测试文件放在 `tests/` 子目录：
```
services/
├── mod.rs
├── connection_service.rs
└── tests/
    ├── mod.rs
    └── connection_service_tests.rs
```

### 4. 文档位置

文档放在 `docs/` 目录：
```
src/
├── docs/
│   ├── README.md
│   ├── 01-architecture-overview.md
│   └── ...
├── lib.rs
└── ...
```

## 命名规范

### 文件命名

| 类型 | 规范 | 示例 |
|------|------|------|
| 模块文件 | snake_case.rs | `connection_service.rs` |
| 测试文件 | {module}_tests.rs | `connection_service_tests.rs` |
| 文档文件 | {number}-{name}.md | `01-architecture-overview.md` |

### 目录命名

| 类型 | 规范 | 示例 |
|------|------|------|
| 模块目录 | snake_case | `connection_manager/` |
| 测试目录 | tests | `tests/` |
| 文档目录 | docs | `docs/` |

## 导入规范

### 导入顺序

```rust
// 1. 标准库
use std::sync::Arc;

// 2. 第三方库
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

// 3. 本地模块（绝对路径）
use crate::core::error::CoreError;
use crate::core::models::QueryResult;

// 4. 本地模块（相对路径）
use super::ConnectionConfig;
use super::pool::DbPool;
```

### 重新导出

在 `mod.rs` 中重新导出公共类型：
```rust
// core/mod.rs
pub use error::{CoreError, CoreResult};
pub use models::{QueryResult, Row, Value};
pub use services::{ConnectionManager, ConnectionService};
```

## 新增模块指南

### 添加新数据库驱动

1. 在 `driver/native/` 创建 `{db}.rs`
2. 实现 `Database` trait
3. 在 `driver/native/mod.rs` 导出
4. 在 `lib.rs` 注册驱动

### 添加新服务

1. 在 `services/` 创建 `{name}_service.rs`
2. 实现服务结构体
3. 在 `services/mod.rs` 导出
4. 在 `core/mod.rs` 重新导出

### 添加新命令

1. 在 `commands/` 创建 `{name}_commands.rs`
2. 实现命令函数
3. 在 `commands/mod.rs` 导出
4. 在 `lib.rs` 注册命令
