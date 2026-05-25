use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::core::driver::connection::config::ConnectionMethod;
use crate::core::driver::connection::connector::TunnelGuard;
use crate::core::driver::registry::DriverConnectionConfig;
use crate::core::driver::router::DataSourceRouter;
use crate::core::driver::traits::{DataSourceMeta, DynDatabase};
use crate::core::error::{ConnectionError, CoreError};
use crate::core::persistence::connection_store::{self, RecentConnectionInput};
use crate::core::persistence::global_db::GlobalConnectionSaveInput;
use crate::core::persistence::MetadataCacheManager;
use crate::core::services::connection_manager::{
    ConnectionInfo, ConnectionManager, ConnectionType,
};

/// 保存全局连接输入参数
pub struct SaveGlobalConnectionInput<'a> {
    pub conn_id: &'a str,
    pub name: &'a str,
    pub db_type: &'a str,
    pub url: &'a str,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub tags: Option<&'a str>,
    pub server_version: Option<&'a str>,
    pub description: Option<&'a str>,
    pub driver_id: Option<&'a str>,
    pub environment_id: Option<&'a str>,
    pub auth_config_id: Option<&'a str>,
    pub auth_method: Option<&'a str>,
    pub network_config_id: Option<&'a str>,
    pub options: Option<&'a str>,
    pub driver_properties: Option<&'a str>,
    pub advanced_options: Option<&'a str>,
    pub use_duckdb_fed: Option<bool>,
    pub metadata_path: Option<&'a str>,
    pub schema_name: Option<&'a str>,
}

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
    tunnels: Arc<tokio::sync::Mutex<HashMap<String, Vec<TunnelGuard>>>>,
}

