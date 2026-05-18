use std::sync::Arc;

use crate::commands::project_commands::ProjectState;
use crate::core::error::CoreError;
use crate::core::migration::get_global_db_manager;
use crate::core::persistence::auth_store;
use crate::core::persistence::driver_store::{DataSourceType, Driver, DriverFile};
use crate::core::persistence::env_store;
use crate::core::persistence::network_store;
use crate::core::persistence::project_connection_store::ProjectConnectionStore;
use crate::core::persistence::project_db::ProjectDatabaseManager;
use crate::core::services::driver_service::{self, DriverService};

fn get_driver_service() -> Result<DriverService, CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    Ok(DriverService::new(db))
}

/// 驱动列表响应（含缺失驱动自检）
#[derive(serde::Serialize, Debug)]
pub struct DriverListResponse {
    pub drivers: Vec<Driver>,
    pub missing: Vec<driver_service::MissingDriver>,
}

/// 检查项目缺失的外部驱动文件（预置辅助函数，供后续功能扩展使用）
#[allow(dead_code)]
async fn check_missing_drivers_impl(
    _project_id: &str,
) -> Result<Vec<driver_service::MissingDriver>, CoreError> {
    let global_db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;

    let all_drivers = global_db.get_all_drivers().await?;

    let mut missing = Vec::new();
    for driver in &all_drivers {
        if driver.driver_kind == "native" {
            continue;
        }
        let version = driver.version.as_deref().unwrap_or("1.0.0");
        let installed = global_db.is_driver_installed(&driver.id, version).await.unwrap_or(false);
        if !installed {
            missing.push(driver_service::MissingDriver {
                driver_id: driver.id.clone(),
                driver_name: driver.name.clone(),
                download_url: driver.download_url.clone().unwrap_or_default(),
            });
        }
    }
    Ok(missing)
}

/// 获取数据源类型目录（供前端数据源树渲染）
#[tauri::command]
pub async fn get_data_source_types(
    category: Option<String>,
) -> Result<Vec<DataSourceType>, CoreError> {
    let service = get_driver_service()?;
    let types = service.get_data_source_types().await?;
    let filtered: Vec<DataSourceType> = types
        .into_iter()
        .filter(|t| category.as_ref().map_or(true, |c| t.category == *c))
        .collect();
    Ok(filtered)
}

/// 获取驱动列表，传入 project_path 时自动检测缺失驱动
#[tauri::command]
pub async fn get_available_drivers(
    project_path: Option<String>,
    state: tauri::State<'_, ProjectState>,
) -> Result<DriverListResponse, CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    let drivers = db.get_all_drivers().await?;
    let mut missing = Vec::new();

    if let Some(path) = project_path {
        let proj_db = get_project_db_manager(&path, &state).await?;
        let store = ProjectConnectionStore::new(proj_db);
        let enabled_drivers = store.list_enabled_drivers().await?;

        for driver in &drivers {
            if !enabled_drivers.contains(&driver.id) {
                continue;
            }
            if driver.driver_kind != "native" {
                let version = driver.version.as_deref().unwrap_or("0.0.0");
                let installed = db
                    .is_driver_installed(&driver.id, version)
                    .await
                    .unwrap_or(false);
                if !installed {
                    missing.push(driver_service::MissingDriver {
                        driver_id: driver.id.clone(),
                        driver_name: driver.name.clone(),
                        download_url: driver.download_url.clone().unwrap_or_default(),
                    });
                }
            }
        }
    }

    Ok(DriverListResponse { drivers, missing })
}

/// 驱动详情响应（含可用性状态）
#[derive(serde::Serialize, Debug)]
pub struct DriverDetailResponse {
    pub driver: Driver,
    pub availability: String,
}

/// 获取驱动详情（含 config_schema + 可用性状态）
#[tauri::command]
pub async fn get_driver_detail(
    driver_id: String,
    project_path: Option<String>,
    state: tauri::State<'_, ProjectState>,
) -> Result<DriverDetailResponse, CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    let driver = db
        .get_driver(&driver_id)
        .await?
        .ok_or_else(|| CoreError::from(format!("驱动 {} 不存在", driver_id)))?;

    let mut availability = "ready".to_string();
    if let Some(path) = project_path {
        let proj_db = get_project_db_manager(&path, &state).await?;
        let store = ProjectConnectionStore::new(proj_db);
        if !store.is_driver_enabled(&driver_id).await.unwrap_or(false) {
            availability = "not_enabled".to_string();
        } else if driver.driver_kind != "native" {
            let version = driver.version.as_deref().unwrap_or("0.0.0");
            if !db
                .is_driver_installed(&driver_id, version)
                .await
                .unwrap_or(false)
            {
                availability = "not_installed".to_string();
            }
        }
    }

    Ok(DriverDetailResponse {
        driver,
        availability,
    })
}

/// 安装外部驱动文件（下载到本机并注册）
#[tauri::command]
pub async fn install_driver(driver_id: String) -> Result<(), CoreError> {
    let service = get_driver_service()?;
    service.install_driver(&driver_id).await
}

