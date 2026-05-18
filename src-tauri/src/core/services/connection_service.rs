use std::path::PathBuf;
use std::sync::Arc;

use crate::core::driver::connection::config::ConnectionMethod;
use crate::core::driver::registry::DriverConnectionConfig;
use crate::core::driver::router::DataSourceRouter;
use crate::core::driver::traits::{DataSourceMeta, DynDatabase};
use crate::core::error::{ConnectionError, CoreError};
use crate::core::persistence::connection_store;
use crate::core::persistence::MetadataCacheManager;
use crate::core::services::connection_manager::{
    ConnectionInfo, ConnectionManager, ConnectionType,
};

/// 连接服务
///
/// 负责数据库连接的生命周期管理，包括：
/// - 创建新连接
/// - 连接复用
/// - 连接切换
/// - 连接关闭
/// - 最近连接记录
/// - 连接类型转换（全局↔项目）
pub struct ConnectionService {
    manager: Arc<ConnectionManager>,
}

impl ConnectionService {
    /// 创建新的连接服务
    pub fn new(manager: Arc<ConnectionManager>) -> Self {
        Self { manager }
    }

    /// 创建或获取数据库连接（默认全局连接）
    ///
    /// # Arguments
    ///
    /// * `conn_id` - 连接唯一标识（可选，不提供则自动生成）
    /// * `db_type` - 数据库类型 (mysql, postgres, sqlite, duckdb)
    /// * `url` - 数据库连接 URL
    /// * `name` - 连接名称（可选）
    ///
    /// # Returns
    ///
    /// 返回连接 ID 和数据库实例
    pub async fn connect(
        &self,
        conn_id: Option<String>,
        db_type: &str,
        url: &str,
        name: Option<String>,
    ) -> Result<(String, DynDatabase), CoreError> {
        self.connect_with_type(
            conn_id,
            db_type,
            url,
            name,
            ConnectionType::Global,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
    }

    /// 创建或获取数据库连接（指定连接类型）
    ///
    /// # Arguments
    ///
    /// * `conn_id` - 连接唯一标识（可选，不提供则自动生成）
    /// * `db_type` - 数据库类型 (mysql, postgres, sqlite, duckdb)
    /// * `url` - 数据库连接 URL
    /// * `name` - 连接名称（可选）
    /// * `connection_type` - 连接类型（全局/项目）
    /// * `project_path` - 项目路径（仅项目连接时需要）
    /// * `description` - 连接描述（可选）
    /// * `network_method` - 网络连接方式（可选，用于 SSH 隧道/SSL 加密/代理等）
    /// * `driver_id` - 驱动 ID（可选，数据源模块字段）
    /// * `environment_id` - 环境 ID（可选）
    /// * `auth_config_id` - 认证配置 ID（可选）
    /// * `network_config_id` - 网络配置 ID（可选）
    /// * `driver_properties` - 驱动属性 JSON（可选）
    /// * `advanced_options` - 高级选项 JSON（可选）
    ///
    /// # Returns
    ///
    /// 返回连接 ID 和数据库实例
    #[allow(clippy::too_many_arguments)]
    pub async fn connect_with_type(
        &self,
        conn_id: Option<String>,
        db_type: &str,
        url: &str,
        name: Option<String>,
        connection_type: ConnectionType,
        project_path: Option<String>,
        description: Option<String>,
        driver_id: Option<String>,
        environment_id: Option<String>,
        auth_config_id: Option<String>,
        network_config_id: Option<String>,
        driver_properties: Option<String>,
        advanced_options: Option<String>,
        network_method: Option<ConnectionMethod>,
    ) -> Result<(String, DynDatabase), CoreError> {
        // 参数校验
        if url.is_empty() {
            return Err(CoreError::connection(ConnectionError::InvalidConfig {
                conn_id: "unknown".to_string(),
                reason: "Database URL cannot be empty".to_string(),
            }));
        }

        // 生成连接 ID：从 URL 中提取干净的路径部分，并对文件路径进行安全处理
        // 加入连接类型前缀来区分全局连接和项目连接，避免 ID 重复
        let conn_id = conn_id.unwrap_or_else(|| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            // 连接类型前缀
            let type_prefix = match connection_type {
                ConnectionType::Global => "global",
                ConnectionType::Project => "project",
            };

            // 对于文件数据库，使用路径的哈希值避免非法字符
            if url.starts_with("sqlite://") || url.starts_with("duckdb://") {
                let path = if url.starts_with("sqlite://") {
                    url.trim_start_matches("sqlite://")
                } else {
                    url.trim_start_matches("duckdb://")
                };
                // 使用路径的哈希值作为 ID 的一部分
                let mut hasher = DefaultHasher::new();
                path.hash(&mut hasher);
                let hash = hasher.finish();
                format!("{}-{}-{:x}", type_prefix, db_type, hash)
            } else {
                // 网络数据库：使用主机:端口/数据库格式
                let clean_url = if url.starts_with("mysql://") {
                    url.trim_start_matches("mysql://")
                } else if url.starts_with("postgres://") {
                    url.trim_start_matches("postgres://")
                } else {
                    url
                };
                format!("{}-{}-{}", type_prefix, db_type, clean_url)
            }
        });
        let connection_name = name.unwrap_or_else(|| conn_id.clone());

        // 检查是否已有连接（基于 conn_id）
        if let Some(db) = self.manager.get_connection(&conn_id).await {
            tracing::info!(conn_id = %conn_id, "Connection already exists, reusing it");
            return Ok((conn_id, db));
        }

        // 对于文件型数据库，额外检查是否有相同 URL 的连接（避免文件锁冲突）
        if url.starts_with("sqlite://") || url.starts_with("duckdb://") {
            let all_connections = self.manager.get_all_connection_info().await;
            for conn_info in all_connections {
                if conn_info.url == url {
                    tracing::info!(url = %url, conn_id = %conn_info.id, "Connection with URL already exists");
                    if let Some(db) = self.manager.get_connection(&conn_info.id).await {
                        tracing::info!(conn_id = %conn_info.id, "Reusing existing connection");
                        return Ok((conn_info.id, db));
                    }

                    // 如果连接管理器中没有该连接（可能已被关闭），但连接信息仍存在
                    // 需要先移除旧的连接信息，然后创建新连接
                    tracing::info!(
                        "旧连接 {} 已不存在于连接管理器中，准备移除旧信息并创建新连接",
                        conn_info.id
                    );
                    self.manager.remove_connection(&conn_info.id).await;
                    // 继续创建新连接
                    break;
                }
            }
        }

        // 创建新连接
        tracing::info!("Creating new connection with ID: {}", conn_id);

        // 应用网络连接方式（SSH 隧道 / SSL / 代理）
        let effective_url = self
            .apply_network_method(url, &network_method, &conn_id)
            .await?;

        let db = self.create_database(db_type, &effective_url).await?;
        let server_version = db.meta().server_version.clone();
        let safe_url = Self::mask_password_in_url(url);

        // 创建连接信息
        let info = ConnectionInfo {
            id: conn_id.clone(),
            name: connection_name.clone(),
            db_type: db_type.to_string(),
            url: safe_url.clone(),
            server_version: server_version.clone(),
            connection_type,
            project_id: project_path.clone(),
            driver_id: driver_id.clone(),
            environment_id: environment_id.clone(),
            auth_config_id: auth_config_id.clone(),
            network_config_id: network_config_id.clone(),
            driver_properties: driver_properties.clone(),
            advanced_options: advanced_options.clone(),
            description: description.clone(),
            created_at: std::time::Instant::now(),
        };

        // 转换为 DriverConnectionConfig（用于重连）
        let driver_config = crate::core::driver::registry::DriverConnectionConfig::new(db_type)
            .with_url_override(url)
            .with_name(&connection_name);

        // 添加到连接管理器
        self.manager
            .add_connection(conn_id.clone(), Arc::clone(&db), info, driver_config)
            .await?;

        // 初始化连接级元数据库（根据连接类型选择路径）
        self.initialize_connection_metadata(
            &conn_id,
            db_type,
            url,
            connection_type,
            project_path.as_deref(),
        )
        .await?;

        // 对于全局连接，保存到全局 SQLite 数据库
        if connection_type == ConnectionType::Global {
            // 从 URL 中解析 username 和 password
            let (username, password) = Self::extract_credentials_from_url(url);

            // 默认添加 "global" 标签
            let tags = Some("[\"global\"]");
            if let Err(e) = self
                .save_global_connection_to_db(
                    &conn_id,
                    &connection_name,
                    db_type,
                    &safe_url,
                    username.as_deref(),
                    password.as_deref(),
                    tags,
                    server_version.as_deref(),
                    description.as_deref(),
                    driver_id.as_deref(),
                    environment_id.as_deref(),
                    auth_config_id.as_deref(),
                    network_config_id.as_deref(),
                    driver_properties.as_deref(),
                    advanced_options.as_deref(),
                )
                .await
            {
                tracing::warn!("保存全局连接信息到 SQLite 失败: {}", e);
            }
        }

        // 保存到最近连接记录
        if let Err(e) = connection_store::save_recent_connection(
            &connection_name,
            db_type,
            &safe_url,
            description.as_deref(),
            driver_id.as_deref(),
            environment_id.as_deref(),
            auth_config_id.as_deref(),
            network_config_id.as_deref(),
            driver_properties.as_deref(),
            advanced_options.as_deref(),
        ) {
            tracing::warn!("Failed to save connection history: {}", e);
        }

        Ok((conn_id, db))
    }