impl ConnectionService {
    /// 创建新的连接服务
    pub fn new(manager: Arc<ConnectionManager>) -> Self {
        Self {
            manager,
            tunnels: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
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
    /// * `skip_persistence` - 跳过持久化到 SQLite（测试连接等场景，默认 false）
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
        auth_method: Option<String>,
        network_config_id: Option<String>,
        driver_properties: Option<String>,
        advanced_options: Option<String>,
        options: Option<String>,
        tags: Option<String>,
        metadata_path: Option<String>,
        schema_name: Option<String>,
        use_duckdb_fed: Option<bool>,
        skip_persistence: Option<bool>,
        network_method: Option<ConnectionMethod>,
    ) -> Result<(String, DynDatabase), CoreError> {
        // 参数校验
        if url.is_empty() {
            return Err(CoreError::connection(ConnectionError::InvalidConfig {
                conn_id: "unknown".to_string(),
                reason: "Database URL cannot be empty".to_string(),
            }));
        }

        // 生成连接 ID：统一使用 URL 哈希，确保：
        //   - 短且唯一（8位 hex）
        //   - 文件系统安全（无 Windows 非法字符 : @ / \ 等）
        //   - 全局/项目双链路一致
        let conn_id = conn_id.unwrap_or_else(|| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let type_prefix = match connection_type {
                ConnectionType::Global => "global",
                ConnectionType::Project => "project",
            };

            let mut hasher = DefaultHasher::new();
            url.hash(&mut hasher);
            let hash = hasher.finish();
            format!("{}-{}-{:x}", type_prefix, db_type, hash)
        });

        // 连接显示名称：用户指定 > 自动生成（db_type@host 简洁格式）
        let connection_name = name.unwrap_or_else(|| {
            let host = url
                .split('@')
                .nth(1)
                .and_then(|s| s.split('/').next())
                .map(|s| s.split('?').next().unwrap_or(s))
                .unwrap_or("localhost");
            format!("{}@{}", db_type.to_uppercase(), host)
        });

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
        let (effective_url, tunnel_guards) = self
            .apply_network_method(url, &network_method, &conn_id, db_type)
            .await?;

        // 注册隧道守卫，确保隧道在连接生命周期内保持存活
        if !tunnel_guards.is_empty() {
            self.tunnels
                .lock()
                .await
                .insert(conn_id.clone(), tunnel_guards);
            tracing::info!(
                conn_id = %conn_id,
                "已注册 {} 个隧道守卫",
                self.tunnels.lock().await.get(&conn_id).map(|v| v.len()).unwrap_or(0)
            );
        }

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
            auth_method: auth_method.clone(),
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

        // NOTE: 元数据缓存不在连接时立即创建，改为懒加载。
        // 首次查询 schema / table / column 时通过 L2 cache write 路径自动创建。

        // 对于全局连接，保存到全局 SQLite 数据库（skip_persistence 时跳过）
        if !skip_persistence.unwrap_or(false) && connection_type == ConnectionType::Global {
            // 从 URL 中解析 username 和 password
            let (username, password) = Self::extract_credentials_from_url(url);

            // 标签：优先使用输入值，无输入时默认 ["global"]
            let final_tags = tags.clone().or(Some("[\"global\"]".to_string()));
            if let Err(e) = self
                .save_global_connection_to_db(SaveGlobalConnectionInput {
                    conn_id: &conn_id,
                    name: &connection_name,
                    db_type,
                    url: &safe_url,
                    username: username.as_deref(),
                    password: password.as_deref(),
                    tags: final_tags.as_deref(),
                    server_version: server_version.as_deref(),
                    description: description.as_deref(),
                    driver_id: driver_id.as_deref(),
                    environment_id: environment_id.as_deref(),
                    auth_config_id: auth_config_id.as_deref(),
                    auth_method: auth_method.as_deref(),
                    network_config_id: network_config_id.as_deref(),
                    options: options.as_deref(),
                    driver_properties: driver_properties.as_deref(),
                    advanced_options: advanced_options.as_deref(),
                    use_duckdb_fed: use_duckdb_fed,
                    metadata_path: metadata_path.as_deref(),
                    schema_name: schema_name.as_deref(),
                })
                .await
            {
                tracing::warn!("保存全局连接信息到 SQLite 失败: {}", e);
            }
        }

        // 保存到最近连接记录（skip_persistence 时跳过）
        if !skip_persistence.unwrap_or(false) {
        if let Err(e) = connection_store::save_recent_connection(RecentConnectionInput {
            name: &connection_name,
            db_type,
            url: &safe_url,
            conn_id: Some(&conn_id),
            description: description.as_deref(),
            driver_id: driver_id.as_deref(),
            environment_id: environment_id.as_deref(),
            auth_config_id: auth_config_id.as_deref(),
            auth_method: auth_method.as_deref(),
            network_config_id: network_config_id.as_deref(),
            driver_properties: driver_properties.as_deref(),
            advanced_options: advanced_options.as_deref(),
        }) {
            tracing::warn!("Failed to save connection history: {}", e);
        }
        }

        Ok((conn_id, db))
    }

