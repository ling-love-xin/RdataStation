//! DuckDB 分析引擎模块
//!
//! 本模块提供 DuckDB 本地分析引擎的完整封装，包括连接池管理、临时表、
//! 联邦查询、数据导入导出、全文搜索、查询计划分析、插件系统接口和扩展管理。
//!
//! # 模块结构
//! - `manager.rs` - 连接池管理
//! - `executor.rs` - 统一SQL执行接口
//! - `temp_table.rs` - 临时表管理
//! - `federation.rs` - 联邦查询
//! - `import_export.rs` - 数据导入导出
//! - `fts.rs` - 全文搜索
//! - `explain.rs` - 查询计划分析
//! - `plugin.rs` - 插件系统接口
//! - `extensions.rs` - DuckDB扩展管理

mod explain;
mod executor;
mod extensions;
mod federation;
mod fts;
mod import_export;
mod manager;
mod plugin;
mod temp_table;

// 导出所有核心类型
pub use explain::{ExplainAnalyzer, PlanNode, PlanNodeType};
pub use executor::{DuckDBExecutor, DuckDBResult};
pub use extensions::{ExtensionInfo, ExtensionManager, ExtensionStatus};
pub use federation::{DataSourceConfig, DataSourceType, FederationManager};
pub use fts::FTSManager;
pub use import_export::{DataFormat, ExportConfig, ImportConfig, ImportExportManager};
pub use manager::DuckDBManager;
pub use plugin::{PluginConnection, PluginManager, PluginPermissionLevel};
pub use temp_table::{TempTableConfig, TempTableManager, TempTableSource};