    /// 保存全局连接信息到全局 SQLite 数据库
    #[allow(clippy::too_many_arguments)]
    async fn save_global_connection_to_db(
        &self,
        conn_id: &str,
        name: &str,
        db_type: &str,
        url: &str,
        username: Option<&str>,
        password: Option<&str>,
        tags: Option<&str>,
        server_version: Option<&str>,
        description: Option<&str>,
        driver_id: Option<&str>,
        environment_id: Option<&str>,
        auth_config_id: Option<&str>,
        network_config_id: Option<&str>,
        driver_properties: Option<&str>,
        advanced_options: Option<&str>,
    ) -> Result<(), CoreError> {
        use crate::core::migration::global_init;

        let global_db = global_init::get_global_db_manager().ok_or_else(|| {
            CoreError::common(crate::core::error::CommonError::General(
                "Global database manager not initialized".to_string(),
            ))
        })?;

        let encrypted_password = match password {
            Some(p) if !p.is_empty() => {
                Some(crate::core::crypto::encrypt_password(p).map_err(|e| {
                    CoreError::common(crate::core::error::CommonError::Internal(format!(
                        "Password encryption failed: {}",
                        e
                    )))
                })?)
            }
            _ => password.map(|p| p.to_string()),
        };

        global_db
            .save_global_connection(
                conn_id,
                name,
                db_type,
                url,
                username,
                encrypted_password.as_deref(),
                tags,
                server_version,
                description,
                driver_id,
                environment_id,
                auth_config_id,
                network_config_id,
                driver_properties,
                advanced_options,
            )
            .await
    }

