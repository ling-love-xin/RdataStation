//! 项目存储相关命令
//!
//! 处理项目级数据的持久化操作

use std::sync::Arc;

use crate::commands::project_commands::ProjectState;
use crate::core::error::CoreError;
use crate::core::persistence::project_connection_store::ProjectConnection;
use crate::core::persistence::project_db::ProjectDatabaseManager;

// ==================== Project Connection Commands ====================

/// 项目连接响应（与 GlobalConnection 结构一致）
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProjectConnectionResponse {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub database: Option<String>,
    pub schema_name: Option<String>,
    pub username: Option<String>,
    pub options: Option<String>,
    pub tags: Option<String>,
    pub use_duckdb_fed: bool,
    pub metadata_path: Option<String>,
    pub is_active: bool,
    pub server_version: Option<String>,
    pub description: Option<String>,
    pub driver_id: Option<String>,
    pub environment_id: Option<String>,
    pub auth_config_id: Option<String>,
    pub network_config_id: Option<String>,
    pub driver_properties: Option<String>,
    pub advanced_options: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ProjectConnection> for ProjectConnectionResponse {
    fn from(conn: ProjectConnection) -> Self {
        Self {
            id: conn.id,
            name: conn.name,
            driver: conn.driver,
            host: conn.host,
            port: conn.port,
            database: conn.database,
            schema_name: conn.schema_name,
            username: conn.username,
            options: conn.options,
            tags: conn.tags,
            use_duckdb_fed: conn.use_duckdb_fed,
            metadata_path: conn.metadata_path,
            is_active: conn.is_active,
            server_version: conn.server_version,
            description: conn.description,
            driver_id: conn.driver_id,
            environment_id: conn.environment_id,
            auth_config_id: conn.auth_config_id,
            network_config_id: conn.network_config_id,
            driver_properties: conn.driver_properties,
            advanced_options: conn.advanced_options,
            created_at: conn.created_at,
            updated_at: conn.updated_at,
        }
    }
}

/// 创建项目连接请求
#[derive(serde::Deserialize, Debug)]
pub struct CreateProjectConnectionInput {
    pub project_path: String,
    pub name: String,
    pub driver: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub database: Option<String>,
    pub schema_name: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub options: Option<String>,
    pub tags: Option<String>,
    pub use_duckdb_fed: Option<bool>,
    pub metadata_path: Option<String>,
    pub description: Option<String>,
    pub driver_id: Option<String>,
    pub environment_id: Option<String>,
    pub auth_config_id: Option<String>,
    pub network_config_id: Option<String>,
    pub driver_properties: Option<String>,
    pub advanced_options: Option<String>,
}

/// 获取 ProjectDatabaseManager 的辅助函数
async fn get_db_manager(
    project_path: &str,
    state: tauri::State<'_, ProjectState>,
) -> Result<Arc<ProjectDatabaseManager>, CoreError> {
    let guard = state.store.lock().await;
    let store = guard
        .as_ref()
        .ok_or_else(|| CoreError::from("项目存储未初始化，请先调用 init_project_store"))?;

    if store.db_manager.project_path().to_string_lossy() != project_path {
        return Err(CoreError::from("项目路径不匹配"));
    }

    Ok(store.db_manager.clone())
}

/// 创建项目连接
#[tauri::command]
pub async fn create_project_connection(
    input: CreateProjectConnectionInput,
    state: tauri::State<'_, ProjectState>,
) -> Result<ProjectConnectionResponse, CoreError> {
    let db_manager = get_db_manager(&input.project_path, state).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let id = format!("project-{}-{}", input.driver, uuid::Uuid::new_v4());

    let conn = ProjectConnection {
        id: id.clone(),
        name: input.name,
        driver: input.driver,
        host: input.host,
        port: input.port,
        database: input.database,
        schema_name: input.schema_name,
        username: input.username,
        password_encrypted: input.password,
        options: input.options,
        tags: input.tags.or_else(|| Some("[\"project\"]".to_string())),
        use_duckdb_fed: input.use_duckdb_fed.unwrap_or(false),
        metadata_path: input.metadata_path,
        is_active: true,
        server_version: None,
        description: input.description,
        driver_id: input.driver_id,
        environment_id: input.environment_id,
        auth_config_id: input.auth_config_id,
        network_config_id: input.network_config_id,
        driver_properties: input.driver_properties,
        advanced_options: input.advanced_options,
        created_at: now.clone(),
        updated_at: now,
    };

    let connection_store =
        crate::core::persistence::project_connection_store::ProjectConnectionStore::new(db_manager);
    connection_store
        .create_connection(&conn)
        .await
        .map_err(|e| CoreError::from(e.to_string()))?;

    Ok(conn.into())
}

/// 获取项目所有连接
#[tauri::command]
pub async fn get_project_connections(
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<Vec<ProjectConnectionResponse>, CoreError> {
    let db_manager = get_db_manager(&project_path, state).await?;

    let connection_store =
        crate::core::persistence::project_connection_store::ProjectConnectionStore::new(db_manager);
    let connections = connection_store
        .get_all_connections()
        .await
        .map_err(|e| CoreError::from(e.to_string()))?;

    Ok(connections.into_iter().map(|c| c.into()).collect())
}

/// 获取单个项目连接
#[tauri::command]
pub async fn get_project_connection(
    project_path: String,
    connection_id: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<Option<ProjectConnectionResponse>, CoreError> {
    let db_manager = get_db_manager(&project_path, state).await?;

    let connection_store =
        crate::core::persistence::project_connection_store::ProjectConnectionStore::new(db_manager);
    let conn = connection_store
        .get_connection(&connection_id)
        .await
        .map_err(|e| CoreError::from(e.to_string()))?;

    Ok(conn.map(|c| c.into()))
}

/// 更新项目连接
#[tauri::command]
pub async fn update_project_connection(
    project_path: String,
    connection: ProjectConnectionResponse,
    state: tauri::State<'_, ProjectState>,
) -> Result<(), CoreError> {
    let db_manager = get_db_manager(&project_path, state).await?;

    let conn = ProjectConnection {
        id: connection.id,
        name: connection.name,
        driver: connection.driver,
        host: connection.host,
        port: connection.port,
        database: connection.database,
        schema_name: connection.schema_name,
        username: connection.username,
        password_encrypted: None,
        options: connection.options,
        tags: connection.tags,
        use_duckdb_fed: connection.use_duckdb_fed,
        metadata_path: connection.metadata_path,
        is_active: connection.is_active,
        server_version: connection.server_version,
        description: connection.description,
        driver_id: connection.driver_id,
        environment_id: connection.environment_id,
        auth_config_id: connection.auth_config_id,
        network_config_id: connection.network_config_id,
        driver_properties: connection.driver_properties,
        advanced_options: connection.advanced_options,
        created_at: connection.created_at,
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let connection_store =
        crate::core::persistence::project_connection_store::ProjectConnectionStore::new(db_manager);
    connection_store
        .update_connection(&conn)
        .await
        .map_err(|e| CoreError::from(e.to_string()))?;

    Ok(())
}

/// 删除项目连接
#[tauri::command]
pub async fn delete_project_connection(
    project_path: String,
    connection_id: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<(), CoreError> {
    let db_manager = get_db_manager(&project_path, state).await?;

    let connection_store =
        crate::core::persistence::project_connection_store::ProjectConnectionStore::new(db_manager);
    connection_store
        .delete_connection(&connection_id)
        .await
        .map_err(|e| CoreError::from(e.to_string()))?;

    Ok(())
}

/// 更新项目连接状态
#[tauri::command]
pub async fn update_project_connection_status(
    project_path: String,
    connection_id: String,
    is_active: bool,
    state: tauri::State<'_, ProjectState>,
) -> Result<(), CoreError> {
    let db_manager = get_db_manager(&project_path, state).await?;

    let connection_store =
        crate::core::persistence::project_connection_store::ProjectConnectionStore::new(db_manager);
    connection_store
        .update_connection_status(&connection_id, is_active)
        .await
        .map_err(|e| CoreError::from(e.to_string()))?;

    Ok(())
}

/// 搜索项目连接
#[tauri::command]
pub async fn search_project_connections(
    project_path: String,
    query: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<Vec<ProjectConnectionResponse>, CoreError> {
    let db_manager = get_db_manager(&project_path, state).await?;

    let connection_store =
        crate::core::persistence::project_connection_store::ProjectConnectionStore::new(db_manager);
    let connections = connection_store
        .search_connections(&query)
        .await
        .map_err(|e| CoreError::from(e.to_string()))?;

    Ok(connections.into_iter().map(|c| c.into()).collect())
}
