
//! 插件系统核心模块
//!
//! 提供完整的插件管理、加载、生命周期控制功能

pub mod events;
pub mod loader;
pub mod manager;
pub mod manifest;
pub mod installer;
pub mod dependency;
pub mod permission;

pub use events::*;
pub use loader::*;
pub use manager::*;
pub use manifest::*;
pub use dependency::*;
pub use permission::*;