    /// 从 URL 中提取用户名和密码
    ///
    /// 支持的 URL 格式：
    /// - mysql://user:pass@host:port/database
    /// - postgresql://user:pass@host:port/database
    /// - sqlite:///path/to/file.sqlite (无认证)
    /// - duckdb:///path/to/file.duckdb (无认证)
    fn extract_credentials_from_url(url: &str) -> (Option<String>, Option<String>) {
        // 移除协议前缀
        let clean_url = if let Some(pos) = url.find("://") {
            &url[pos + 3..]
        } else {
            return (None, None);
        };

        // 查找 @ 符号
        if let Some(at_pos) = clean_url.find('@') {
            let auth_part = &clean_url[..at_pos];

            // 解析 user:pass
            if let Some(colon_pos) = auth_part.find(':') {
                let username = auth_part[..colon_pos].to_string();
                let password = auth_part[colon_pos + 1..].to_string();
                (Some(username), Some(password))
            } else {
                // 只有用户名，没有密码
                (Some(auth_part.to_string()), None)
            }
        } else {
            // 没有认证信息
            (None, None)
        }
    }

    /// 脱敏 URL 中的密码，替换为 ***
    pub(crate) fn mask_password_in_url(url: &str) -> String {
        if let Some(scheme_end) = url.find("://") {
            let prefix = &url[..scheme_end + 3];
            let rest = &url[scheme_end + 3..];
            if let Some(at_pos) = rest.find('@') {
                let auth = &rest[..at_pos];
                let host_part = &rest[at_pos..];
                if let Some(colon_pos) = auth.find(':') {
                    let username = &auth[..colon_pos];
                    return format!("{}{}:******{}", prefix, username, host_part);
                }
                return format!("{}{}******{}", prefix, auth, host_part);
            }
        }
        url.to_string()
    }

    /// 检查 URL 是否含明文密码，用于旧数据迁移检测
    #[allow(dead_code)]
    fn url_has_plaintext_password(url: &str) -> bool {
        if let Some(scheme_end) = url.find("://") {
            let rest = &url[scheme_end + 3..];
            if let Some(at_pos) = rest.find('@') {
                let auth = &rest[..at_pos];
                return auth.contains(':') && !auth.contains("******");
            }
        }
        false
    }

