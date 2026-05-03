//! 数据源路由层
//!
//! 职责：数据源注册、路由、能力发现
//! 具体驱动实现在 driver/native/ 中

pub mod router;

pub use router::DataSourceRouter;
