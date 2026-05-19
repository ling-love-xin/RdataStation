//! 连接相关命令
//!
//! 处理数据库连接的创建、管理、关闭等操作

use crate::core::driver::connection::config::ConnectionMethod;
use crate::core::driver::DriverConnectionConfig;
use crate::core::error::CoreError;
use crate::core::services::{ConnectionService, ConnectionType};
use crate::core::{get_connection_manager, DataSourceMeta};

// ==================== Connection Commands ====================

/// 创建数据库连接请求参数
#[derive(serde::Deserialize, Debug)]
pub struct ConnectDatabaseInput {
    pub db_type: String,
    pub url: String,
    pub name: Option<String>,
    pub connection_type: Option<String>, // "global" 或 "project"
    pub project_id: Option<String>,      // 仅项目连接时需要
    pub description: Option<String>,
    pub driver_id: Option<String>,
    pub environment_id: Option<String>,
    pub auth_config_id: Option<String>,
    pub network_config_id: Option<String>,
    pub driver_properties: Option<String>,
    pub advanced_options: Option<String>,
}

/// 连接响应
#[derive(serde::Serialize, Debug)]
pub struct ConnectDatabaseResponse {
    pub conn_id: String,
    pub name: String,
    pub db_type: String,
    pub url: String,
    pub status: String,
    pub meta: DataSourceMetaResponse,
}

/// 数据源元数据响应
#[derive(serde::Serialize, Debug)]
pub struct DataSourceMetaResponse {
    pub supports_transaction: bool,
    pub supports_streaming: bool,
    pub supports_arrow: bool,
    pub supports_federated: bool,
    pub supports_concurrent_write: bool,
    pub is_in_memory: bool,
    pub server_version: Option<String>,
}

impl From<DataSourceMeta> for DataSourceMetaResponse {
    fn from(meta: DataSourceMeta) -> Self {
        Self {
            supports_transaction: meta.supports_transaction,
            supports_streaming: meta.supports_streaming,
            supports_arrow: meta.supports_arrow,
            supports_federated: meta.supports_federated,
            supports_concurrent_write: meta.supports_concurrent_write,
            is_in_memory: meta.is_in_memory,
            server_version: meta.server_version,
        }
    }
}