    /// 应用网络连接方式（SSH 隧道 / SSL / 代理）
    ///
    /// 根据 ConnectionMethod 对连接 URL 做改写：
    /// - SSH: 建立本地端口转发隧道，将 URL 中 host:port 改写为 localhost:tunnel_port
    /// - SSL: 将 SSL 参数注入 URL（如 ssl-ca、ssl-cert 等）
    /// - Proxy: 暂不支持（sqlx 不原生支持代理，后续通过 wrapping stream 实现）
    /// - Direct/None: 原样返回
    async fn apply_network_method(
        &self,
        url: &str,
        method: &Option<ConnectionMethod>,
        conn_id: &str,
    ) -> Result<String, CoreError> {
        match method {
            None | Some(ConnectionMethod::Direct) => Ok(url.to_string()),
            Some(ConnectionMethod::Chain(hops)) => {
                self.process_chain(url, hops, conn_id).await
            }
            Some(ConnectionMethod::Ssh(ssh_config)) => {
                let (tunnel_stream, local_port) =
                    create_ssh_tunnel_port(ssh_config, None).await?;
                drop(tunnel_stream);

                let rewritten = Self::rewrite_url_host_port(url, "127.0.0.1", local_port)?;
                tracing::info!(
                    conn_id = %conn_id,
                    original = %Self::mask_password_in_url(url),
                    tunnel = %rewritten,
                    "SSH 隧道已建立，URL 已改写为本地端口"
                );
                Ok(rewritten)
            }
            Some(ConnectionMethod::Ssl(_ssl_config)) => {
                // SSL 参数由 sqlx 原生支持，通过 URL query 参数传递
                // 例如：mysql://host/db?ssl-mode=VERIFY_CA&ssl-ca=/path/ca.pem
                // 当前 SslConfig 为一站式 TLS 流加密（connector 层），
                // sqlx 走自身 SSL 逻辑，暂不在 service 层注入 SSL 参数
                Ok(url.to_string())
            }
            Some(ConnectionMethod::HttpProxy(_) | ConnectionMethod::SocksProxy(_)) => {
                let (target_host, target_port) = Self::parse_host_port_from_url(url)?;
                let proxy_config = match method {
                    Some(ConnectionMethod::HttpProxy(c)) => c,
                    Some(ConnectionMethod::SocksProxy(c)) => c,
                    _ => unreachable!(),
                };
                let is_socks = matches!(method, Some(ConnectionMethod::SocksProxy(_)));

                let (tunnel_stream, local_port) =
                    create_proxy_tunnel_port(proxy_config, &target_host, target_port, is_socks)
                        .await?;
                drop(tunnel_stream);

                let rewritten = Self::rewrite_url_host_port(url, "127.0.0.1", local_port)?;
                tracing::info!(
                    conn_id = %conn_id,
                    original = %Self::mask_password_in_url(url),
                    proxy = %rewritten,
                    "代理隧道已建立，URL 已改写为本地端口"
                );
                Ok(rewritten)
            }
        }
    }

    /// 处理协议链路（外层 → 内层迭代）
    ///
    /// 每跳建立本地端口转发，将目标地址作为下一跳的连接入口：
    /// - Proxy 跳的目标 = 下一跳的 host:port
    /// - SSH 跳的 connect_to = 上一跳的 localhost 端口
    /// - SSL 跳由 sqlx 原生处理
    async fn process_chain(
        &self,
        url: &str,
        hops: &[crate::core::driver::connection::config::ChainHop],
        conn_id: &str,
    ) -> Result<String, CoreError> {
        use crate::core::driver::connection::config::ChainHop;

        let (final_db_host, final_db_port) = Self::parse_host_port_from_url(url)?;
        let mut tunnel_port: Option<u16> = None;

        for (i, hop) in hops.iter().enumerate() {
            let next_hop = hops.get(i + 1);

            match hop {
                ChainHop::Ssh(ssh_config) => {
                    let connect_override =
                        tunnel_port.map(|p| ("127.0.0.1".to_string(), p));
                    let (_, lp) =
                        create_ssh_tunnel_port(ssh_config, connect_override).await?;
                    tunnel_port = Some(lp);
                    tracing::info!(
                        conn_id = %conn_id,
                        hop = i,
                        port = lp,
                        "SSH 隧道跳已建立"
                    );
                }
                ChainHop::HttpProxy(proxy) | ChainHop::SocksProxy(proxy) => {
                    let (target_host, target_port) = if let Some(next) = next_hop {
                        match next {
                            ChainHop::Ssh(s) => (s.host.clone(), s.port),
                            ChainHop::HttpProxy(p) | ChainHop::SocksProxy(p) => {
                                (p.host.clone(), p.port)
                            }
                            ChainHop::Ssl(_) => {
                                (final_db_host.clone(), final_db_port)
                            }
                        }
                    } else {
                        (final_db_host.clone(), final_db_port)
                    };
                    let is_socks = matches!(hop, ChainHop::SocksProxy(_));
                    let (_, lp) = create_proxy_tunnel_port(
                        proxy,
                        &target_host,
                        target_port,
                        is_socks,
                    )
                    .await?;
                    tunnel_port = Some(lp);
                    tracing::info!(
                        conn_id = %conn_id,
                        hop = i,
                        port = lp,
                        target = %format!("{}:{}", target_host, target_port),
                        "代理跳已建立"
                    );
                }
                ChainHop::Ssl(_) => {
                    tracing::info!(
                        conn_id = %conn_id,
                        hop = i,
                        "SSL 跳（由 sqlx 原生处理）"
                    );
                }
            }
        }

        match tunnel_port {
            Some(port) => {
                let rewritten =
                    Self::rewrite_url_host_port(url, "127.0.0.1", port)?;
                tracing::info!(
                    conn_id = %conn_id,
                    original = %Self::mask_password_in_url(url),
                    tunnel = %rewritten,
                    hops = hops.len(),
                    "协议链已建立"
                );
                Ok(rewritten)
            }
            None => Ok(url.to_string()),
        }
    }

