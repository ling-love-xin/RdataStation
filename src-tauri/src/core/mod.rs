//! Core 业务层
//!
//! 纯业务逻辑，无外部框架依赖。
//! 负责：
//! - 连接生命周期管理
//! - SQL 执行与事务
//! - 数据库驱动抽象
//! - 数据持久化
//! - 底层连接管理（SSL/SSH/Proxy）
//! - 多级缓存管理
//! - DBI 统一数据访问入口
//!
//! 注意：Core 不直接暴露给外部，需通过 Adapter 层（Tauri/CLI/HTTP）访问
//!
//! # 模块依赖规则
//!
//! ## 允许依赖：
//! - `models` → 无（基础层）
//! - `error`, `macros` → 无（基础层）
//! - `driver` → `error`, `macros`, `models`
//! - `connection` → `error`, `models`
//! - `datasource` → `driver`, `connection`, `error`, `models`
//! - `persistence` → `error`, `models`
//! - `project` → `error`, `models`, `persistence`
//! - `services` → `driver`, `persistence`, `connection`, `error`, `models`, `project`, `cache`
//! - `cache` → `error`, `models`
//! - `dbi` → `driver`, `error`, `models`, `stream`
//!
//! ## 禁止依赖：
//! - 任何 core 内部模块 → `api`（api 应依赖 core，而非相反）
//! - `driver` → `connection`（应通过 trait 解耦）
//! - `datasource` → `api`

pub mod connection;
pub mod datasource;
pub mod dbi;
pub mod driver;
pub mod models;
pub mod persistence;
pub mod performance;
pub mod port_negotiation;
pub mod project;
pub mod services;
pub mod cache;
pub mod utils;
pub mod migration;
pub mod insight;
pub mod scratchpad;

// 错误处理宏模块
#[macro_use]
pub mod macros;

pub mod error;
pub mod arrow;
pub mod stream;

// 重新导出常用错误类型
pub use error::{
    CommonError, ConnectionError, CoreError, CoreResult, DatabaseError, ErrorCategory,
    StorageError, TransactionState, common_err, conn_err, invalid_arg, not_supported, query_err,
    storage_err, timeout,
};

// 重新导出 Service 层（供 Adapters 使用）
pub use services::{ConnectionService, SqlService, ConnectionManager, ConnectionInfo, get_connection_manager};

// 重新导出驱动层
pub use driver::{
    Database, Transaction, DbPool, PoolStatus, DynDatabase,
    DataSourceMeta, SchemaObject, SchemaObjectKind,
    DriverConnectionConfig, DriverDescriptor,
    DriverFactory, DriverRegistry,
    // 驱动配置和自动注册
    AutoDriverRegistrar,
};

// 重新导出驱动注册表函数
pub use driver::registry::{get_all_drivers, get_driver};

// 重新导出数据源路由层
pub use datasource::router::DataSourceRouter;

// 重新导出连接层
pub use connection::{
    ConnectionConfig, ConnectionMethod, SshConfig, ProxyConfig, SslConfig,
    config::TlsVersion,
    Connection, Connector, ConnectionFactory, ConnectionStream,
};

// 重新导出模型层
pub use models::{QueryResult, Row, Value};

// 重新导出项目层
pub use project::{
    Project, ProjectConfig, ProjectInfo, ProjectPath, ProjectStatus,
    Version, Versioned, VersionInfo,
    ConnectionRef, QueryRef,
    ProjectStore, ProjectManager,
};

// 重新导出端口协商层
pub use port_negotiation::{
    PortNegotiator, AdvancedPortNegotiator, PortRange, PortNegotiationResult,
    DEFAULT_PORT_RANGE, COMMON_DB_PORTS,
};

// 重新导出缓存层
pub use cache::{
    CacheManager, CacheConfig, CacheLevel, CacheManagerStats,
    MetadataCache, MetadataCacheConfig, MetadataCacheKey, MetadataCacheValue,
    LruCache, CacheStats, CachePolicy, CacheEntry,
};

// 重新导出 Arrow 相关
pub use arrow::{
    ArrowBatch, ArrowBatchStream, ArrowHandler,
};

// 重新导出流相关
pub use stream::{
    Stream, ArrowBatchStream as CoreArrowBatchStream, StreamQueryResult,
};

// 重新导出工具模块
pub use utils::{
    hash, time, string,
};

// 重新导出草稿箱模块
pub use scratchpad::{
    ScratchpadEntry, ScratchpadEntryKind, ScratchpadConfig,
    ExternalReference, ScratchpadResponse, ScratchpadStore,
};