/// 创建数据库连接
#[tauri::command]
pub async fn connect_database(
    input: ConnectDatabaseInput,
) -> Result<ConnectDatabaseResponse, CoreError> {
    if input.url.is_empty() {
        return Err("Database URL cannot be empty".into());
    }

    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    // 解析连接类型
    let connection_type = match input.connection_type.as_deref() {
        Some("global") | None => ConnectionType::Global,
        Some("project") => ConnectionType::Project,
        Some(other) => return Err(format!("Invalid connection type: {}", other).into()),
    };

    // 项目连接必须有 project_id
    if connection_type == ConnectionType::Project && input.project_id.is_none() {
        return Err("project_id is required for project connections".into());
    }

    // ===== 数据源模块：驱动校验 =====
    if let Some(ref driver_id) = input.driver_id {
        let global_db = crate::core::migration::get_global_db_manager()
            .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;

        let driver = global_db
            .get_driver(driver_id)
            .await?
            .ok_or_else(|| CoreError::from(format!("驱动 {} 不存在于全局目录中", driver_id)))?;

        if connection_type == ConnectionType::Project {
            if let Some(ref proj_path) = input.project_id {
                let meta_dir = std::path::Path::new(proj_path).join(".RSmeta");
                let db_path = meta_dir.join("project.db");
                if db_path.exists() {
                    let conn = rusqlite::Connection::open(&db_path)
                        .map_err(|e| CoreError::from(format!("打开项目数据库失败: {}", e)))?;
                    let enabled: bool = conn
                        .query_row(
                            "SELECT enabled FROM project_drivers WHERE driver_id = ?1",
                            rusqlite::params![driver_id],
                            |row| row.get(0),
                        )
                        .unwrap_or(false);
                    if !enabled {
                        return Err(CoreError::from(format!(
                            "驱动 {} 未在当前项目中启用，请先在驱动管理中启用",
                            driver_id
                        )));
                    }
                }
            }
        }

        if driver.driver_kind != "native" {
            let version = driver.version.as_deref().unwrap_or("0.0.0");
            let installed = global_db
                .is_driver_installed(driver_id, version)
                .await
                .map_err(|e| CoreError::from(format!("检查驱动安装状态失败: {}", e)))?;
            if !installed {
                return Err(CoreError::from(format!(
                    "驱动 {} 的文件未在本机安装，请先下载安装",
                    driver_id
                )));
            }
        }
    }

    // ===== 数据源模块：环境/认证/网络校验 =====
    // 辅助闭包：打开项目DB连接（复用于项目级校验）
    let open_project_db = |proj_path: &str| -> Option<rusqlite::Connection> {
        let db_path = std::path::Path::new(proj_path)
            .join(".RSmeta")
            .join("project.db");
        if db_path.exists() {
            rusqlite::Connection::open(&db_path).ok()
        } else {
            None
        }
    };

    let is_project = connection_type == ConnectionType::Project;
    let proj_db = input.project_id.as_deref().and_then(open_project_db);

    if let Some(ref env_id) = input.environment_id {
        let mut found = false;
        // 先查全局环境
        if let Some(gdb) = crate::core::migration::get_global_db_manager() {
            if let Ok(envs) = gdb.list_environments().await {
                found = envs.iter().any(|e| e.id == *env_id);
            }
        }
        // 项目连接时也查项目级环境
        if !found && is_project {
            if let Some(ref conn) = proj_db {
                found = conn
                    .query_row::<i64, _, _>(
                        "SELECT COUNT(*) FROM environments WHERE id = ?1",
                        rusqlite::params![env_id],
                        |row| row.get(0),
                    )
                    .map(|c| c > 0)
                    .unwrap_or(false);
            }
        }
        if !found {
            return Err(CoreError::from(format!("环境 {} 不存在", env_id)));
        }
    }

    if let Some(ref auth_id) = input.auth_config_id {
        let mut found = false;
        if let Some(gdb) = crate::core::migration::get_global_db_manager() {
            if let Ok(auths) = gdb.list_auth_configs(None).await {
                found = auths.iter().any(|a| a.id == *auth_id);
            }
        }
        if !found && is_project {
            if let Some(ref conn) = proj_db {
                found = conn
                    .query_row::<i64, _, _>(
                        "SELECT COUNT(*) FROM auth_configs WHERE id = ?1",
                        rusqlite::params![auth_id],
                        |row| row.get(0),
                    )
                    .map(|c| c > 0)
                    .unwrap_or(false);
            }
        }
        if !found {
            return Err(CoreError::from(format!("认证配置 {} 不存在", auth_id)));
        }
    }

    if let Some(ref net_id) = input.network_config_id {
        let mut found = false;
        if let Some(gdb) = crate::core::migration::get_global_db_manager() {
            if let Ok(nets) = gdb.list_network_configs(None).await {
                found = nets.iter().any(|n| n.id == *net_id);
            }
        }
        if !found && is_project {
            if let Some(ref conn) = proj_db {
                found = conn
                    .query_row::<i64, _, _>(
                        "SELECT COUNT(*) FROM network_configs WHERE id = ?1",
                        rusqlite::params![net_id],
                        |row| row.get(0),
                    )
                    .map(|c| c > 0)
                    .unwrap_or(false);
            }
        }
        if !found {
            return Err(CoreError::from(format!("网络配置 {} 不存在", net_id)));
        }
    }

    // ===== 数据源模块：解析网络配置为 ConnectionMethod =====
    let network_method = parse_network_method(&input).await?;

    let (conn_id, db) = service
        .connect_with_type(
            None,
            &input.db_type,
            &input.url,
            input.name.clone(),
            connection_type,
            input.project_id.clone(),
            input.description.clone(),
            input.driver_id.clone(),
            input.environment_id.clone(),
            input.auth_config_id.clone(),
            input.network_config_id.clone(),
            input.driver_properties.clone(),
            input.advanced_options.clone(),
            network_method,
        )
        .await?;

    let meta = db.meta();
    let safe_url = ConnectionService::mask_password_in_url(&input.url);

    Ok(ConnectDatabaseResponse {
        conn_id,
        name: input.name.unwrap_or_else(|| safe_url.clone()),
        db_type: input.db_type,
        url: safe_url,
        status: "connected".to_string(),
        meta: meta.into(),
    })
}