    /// 将 URL 中的 host:port 改写为新的 host:port
    ///
    /// 支持 mysql://, postgres://, sqlite://, duckdb:// 等常见 schema
    fn rewrite_url_host_port(
        url: &str,
        new_host: &str,
        new_port: u16,
    ) -> Result<String, CoreError> {
        if url.starts_with("mysql://") || url.starts_with("postgres://") {
            let (prefix, rest) = url.split_once("://").unwrap();
            let after_auth = if let Some(at_pos) = rest.find('@') {
                let (auth, _host_part) = rest.split_at(at_pos + 1);
                format!("{}{}:{}", auth, new_host, new_port)
            } else {
                format!("{}:{}", new_host, new_port)
            };
            let last_part = rest
                .find('@')
                .map(|p| {
                    let host_section = &rest[p + 1..];
                    host_section
                        .find('/')
                        .map(|s| &host_section[s..])
                        .unwrap_or("")
                })
                .unwrap_or("");
            Ok(format!("{}://{}{}", prefix, after_auth, last_part))
        } else if url.starts_with("sqlite://") || url.starts_with("duckdb://") {
            Ok(url.to_string())
        } else {
            Err(CoreError::connection(ConnectionError::InvalidConfig {
                conn_id: url.to_string(),
                reason: "无法改写 URL，不支持的协议".to_string(),
            }))
        }
    }

    /// 从数据库 URL 中解析目标主机和端口
    ///
    /// 支持 mysql://user:pass@host:port/db 和 postgres://user:pass@host:port/db 格式
    fn parse_host_port_from_url(url: &str) -> Result<(String, u16), CoreError> {
        let after_scheme = if let Some(pos) = url.find("://") {
            &url[pos + 3..]
        } else {
            return Err(CoreError::connection(ConnectionError::InvalidConfig {
                conn_id: url.to_string(),
                reason: "URL 中没有找到协议前缀".to_string(),
            }));
        };

        let host_part = if let Some(at_pos) = after_scheme.find('@') {
            &after_scheme[at_pos + 1..]
        } else {
            after_scheme
        };

        let host = if let Some(slash_pos) = host_part.find('/') {
            &host_part[..slash_pos]
        } else {
            host_part
        };

        let (hostname, port) = if let Some(colon_pos) = host.rfind(':') {
            let h = &host[..colon_pos];
            let p_str = &host[colon_pos + 1..];
            let p = p_str.parse::<u16>().map_err(|_| {
                CoreError::connection(ConnectionError::InvalidConfig {
                    conn_id: url.to_string(),
                    reason: format!("无效端口号: {}", p_str),
                })
            })?;
            (h.to_string(), p)
        } else {
            let default_port = if url.starts_with("postgres://") { 5432 } else { 3306 };
            (host.to_string(), default_port)
        };

        Ok((hostname, port))
    }

    /// 根据数据库类型创建对应的数据库实例
    /// 通过 DataSourceRouter 路由到 DriverRegistry 动态创建
    async fn create_database(&self, db_type: &str, url: &str) -> Result<DynDatabase, CoreError> {
        let config = DriverConnectionConfig::new(db_type).with_url_override(url);

        DataSourceRouter::route(config).await
    }

    /// 初始化连接级元数据库（根据连接类型选择路径）
    ///
    /// 元数据缓存跟随连接信息：
    /// - 全局连接：system/global_metadata/conn_{id}.sqlite
    /// - 项目连接：project/meta/connection_metadata/conn_{id}.sqlite
    async fn initialize_connection_metadata(
        &self,
        conn_id: &str,
        _db_type: &str,
        _url: &str,
        connection_type: ConnectionType,
        project_path: Option<&str>,
    ) -> Result<(), CoreError> {
        let cache_manager = MetadataCacheManager::new(
            conn_id,
            match connection_type {
                ConnectionType::Global => crate::core::persistence::ConnectionType::Global,
                ConnectionType::Project => crate::core::persistence::ConnectionType::Project,
            },
            project_path,
        )?;

        // 打开元数据缓存数据库（自动执行迁移）
        let _conn = cache_manager.open()?;

        tracing::debug!(
            "Connection metadata initialized ({:?}): {:?}",
            connection_type,
            cache_manager.db_path()
        );
        Ok(())
    }

    /// 获取现有连接
    pub async fn get_connection(&self, conn_id: &str) -> Option<DynDatabase> {
        self.manager.get_connection(&conn_id.to_string()).await
    }

    /// 切换活动连接
    pub async fn switch_connection(&self, conn_id: &str) -> Result<(), CoreError> {
        self.manager.switch_connection(&conn_id.to_string()).await
    }

    /// 获取当前活动连接
    pub async fn get_active_connection(&self) -> Option<DynDatabase> {
        self.manager.get_active_connection().await.map(|(_, db)| db)
    }

    /// 获取当前活动连接 ID
    pub async fn get_active_conn_id(&self) -> Option<String> {
        self.manager.get_active_conn_id().await
    }

