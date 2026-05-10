//! 连接相关命令
//!
//! 处理数据库连接的创建、管理、关闭等操作

use crate::core::driver::DriverConnectionConfig;
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
) -> Result<ConnectDatabaseResponse, String> {
    if input.url.is_empty() {
        return Err("Database URL cannot be empty".into());
    }

    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    // 解析连接类型
    let connection_type = match input.connection_type.as_deref() {
        Some("global") | None => ConnectionType::Global,
        Some("project") => ConnectionType::Project,
        Some(other) => return Err(format!("Invalid connection type: {}", other)),
    };

    // 项目连接必须有 project_id
    if connection_type == ConnectionType::Project && input.project_id.is_none() {
        return Err("project_id is required for project connections".into());
    }

    let (conn_id, db) = service
        .connect_with_type(
            None,
            &input.db_type,
            &input.url,
            input.name.clone(),
            connection_type,
            input.project_id.clone(),
        )
        .await
        .map_err(|e| e.to_string())?;

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
}

/// 获取所有连接
#[tauri::command]
pub async fn get_connections() -> Result<Vec<ConnectionInfoResponse>, String> {
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
            }
        })
        .collect())
}

/// 切换活动连接
#[tauri::command]
pub async fn switch_connection(conn_id: String) -> Result<(), String> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    service
        .switch_connection(&conn_id)
        .await
        .map_err(|e| e.to_string())
}

/// 关闭指定连接
#[tauri::command]
pub async fn close_connection(conn_id: String) -> Result<(), String> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    service
        .close_connection(&conn_id)
        .await
        .map_err(|e| e.to_string())
}

/// 关闭所有连接
#[tauri::command]
pub async fn close_all_connections() -> Result<(), String> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    service
        .close_all_connections()
        .await
        .map_err(|e| e.to_string())
}

/// 获取当前活动连接
#[tauri::command]
pub async fn get_active_connection() -> Result<Option<ConnectionInfoResponse>, String> {
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
        }))
}

/// 最近连接记录响应
#[derive(serde::Serialize, Debug)]
pub struct RecentConnectionResponse {
    pub name: String,
    pub db_type: String,
    pub url: String,
    pub last_used_at: String,
}

/// 获取最近连接列表
#[tauri::command]
pub async fn get_recent_connections() -> Result<Vec<RecentConnectionResponse>, String> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    let connections = service
        .get_recent_connections()
        .map_err(|e| e.to_string())?;

    Ok(connections
        .into_iter()
        .map(|c| RecentConnectionResponse {
            name: c.name,
            db_type: c.db_type,
            url: c.url,
            last_used_at: c.last_used_at.to_rfc3339(),
        })
        .collect())
}

/// 删除最近连接记录
#[tauri::command]
pub async fn remove_recent_connection(name: String) -> Result<(), String> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    service
        .remove_recent_connection(&name)
        .map_err(|e| e.to_string())
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
) -> Result<ConvertConnectionResponse, String> {
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
        other => return Err(format!("Invalid target type: {}", other)),
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
) -> Result<Vec<ConnectionInfoResponse>, String> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    let connections = service
        .detect_global_connections_in_project(&project_id)
        .await
        .map_err(|e| e.to_string())?;

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
) -> Result<TestConnectionResponse, String> {
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
            .map(|db| db.meta().server_version)
            .flatten()
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
            return Err(format!("连接失败: {}", e));
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
) -> Result<CreateDatabaseFileResponse, String> {
    use std::path::Path;

    // 验证数据库类型
    if input.db_type != "sqlite" && input.db_type != "duckdb" {
        return Err(format!(
            "不支持的数据库类型: {}. 仅支持 sqlite 和 duckdb",
            input.db_type
        ));
    }

    let path = Path::new(&input.file_path);

    // 检查文件是否已存在
    if path.exists() {
        return Err("文件已存在".to_string());
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
pub async fn test_connection_config(config: DriverConnectionConfig) -> Result<(), String> {
    let url = config.to_url().map_err(|e| e.to_string())?;

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
) -> Result<ConnectionPoolStatusResponse, String> {
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
pub async fn get_global_connections() -> Result<Vec<GlobalConnectionInfoResponse>, String> {
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
            }
        })
        .collect())
}