/// 解析 network_config_id → ConnectionMethod
///
/// 从 global/project 数据库中加载网络配置，
/// 根据 network_type 将 config JSON 反序列化为对应的连接方式
///
/// ## ID 前缀解析优先级
///
/// | 前缀  | 查找顺序                                          |
/// |-------|--------------------------------------------------|
/// | `G_`  | 1. global.db.network_configs                       |
/// | `P_`  | 1. project.db.network_configs (GP_ 优先)           |
/// | `GP_` | 1. project.db.network_configs（origin='global_snapshot'）|
/// | 无前缀 | 向后兼容：先查 global，再查 project（历史数据）    |
async fn parse_network_method(
    input: &ConnectDatabaseInput,
) -> Result<Option<ConnectionMethod>, CoreError> {
    let Some(ref net_id) = input.network_config_id else {
        return Ok(None);
    };

    let is_project = input.connection_type.as_deref() == Some("project");

    if net_id.starts_with("GP_") {
        if let Some(ref proj_path) = input.project_id {
            let db_path = std::path::Path::new(proj_path)
                .join(".RSmeta")
                .join("project.db");
            if db_path.exists() {
                if let Ok(config_str) = project_query_network_config(&db_path, net_id) {
                    return parse_config_json("unknown", &config_str).await;
                }
            }
        }
        return Ok(None);
    }

    if net_id.starts_with("P_") && is_project {
        if let Some(ref proj_path) = input.project_id {
            let db_path = std::path::Path::new(proj_path)
                .join(".RSmeta")
                .join("project.db");
            if db_path.exists() {
                if let Ok(config_str) = project_query_network_config(&db_path, net_id) {
                    return parse_config_json("unknown", &config_str).await;
                }
            }
        }
        return Ok(None);
    }

    if let Some(gdb) = crate::core::migration::get_global_db_manager() {
        if let Ok(nets) = gdb.list_network_configs(None).await {
            if let Some(net) = nets.iter().find(|n| n.id == *net_id) {
                return parse_config_json(&net.network_type, &net.config).await;
            }
        }
    }

    if is_project {
        if let Some(ref proj_path) = input.project_id {
            let db_path = std::path::Path::new(proj_path)
                .join(".RSmeta")
                .join("project.db");
            if db_path.exists() {
                if let Ok(config_str) = project_query_network_config(&db_path, net_id) {
                    return parse_config_json("unknown", &config_str).await;
                }
            }
        }
    }

    Ok(None)
}

fn project_query_network_config(db_path: &std::path::Path, net_id: &str) -> Result<String, String> {
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.query_row::<String, _, _>(
        "SELECT config FROM network_configs WHERE id = ?1",
        rusqlite::params![net_id],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

/// 根据 network_type 将 config JSON 解析为 ConnectionMethod
async fn parse_config_json(
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
            tracing::warn!(
                network_type = %network_type,
                "未知网络配置类型，将使用直连"
            );
            Ok(None)
        }
    }
}

/// 连接信息响应
#[derive(serde::Serialize, Debug)]
pub struct ConnectionInfoResponse {
    pub id: String,
    pub name: String,
    pub db_type: String,
    pub url: String,
    pub connection_type: String,
    pub project_id: Option<String>,
    pub status: String,
    pub is_active: bool,
    pub created_at_ms: u64,
    pub server_version: Option<String>,
    pub driver_id: Option<String>,
    pub description: Option<String>,
    pub environment_id: Option<String>,
    pub auth_config_id: Option<String>,
    pub network_config_id: Option<String>,
    pub driver_properties: Option<String>,
    pub advanced_options: Option<String>,
}