    /// 关闭指定连接
    pub async fn close_connection(&self, conn_id: &str) -> Result<(), CoreError> {
        // 获取连接信息以清理元数据缓存
        let conn_info = self.manager.get_connection_info(&conn_id.to_string()).await;

        // 从连接管理器中移除连接
        self.manager.remove_connection(&conn_id.to_string()).await;

        // 清理元数据缓存文件
        if let Some(info) = conn_info {
            let cache_manager = MetadataCacheManager::new(
                conn_id,
                match info.connection_type {
                    ConnectionType::Global => crate::core::persistence::ConnectionType::Global,
                    ConnectionType::Project => crate::core::persistence::ConnectionType::Project,
                },
                info.project_id.as_deref(),
            )?;

            if let Err(e) = cache_manager.delete() {
                tracing::warn!(
                    "Failed to delete metadata cache for connection {}: {}",
                    conn_id,
                    e
                );
            }
        }

        Ok(())
    }

    /// 关闭所有连接
    pub async fn close_all_connections(&self) -> Result<(), CoreError> {
        self.manager.close_all_connections().await;
        Ok(())
    }

    /// 获取所有连接信息
    pub async fn list_connections(&self) -> Vec<ConnectionInfo> {
        self.manager.get_all_connection_info().await
    }

    /// 获取连接的数据源元数据
    pub async fn get_connection_meta(&self, conn_id: &str) -> Option<DataSourceMeta> {
        self.manager
            .get_connection(&conn_id.to_string())
            .await
            .map(|db| db.meta())
    }

    /// 检查连接是否存在
    pub async fn has_connection(&self, conn_id: &str) -> bool {
        self.manager
            .get_connection(&conn_id.to_string())
            .await
            .is_some()
    }

    /// 获取最近使用的连接列表
    pub fn get_recent_connections(
        &self,
    ) -> Result<Vec<connection_store::ConnectionRecord>, CoreError> {
        connection_store::get_recent_connections().map_err(|e| {
            CoreError::storage(crate::core::error::StorageError::read(
                "recent_connections",
                e.to_string(),
            ))
        })
    }

    /// 删除最近连接记录
    pub fn remove_recent_connection(&self, name: &str) -> Result<(), CoreError> {
        connection_store::remove_recent_connection(name).map_err(|e| {
            CoreError::storage(crate::core::error::StorageError::write(
                "recent_connections",
                e.to_string(),
            ))
        })
    }

    /// 获取连接信息
    pub async fn get_connection_info(&self, conn_id: &str) -> Result<ConnectionInfo, CoreError> {
        self.manager
            .get_connection_info(&conn_id.to_string())
            .await
            .ok_or_else(|| CoreError::connection(ConnectionError::NotFound(conn_id.to_string())))
    }

    /// 执行 SQL 查询
    pub async fn execute_sql(
        &self,
        conn_id: Option<String>,
        sql: &str,
    ) -> Result<crate::api::dto::QueryResult, CoreError> {
        let db = if let Some(id) = conn_id {
            self.manager
                .get_connection(&id)
                .await
                .ok_or_else(|| CoreError::connection(ConnectionError::NotFound(id.clone())))?
        } else {
            self.manager
                .get_active_connection()
                .await
                .map(|(_, db)| db)
                .ok_or_else(|| CoreError::connection(ConnectionError::NoActiveConnection))?
        };

        let result = db.query(sql).await?;
        Ok(result)
    }

