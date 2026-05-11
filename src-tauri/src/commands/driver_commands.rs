//! 驱动相关命令
//!
//! 处理数据库驱动的查询、连接配置等操作

use crate::commands::connection_commands::DataSourceMetaResponse;
use crate::core::error::CoreError;
use crate::core::get_connection_manager;
use crate::core::services::ConnectionService;
use crate::core::{get_all_drivers, get_driver, DriverConnectionConfig, DriverDescriptor};

/// 创建连接响应
#[derive(serde::Serialize, Debug)]
pub struct CreateConnectionResponse {
    pub conn_id: String,
    pub name: String,
    pub driver: String,
    pub url: String,
    pub meta: DataSourceMetaResponse,
}

// ==================== Driver Commands ====================

/// 获取所有支持的驱动列表
#[tauri::command]
pub async fn get_drivers() -> Result<Vec<DriverDescriptor>, CoreError> {
    Ok(get_all_drivers())
}

/// 获取指定驱动的描述符
#[tauri::command]
pub async fn get_driver_info(driver_id: String) -> Result<Option<DriverDescriptor>, CoreError> {
    Ok(get_driver(&driver_id))
}

/// 使用 ConnectionConfig 创建连接
#[derive(serde::Deserialize, Debug)]
pub struct CreateConnectionInput {
    pub config: DriverConnectionConfig,
}

#[tauri::command]
pub async fn create_connection(
    input: CreateConnectionInput,
) -> Result<CreateConnectionResponse, CoreError> {
    let url = input
        .config
        .to_url()
        .map_err(|e| CoreError::from(e.to_string()))?;

    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    let name = input
        .config
        .name
        .clone()
        .unwrap_or_else(|| format!("{}-{}", input.config.driver, url));

    let (conn_id, db) = service
        .connect(None, &input.config.driver, &url, Some(name.clone()))
        .await
        .map_err(|e| CoreError::from(e.to_string()))?;

    let meta = db.meta();

    Ok(CreateConnectionResponse {
        conn_id,
        name,
        driver: input.config.driver,
        url,
        meta: meta.into(),
    })
}

/// 使用 ConnectionConfig 创建连接（新方法）
#[derive(serde::Deserialize, Debug)]
pub struct CreateConnectionWithConfigInput {
    pub config: DriverConnectionConfig,
}

#[tauri::command]
pub async fn create_connection_with_config(
    input: CreateConnectionWithConfigInput,
) -> Result<CreateConnectionResponse, CoreError> {
    let url = input
        .config
        .to_url()
        .map_err(|e| CoreError::from(e.to_string()))?;

    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager);

    let name = input
        .config
        .name
        .clone()
        .unwrap_or_else(|| format!("{}-{}", input.config.driver, url));

    let (conn_id, db) = service
        .connect(None, &input.config.driver, &url, Some(name.clone()))
        .await
        .map_err(|e| CoreError::from(e.to_string()))?;

    let meta = db.meta();

    Ok(CreateConnectionResponse {
        conn_id,
        name,
        driver: input.config.driver,
        url,
        meta: meta.into(),
    })
}

/// 更新连接配置
#[derive(serde::Deserialize, Debug)]
pub struct UpdateConnectionInput {
    pub id: String,
    pub config: DriverConnectionConfig,
}

#[tauri::command]
pub async fn update_connection(input: UpdateConnectionInput) -> Result<(), CoreError> {
    let manager = get_connection_manager().clone();
    let service = ConnectionService::new(manager.clone());

    if manager.get_connection_info(&input.id).await.is_some() {
        service
            .close_connection(&input.id)
            .await
            .map_err(|e| CoreError::from(e.to_string()))?;
    }

    let url = input
        .config
        .to_url()
        .map_err(|e| CoreError::from(e.to_string()))?;
    let name = input
        .config
        .name
        .clone()
        .unwrap_or_else(|| format!("{}-{}", input.config.driver, url));

    service
        .connect(Some(input.id), &input.config.driver, &url, Some(name))
        .await
        .map_err(|e| CoreError::from(e.to_string()))?;

    Ok(())
}
