pub mod auto_register;
pub mod connection;
pub mod factory;
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