    /// 转换连接类型：全局 → 项目
    ///
    /// # Arguments
    ///
    /// * `conn_id` - 连接 ID
    /// * `project_id` - 目标项目 ID
    ///
    /// # Returns
    ///
    /// 返回新的连接信息
    pub async fn convert_to_project_connection(
        &self,
        conn_id: &str,
        project_id: &str,
    ) -> Result<ConnectionInfo, CoreError> {
        use crate::core::error::CommonError;

        let conn_id_string = conn_id.to_string();

        // 获取原连接信息
        let old_info = self
            .manager
            .get_connection_info(&conn_id_string)
            .await
            .ok_or_else(|| CoreError::connection(ConnectionError::NotFound(conn_id.to_string())))?;

        // 验证原连接是全局连接
        if old_info.connection_type != ConnectionType::Global {
            return Err(CoreError::common(CommonError::General(format!(
                "Connection {} is not a global connection",
                conn_id
            ))));
        }

        // 复制元数据文件到项目目录
        let old_meta_path = self.get_global_metadata_path(conn_id);
        let new_meta_path = self.get_project_metadata_path(conn_id, project_id);

        if old_meta_path.exists() {
            if let Some(parent) = new_meta_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    CoreError::common(CommonError::General(format!(
                        "Failed to create project metadata directory: {}",
                        e
                    )))
                })?;
            }

            std::fs::copy(&old_meta_path, &new_meta_path).map_err(|e| {
                CoreError::common(CommonError::General(format!(
                    "Failed to copy metadata file: {}",
                    e
                )))
            })?;

            tracing::info!(
                "Copied metadata from {:?} to {:?}",
                old_meta_path,
                new_meta_path
            );
        }

        // 更新连接信息
        let new_info = ConnectionInfo {
            id: old_info.id.clone(),
            name: old_info.name.clone(),
            db_type: old_info.db_type.clone(),
            url: old_info.url.clone(),
            server_version: old_info.server_version.clone(),
            connection_type: ConnectionType::Project,
            project_id: Some(project_id.to_string()),
            driver_id: old_info.driver_id.clone(),
            environment_id: old_info.environment_id.clone(),
            auth_config_id: old_info.auth_config_id.clone(),
            network_config_id: old_info.network_config_id.clone(),
            driver_properties: old_info.driver_properties.clone(),
            advanced_options: old_info.advanced_options.clone(),
            description: old_info.description.clone(),
            created_at: old_info.created_at,
        };

        // 更新连接管理器中的信息
        self.manager
            .update_connection_info(&conn_id_string, new_info.clone())
            .await?;

        tracing::info!("Connection {} converted from global to project", conn_id);
        Ok(new_info)
    }

    /// 转换连接类型：项目 → 全局
    ///
    /// # Arguments
    ///
    /// * `conn_id` - 连接 ID
    ///
    /// # Returns
    ///
    /// 返回新的连接信息
    pub async fn convert_to_global_connection(
        &self,
        conn_id: &str,
    ) -> Result<ConnectionInfo, CoreError> {
        use crate::core::error::CommonError;

        let conn_id_string = conn_id.to_string();

        // 获取原连接信息
        let old_info = self
            .manager
            .get_connection_info(&conn_id_string)
            .await
            .ok_or_else(|| CoreError::connection(ConnectionError::NotFound(conn_id.to_string())))?;

        // 验证原连接是项目连接
        if old_info.connection_type != ConnectionType::Project {
            return Err(CoreError::common(CommonError::General(format!(
                "Connection {} is not a project connection",
                conn_id
            ))));
        }

        // 移动元数据文件到全局目录
        let project_id = old_info.project_id.as_deref().ok_or_else(|| {
            CoreError::common(CommonError::General("Project ID is missing".to_string()))
        })?;

        let old_meta_path = self.get_project_metadata_path(conn_id, project_id);
        let new_meta_path = self.get_global_metadata_path(conn_id);

        if old_meta_path.exists() {
            if let Some(parent) = new_meta_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    CoreError::common(CommonError::General(format!(
                        "Failed to create global metadata directory: {}",
                        e
                    )))
                })?;
            }

            std::fs::copy(&old_meta_path, &new_meta_path).map_err(|e| {
                CoreError::common(CommonError::General(format!(
                    "Failed to copy metadata file: {}",
                    e
                )))
            })?;

            // 删除原项目元数据文件
            let _ = std::fs::remove_file(&old_meta_path);

            tracing::info!(
                "Moved metadata from {:?} to {:?}",
                old_meta_path,
                new_meta_path
            );
        }

        // 更新连接信息
        let new_info = ConnectionInfo {
            id: old_info.id.clone(),
            name: old_info.name.clone(),
            db_type: old_info.db_type.clone(),
            url: old_info.url.clone(),
            server_version: old_info.server_version.clone(),
            connection_type: ConnectionType::Global,
            project_id: None,
            driver_id: old_info.driver_id.clone(),
            environment_id: old_info.environment_id.clone(),
            auth_config_id: old_info.auth_config_id.clone(),
            network_config_id: old_info.network_config_id.clone(),
            driver_properties: old_info.driver_properties.clone(),
            advanced_options: old_info.advanced_options.clone(),
            description: old_info.description.clone(),
            created_at: old_info.created_at,
        };

        // 更新连接管理器中的信息
        self.manager
            .update_connection_info(&conn_id_string, new_info.clone())
            .await?;

        tracing::info!("Connection {} converted from project to global", conn_id);
        Ok(new_info)
    }

    /// 获取全局连接的元数据路径
    fn get_global_metadata_path(&self, conn_id: &str) -> PathBuf {
        let cache_manager = MetadataCacheManager::new(
            conn_id,
            crate::core::persistence::ConnectionType::Global,
            None,
        );
        cache_manager
            .map(|m| m.db_path().clone())
            .unwrap_or_else(|_| PathBuf::from(".").join(format!("conn_global_{}.sqlite", conn_id)))
    }

    /// 获取项目连接的元数据路径
    fn get_project_metadata_path(&self, conn_id: &str, project_id: &str) -> PathBuf {
        let cache_manager = MetadataCacheManager::new(
            conn_id,
            crate::core::persistence::ConnectionType::Project,
            Some(project_id),
        );
        cache_manager
            .map(|m| m.db_path().clone())
            .unwrap_or_else(|_| {
                PathBuf::from(project_id)
                    .join(".RSmeta")
                    .join("metadata")
                    .join(format!("conn_project_{}.sqlite", conn_id))
            })
    }

    /// 检测项目中是否存在全局连接
    ///
    /// # Arguments
    ///
    /// * `project_id` - 项目 ID
    ///
    /// # Returns
    ///
    /// 返回全局连接列表
    pub async fn detect_global_connections_in_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ConnectionInfo>, CoreError> {
        let connections = self.manager.get_all_connection_info().await;
        let global_connections: Vec<ConnectionInfo> = connections
            .into_iter()
            .filter(|info| {
                info.connection_type == ConnectionType::Global
                    && info.project_id.as_deref() == Some(project_id)
            })
            .collect();

        Ok(global_connections)
    }
}

