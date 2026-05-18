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
use uuid::Uuid;

use chrono::Utc;

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

/// 获取数据源类型目录（供前端数据源树渲染）
#[tauri::command]
pub async fn get_data_source_types(
    category: Option<String>,
) -> Result<Vec<DataSourceType>, CoreError> {
    let service = get_driver_service()?;
    let types = service.get_data_source_types().await?;
    let filtered: Vec<DataSourceType> = types
        .into_iter()
        .filter(|t| category.as_ref().is_none_or(|c| t.category == *c))
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
pub async fn create_environment(mut env: env_store::Environment) -> Result<(), CoreError> {
    if env.id.is_empty() {
        env.id = format!("env_{}", Uuid::new_v4().to_string().replace('-', "_"));
    }
    if env.created_at.is_empty() {
        env.created_at = Utc::now().to_rfc3339();
    }
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
    mut policy: env_store::EnvironmentPolicy,
) -> Result<(), CoreError> {
    if policy.id.is_empty() {
        policy.id = format!("env_pol_{}", Uuid::new_v4().to_string().replace('-', "_"));
    }
    if policy.created_at.is_empty() {
        policy.created_at = Utc::now().to_rfc3339();
    }
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
pub async fn create_auth_config(mut ac: auth_store::AuthConfig) -> Result<(), CoreError> {
    let now = Utc::now().to_rfc3339();
    if ac.id.is_empty() {
        ac.id = format!("auth_{}", Uuid::new_v4().to_string().replace('-', "_"));
    }
    if ac.created_at.is_empty() {
        ac.created_at = now.clone();
    }
    if ac.updated_at.is_empty() {
        ac.updated_at = now;
    }
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

/// 更新认证配置
#[tauri::command]
pub async fn update_auth_config(ac: auth_store::AuthConfig) -> Result<(), CoreError> {
    let db = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;
    db.update_auth_config(&ac).await
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
pub async fn create_network_config(mut nc: network_store::NetworkConfig) -> Result<(), CoreError> {
    let now = Utc::now().to_rfc3339();
    if nc.id.is_empty() {
        nc.id = format!("net_{}", Uuid::new_v4().to_string().replace('-', "_"));
    }
    if nc.created_at.is_empty() {
        nc.created_at = now.clone();
    }
    if nc.updated_at.is_empty() {
        nc.updated_at = now;
    }
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
                .is_none_or(|k| d.driver_kind == *k)
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

// ========== 项目级环境命令 ==========

/// 在指定项目中创建环境
#[tauri::command]
pub async fn project_create_environment(
    name: String,
    description: Option<String>,
    color: Option<String>,
    sort_order: Option<i32>,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<env_store::Environment, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.create_project_environment(&name, description.as_deref(), color.as_deref(), sort_order).await
}

/// 列出指定项目中的所有环境
#[tauri::command]
pub async fn project_list_environments(
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<Vec<env_store::Environment>, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.list_project_environments().await
}

/// 更新指定项目中的环境
#[tauri::command]
pub async fn project_update_environment(
    id: String,
    name: Option<String>,
    description: Option<String>,
    color: Option<String>,
    sort_order: Option<i32>,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<bool, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.update_project_environment(&id, name.as_deref(), description.as_deref(), color.as_deref(), sort_order).await?;
    Ok(true)
}

/// 从指定项目中删除环境
#[tauri::command]
pub async fn project_delete_environment(
    id: String,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<bool, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.delete_project_environment(&id).await?;
    Ok(true)
}

// ========== 项目级环境策略命令 ==========

/// 在指定项目中创建环境策略
#[tauri::command]
pub async fn project_create_environment_policy(
    environment_id: String,
    policy_type: String,
    policy_config: Option<String>,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<env_store::EnvironmentPolicy, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.create_project_environment_policy(&environment_id, &policy_type, policy_config.as_deref()).await
}

/// 列出指定项目中某环境的所有策略
#[tauri::command]
pub async fn project_list_environment_policies(
    environment_id: String,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<Vec<env_store::EnvironmentPolicy>, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.list_project_environment_policies(&environment_id).await
}

/// 更新指定项目中的环境策略
#[tauri::command]
pub async fn project_update_environment_policy(
    id: String,
    policy_config: Option<String>,
    enabled: Option<bool>,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<bool, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.update_project_environment_policy(&id, policy_config.as_deref(), enabled).await?;
    Ok(true)
}

/// 从指定项目中删除环境策略
#[tauri::command]
pub async fn project_delete_environment_policy(
    id: String,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<bool, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.delete_project_environment_policy(&id).await?;
    Ok(true)
}

// ========== 项目级认证配置命令 ==========

/// 在指定项目中创建认证配置
#[tauri::command]
pub async fn project_create_auth_config(
    name: Option<String>,
    auth_type: String,
    auth_data: String,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<auth_store::AuthConfig, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.create_project_auth_config(name.as_deref(), &auth_type, &auth_data).await
}

/// 列出指定项目中的所有认证配置
#[tauri::command]
pub async fn project_list_auth_configs(
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<Vec<auth_store::AuthConfig>, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.list_project_auth_configs().await
}

/// 从指定项目中删除认证配置
#[tauri::command]
pub async fn project_delete_auth_config(
    id: String,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<bool, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.delete_project_auth_config(&id).await?;
    Ok(true)
}

// ========== 项目级网络配置命令 ==========

/// 在指定项目中创建网络配置
#[tauri::command]
pub async fn project_create_network_config(
    name: Option<String>,
    network_type: String,
    config: String,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<network_store::NetworkConfig, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.create_project_network_config(name.as_deref(), &network_type, &config).await
}

/// 列出指定项目中的所有网络配置
#[tauri::command]
pub async fn project_list_network_configs(
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<Vec<network_store::NetworkConfig>, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.list_project_network_configs().await
}

/// 更新指定项目中的网络配置
#[tauri::command]
pub async fn project_update_network_config(
    id: String,
    name: Option<String>,
    config: Option<String>,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<bool, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.update_project_network_config(&id, name.as_deref(), config.as_deref()).await?;
    Ok(true)
}

/// 从指定项目中删除网络配置
#[tauri::command]
pub async fn project_delete_network_config(
    id: String,
    project_path: String,
    state: tauri::State<'_, ProjectState>,
) -> Result<bool, CoreError> {
    let db_manager = get_project_db_manager(&project_path, &state).await?;
    db_manager.delete_project_network_config(&id).await?;
    Ok(true)
}