/// 获取所有连接
#[tauri::command]
pub async fn get_connections() -> Result<Vec<ConnectionInfoResponse>, CoreError> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    let connections = service.list_connections().await;
    let active_id = service.get_active_conn_id().await;

    Ok(connections
        .into_iter()
        .map(|info| {
            let is_active = active_id.as_ref() == Some(&info.id);
            ConnectionInfoResponse {
                id: info.id,
                name: info.name,
                db_type: info.db_type,
                url: info.url,
                connection_type: info.connection_type.to_string(),
                project_id: info.project_id,
                status: "connected".to_string(),
                is_active,
                created_at_ms: info.created_at.elapsed().as_millis() as u64,
                server_version: info.server_version,
                driver_id: info.driver_id,
                environment_id: info.environment_id,
                description: info.description,
                auth_config_id: info.auth_config_id,
                network_config_id: info.network_config_id,
                driver_properties: info.driver_properties,
                advanced_options: info.advanced_options,
            }
        })
        .collect())
}

/// 切换活动连接
#[tauri::command]
pub async fn switch_connection(conn_id: String) -> Result<(), CoreError> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    service.switch_connection(&conn_id).await
}

/// 关闭指定连接
#[tauri::command]
pub async fn close_connection(conn_id: String) -> Result<(), CoreError> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    service.close_connection(&conn_id).await
}

/// 关闭所有连接
#[tauri::command]
pub async fn close_all_connections() -> Result<(), CoreError> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    service.close_all_connections().await
}

/// 获取当前活动连接
#[tauri::command]
pub async fn get_active_connection() -> Result<Option<ConnectionInfoResponse>, CoreError> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    let connections = service.list_connections().await;
    let active_id = service.get_active_conn_id().await;

    Ok(connections
        .into_iter()
        .find(|info| active_id.as_ref() == Some(&info.id))
        .map(|info| ConnectionInfoResponse {
            id: info.id,
            name: info.name,
            db_type: info.db_type,
            url: info.url,
            connection_type: info.connection_type.to_string(),
            project_id: info.project_id,
            status: "connected".to_string(),
            is_active: true,
            created_at_ms: info.created_at.elapsed().as_millis() as u64,
            server_version: info.server_version,
            driver_id: info.driver_id,
            environment_id: info.environment_id,
            description: info.description,
            auth_config_id: info.auth_config_id,
            network_config_id: info.network_config_id,
            driver_properties: info.driver_properties,
            advanced_options: info.advanced_options,
        }))
}

/// 最近连接记录响应
#[derive(serde::Serialize, Debug)]
pub struct RecentConnectionResponse {
    pub name: String,
    pub db_type: String,
    pub url: String,
    pub last_used_at: String,
    pub description: Option<String>,
    pub driver_id: Option<String>,
    pub environment_id: Option<String>,
    pub auth_config_id: Option<String>,
    pub network_config_id: Option<String>,
}

/// 获取最近连接列表
#[tauri::command]
pub async fn get_recent_connections() -> Result<Vec<RecentConnectionResponse>, CoreError> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    let connections = service.get_recent_connections()?;

    Ok(connections
        .into_iter()
        .map(|c| RecentConnectionResponse {
            name: c.name,
            db_type: c.db_type,
            url: c.url,
            last_used_at: c.last_used_at.to_rfc3339(),
            description: c.description,
            driver_id: c.driver_id,
            environment_id: c.environment_id,
            auth_config_id: c.auth_config_id,
            network_config_id: c.network_config_id,
        })
        .collect())
}

/// 删除最近连接记录
#[tauri::command]
pub async fn remove_recent_connection(name: String) -> Result<(), CoreError> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    service
        .remove_recent_connection(&name)
        .map_err(|e| CoreError::from(e.to_string()))
}

/// 连接类型转换请求参数
#[derive(serde::Deserialize, Debug)]
pub struct ConvertConnectionInput {
    pub conn_id: String,
    pub target_type: String,        // "global" 或 "project"
    pub project_id: Option<String>, // 转为项目连接时需要
}

/// 连接类型转换响应
#[derive(serde::Serialize, Debug)]
pub struct ConvertConnectionResponse {
    pub conn_id: String,
    pub connection_type: String,
    pub project_id: Option<String>,
    pub message: String,
}

