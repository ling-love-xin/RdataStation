//! 命令模块
//!
//! 包含所有 Tauri 命令的实现，统一从 adapters/tauri/command.rs 迁移至此

pub mod cache_warming_commands;
pub mod connection_commands;
pub mod driver_commands;
pub mod memory_commands;
pub mod metadata_cache_commands;
pub mod navigator_commands;
pub mod performance_commands;
pub mod port_commands;
pub mod project_commands;
pub mod project_store_commands;
pub mod result_commands;
pub mod sql_commands;
pub mod sql_parser_commands;
pub mod sql_template_commands;
pub mod analytics_resource_commands;
pub mod mock_commands;

pub mod scratchpad_commands;

// 重新导出所有命令，方便 lib.rs 统一导入
pub use cache_warming_commands::*;
pub use connection_commands::*;
pub use driver_commands::*;
pub use memory_commands::*;
pub use metadata_cache_commands::*;
pub use navigator_commands::*;
pub use performance_commands::*;
pub use port_commands::*;
pub use project_commands::*;
pub use project_store_commands::*;
pub use result_commands::*;
pub use sql_commands::*;
pub use sql_template_commands::*;
pub use sql_parser_commands::*;
pub use analytics_resource_commands::*;
pub use scratchpad_commands::*;
pub use mock_commands::*;
