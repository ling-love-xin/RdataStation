//! 分析资源管理相关命令
//!
//! 处理分析资源（连接、表、文件）的增删改查、文件夹、标签等操作

use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use crate::core::persistence::{
    AnalyticsResourceStore, AnalyticsResource, AnalyticsFolder, AnalyticsTag, AnalyticsRecycleItem,
    CreateResourceRequest, CreateFolderRequest, CreateTagRequest, ListResourcesOutput,
};
use crate::commands::project_commands::ProjectState;

/// 分析资源状态
pub struct AnalyticsResourceState {
    pub store: Arc<Mutex<Option<AnalyticsResourceStore>>>,
}

impl AnalyticsResourceState {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
        }
    }
}

// ==================== Resource Commands ====================

/// 创建资源
#[tauri::command]
pub async fn create_analytics_resource(
    input: CreateResourceRequest,
    analytics_state: State<'_, AnalyticsResourceState>,
    _project_state: State<'_, ProjectState>,
) -> Result<AnalyticsResource, String> {
    tracing::info!(resource_type = %input.resource_type, name = %input.name, "Creating analytics resource");
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    let resource = store.create_resource(input).await.map_err(|e| e.to_string())?;
    
    tracing::info!(resource_id = %resource.id, "Analytics resource created successfully");
    
    Ok(resource)
}

