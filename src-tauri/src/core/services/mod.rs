//! Service 层
//!
//! 负责业务逻辑处理，将 Command 层与核心逻辑解耦。
//! 这是大型 Rust 项目的常见架构模式：
//! - Command 层只负责参数解析和调用 Service
//! - Service 层负责业务逻辑
//! - 便于单元测试和代码复用

pub mod connection_manager;
pub mod connection_service;
pub mod duckdb_service;
pub mod execution_service;
pub mod insight_engine;
pub mod persistence_service;
pub mod quality_scorer;
pub mod result_service;
pub mod sql_parser_service;
pub mod sql_service;
pub mod table_profile_service;

#[cfg(test)]
mod tests;

pub use connection_manager::{
    get_connection_manager, ConnId, ConnectionConfig, ConnectionInfo, ConnectionManager,
    ConnectionType,
};
pub use connection_service::ConnectionService;
pub use result_service::{ColumnStats, ResultService, ResultSet};
pub use sql_service::SqlService;
