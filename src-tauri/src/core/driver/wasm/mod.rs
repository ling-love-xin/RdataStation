//! WASM 驱动模块
//!
//! 通过 WASM 运行时支持自定义数据库驱动
//!
//! # 架构
//!
//! ```
//! Rust Core
//!     │
//!     ▼
//! WASM Driver (core/driver/wasm/)
//!     │
//!     ▼
//! WASM Runtime (Extism)
//!     │
//!     ▼
//! Custom Driver (.wasm)
//! ```
//!
//! # 支持场景
//!
//! - 自定义协议数据库
//! - 第三方闭源驱动
//! - 热插拔驱动

pub mod driver;
pub mod adapter;
pub mod pool;

pub use driver::WasmDriver;
pub use adapter::WasmAdapter;
pub use pool::WasmPool;