    /// 保存全局连接信息到全局 SQLite 数据库
    async fn save_global_connection_to_db(
        &self,
        input: SaveGlobalConnectionInput<'_>,
    ) -> Result<(), CoreError> {
        use crate::core::migration::global_init;

        let global_db = global_init::get_global_db_manager().ok_or_else(|| {
            CoreError::common(crate::core::error::CommonError::General(
                "Global database manager not initialized".to_string(),
            ))
        })?;

        let encrypted_password = match input.password {
            Some(p) if !p.is_empty() => {
                Some(crate::core::crypto::encrypt_password(p).map_err(|e| {
                    CoreError::common(crate::core::error::CommonError::Internal(format!(
                        "Password encryption failed: {}",
                        e
                    )))
                })?)
            }
            _ => input.password.map(|p| p.to_string()),
        };

        global_db
            .save_global_connection(GlobalConnectionSaveInput {
                conn_id: input.conn_id,
                name: input.name,
                db_type: input.db_type,
                url: input.url,
                username: input.username,
                password: encrypted_password.as_deref(),
                tags: input.tags,
                server_version: input.server_version,
                description: input.description,
                driver_id: input.driver_id,
                environment_id: input.environment_id,
                auth_config_id: input.auth_config_id,
                auth_method: input.auth_method,
                network_config_id: input.network_config_id,
                options: input.options,
                driver_properties: input.driver_properties,
                advanced_options: input.advanced_options,
                use_duckdb_fed: input.use_duckdb_fed,
                metadata_path: input.metadata_path,
                schema_name: input.schema_name,
            })
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
        db_type: &str,
    ) -> Result<(String, Vec<TunnelGuard>), CoreError> {
        match method {
            None | Some(ConnectionMethod::Direct) => Ok((url.to_string(), vec![])),
            Some(ConnectionMethod::Chain(hops)) => {
                self.process_chain(url, hops, conn_id, db_type).await
            }
            Some(ConnectionMethod::Ssh(ssh_config)) => {
                let guard =
                    create_ssh_tunnel_port(ssh_config, None).await?;
                let local_port = guard.port();
                let rewritten = Self::rewrite_url_host_port(url, "127.0.0.1", local_port)?;
                tracing::info!(
                    conn_id = %conn_id,
                    original = %Self::mask_password_in_url(url),
                    tunnel = %rewritten,
                    "SSH 隧道已建立，URL 已改写为本地端口"
                );
                Ok((rewritten, vec![guard]))
            }
            Some(ConnectionMethod::Ssl(ssl_config)) => {
                // SSL 参数由 sqlx 原生支持，通过 URL query 参数传递
                // 根据数据库类型自动映射 ssl_mode/sslmode 与证书路径
                let url_with_ssl = Self::append_ssl_params(url, db_type, ssl_config)?;
                Ok((url_with_ssl, vec![]))
            }
            Some(ConnectionMethod::HttpProxy(_) | ConnectionMethod::SocksProxy(_)) => {
                let (target_host, target_port) = Self::parse_host_port_from_url(url)?;
                let proxy_config = match method {
                    Some(ConnectionMethod::HttpProxy(c)) => c,
                    Some(ConnectionMethod::SocksProxy(c)) => c,
                    _ => unreachable!(),
                };
                let is_socks = matches!(method, Some(ConnectionMethod::SocksProxy(_)));

                if Self::matches_no_proxy(&target_host, &proxy_config.no_proxy) {
                    tracing::info!(
                        conn_id = %conn_id,
                        host = %target_host,
                        rules = ?proxy_config.no_proxy,
                        "目标主机匹配 no_proxy 规则，跳过代理"
                    );
                    return Ok((url.to_string(), vec![]));
                }

                let guard =
                    create_proxy_tunnel_port(proxy_config, &target_host, target_port, is_socks, None, None)
                        .await?;
                let local_port = guard.port();

                let rewritten = Self::rewrite_url_host_port(url, "127.0.0.1", local_port)?;
                tracing::info!(
                    conn_id = %conn_id,
                    original = %Self::mask_password_in_url(url),
                    proxy = %rewritten,
                    "代理隧道已建立，URL 已改写为本地端口"
                );
                Ok((rewritten, vec![guard]))
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
        db_type: &str,
    ) -> Result<(String, Vec<TunnelGuard>), CoreError> {
        use crate::core::driver::connection::config::ChainHop;

        let (final_db_host, final_db_port) = Self::parse_host_port_from_url(url)?;
        let mut tunnel_port: Option<u16> = None;
        let mut guards: Vec<TunnelGuard> = Vec::new();

        for (i, hop) in hops.iter().enumerate() {
            let next_hop = hops.get(i + 1);

            match hop {
                ChainHop::Ssh(ssh_config) => {
                    let connect_override =
                        tunnel_port.map(|p| ("127.0.0.1".to_string(), p));
                    let guard =
                        create_ssh_tunnel_port(ssh_config, connect_override).await?;
                    let lp = guard.port();
                    tunnel_port = Some(lp);
                    tracing::info!(
                        conn_id = %conn_id,
                        hop = i,
                        port = lp,
                        "SSH 隧道跳已建立"
                    );
                    guards.push(guard);
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
                    let connect_override =
                        tunnel_port.map(|p| ("127.0.0.1".to_string(), p));

                    if Self::matches_no_proxy(&target_host, &proxy.no_proxy) {
                        tracing::info!(
                            conn_id = %conn_id,
                            hop = i,
                            host = %target_host,
                            rules = ?proxy.no_proxy,
                            "链路中目标主机匹配 no_proxy 规则，跳过此代理跳"
                        );
                        continue;
                    }

                    let wrap_ssl = match next_hop {
                        Some(ChainHop::Ssl(ref ssl_cfg)) => Some(ssl_cfg.clone()),
                        _ => None,
                    };
                    let guard = create_proxy_tunnel_port(
                        proxy,
                        &target_host,
                        target_port,
                        is_socks,
                        connect_override,
                        wrap_ssl,
                    )
                    .await?;
                    let lp = guard.port();
                    tunnel_port = Some(lp);
                    guards.push(guard);
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
                let url_with_ssl = Self::inject_chain_ssl_params(&rewritten, hops, db_type)?;
                Ok((url_with_ssl, guards))
            }
            None => {
                let url_with_ssl = Self::inject_chain_ssl_params(url, hops, db_type)?;
                Ok((url_with_ssl, guards))
            }
        }
    }

    /// 从链路 hops 中提取 SSL 配置并注入 URL 参数
    ///
    /// 跳过已被 Proxy→SSL 嵌套层处理的 SSL hop（前一个 hop 是代理）
    fn inject_chain_ssl_params(
        url: &str,
        hops: &[crate::core::driver::connection::config::ChainHop],
        db_type: &str,
    ) -> Result<String, CoreError> {
        use crate::core::driver::connection::config::ChainHop;
        for (i, hop) in hops.iter().enumerate() {
            if let ChainHop::Ssl(ssl_config) = hop {
                let previous_is_proxy = i > 0
                    && matches!(
                        hops[i - 1],
                        ChainHop::HttpProxy(_) | ChainHop::SocksProxy(_)
                    );
                if previous_is_proxy {
                    tracing::info!(
                        target: "chain",
                        hop = i,
                        "SSL 跳已由 Proxy→SSL 嵌套层处理，跳过 URL 参数注入"
                    );
                    continue;
                }
                return Self::append_ssl_params(url, db_type, ssl_config);
            }
        }
        Ok(url.to_string())
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
            let (prefix, rest) = url.split_once("://")
                .ok_or_else(|| CoreError::from("Invalid connection URL format"))?;
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

    /// 根据 SslConfig 和数据库类型向 URL 追加 SSL 连接参数
    ///
    /// MySQL: ssl-mode, ssl-ca, ssl-cert, ssl-key
    /// PostgreSQL: sslmode, sslrootcert, sslcert, sslkey
    fn append_ssl_params(
        url: &str,
        db_type: &str,
        ssl_config: &crate::core::driver::connection::config::SslConfig,
    ) -> Result<String, CoreError> {
        let params = match db_type {
            "mysql" | "mariadb" => {
                let mode = if ssl_config.verify_server_cert {
                    if ssl_config.ca_cert_path.is_some() {
                        "VERIFY_CA"
                    } else {
                        "REQUIRED"
                    }
                } else {
                    "REQUIRED"
                };
                let mut p = format!("ssl-mode={}", mode);
                if let Some(ref ca) = ssl_config.ca_cert_path {
                    p.push_str(&format!("&ssl-ca={}", ca));
                }
                if let Some(ref cert) = ssl_config.client_cert_path {
                    p.push_str(&format!("&ssl-cert={}", cert));
                }
                if let Some(ref key) = ssl_config.client_key_path {
                    p.push_str(&format!("&ssl-key={}", key));
                }
                p
            }
            "postgres" | "postgresql" | "pgsql" => {
                let mode = if ssl_config.verify_server_cert {
                    if ssl_config.ca_cert_path.is_some() {
                        "verify-ca"
                    } else {
                        "require"
                    }
                } else {
                    "require"
                };
                let mut p = format!("sslmode={}", mode);
                if let Some(ref ca) = ssl_config.ca_cert_path {
                    p.push_str(&format!("&sslrootcert={}", ca));
                }
                if let Some(ref cert) = ssl_config.client_cert_path {
                    p.push_str(&format!("&sslcert={}", cert));
                }
                if let Some(ref key) = ssl_config.client_key_path {
                    p.push_str(&format!("&sslkey={}", key));
                }
                p
            }
            _ => {
                tracing::info!(db_type=%db_type, "非 SQL 数据库，跳过 SSL 参数注入");
                return Ok(url.to_string());
            }
        };

        Ok(Self::append_url_params(url, &params))
    }

    /// 向 URL 追加 &params 或 ?params
    fn append_url_params(url: &str, params: &str) -> String {
        if url.contains('?') {
            format!("{}&{}", url, params)
        } else {
            format!("{}?{}", url, params)
        }
    }

    /// 检查目标主机是否匹配 no_proxy 规则列表
    ///
    /// 支持格式：精确主机名、IP 地址、`.domain` 后缀通配（匹配 `*.domain`）
    fn matches_no_proxy(host: &str, rules: &[String]) -> bool {
        if rules.is_empty() {
            return false;
        }
        let host_lower = host.to_lowercase();
        for rule in rules {
            let rule = rule.trim().to_lowercase();
            if rule.is_empty() {
                continue;
            }
            if rule == host_lower {
                return true;
            }
            if rule == "localhost" && (host_lower == "127.0.0.1" || host_lower == "::1") {
                return true;
            }
            if rule == "127.0.0.1" && host_lower == "localhost" {
                return true;
            }
            if let Some(suffix) = rule.strip_prefix('.') {
                if host_lower == suffix || host_lower.ends_with(&format!(".{}", suffix)) {
                    return true;
                }
            }
        }
        false
    }

    /// 根据数据库类型创建对应的数据库实例
    /// 通过 DataSourceRouter 路由到 DriverRegistry 动态创建
    async fn create_database(&self, db_type: &str, url: &str) -> Result<DynDatabase, CoreError> {
        let config = DriverConnectionConfig::new(db_type).with_url_override(url);

        DataSourceRouter::route(config).await
    }

    /// 确保连接级元数据缓存已初始化（懒加载，幂等）
    ///
    /// 元数据缓存跟随连接信息：
    /// - 全局连接：{data_dir}/system/global_metadata/conn_{id}.sqlite
    /// - 项目连接：{project_path}/meta/connection_metadata/conn_{id}.sqlite
    ///
    /// 如果缓存文件已存在则跳过，否则创建并执行迁移。
    /// 调用时机：首次查询 schema / table / column 时。
    pub fn ensure_metadata_cache(
        conn_id: &str,
        connection_type: crate::core::persistence::ConnectionType,
        project_path: Option<&str>,
    ) -> Result<(), CoreError> {
        let cache_manager = MetadataCacheManager::new(conn_id, connection_type, project_path)?;

        if cache_manager.exists() {
            tracing::debug!(path = ?cache_manager.db_path(), "Metadata cache already exists, skipping init");
            return Ok(());
        }

        let _conn = cache_manager.open()?;

        tracing::debug!(
            "Metadata cache lazily initialized ({:?}): {:?}",
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

        // 清理隧道守卫（释放本地端口 + 取消后台任务）
        if let Some(guards) = self.tunnels.lock().await.remove(conn_id) {
            tracing::info!(
                conn_id = %conn_id,
                count = guards.len(),
                "正在清理 {} 个隧道守卫",
                guards.len()
            );
            drop(guards);
        }

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
        self.tunnels.lock().await.clear();
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
            auth_method: old_info.auth_method.clone(),
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
            auth_method: old_info.auth_method.clone(),
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

/// 创建 SSH 隧道端口转发，返回隧道生命周期守卫
///
/// connect_override: 链式跳转时，覆盖 SSH 连接目标（通过上一跳的 localhost 端口间接连接）
async fn create_ssh_tunnel_port(
    ssh_config: &crate::core::driver::connection::config::SshConfig,
    connect_override: Option<(String, u16)>,
) -> Result<crate::core::driver::connection::connector::TunnelGuard, CoreError> {
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

    connector::establish_ssh_tunnel(&dummy_config, &effective_config).await
}

/// 创建代理隧道端口转发，返回隧道生命周期守卫
///
/// 建立本地端口转发：绑定本地端口 → accept 循环 → 每个连接的桥接通过代理到目标
///
/// connect_override: 链式跳转时，覆盖代理连接目标（通过上一跳的 localhost 端口间接连接代理服务器）
/// wrap_ssl: 代理 CONNECT 成功后对连接进行 TLS 封装（Proxy → SSL 嵌套）
async fn create_proxy_tunnel_port(
    proxy_config: &crate::core::driver::connection::config::ProxyConfig,
    target_host: &str,
    target_port: u16,
    is_socks: bool,
    connect_override: Option<(String, u16)>,
    wrap_ssl: Option<crate::core::driver::connection::config::SslConfig>,
) -> Result<crate::core::driver::connection::connector::TunnelGuard, CoreError> {
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
    let effective_pc = if let Some((ref host, port)) = connect_override {
        let mut modified = pc.clone();
        modified.host = host.clone();
        modified.port = port;
        modified
    } else {
        pc.clone()
    };
    let th = target_host.to_string();
    let ssl_config = wrap_ssl.clone();
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let task = tokio::spawn(async move {
        tracing::info!(
            target: "proxy_tunnel",
            target = %format!("{}:{}", th, target_port),
            local_port,
            is_socks,
            has_tls = ssl_config.is_some(),
            "代理隧道后台任务启动 (accept 循环)"
        );
        let th_outer = th.clone();
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((local_stream, _)) => {
                            let th2 = th_outer.clone();
                            let dummy = ConnectionConfig::direct(&th2, target_port);
                            let epc = effective_pc.clone();
                            let ssl = ssl_config.clone();
                            tokio::spawn(async move {
                                let tunneled = if is_socks {
                                    connector::establish_socks_proxy(&dummy, &epc).await
                                } else {
                                    connector::establish_http_proxy(&dummy, &epc).await
                                };
                                match tunneled {
                                    Ok(proxy_stream) => {
                                        if let Some(ref ssl_cfg) = ssl {
                                            match connector::wrap_tls_stream(
                                                proxy_stream,
                                                ssl_cfg,
                                                &th2,
                                            )
                                            .await
                                            {
                                                Ok(tls_stream) => {
                                                    let (mut lr, mut lw) =
                                                        tokio::io::split(local_stream);
                                                    let (mut pr, mut pw) =
                                                        tokio::io::split(tls_stream);
                                                    let _ = tokio::join!(
                                                        tokio::io::copy(&mut lr, &mut pw),
                                                        tokio::io::copy(&mut pr, &mut lw),
                                                    );
                                                    tracing::debug!(target: "proxy_tunnel", "TLS 加密代理桥接结束");
                                                }
                                                Err(e) => {
                                                    tracing::warn!(target: "proxy_tunnel", host = %th2, "TLS 封装失败: {}", e);
                                                }
                                            }
                                        } else {
                                            let (mut lr, mut lw) =
                                                tokio::io::split(local_stream);
                                            let (mut pr, mut pw) =
                                                tokio::io::split(proxy_stream);
                                            let _ = tokio::join!(
                                                tokio::io::copy(&mut lr, &mut pw),
                                                tokio::io::copy(&mut pr, &mut lw),
                                            );
                                            tracing::debug!(target: "proxy_tunnel", "代理桥接结束");
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!(target: "proxy_tunnel", host = %th2, port = %target_port, "代理隧道连接失败: {}", e);
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            tracing::warn!(target: "proxy_tunnel", "接受本地代理连接失败: {}", e);
                            break;
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    tracing::info!(target: "proxy_tunnel", local_port, "代理隧道收到关闭信号，退出 accept 循环");
                    break;
                }
            }
        }
        drop(listener);
        drop(effective_pc);
        tracing::info!(target: "proxy_tunnel", local_port, "代理隧道后台任务已退出");
    });

    tracing::info!(
        target: "proxy_tunnel",
        target = %format!("{}:{}", target_host, target_port),
        local_port,
        is_socks,
        "代理隧道已建立"
    );

    Ok(crate::core::driver::connection::connector::TunnelGuard::new(
        local_port,
        shutdown_tx,
        task,
        format!("proxy:{}", target_host),
    ))
}

/// 从全局 DB 解析 network_config_id → ConnectionMethod
///
/// 只查询全局 network_configs 表（测试连接场景）
/// 根据 config 中的 network_type 字段进行 JSON 反序列化
pub async fn resolve_network_method(
    network_config_id: Option<&str>,
) -> Result<Option<ConnectionMethod>, CoreError> {
    let Some(net_id) = network_config_id else {
        return Ok(None);
    };

    let gdb = match crate::core::migration::get_global_db_manager() {
        Some(gdb) => gdb,
        None => return Ok(None),
    };

    let nets = gdb.list_network_configs(None).await?;
    let net = match nets.iter().find(|n| n.id == net_id) {
        Some(net) => net,
        None => {
            tracing::warn!("未找到网络配置 ID={}", net_id);
            return Ok(None);
        }
    };

    parse_network_config_json(&net.network_type, &net.config).await
}

/// 根据 network_type 将 config JSON 解析为 ConnectionMethod
///
/// 公共函数，commands 层和 service 层共享
pub async fn parse_network_config_json(
    network_type: &str,
    config_json: &str,
) -> Result<Option<ConnectionMethod>, CoreError> {
    match network_type {
        "chain" => {
            let hops: Vec<crate::core::driver::connection::config::ChainHop> =
                serde_json::from_str(config_json).map_err(|e| {
                    CoreError::from(format!("解析协议链配置 JSON 失败: {}", e))
                })?;
            if hops.is_empty() {
                return Ok(None);
            }
            Ok(Some(ConnectionMethod::Chain(hops)))
        }
        "ssh" => {
            let ssh_config: crate::core::driver::connection::config::SshConfig =
                serde_json::from_str(config_json)
                    .map_err(|e| CoreError::from(format!("解析 SSH 隧道配置 JSON 失败: {}", e)))?;
            Ok(Some(ConnectionMethod::Ssh(ssh_config)))
        }
        "ssl" => {
            let ssl_config: crate::core::driver::connection::config::SslConfig =
                serde_json::from_str(config_json)
                    .map_err(|e| CoreError::from(format!("解析 SSL 配置 JSON 失败: {}", e)))?;
            Ok(Some(ConnectionMethod::Ssl(ssl_config)))
        }
        "proxy" | "http_proxy" | "socks" | "socks5" => {
            let proxy_config: crate::core::driver::connection::config::ProxyConfig =
                serde_json::from_str(config_json)
                    .map_err(|e| CoreError::from(format!("解析代理配置 JSON 失败: {}", e)))?;
            if network_type == "socks" || network_type == "socks5" {
                Ok(Some(ConnectionMethod::SocksProxy(proxy_config)))
            } else {
                Ok(Some(ConnectionMethod::HttpProxy(proxy_config)))
            }
        }
        _ => {
            tracing::warn!("未知的网络配置类型: {}", network_type);
            Ok(None)
        }
    }
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
