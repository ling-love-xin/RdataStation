use std::sync::Arc;
use uuid::Uuid;

use crate::adapters::tauri::state::AppState;
use crate::core::mock::persistence::{
    MockGenerationColumn, MockGenerationDetail, MockGenerationStore, MockGenerationTask,
    MockTemplateColumn, MockUserTemplate,
};
use crate::core::persistence::project_db::ProjectSqlitePool;

async fn open_store(project_path: &str) -> Result<MockGenerationStore, String> {
    let db_path = format!("{}/.RSMETA/project.db", project_path);
    let pool = ProjectSqlitePool::new(
        std::path::PathBuf::from(&db_path),
        1,
    )
    .await
    .map_err(|e| format!("Failed to open project database: {}", e))?;

    Ok(MockGenerationStore::new(Arc::new(pool)))
}

#[tauri::command]
pub async fn save_mock_generation_task(
    _state: tauri::State<'_, AppState>,
    project_path: String,
    task: MockGenerationTask,
    columns: Vec<MockGenerationColumn>,
) -> Result<String, String> {
    let store = open_store(&project_path).await?;
    store
        .save_task(&task, &columns)
        .await
        .map_err(|e| format!("Failed to save task: {}", e))?;
    Ok(task.id.clone())
}

#[tauri::command]
pub async fn get_mock_generation_history(
    _state: tauri::State<'_, AppState>,
    project_path: String,
    limit: Option<u32>,
) -> Result<Vec<MockGenerationTask>, String> {
    let store = open_store(&project_path).await?;
    store
        .get_history(limit.unwrap_or(20))
        .await
        .map_err(|e| format!("Failed to query history: {}", e))
}

#[tauri::command]
pub async fn get_mock_generation_detail(
    _state: tauri::State<'_, AppState>,
    project_path: String,
    task_id: String,
) -> Result<MockGenerationDetail, String> {
    let store = open_store(&project_path).await?;
    store
        .get_detail(&task_id)
        .await
        .map_err(|e| format!("Failed to query detail: {}", e))
}

#[tauri::command]
pub async fn delete_mock_generation_task(
    _state: tauri::State<'_, AppState>,
    project_path: String,
    task_id: String,
) -> Result<(), String> {
    let store = open_store(&project_path).await?;
    store
        .delete_task(&task_id)
        .await
        .map_err(|e| format!("Failed to delete task: {}", e))
}

#[tauri::command]
pub async fn save_mock_template(
    _state: tauri::State<'_, AppState>,
    project_path: String,
    template: MockUserTemplate,
    columns: Vec<MockTemplateColumn>,
) -> Result<String, String> {
    let store = open_store(&project_path).await?;
    store
        .save_template(&template, &columns)
        .await
        .map_err(|e| format!("Failed to save template: {}", e))?;
    Ok(template.id.clone())
}

#[tauri::command]
pub async fn get_mock_templates(
    _state: tauri::State<'_, AppState>,
    project_path: String,
) -> Result<Vec<MockUserTemplate>, String> {
    let store = open_store(&project_path).await?;
    store
        .get_templates()
        .await
        .map_err(|e| format!("Failed to query templates: {}", e))
}

#[tauri::command]
pub async fn get_mock_template_detail(
    _state: tauri::State<'_, AppState>,
    project_path: String,
    template_id: String,
) -> Result<(MockUserTemplate, Vec<MockTemplateColumn>), String> {
    let store = open_store(&project_path).await?;
    store
        .get_template_detail(&template_id)
        .await
        .map_err(|e| format!("Failed to query template detail: {}", e))
}