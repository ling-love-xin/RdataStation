
//! 插件相关命令
//!
//! 处理插件的安装、卸载、项目级管理等操作

use crate::core::error::CoreError;
use crate::core::migration::get_global_db_manager;
use crate::core::plugin::manager::{PluginManager, PluginInfo};
use crate::core::services::plugin_bridge::get_plugin_bridge;
use crate::core::services::plugin_service::{
    PluginService, PluginWithStatus
};
use crate::core::persistence::plugin_store::{
    Plugin, ProjectUsedPlugin, ProjectPluginConfig
};
use crate::core::persistence::project_connection_store::ProjectConnectionStore;
use crate::core::persistence::project_db::ProjectDatabaseManager;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginStatus {
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub plugin_type: String,
    pub status: String,
    pub config: Option&lt;serde_json::Value&gt;,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallPluginRequest {
    pub code: String,
    pub name: String,
    pub version: String,
    pub author: Option&lt;String&gt;,
    pub description: Option&lt;String&gt;,
    pub repo_url: Option&lt;String&gt;,
    pub plugin_type: String,
    pub manifest_json: Option&lt;String&gt;,
    pub install_path: String,
    pub is_builtin: Option&lt;bool&gt;,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnablePluginInProjectRequest {
    pub plugin_code: String,
    pub plugin_version: String,
    pub required: Option&lt;bool&gt;,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadPluginRequest {
    pub plugin_id: String,
    pub plugin_path: String,
}

// 项目状态包装，便于命令访问
pub(crate) use crate::commands::project_commands::ProjectState;

// ==================== 通用插件命令 ====================

#[tauri::command]
pub async fn plugin_db_query(
    plugin_id: String,
    conn_id: String,
    sql: String,
    timeout: Option&lt;u64&gt;,
) -&gt; Result&lt;crate::api::dto::QueryResult, CoreError&gt; {
    let _ = timeout;
    let bridge = get_plugin_bridge();
    let result = bridge.query(&amp;plugin_id, &amp;conn_id, &amp;sql).await?;
    Ok(result)
}

#[tauri::command]
pub async fn plugin_db_metadata(
    plugin_id: String,
    conn_id: String,
    catalog: String,
    schema: String,
    kind: String,
) -&gt; Result&lt;serde_json::Value, CoreError&gt; {
    let bridge = get_plugin_bridge();
    let result = bridge
        .metadata(&amp;plugin_id, &amp;conn_id, &amp;catalog, &amp;schema, &amp;kind)
        .await?;
    Ok(result)
}

#[tauri::command]
pub async fn plugin_list(
    plugin_manager: State&lt;'_, Arc&lt;PluginManager&gt;&gt;,
) -&gt; Result&lt;Vec&lt;PluginInfo&gt;, CoreError&gt; {
    Ok(plugin_manager.list_plugins())
}

#[tauri::command]
pub async fn plugin_load(
    plugin_manager: State&lt;'_, Arc&lt;PluginManager&gt;&gt;,
    plugin_id: String,
    plugin_path: String,
) -&gt; Result&lt;PluginInfo, CoreError&gt; {
    use std::path::PathBuf;
    let path = PathBuf::from(plugin_path);
    let info = plugin_manager.load_plugin(&amp;plugin_id, &amp;path)?;
    Ok(info)
}

#[tauri::command]
pub async fn plugin_activate(
    plugin_manager: State&lt;'_, Arc&lt;PluginManager&gt;&gt;,
    plugin_id: String,
) -&gt; Result&lt;(), CoreError&gt; {
    plugin_manager.activate_plugin(&amp;plugin_id)?;
    Ok(())
}

#[tauri::command]
pub async fn plugin_deactivate(
    plugin_manager: State&lt;'_, Arc&lt;PluginManager&gt;&gt;,
    plugin_id: String,
) -&gt; Result&lt;(), CoreError&gt; {
    plugin_manager.deactivate_plugin(&amp;plugin_id)?;
    Ok(())
}

#[tauri::command]
pub async fn plugin_unload(
    plugin_manager: State&lt;'_, Arc&lt;PluginManager&gt;&gt;,
    plugin_id: String,
) -&gt; Result&lt;(), CoreError&gt; {
    plugin_manager.unload_plugin(&amp;plugin_id)?;
    Ok(())
}

#[tauri::command]
pub async fn plugin_get_status(
    plugin_manager: State&lt;'_, Arc&lt;PluginManager&gt;&gt;,
    plugin_id: String,
) -&gt; Result&lt;PluginStatus, CoreError&gt; {
    let plugins = plugin_manager.list_plugins();
    let plugin = plugins.into_iter().find(|p| p.id == plugin_id)
        .ok_or_else(|| CoreError::from("Plugin not found"))?;

    Ok(PluginStatus {
        plugin_id: plugin.id,
        name: plugin.name,
        version: plugin.version,
        plugin_type: format!("{:?}", plugin.kind),
        status: format!("{:?}", plugin.state),
        config: None,
    })
}

#[tauri::command]
pub async fn plugin_add_directory(
    plugin_manager: State&lt;'_, Arc&lt;PluginManager&gt;&gt;,
    directory: String,
) -&gt; Result&lt;Vec&lt;PluginInfo&gt;, CoreError&gt; {
    use std::path::PathBuf;
    let path = PathBuf::from(directory);
    let mut manager = plugin_manager.as_ref().clone();
    
    let manager_ptr = Arc::make_mut(&amp;mut manager);
    manager_ptr.add_plugin_dir(path);
    
    let discovered = manager_ptr.scan_plugins()?;
    Ok(discovered)
}

// ==================== 插件安装管理 ====================

#[tauri::command]
pub async fn plugin_get_all_installed() -&gt; Result&lt;Vec&lt;Plugin&gt;, CoreError&gt; {
    let db_manager = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database manager not available"))?;
    let service = PluginService::new(db_manager);
    service.get_installed_plugins().await
}

#[tauri::command]
pub async fn plugin_get_with_status(
    project_state: State&lt;'_, ProjectState&gt;,
) -&gt; Result&lt;Vec&lt;PluginWithStatus&gt;, CoreError&gt; {
    let db_manager = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database manager not available"))?;
    let service = PluginService::new(db_manager);
    
    let store = project_state.store.lock().await;
    let project_db = store.as_ref().and_then(|s| s.project_db());
    
    match project_db {
        Some(db) =&gt; {
            let conn_store = ProjectConnectionStore::new(db);
            service.get_plugins_with_status(Some(&amp;conn_store)).await
        },
        None =&gt; service.get_plugins_with_status(None).await
    }
}

#[tauri::command]
pub async fn plugin_get(plugin_id: String) -&gt; Result&lt;Option&lt;Plugin&gt;, CoreError&gt; {
    let db_manager = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database manager not available"))?;
    let service = PluginService::new(db_manager);
    service.get_plugin(&amp;plugin_id).await
}

#[tauri::command]
pub async fn plugin_install(request: InstallPluginRequest) -&gt; Result&lt;Plugin, CoreError&gt; {
    let db_manager = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database manager not available"))?;
    let service = PluginService::new(db_manager);
    service.install_plugin(
        request.code,
        request.name,
        request.version,
        request.author,
        request.description,
        request.repo_url,
        request.plugin_type,
        request.manifest_json,
        request.install_path,
        request.is_builtin,
    ).await
}

#[tauri::command]
pub async fn plugin_enable(plugin_id: String) -&gt; Result&lt;(), CoreError&gt; {
    let db_manager = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database manager not available"))?;
    let service = PluginService::new(db_manager);
    service.enable_plugin(&amp;plugin_id).await
}

#[tauri::command]
pub async fn plugin_disable(plugin_id: String) -&gt; Result&lt;(), CoreError&gt; {
    let db_manager = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database manager not available"))?;
    let service = PluginService::new(db_manager);
    service.disable_plugin(&amp;plugin_id).await
}

#[tauri::command]
pub async fn plugin_uninstall(plugin_id: String) -&gt; Result&lt;(), CoreError&gt; {
    let db_manager = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database manager not available"))?;
    let service = PluginService::new(db_manager);
    service.uninstall_plugin(&amp;plugin_id).await
}

// ==================== 项目插件管理 ====================

#[tauri::command]
pub async fn project_plugin_enable(
    project_state: State&lt;'_, ProjectState&gt;,
    request: EnablePluginInProjectRequest,
) -&gt; Result&lt;(), CoreError&gt; {
    let db_manager = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database manager not available"))?;
    let service = PluginService::new(db_manager);
    
    let store = project_state.store.lock().await;
    let project_db = store.as_ref().and_then(|s| s.project_db())
        .ok_or_else(|| CoreError::from("Project not open"))?;
    let conn_store = ProjectConnectionStore::new(project_db);
    
    service.enable_plugin_in_project(
        &amp;conn_store,
        request.plugin_code,
        request.plugin_version,
        request.required,
    ).await
}

#[tauri::command]
pub async fn project_plugin_disable(
    project_state: State&lt;'_, ProjectState&gt;,
    plugin_code: String,
    plugin_version: String,
) -&gt; Result&lt;(), CoreError&gt; {
    let db_manager = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database manager not available"))?;
    let service = PluginService::new(db_manager);
    
    let store = project_state.store.lock().await;
    let project_db = store.as_ref().and_then(|s| s.project_db())
        .ok_or_else(|| CoreError::from("Project not open"))?;
    let conn_store = ProjectConnectionStore::new(project_db);
    
    service.disable_plugin_in_project(&amp;conn_store, plugin_code, plugin_version).await
}

#[tauri::command]
pub async fn project_plugin_remove(
    project_state: State&lt;'_, ProjectState&gt;,
    plugin_code: String,
    plugin_version: String,
) -&gt; Result&lt;(), CoreError&gt; {
    let store = project_state.store.lock().await;
    let project_db = store.as_ref().and_then(|s| s.project_db())
        .ok_or_else(|| CoreError::from("Project not open"))?;
    let conn_store = ProjectConnectionStore::new(project_db);
    
    crate::core::persistence::plugin_store::project_remove_plugin(&amp;conn_store, &amp;plugin_code, &amp;plugin_version).await?;
    Ok(())
}

#[tauri::command]
pub async fn project_plugin_list(
    project_state: State&lt;'_, ProjectState&gt;,
) -&gt; Result&lt;Vec&lt;ProjectUsedPlugin&gt;, CoreError&gt; {
    let store = project_state.store.lock().await;
    let project_db = store.as_ref().and_then(|s| s.project_db())
        .ok_or_else(|| CoreError::from("Project not open"))?;
    let conn_store = ProjectConnectionStore::new(project_db);
    
    crate::core::persistence::plugin_store::project_get_plugins(&amp;conn_store).await
}

#[tauri::command]
pub async fn project_plugin_set_config(
    project_state: State&lt;'_, ProjectState&gt;,
    plugin_code: String,
    plugin_version: String,
    key: String,
    value: Option&lt;String&gt;,
) -&gt; Result&lt;(), CoreError&gt; {
    let config = ProjectPluginConfig {
        plugin_code,
        plugin_version,
        key,
        value,
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    
    let store = project_state.store.lock().await;
    let project_db = store.as_ref().and_then(|s| s.project_db())
        .ok_or_else(|| CoreError::from("Project not open"))?;
    let conn_store = ProjectConnectionStore::new(project_db);
    
    crate::core::persistence::plugin_store::project_set_plugin_config(&amp;conn_store, &amp;config).await?;
    Ok(())
}

#[tauri::command]
pub async fn project_plugin_get_configs(
    project_state: State&lt;'_, ProjectState&gt;,
    plugin_code: String,
    plugin_version: String,
) -&gt; Result&lt;Vec&lt;ProjectPluginConfig&gt;, CoreError&gt; {
    let store = project_state.store.lock().await;
    let project_db = store.as_ref().and_then(|s| s.project_db())
        .ok_or_else(|| CoreError::from("Project not open"))?;
    let conn_store = ProjectConnectionStore::new(project_db);
    
    crate::core::persistence::plugin_store::project_get_plugin_configs(&amp;conn_store, &amp;plugin_code, &amp;plugin_version).await
}

// ==================== 启动相关 ====================

#[tauri::command]
pub async fn plugin_load_enabled_on_startup() -&gt; Result&lt;Vec&lt;Plugin&gt;, CoreError&gt; {
    let db_manager = get_global_db_manager()
        .ok_or_else(|| CoreError::from("Global database manager not available"))?;
    let service = PluginService::new(db_manager);
    service.load_enabled_plugins_on_startup().await
}
