pub mod traits;
pub mod registry;
pub mod factory;
pub mod manager;
pub mod metadata;
pub mod loader;
pub mod utils;
pub mod native;
pub mod jdbc;
pub mod wasm;
pub mod smart_pool;

#[cfg(test)]
mod tests;

// 重新导出核心类型
pub use traits::{Database, Transaction, DbPool, DataSourceMeta, SchemaObject, SchemaObjectKind, DynDatabase, PoolStatus};
pub use registry::{DriverRegistry, ConnectionConfig as DriverConnectionConfig, DriverDescriptor, DriverFactory};
pub use factory::{DriverFactoryManager, DRIVER_FACTORY_MANAGER, MySqlDriverFactory, PostgresDriverFactory, SqliteDriverFactory, DuckDbDriverFactory};
pub use manager::{DriverManager, DRIVER_MANAGER, get_driver_manager, init_driver_manager, DriverStatus, DriverInfo};
pub use metadata::{DriverMetadata, DriverType, DriverIcon, DriverFormField};
pub use loader::{DriverLoader, BuiltinDriverDiscovery, WasmDriverDiscovery, JdbcDriverDiscovery};
pub use utils::{build_connection_url, validate_driver_config, parse_driver_id};
pub use smart_pool::{SmartPool, SmartPoolConfig, SmartPoolBuilder, PoolStats};

// 重新导出自动注册模块
pub mod auto_register;
pub use auto_register::{AutoDriverRegistrar};