/// 转换连接类型（全局↔项目）
#[tauri::command]
pub async fn convert_connection_type(
    input: ConvertConnectionInput,
) -> Result<ConvertConnectionResponse, CoreError> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    let new_info = match input.target_type.as_str() {
        "project" => {
            let project_id = input.project_id.ok_or_else(|| {
                "project_id is required when converting to project connection".to_string()
            })?;
            service
                .convert_to_project_connection(&input.conn_id, &project_id)
                .await
                .map_err(|e| e.to_string())?
        }
        "global" => service
            .convert_to_global_connection(&input.conn_id)
            .await
            .map_err(|e| e.to_string())?,
        other => return Err(format!("Invalid target type: {}", other).into()),
    };

    let message = format!(
        "Connection {} converted to {} connection",
        input.conn_id, new_info.connection_type
    );

    Ok(ConvertConnectionResponse {
        conn_id: new_info.id,
        connection_type: new_info.connection_type.to_string(),
        project_id: new_info.project_id,
        message,
    })
}

/// 检测项目中的全局连接
#[tauri::command]
pub async fn detect_global_connections_in_project(
    project_id: String,
) -> Result<Vec<ConnectionInfoResponse>, CoreError> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    let connections = service
        .detect_global_connections_in_project(&project_id)
        .await?;

    let active_id = service.get_active_conn_id().await;

    Ok(connections
        .into_iter()
        .map(|info| {
            let is_active = active_id.as_ref() == Some(&info.id);
            ConnectionInfoResponse {
                id: info.id,
                name: info.name,
                db_type: info.db_type,
                url: info.url,
                connection_type: info.connection_type.to_string(),
                project_id: info.project_id,
                status: "connected".to_string(),
                is_active,
                created_at_ms: info.created_at.elapsed().as_millis() as u64,
                server_version: info.server_version,
                driver_id: info.driver_id,
                environment_id: info.environment_id,
                description: info.description,
                auth_config_id: info.auth_config_id,
                network_config_id: info.network_config_id,
                driver_properties: info.driver_properties,
                advanced_options: info.advanced_options,
            }
        })
        .collect())
}

/// 测试连接响应
#[derive(serde::Serialize, Debug)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
    pub server_version: String,
    pub response_time_ms: u64,
}

/// 测试数据库连接
#[tauri::command]
pub async fn test_connection(
    db_type: String,
    url: String,
) -> Result<TestConnectionResponse, CoreError> {
    use std::time::Instant;

    if url.is_empty() {
        return Err("Database URL cannot be empty".into());
    }

    let start = Instant::now();
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager.clone());

    // 检查是否已有相同 URL 的正式连接
    let all_connections = service.list_connections().await;
    let existing_conn = all_connections.iter().find(|info| info.url == url);

    // 如果已有正式连接，直接返回成功信息，不创建新连接
    if let Some(info) = existing_conn {
        tracing::info!("测试连接：发现已有正式连接（ID={}），直接返回成功", info.id);

        let server_version = manager
            .get_connection(&info.id)
            .await
            .and_then(|db| db.meta().server_version)
            .unwrap_or_else(|| format!("{} (未知版本)", db_type));

        let response_time_ms = start.elapsed().as_millis() as u64;

        return Ok(TestConnectionResponse {
            success: true,
            message: format!("连接成功（已有连接：{}）", info.name),
            server_version,
            response_time_ms,
        });
    }

    // 没有已有连接，创建临时测试连接
    tracing::info!("测试连接：创建临时连接进行测试（URL={}）", url);
    let (conn_id, db) = match service
        .connect(None, &db_type, &url, Some("test_connection".to_string()))
        .await
    {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("测试连接失败：{}", e);
            return Err(format!("连接失败: {}", e).into());
        }
    };

    let server_version = db
        .meta()
        .server_version
        .unwrap_or_else(|| format!("{} (未知版本)", db_type));

    let response_time_ms = start.elapsed().as_millis() as u64;

    // 关键：测试成功后，必须彻底关闭临时连接
    // 1. 先释放 db 的 Arc 引用
    drop(db);
    tracing::info!("测试连接：已释放数据库连接引用（ID={}）", conn_id);

    // 2. 从连接管理器中关闭并移除连接
    if let Err(e) = service.close_connection(&conn_id).await {
        tracing::error!("测试连接：关闭临时连接失败（ID={}）：{}", conn_id, e);
        // 即使关闭失败，也返回成功（因为连接测试本身是成功的）
        return Ok(TestConnectionResponse {
            success: true,
            message: format!("连接成功，但清理临时连接时出现警告: {}", e),
            server_version,
            response_time_ms,
        });
    }

    tracing::info!("测试连接：临时连接已彻底关闭并清理（ID={}）", conn_id);

    Ok(TestConnectionResponse {
        success: true,
        message: "连接成功（临时测试连接已关闭）".to_string(),
        server_version,
        response_time_ms,
    })
}

