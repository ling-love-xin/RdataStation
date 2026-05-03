//! 连接器 trait 和实现
//!
//! 提供统一的连接器接口，支持多种连接方式

use async_trait::async_trait;
use tokio::net::TcpStream;

use crate::core::connection::config::{
    ConnectionConfig, ConnectionMethod, ProxyConfig, SshConfig, SslConfig,
};
use crate::core::connection::stream::ConnectionStream;
use crate::core::error::{ConnectionError, CoreError};

/// 连接器 trait
///
/// 定义统一的连接接口，所有连接方式都需要实现此 trait
#[async_trait]
pub trait Connector: Send + Sync {
    /// 建立连接
    ///
    /// # Arguments
    ///
    /// * `config` - 连接配置
    ///
    /// # Returns
    ///
    /// 返回连接流或错误
    async fn connect(&self, config: &ConnectionConfig) -> Result<ConnectionStream, CoreError>;

    /// 检查是否支持此连接方式
    fn supports(&self, method: &ConnectionMethod) -> bool;

    /// 获取连接器名称
    fn name(&self) -> &'static str;
}

/// 连接句柄
///
/// 表示一个已建立的连接
pub struct Connection {
    /// 连接流
    pub stream: ConnectionStream,
    /// 连接配置
    pub config: ConnectionConfig,
    /// 连接建立时间
    pub established_at: std::time::Instant,
}

impl Connection {
    /// 创建新的连接
    pub fn new(stream: ConnectionStream, config: ConnectionConfig) -> Self {
        Self {
            stream,
            config,
            established_at: std::time::Instant::now(),
        }
    }

    /// 获取连接持续时间
    pub fn duration(&self) -> std::time::Duration {
        self.established_at.elapsed()
    }

    /// 检查连接是否加密
    pub fn is_encrypted(&self) -> bool {
        self.stream.is_encrypted()
    }
}

/// 直接连接连接器
pub struct DirectConnector;

#[async_trait]
impl Connector for DirectConnector {
    async fn connect(&self, config: &ConnectionConfig) -> Result<ConnectionStream, CoreError> {
        let addr = format!("{}:{}", config.host, config.port);
        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| {
                CoreError::connection(ConnectionError::Network {
                    conn_id: addr.clone(),
                    reason: e.to_string(),
                })
            })?;

        Ok(ConnectionStream::tcp(stream))
    }

    fn supports(&self, method: &ConnectionMethod) -> bool {
        matches!(method, ConnectionMethod::Direct)
    }

    fn name(&self) -> &'static str {
        "direct"
    }
}

/// SSL/TLS 连接连接器
pub struct SslConnector;

#[async_trait]
impl Connector for SslConnector {
    async fn connect(&self, config: &ConnectionConfig) -> Result<ConnectionStream, CoreError> {
        let ConnectionMethod::Ssl(ssl_config) = &config.method else {
            return Err(CoreError::connection(ConnectionError::InvalidConfig {
                conn_id: format!("{}:{}", config.host, config.port),
                reason: "Expected SSL connection method".to_string(),
            }));
        };

        // 首先建立 TCP 连接
        let addr = format!("{}:{}", config.host, config.port);
        let tcp_stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| {
                CoreError::connection(ConnectionError::Network {
                    conn_id: addr.clone(),
                    reason: e.to_string(),
                })
            })?;

        // 建立 TLS 连接
        let tls_stream = establish_tls(tcp_stream, &config.host, ssl_config).await?;

        Ok(ConnectionStream::tls(tls_stream))
    }

    fn supports(&self, method: &ConnectionMethod) -> bool {
        matches!(method, ConnectionMethod::Ssl(_))
    }

    fn name(&self) -> &'static str {
        "ssl"
    }
}

/// 建立 TLS 连接
async fn establish_tls(
    stream: TcpStream,
    domain: &str,
    config: &SslConfig,
) -> Result<tokio_native_tls::TlsStream<TcpStream>, CoreError> {
    use native_tls::TlsConnector;
    use tokio_native_tls::TlsConnector as TokioTlsConnector;

    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(!config.verify_server_cert)
        .build()
        .map_err(|e| {
            CoreError::connection(ConnectionError::Tls {
                conn_id: domain.to_string(),
                reason: format!("Failed to build TLS connector: {}", e),
            })
        })?;

    let tokio_connector = TokioTlsConnector::from(connector);
    let tls_stream = tokio_connector
        .connect(domain, stream)
        .await
        .map_err(|e| {
            CoreError::connection(ConnectionError::Tls {
                conn_id: domain.to_string(),
                reason: e.to_string(),
            })
        })?;

    Ok(tls_stream)
}

