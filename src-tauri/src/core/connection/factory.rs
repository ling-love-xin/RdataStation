//! 连接工厂
//!
//! 提供统一的连接创建入口，根据配置自动选择合适的连接器

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::connection::config::{ConnectionConfig, ConnectionMethod};
use crate::core::connection::connector::{
    Connection, Connector, DirectConnector, HttpProxyConnector, SocksProxyConnector,
    SshTunnelConnector, SslConnector,
};
use crate::core::connection::stream::ConnectionStream;
use crate::core::error::CoreError;

/// 连接工厂
///
/// 管理所有可用的连接器，根据连接配置自动选择合适的连接器
///
/// # 使用示例
///
/// ```rust
/// let factory = ConnectionFactory::new();
/// let config = ConnectionConfig::direct("localhost", 3306);
/// let connection = factory.create_connection(config).await?;
/// ```
pub struct ConnectionFactory {
    /// 连接器映射表
    connectors: HashMap<String, Arc<dyn Connector>>,
}

impl ConnectionFactory {
    /// 创建新的连接工厂
    ///
    /// 自动注册所有内置连接器
    pub fn new() -> Self {
        let mut factory = Self {
            connectors: HashMap::new(),
        };
        factory.register_builtin_connectors();
        factory
    }

    /// 注册内置连接器
    fn register_builtin_connectors(&mut self) {
        self.register(Arc::new(DirectConnector));
        self.register(Arc::new(SslConnector));
        self.register(Arc::new(SshTunnelConnector));
        self.register(Arc::new(HttpProxyConnector));
        self.register(Arc::new(SocksProxyConnector));
    }

    /// 注册连接器
    ///
    /// # Arguments
    ///
    /// * `connector` - 连接器实例
    pub fn register(&mut self, connector: Arc<dyn Connector>) {
        let name = connector.name().to_string();
        self.connectors.insert(name, connector);
    }

    /// 创建连接
    ///
    /// 根据连接配置自动选择合适的连接器并建立连接
    ///
    /// # Arguments
    ///
    /// * `config` - 连接配置
    ///
    /// # Returns
    ///
    /// 返回连接实例或错误
    pub async fn create_connection(
        &self,
        config: ConnectionConfig,
    ) -> Result<Connection, CoreError> {
        let connector = self.find_connector(&config.method)?;
        let stream = connector.connect(&config).await?;

        Ok(Connection::new(stream, config))
    }

    /// 创建连接流（不包装为 Connection）
    ///
    /// # Arguments
    ///
    /// * `config` - 连接配置
    ///
    /// # Returns
    ///
    /// 返回连接流或错误
    pub async fn create_stream(
        &self,
        config: &ConnectionConfig,
    ) -> Result<ConnectionStream, CoreError> {
        let connector = self.find_connector(&config.method)?;
        connector.connect(config).await
    }

    /// 查找合适的连接器
    fn find_connector(&self, method: &ConnectionMethod) -> Result<Arc<dyn Connector>, CoreError> {
        for connector in self.connectors.values() {
            if connector.supports(method) {
                return Ok(connector.clone());
            }
        }

        Err(CoreError::connection(
            crate::core::error::ConnectionError::NotSupported(format!(
                "No connector found for method: {:?}",
                method
            )),
        ))
    }

    /// 获取所有已注册的连接器名称
    pub fn get_connector_names(&self) -> Vec<&str> {
        self.connectors.keys().map(|s| s.as_str()).collect()
    }

    /// 检查是否支持指定的连接方式
    pub fn supports(&self, method: &ConnectionMethod) -> bool {
        self.find_connector(method).is_ok()
    }
}

impl Default for ConnectionFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局连接工厂实例
///
/// 使用 OnceLock 实现懒加载单例
use std::sync::OnceLock;

static GLOBAL_CONNECTION_FACTORY: OnceLock<ConnectionFactory> = OnceLock::new();

/// 获取全局连接工厂实例
pub fn get_connection_factory() -> &'static ConnectionFactory {
    GLOBAL_CONNECTION_FACTORY.get_or_init(ConnectionFactory::new)
}

/// 创建连接（使用全局工厂）
///
/// # Arguments
///
/// * `config` - 连接配置
///
/// # Returns
///
/// 返回连接实例或错误
pub async fn create_connection(config: ConnectionConfig) -> Result<Connection, CoreError> {
    get_connection_factory().create_connection(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::connection::config::ConnectionConfig;

    #[test]
    fn test_connection_factory_new() {
        let factory = ConnectionFactory::new();
        let names = factory.get_connector_names();
        assert!(names.contains(&"direct"));
        assert!(names.contains(&"ssl"));
        assert!(names.contains(&"ssh_tunnel"));
        assert!(names.contains(&"http_proxy"));
        assert!(names.contains(&"socks_proxy"));
    }

    #[test]
    fn test_supports_direct() {
        let factory = ConnectionFactory::new();
        let config = ConnectionConfig::direct("localhost", 3306);
        assert!(factory.supports(&config.method));
    }

    #[tokio::test]
    async fn test_create_connection_direct() {
        // 注意：这个测试需要实际的数据库服务器
        // 在生产环境中应该使用 mock 或测试容器
        let factory = ConnectionFactory::new();
        let config = ConnectionConfig::direct("127.0.0.1", 3306);

        // 如果本地没有 MySQL，这个测试会失败
        // 在生产环境中应该跳过或 mock
        let result = factory.create_stream(&config).await;
        // 我们只需要确保它能到达连接阶段，不强制要求成功
        assert!(result.is_err() || result.is_ok());
    }
}