/// 创建数据库文件请求参数
#[derive(serde::Deserialize, Debug)]
pub struct CreateDatabaseFileInput {
    pub db_type: String, // "sqlite" 或 "duckdb"
    pub file_path: String,
}

/// 创建数据库文件响应
#[derive(serde::Serialize, Debug)]
pub struct CreateDatabaseFileResponse {
    pub file_path: String,
    pub success: bool,
    pub message: String,
}

/// 创建数据库文件（SQLite/DuckDB）
#[tauri::command]
pub async fn create_database_file(
    input: CreateDatabaseFileInput,
) -> Result<CreateDatabaseFileResponse, CoreError> {
    use std::path::Path;

    // 验证数据库类型
    if input.db_type != "sqlite" && input.db_type != "duckdb" {
        return Err(format!(
            "不支持的数据库类型: {}. 仅支持 sqlite 和 duckdb",
            input.db_type
        )
        .into());
    }

    let path = Path::new(&input.file_path);

    // 检查文件是否已存在
    if path.exists() {
        return Err("文件已存在".to_string().into());
    }

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
    }

    // 根据数据库类型创建文件
    match input.db_type.as_str() {
        "sqlite" => {
            // SQLite: 创建空文件，实际连接时会自动初始化
            std::fs::File::create(path).map_err(|e| format!("创建 SQLite 文件失败: {}", e))?;
        }
        "duckdb" => {
            // DuckDB: 需要初始化数据库文件
            // 先创建临时连接来初始化文件，然后立即关闭
            let url = format!("duckdb://{}", input.file_path);
            let manager = get_connection_manager().clone();
            let service = ConnectionService::new(manager);

            tracing::info!("创建 DuckDB 文件：初始化数据库文件（{}）", input.file_path);
            let (conn_id, db) = service
                .connect(None, "duckdb", &url, Some("init_duckdb".to_string()))
                .await
                .map_err(|e| format!("初始化 DuckDB 失败: {}", e))?;

            // 立即关闭连接，释放文件锁
            drop(db);
            service
                .close_connection(&conn_id)
                .await
                .map_err(|e| format!("关闭初始化连接失败: {}", e))?;

            tracing::info!(
                "创建 DuckDB 文件：初始化完成并关闭连接（{}）",
                input.file_path
            );
        }
        _ => unreachable!(),
    }

    Ok(CreateDatabaseFileResponse {
        file_path: input.file_path,
        success: true,
        message: format!("{} 数据库文件创建成功", input.db_type),
    })
}

/// 测试连接配置（不保存）
#[tauri::command]
pub async fn test_connection_config(config: DriverConnectionConfig) -> Result<(), CoreError> {
    let url = config.to_url()?;

    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    // 创建临时测试连接
    tracing::info!(
        "测试连接配置：创建临时连接（driver={}, url={}）",
        config.driver,
        url
    );
    let (conn_id, db) = service
        .connect(None, &config.driver, &url, Some("test".to_string()))
        .await
        .map_err(|e| {
            tracing::error!("测试连接配置失败：{}", e);
            format!("连接失败: {}", e)
        })?;

    // 彻底关闭临时连接
    // 1. 释放 db 的 Arc 引用
    drop(db);
    tracing::info!("测试连接配置：已释放数据库连接引用（ID={}）", conn_id);

    // 2. 从连接管理器中关闭并移除连接
    service.close_connection(&conn_id).await.map_err(|e| {
        tracing::error!("测试连接配置：关闭临时连接失败（ID={}）：{}", conn_id, e);
        format!("关闭测试连接失败: {}", e)
    })?;

    tracing::info!("测试连接配置：临时连接已彻底关闭并清理（ID={}）", conn_id);

    Ok(())
}