/// 列出某驱动在本机已安装的所有文件版本
#[tauri::command]
pub async fn list_driver_files(driver_id: String) -> Result<Vec<DriverFile>, CoreError> {
    let service = get_driver_service()?;
    service.list_driver_files(&driver_id).await
}

/// 列出所有环境
#[tauri::command]
pub async fn list_environments() -> Result<Vec<env_store::Environment>, CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.list_environments().await
}

/// 创建环境
#[tauri::command]
pub async fn create_environment(env: env_store::Environment) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.create_environment(&env).await
}

/// 更新环境
#[tauri::command]
pub async fn update_environment(env: env_store::Environment) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.update_environment(&env).await
}

/// 删除环境
#[tauri::command]
pub async fn delete_environment(id: String) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.delete_environment(&id).await
}

/// 列出环境策略
#[tauri::command]
pub async fn list_environment_policies(
    environment_id: String,
) -> Result<Vec<env_store::EnvironmentPolicy>, CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.list_environment_policies(&environment_id).await
}

/// 创建环境策略
#[tauri::command]
pub async fn create_environment_policy(
    policy: env_store::EnvironmentPolicy,
) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.create_environment_policy(&policy).await
}

/// 更新环境策略
#[tauri::command]
pub async fn update_environment_policy(
    policy: env_store::EnvironmentPolicy,
) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.update_environment_policy(&policy).await
}

/// 删除环境策略
#[tauri::command]
pub async fn delete_environment_policy(id: String) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.delete_environment_policy(&id).await
}

/// 列出认证配置
#[tauri::command]
pub async fn list_auth_configs(
    auth_type: Option<String>,
) -> Result<Vec<auth_store::AuthConfig>, CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.list_auth_configs(auth_type.as_deref()).await
}

/// 创建认证配置
#[tauri::command]
pub async fn create_auth_config(ac: auth_store::AuthConfig) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.create_auth_config(&ac).await
}

/// 删除认证配置
#[tauri::command]
pub async fn delete_auth_config(id: String) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.delete_auth_config(&id).await
}

/// 列出网络配置
#[tauri::command]
pub async fn list_network_configs(
    network_type: Option<String>,
) -> Result<Vec<network_store::NetworkConfig>, CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.list_network_configs(network_type.as_deref()).await
}

/// 创建网络配置
#[tauri::command]
pub async fn create_network_config(nc: network_store::NetworkConfig) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.create_network_config(&nc).await
}

/// 更新网络配置
#[tauri::command]
pub async fn update_network_config(nc: network_store::NetworkConfig) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.update_network_config(&nc).await
}

/// 删除网络配置
#[tauri::command]
pub async fn delete_network_config(id: String) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.delete_network_config(&id).await
}

/// 获取全局驱动目录（按 category / driver_kind 过滤，供"驱动市场"展示）
#[tauri::command]
pub async fn get_all_drivers_catalog(
    category: Option<String>,
    driver_kind: Option<String>,
) -> Result<Vec<Driver>, CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;

    let drivers = if let Some(cat) = category {
        let types = db.list_data_source_types().await?;
        let mut result = Vec::new();
        for dt in types {
            if dt.category == cat && dt.enabled {
                let mut d = db.get_drivers_by_type(&dt.id).await?;
                result.append(&mut d);
            }
        }
        result
    } else {
        db.get_all_drivers().await?
    };

    let filtered: Vec<Driver> = drivers
        .into_iter()
        .filter(|d| {
            driver_kind
                .as_ref()
                .map_or(true, |k| d.driver_kind == *k)
        })
        .collect();

    Ok(filtered)
}

async fn get_project_db_manager(
    project_path: &str,
    state: &tauri::State<'_, ProjectState>,
) -> Result<Arc<ProjectDatabaseManager>, CoreError> {
    let guard = state.store.lock().await;
    let store = guard
        .as_ref()
        .ok_or_else(|| CoreError::from("没有打开的项目".to_string()))?;
    if store.db_manager.project_path().to_string_lossy() != project_path {
        return Err(CoreError::from("项目路径不匹配".to_string()));
    }
    Ok(store.db_manager.clone())
}

/// 为项目启用一个驱动
#[tauri::command]
pub async fn enable_driver_for_project(
    driver_id: String,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<(), CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    let store = ProjectConnectionStore::new(db_manager);
    store.enable_driver(&driver_id).await
}

/// 为项目禁用一个驱动
#[tauri::command]
pub async fn disable_driver_for_project(
    driver_id: String,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<(), CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    let store = ProjectConnectionStore::new(db_manager);
    store.disable_driver(&driver_id).await
}

/// 列出项目中所有已启用的驱动
#[tauri::command]
pub async fn list_enabled_project_drivers(
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<Vec<String>, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    let store = ProjectConnectionStore::new(db_manager);
    store.list_enabled_drivers().await
}