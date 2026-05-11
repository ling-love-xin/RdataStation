//! Driver 层 — 数据库驱动 trait 抽象 + 注册 + 连接管理
//!
//! ═══════════ 架构边界 ═══════════
//! Driver 层定位：数据库连接抽象层，位于 dbi 之下、native 实现之上。
//!
//!    commands ──► services ──► dbi ──► driver ──► native
//!                  (业务逻辑)   (引擎)   (本层)    (实现)
//!
//! ## 职责
//! - 定义 Database / Transaction / Stream 核心 trait（[traits]）
//! - 驱动注册与发现（[registry]、[loader]）
//! - 连接池抽象与管理（[smart_pool]、[connection]）
//! - SQL 安全工具函数（[utils::escape_sql_string]、[utils::quote_identifier]）
//!
//! ## 非职责（禁止事项）
//! - ❌ 不处理业务逻辑（属于 services）
//! - ❌ 不路由执行引擎（属于 dbi）
//! - ❌ 不直接返回给前端（属于 commands）
//! - ❌ 不在 trait 中定义与数据访问无关的方法
//!
//! ## 与 dbi 层的边界
//! - driver 提供 `Database` trait，dbi 通过 `DriverEngine` 调用 trait 方法
//! - driver 不感知 dbi 的执行模式和路由策略
//! - dbi 不直接调用 driver/native 的实现，始终通过 trait 接口
pub mod auto_register;
pub mod connection;
pub mod factory;
pub mod introspection;
pub mod jdbc;
pub mod loader;
pub mod manager;
pub mod metadata;
pub mod native;
pub mod registry;
pub mod router;
pub mod smart_pool;
pub mod traits;
pub mod utils;
pub mod wasm;

pub use auto_register::AutoDriverRegistrar;
pub use factory::{
    DuckDbDriverFactory, MySqlDriverFactory, PostgresDriverFactory, SqliteDriverFactory,
};
pub use introspection::{get_level, remove_level, set_level, IntrospectionLevel};
pub use loader::{BuiltinDriverDiscovery, DriverLoader, JdbcDriverDiscovery, WasmDriverDiscovery};
pub use manager::{
    get_driver_manager, init_driver_manager, DriverInfo, DriverManager, DriverStatus,
    DRIVER_MANAGER,
};
pub use metadata::{DriverFormField, DriverIcon, DriverMetadata, DriverType};
pub use registry::{
    DriverConnectionConfig, DriverDescriptor, DriverFactory, DriverKind, DriverRegistry,
};
pub use router::DataSourceRouter;
pub use smart_pool::{PoolStats, SmartPool, SmartPoolBuilder, SmartPoolConfig};
pub use traits::{
    ColumnDetail, DataSourceMeta, Database, DbPool, DynDatabase, MetadataBrowser, NodeDetail,
    NodeInfo, PoolStatus, SchemaObject, SchemaObjectKind, Transaction,
};
pub use utils::{build_connection_url, parse_driver_id, validate_driver_config};