/// 创建 SSH 隧道端口转发，返回（隧道本地流，本地端口号）
///
/// connect_override: 链式跳转时，覆盖 SSH 连接目标（通过上一跳的 localhost 端口间接连接）
async fn create_ssh_tunnel_port(
    ssh_config: &crate::core::driver::connection::config::SshConfig,
    connect_override: Option<(String, u16)>,
) -> Result<(tokio::net::TcpStream, u16), CoreError> {
    use crate::core::driver::connection::config::ConnectionConfig;
    use crate::core::driver::connection::connector;

    let effective_config = if let Some((ref host, port)) = connect_override {
        let mut modified = ssh_config.clone();
        modified.host = host.clone();
        modified.port = port;
        modified
    } else {
        ssh_config.clone()
    };

    let dummy_config = ConnectionConfig::direct("127.0.0.1", 0);

    let stream = connector::establish_ssh_tunnel(&dummy_config, &effective_config).await?;
    let port = stream
        .local_addr()
        .map_err(|e| {
            CoreError::connection(ConnectionError::Network {
                conn_id: "ssh_tunnel".to_string(),
                reason: format!("获取隧道本地端口失败: {}", e),
            })
        })?
        .port();

    Ok((stream, port))
}

/// 创建代理隧道端口转发，返回（隧道本地流，本地端口号）
///
/// 建立本地端口转发：绑定本地端口 → 后台任务通过代理连接到目标 → 双向数据拷贝
/// 调用方通过返回的端口号改写数据库 URL 后即可通过代理连接
async fn create_proxy_tunnel_port(
    proxy_config: &crate::core::driver::connection::config::ProxyConfig,
    target_host: &str,
    target_port: u16,
    is_socks: bool,
) -> Result<(tokio::net::TcpStream, u16), CoreError> {
    use crate::core::driver::connection::config::ConnectionConfig;
    use crate::core::driver::connection::connector;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| {
            CoreError::connection(ConnectionError::Network {
                conn_id: format!("{}:{}", target_host, target_port),
                reason: format!("绑定代理转发本地端口失败: {}", e),
            })
        })?;

    let local_port = listener.local_addr().map_err(|e| {
        CoreError::connection(ConnectionError::Network {
            conn_id: format!("{}:{}", target_host, target_port),
            reason: format!("获取代理转发本地端口失败: {}", e),
        })
    })?.port();

    let pc = proxy_config.clone();
    let th = target_host.to_string();
    tokio::spawn(async move {
        match listener.accept().await {
            Ok((local_stream, _)) => {
                let dummy = ConnectionConfig::direct(&th, target_port);
                let tunneled = if is_socks {
                    connector::establish_socks_proxy(&dummy, &pc).await
                } else {
                    connector::establish_http_proxy(&dummy, &pc).await
                };

                match tunneled {
                    Ok(tunneled) => {
                        let (mut lr, mut lw) = tokio::io::split(local_stream);
                        let (mut tr, mut tw) = tokio::io::split(tunneled);
                        let _ = tokio::join!(
                            tokio::io::copy(&mut lr, &mut tw),
                            tokio::io::copy(&mut tr, &mut lw),
                        );
                    }
                    Err(e) => {
                        tracing::warn!(target: "proxy_tunnel", host = %th, port = %target_port, "代理隧道连接失败: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(target: "proxy_tunnel", "接受本地代理连接失败: {}", e);
            }
        }
    });

    let local_stream =
        tokio::net::TcpStream::connect(format!("127.0.0.1:{}", local_port))
            .await
            .map_err(|e| {
                CoreError::connection(ConnectionError::Network {
                    conn_id: format!("{}:{}", target_host, target_port),
                    reason: format!("连接代理本地端口失败: {}", e),
                })
            })?;

    Ok((local_stream, local_port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_empty_url() {
        let manager = Arc::new(ConnectionManager::new());
        let service = ConnectionService::new(manager);

        let result = service.connect(None, "mysql", "", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_invalid_db_type() {
        let manager = Arc::new(ConnectionManager::new());
        let service = ConnectionService::new(manager);

        let result = service
            .connect(None, "invalid", "mysql://localhost", None)
            .await;
        assert!(result.is_err());
    }
}