/// 全局连接信息响应
#[derive(serde::Serialize, Debug)]
pub struct GlobalConnectionInfoResponse {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub tags: Vec<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub server_version: Option<String>,
    pub driver_id: Option<String>,
    pub environment_id: Option<String>,
    pub auth_config_id: Option<String>,
    pub network_config_id: Option<String>,
    pub driver_properties: Option<String>,
    pub advanced_options: Option<String>,
    pub description: Option<String>,
}

/// 连接池状态响应
#[derive(serde::Serialize, Debug)]
pub struct ConnectionPoolStatusResponse {
    pub conn_id: String,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub max_connections: usize,
    pub min_connections: usize,
    pub connection_timeout_ms: u64,
    pub idle_timeout_ms: u64,
    pub total_connections: usize,
    pub wait_queue_size: usize,
}

/// 获取连接池状态
#[tauri::command]
pub async fn get_connection_pool_status(
    conn_id: String,
) -> Result<ConnectionPoolStatusResponse, CoreError> {
    let manager = get_connection_manager().clone();

    let _connection_info = manager
        .get_connection_info(&conn_id)
        .await
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    let pool_status = match manager.get_connection(&conn_id).await {
        Some(db) => db.pool_status().await,
        None => None,
    };

    match pool_status {
        Some(ps) => Ok(ConnectionPoolStatusResponse {
            conn_id: conn_id.clone(),
            active_connections: ps.active,
            idle_connections: ps.idle,
            max_connections: ps.max_connections,
            min_connections: ps.min_connections,
            connection_timeout_ms: 30000,
            idle_timeout_ms: 300000,
            total_connections: ps.size,
            wait_queue_size: ps.waiting,
        }),
        None => Ok(ConnectionPoolStatusResponse {
            conn_id: conn_id.clone(),
            active_connections: 1,
            idle_connections: 0,
            max_connections: 1,
            min_connections: 1,
            connection_timeout_ms: 30000,
            idle_timeout_ms: 300000,
            total_connections: 1,
            wait_queue_size: 0,
        }),
    }
}

/// 获取所有全局连接
#[tauri::command]
pub async fn get_global_connections() -> Result<Vec<GlobalConnectionInfoResponse>, CoreError> {
    use crate::core::migration::global_init;

    let global_db = global_init::get_global_db_manager()
        .ok_or_else(|| "Global database manager not initialized".to_string())?;

    let connections = global_db
        .get_global_connections()
        .await
        .map_err(|e| format!("获取全局连接失败: {}", e))?;

    Ok(connections
        .into_iter()
        .map(|conn| {
            let tags: Vec<String> = serde_json::from_str(&conn.tags).unwrap_or_else(|_| {
                if conn.tags.is_empty() {
                    vec![]
                } else {
                    vec![conn.tags.clone()]
                }
            });

            let password = conn
                .password
                .and_then(|p| crate::core::crypto::decrypt_password(&p).ok().or(Some(p)));

            GlobalConnectionInfoResponse {
                id: conn.id,
                name: conn.name,
                driver: conn.driver,
                host: conn.host,
                port: conn.port,
                database: conn.database,
                username: conn.username,
                password,
                tags,
                is_active: conn.is_active,
                created_at: conn.created_at,
                updated_at: conn.updated_at,
                server_version: conn.server_version,
                driver_id: conn.driver_id,
                environment_id: conn.environment_id,
                auth_config_id: conn.auth_config_id,
                network_config_id: conn.network_config_id,
                driver_properties: conn.driver_properties,
                advanced_options: conn.advanced_options,
                description: conn.description,
            }
        })
        .collect())
}