/// SSH 隧道连接器
pub struct SshTunnelConnector;

#[async_trait]
impl Connector for SshTunnelConnector {
    async fn connect(&self, config: &ConnectionConfig) -> Result<ConnectionStream, CoreError> {
        let ConnectionMethod::Ssh(ssh_config) = &config.method else {
            return Err(CoreError::connection(ConnectionError::InvalidConfig {
                conn_id: format!("{}:{}", config.host, config.port),
                reason: "Expected SSH connection method".to_string(),
            }));
        };

        // 建立 SSH 隧道并返回本地端口转发流
        let stream = establish_ssh_tunnel(config, ssh_config).await?;
        Ok(ConnectionStream::ssh_tunnel(stream))
    }

    fn supports(&self, method: &ConnectionMethod) -> bool {
        matches!(method, ConnectionMethod::Ssh(_))
    }

    fn name(&self) -> &'static str {
        "ssh_tunnel"
    }
}

/// 建立 SSH 隧道
async fn establish_ssh_tunnel(
    _config: &ConnectionConfig,
    ssh_config: &SshConfig,
) -> Result<TcpStream, CoreError> {
    // 注意：这是一个简化实现
    // 实际实现需要使用 russh 库建立 SSH 连接并创建本地端口转发

    // 1. 连接到 SSH 服务器
    let ssh_addr = format!("{}:{}", ssh_config.host, ssh_config.port);
    let _ssh_stream = TcpStream::connect(&ssh_addr)
        .await
        .map_err(|e| {
            CoreError::connection(ConnectionError::Network {
                conn_id: ssh_addr.clone(),
                reason: format!("Failed to connect to SSH server: {}", e),
            })
        })?;

    // 2. 进行 SSH 认证（根据 auth 类型）
    // TODO: 实现 SSH 认证逻辑

    // 3. 创建本地端口转发
    // 绑定到本地端口
    let local_bind = if ssh_config.local_port == 0 {
        "127.0.0.1:0".to_string()
    } else {
        format!("127.0.0.1:{}", ssh_config.local_port)
    };

    let listener = tokio::net::TcpListener::bind(&local_bind)
        .await
        .map_err(|e| {
            CoreError::connection(ConnectionError::Network {
                conn_id: local_bind.clone(),
                reason: format!("Failed to bind local port: {}", e),
            })
        })?;

    let local_addr = listener.local_addr().map_err(|e| {
        CoreError::connection(ConnectionError::Network {
            conn_id: local_bind.clone(),
            reason: format!("Failed to get local address: {}", e),
        })
    })?;

    // 4. 连接到本地端口（这将通过 SSH 隧道转发到远程）
    let local_stream = TcpStream::connect(local_addr)
        .await
        .map_err(|e| {
            CoreError::connection(ConnectionError::Network {
                conn_id: local_addr.to_string(),
                reason: format!("Failed to connect to local port: {}", e),
            })
        })?;

    Ok(local_stream)
}

/// HTTP 代理连接器
pub struct HttpProxyConnector;

#[async_trait]
impl Connector for HttpProxyConnector {
    async fn connect(&self, config: &ConnectionConfig) -> Result<ConnectionStream, CoreError> {
        let ConnectionMethod::HttpProxy(proxy_config) = &config.method else {
            return Err(CoreError::connection(ConnectionError::InvalidConfig {
                conn_id: format!("{}:{}", config.host, config.port),
                reason: "Expected HTTP proxy connection method".to_string(),
            }));
        };

        let stream = establish_http_proxy(config, proxy_config).await?;
        Ok(ConnectionStream::HttpProxy(stream))
    }

    fn supports(&self, method: &ConnectionMethod) -> bool {
        matches!(method, ConnectionMethod::HttpProxy(_))
    }

    fn name(&self) -> &'static str {
        "http_proxy"
    }
}