/// 更新资源
#[tauri::command]
pub async fn update_analytics_resource(
    id: String,
    input: CreateResourceRequest,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<AnalyticsResource, String> {
    tracing::info!(resource_id = %id, "Updating analytics resource");
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    let resource = store.update_resource(&id, input).await.map_err(|e| e.to_string())?;
    
    tracing::info!(resource_id = %resource.id, "Analytics resource updated successfully");
    
    Ok(resource)
}

/// 获取资源详情
#[tauri::command]
pub async fn get_analytics_resource(
    id: String,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<AnalyticsResource, String> {
    tracing::debug!(resource_id = %id, "Getting analytics resource");
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    let resource = store.get_resource_by_id(&id).await.map_err(|e| e.to_string())?;
    
    Ok(resource)
}

/// 列出资源
#[derive(serde::Deserialize, Debug)]
pub struct ListAnalyticsResourcesInput {
    pub scope: Option<String>,
    pub resource_type: Option<String>,
    pub folder_id: Option<String>,
}

#[tauri::command]
pub async fn list_analytics_resources(
    input: ListAnalyticsResourcesInput,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<Vec<AnalyticsResource>, String> {
    tracing::debug!(
        scope = ?input.scope,
        resource_type = ?input.resource_type,
        folder_id = ?input.folder_id,
        "Listing analytics resources"
    );
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    let resources = store.list_resources(
        input.scope.as_deref(),
        input.resource_type.as_deref(),
        input.folder_id.as_deref(),
    ).await.map_err(|e| e.to_string())?;
    
    Ok(resources)
}

/// 删除资源（软删除，移入回收站）
#[tauri::command]
pub async fn delete_analytics_resource(
    id: String,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<(), String> {
    tracing::info!(resource_id = %id, "Deleting analytics resource");

    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;

    store.delete_resource(&id).await.map_err(|e| e.to_string())?;

    tracing::info!(resource_id = %id, "Analytics resource deleted successfully");

    Ok(())
}

/// 批量删除资源
#[tauri::command]
pub async fn batch_delete_analytics_resources(
    ids: Vec<String>,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<(), String> {
    tracing::info!(count = ids.len(), "Batch deleting analytics resources");

    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;

    for id in &ids {
        store.delete_resource(id).await.map_err(|e| e.to_string())?;
    }

    tracing::info!(count = ids.len(), "Batch delete completed successfully");

    Ok(())
}

/// 克隆资源
#[tauri::command]
pub async fn clone_analytics_resource(
    id: String,
    new_name: Option<String>,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<AnalyticsResource, String> {
    tracing::info!(resource_id = %id, "Cloning analytics resource");

    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;

    let cloned = store.clone_resource(&id, new_name.as_deref()).await.map_err(|e| e.to_string())?;

    tracing::info!(new_resource_id = %cloned.id, "Analytics resource cloned successfully");

    Ok(cloned)
}

/// 分页列出资源
#[derive(serde::Deserialize, Debug)]
pub struct ListResourcesInput {
    pub scope: Option<String>,
    pub resource_type: Option<String>,
    pub folder_id: Option<String>,
    pub search: Option<String>,
    pub pagination: Option<PaginationInput>,
    pub sort: Option<SortInput>,
}

#[derive(serde::Deserialize, Debug)]
pub struct PaginationInput {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(serde::Deserialize, Debug)]
pub struct SortInput {
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[tauri::command]
pub async fn list_analytics_resources_paginated(
    input: ListResourcesInput,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<ListResourcesOutput, String> {
    tracing::debug!(
        scope = ?input.scope,
        resource_type = ?input.resource_type,
        "Listing analytics resources paginated"
    );

    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;

    let page = input.pagination.as_ref().and_then(|p| p.page).unwrap_or(1);
    let page_size = input.pagination.as_ref().and_then(|p| p.page_size).unwrap_or(20);
    let sort_by = input.sort.as_ref().and_then(|s| s.sort_by.clone());
    let sort_order = input.sort.as_ref().and_then(|s| s.sort_order.clone());

    let result = store.list_resources_paginated(
        input.scope.as_deref(),
        input.resource_type.as_deref(),
        input.folder_id.as_deref(),
        input.search.as_deref(),
        page,
        page_size,
        sort_by.as_deref(),
        sort_order.as_deref(),
    ).await.map_err(|e| e.to_string())?;

    Ok(result)
}

// ==================== Folder Commands ====================

/// 创建文件夹
#[tauri::command]
pub async fn create_analytics_folder(
    input: CreateFolderRequest,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<AnalyticsFolder, String> {
    tracing::info!(name = %input.name, scope = %input.scope, "Creating analytics folder");
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    let folder = store.create_folder(input).await.map_err(|e| e.to_string())?;
    
    tracing::info!(folder_id = %folder.id, "Analytics folder created successfully");
    
    Ok(folder)
}

/// 获取文件夹详情
#[tauri::command]
pub async fn get_analytics_folder(
    id: String,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<AnalyticsFolder, String> {
    tracing::debug!(folder_id = %id, "Getting analytics folder");
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    let folder = store.get_folder_by_id(&id).await.map_err(|e| e.to_string())?;
    
    Ok(folder)
}

/// 列出文件夹
#[derive(serde::Deserialize, Debug)]
pub struct ListAnalyticsFoldersInput {
    pub scope: Option<String>,
    pub parent_folder_id: Option<String>,
}

#[tauri::command]
pub async fn list_analytics_folders(
    input: ListAnalyticsFoldersInput,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<Vec<AnalyticsFolder>, String> {
    tracing::debug!(
        scope = ?input.scope,
        parent_folder_id = ?input.parent_folder_id,
        "Listing analytics folders"
    );
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    let folders = store.list_folders(
        input.scope.as_deref(),
        input.parent_folder_id.as_deref(),
    ).await.map_err(|e| e.to_string())?;
    
    Ok(folders)
}

/// 将资源添加到文件夹
#[derive(serde::Deserialize, Debug)]
pub struct AddResourceToFolderInput {
    pub resource_id: String,
    pub folder_id: String,
}

#[tauri::command]
pub async fn add_analytics_resource_to_folder(
    input: AddResourceToFolderInput,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<(), String> {
    tracing::info!(
        resource_id = %input.resource_id,
        folder_id = %input.folder_id,
        "Adding resource to folder"
    );
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    store.add_resource_to_folder(&input.resource_id, &input.folder_id).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 从文件夹移除资源
#[tauri::command]
pub async fn remove_analytics_resource_from_folder(
    input: AddResourceToFolderInput,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<(), String> {
    tracing::info!(
        resource_id = %input.resource_id,
        folder_id = %input.folder_id,
        "Removing resource from folder"
    );
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    store.remove_resource_from_folder(&input.resource_id, &input.folder_id).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

// ==================== Tag Commands ====================

/// 创建标签
#[tauri::command]
pub async fn create_analytics_tag(
    input: CreateTagRequest,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<AnalyticsTag, String> {
    tracing::info!(name = %input.name, scope = %input.scope, "Creating analytics tag");
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    let tag = store.create_tag(input).await.map_err(|e| e.to_string())?;
    
    tracing::info!(tag_id = %tag.id, "Analytics tag created successfully");
    
    Ok(tag)
}

/// 列出标签
#[derive(serde::Deserialize, Debug)]
pub struct ListAnalyticsTagsInput {
    pub scope: Option<String>,
}

#[tauri::command]
pub async fn list_analytics_tags(
    input: ListAnalyticsTagsInput,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<Vec<AnalyticsTag>, String> {
    tracing::debug!(scope = ?input.scope, "Listing analytics tags");
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    let tags = store.list_tags(input.scope.as_deref()).await.map_err(|e| e.to_string())?;
    
    Ok(tags)
}

/// 将标签添加到资源
#[derive(serde::Deserialize, Debug)]
pub struct AddTagToResourceInput {
    pub resource_id: String,
    pub tag_id: String,
}

#[tauri::command]
pub async fn add_analytics_tag_to_resource(
    input: AddTagToResourceInput,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<(), String> {
    tracing::info!(
        resource_id = %input.resource_id,
        tag_id = %input.tag_id,
        "Adding tag to resource"
    );
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    store.add_tag_to_resource(&input.resource_id, &input.tag_id).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 从资源移除标签
#[tauri::command]
pub async fn remove_analytics_tag_from_resource(
    input: AddTagToResourceInput,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<(), String> {
    tracing::info!(
        resource_id = %input.resource_id,
        tag_id = %input.tag_id,
        "Removing tag from resource"
    );
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    store.remove_tag_from_resource(&input.resource_id, &input.tag_id).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

// ==================== Recycle Bin Commands ====================

/// 获取回收站列表
#[tauri::command]
pub async fn get_analytics_recycle_bin(
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<Vec<AnalyticsRecycleItem>, String> {
    tracing::debug!("Getting analytics recycle bin");
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    let items = store.get_recycle_items().await.map_err(|e| e.to_string())?;
    
    Ok(items)
}

/// 从回收站恢复资源
#[tauri::command]
pub async fn restore_analytics_resource_from_recycle(
    recycle_id: String,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<AnalyticsResource, String> {
    tracing::info!(recycle_id = %recycle_id, "Restoring analytics resource from recycle bin");
    
    let store = {
        let analytics_guard = analytics_state.store.lock().await;
        analytics_guard.as_ref().ok_or_else(|| {
            "分析资源存储未初始化".to_string()
        })?.clone()
    };
    
    let resource = store.restore_from_recycle(&recycle_id).await.map_err(|e| e.to_string())?;
    
    tracing::info!(resource_id = %resource.id, "Analytics resource restored successfully");
    
    Ok(resource)
}

/// 永久删除资源
#[tauri::command]
pub async fn permanent_delete_analytics_resource(
    recycle_id: String,
    analytics_state: State<'_, AnalyticsResourceState>,
) -> Result<(), String> {
    tracing::info!(recycle_id = %recycle_id, "Permanent deleting analytics resource");
    
    let analytics_guard = analytics_state.store.lock().await;
    let store = analytics_guard.as_ref().ok_or_else(|| {
        "分析资源存储未初始化".to_string()
    })?;
    
    store.permanent_delete(&recycle_id).await.map_err(|e| e.to_string())?;
    
    tracing::info!(recycle_id = %recycle_id, "Analytics resource permanently deleted");
    
    Ok(())
}

// ==================== Initialization ====================

/// 初始化分析资源存储
#[tauri::command]
pub async fn init_analytics_resource_store(
    analytics_state: State<'_, AnalyticsResourceState>,
    project_state: State<'_, ProjectState>,
) -> Result<(), String> {
    tracing::info!("Initializing analytics resource store");
    
    let project_guard = project_state.store.lock().await;
    let project_store = project_guard.as_ref().ok_or_else(|| {
        "项目存储未初始化".to_string()
    })?;
    
    let mut analytics_guard = analytics_state.store.lock().await;
    
    // 检查是否已初始化
    if analytics_guard.as_ref().is_some() {
        tracing::info!("Analytics resource store already initialized");
        return Ok(());
    }
    
    let store = AnalyticsResourceStore::new(project_store.db_manager.sqlite_pool());
    *analytics_guard = Some(store);
    
    tracing::info!("Analytics resource store initialized successfully");
    
    Ok(())
}
