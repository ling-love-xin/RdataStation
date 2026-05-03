//! 统一连接层
//!
//! 提供支持多种连接方式的统一抽象：
//! - 直接连接（Direct）
//! - SSL/TLS 加密连接
//! - SSH 隧道连接
//! - HTTP/HTTPS 代理连接
//! - SOCKS4/5 代理连接
//!
//! 架构设计：
//! ```
//! ConnectionConfig
//!       │
//!       ▼
//! ConnectionFactory
//!       │
//!       ├── DirectConnector
//!       ├── SslConnector
//!       ├── SshTunnelConnector
//!       ├── HttpProxyConnector
//!       └── SocksProxyConnector
//! ```

pub mod config;
pub mod connector;
pub mod factory;
pub mod stream;

pub use config::{ConnectionConfig, ConnectionMethod, SshConfig, ProxyConfig, SslConfig};
pub use connector::{Connection, Connector};
pub use factory::ConnectionFactory;
pub use stream::{ConnectionStream, StreamWrapper};