/// 建立 HTTP 代理连接
async fn establish_http_proxy(
    config: &ConnectionConfig,
    proxy_config: &ProxyConfig,
) -> Result<TcpStream, CoreError> {
    // 连接到代理服务器
    let proxy_addr = format!("{}:{}", proxy_config.host, proxy_config.port);
    let mut stream = TcpStream::connect(&proxy_addr)
        .await
        .map_err(|e| {
            CoreError::connection(ConnectionError::Network {
                conn_id: proxy_addr.clone(),
                reason: format!("Failed to connect to HTTP proxy: {}", e),
            })
        })?;

    // 发送 HTTP CONNECT 请求
    let target = format!("{}:{}", config.host, config.port);
    let auth_header = if let Some(auth) = &proxy_config.auth {
        let credentials = base64::encode(format!("{}:{}", auth.username, auth.password));
        format!("Proxy-Authorization: Basic {}\r\n", credentials)
    } else {
        String::new()
    };

    let connect_request = format!(
        "CONNECT {} HTTP/1.1\r\nHost: {}\r\n{}\r\n",
        target, target, auth_header
    );

    tokio::io::AsyncWriteExt::write_all(&mut stream, connect_request.as_bytes())
        .await
        .map_err(|e| {
            CoreError::connection(ConnectionError::Network {
                conn_id: proxy_addr.clone(),
                reason: format!("Failed to send CONNECT request: {}", e),
            })
        })?;

    // 读取代理服务器响应
    let mut buffer = vec![0u8; 1024];
    let n = tokio::io::AsyncReadExt::read(&mut stream, &mut buffer)
        .await
        .map_err(|e| {
            CoreError::connection(ConnectionError::Network {
                conn_id: proxy_addr.clone(),
                reason: format!("Failed to read proxy response: {}", e),
            })
        })?;

    let response = String::from_utf8_lossy(&buffer[..n]);
    if !response.contains("200") {
        return Err(CoreError::connection(ConnectionError::Network {
            conn_id: proxy_addr,
            reason: format!("Proxy connection failed: {}", response),
        }));
    }

    Ok(stream)
}

/// SOCKS 代理连接器
pub struct SocksProxyConnector;

#[async_trait]
impl Connector for SocksProxyConnector {
    async fn connect(&self, config: &ConnectionConfig) -> Result<ConnectionStream, CoreError> {
        let ConnectionMethod::SocksProxy(proxy_config) = &config.method else {
            return Err(CoreError::connection(ConnectionError::InvalidConfig {
                conn_id: format!("{}:{}", config.host, config.port),
                reason: "Expected SOCKS proxy connection method".to_string(),
            }));
        };

        let stream = establish_socks_proxy(config, proxy_config).await?;
        Ok(ConnectionStream::SocksProxy(stream))
    }

    fn supports(&self, method: &ConnectionMethod) -> bool {
        matches!(method, ConnectionMethod::SocksProxy(_))
    }

    fn name(&self) -> &'static str {
        "socks_proxy"
    }
}

/// 建立 SOCKS 代理连接
async fn establish_socks_proxy(
    config: &ConnectionConfig,
    proxy_config: &ProxyConfig,
) -> Result<TcpStream, CoreError> {
    use tokio_socks::tcp::Socks5Stream;

    // 使用 &str 而不是 String，因为 ToProxyAddrs 实现了 for &str
    let proxy_addr = format!("{}:{}", proxy_config.host, proxy_config.port);
    let target_addr = format!("{}:{}", config.host, config.port);

    let stream = if let Some(auth) = &proxy_config.auth {
        Socks5Stream::connect_with_password(
            proxy_addr.as_str(),
            target_addr.as_str(),
            &auth.username,
            &auth.password,
        )
        .await
        .map_err(|e| {
            CoreError::connection(ConnectionError::Network {
                conn_id: proxy_addr.clone(),
                reason: format!("SOCKS5 connection failed: {}", e),
            })
        })?
        .into_inner()
    } else {
        Socks5Stream::connect(proxy_addr.as_str(), target_addr.as_str())
            .await
            .map_err(|e| {
                CoreError::connection(ConnectionError::Network {
                    conn_id: proxy_addr.clone(),
                    reason: format!("SOCKS5 connection failed: {}", e),
                })
            })?
            .into_inner()
    };

    Ok(stream)
}

// 添加 base64 编码支持
mod base64 {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    
    pub fn encode(input: impl AsRef<[u8]>) -> String {
        STANDARD.encode(input.as_ref())
    }
}
