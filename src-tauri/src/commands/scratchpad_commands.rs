use std::path::PathBuf;

use tauri::State;

use crate::core::scratchpad::{
    AnalyzableFile, ExternalReference, ScratchpadEntry, ScratchpadResponse, ScratchpadState, ScratchpadStore,
};

async fn get_store(state: &ScratchpadState) -> Result<ScratchpadStore, String> {
    let guard = state.store.lock().await;
    guard
        .clone()
        .ok_or_else(|| "草稿板存储未初始化，请先打开项目".to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn init_scratchpad_store(
    project_path: String,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<(), String> {
    scratchpad_state.init(PathBuf::from(project_path));
    Ok(())
}

#[tauri::command]
pub async fn list_scratchpad_files(
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<ScratchpadResponse, String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.get_full_response().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_scratchpad_entry(
    name: String,
    is_folder: bool,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<ScratchpadEntry, String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.create_entry(&name, is_folder).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_scratchpad_entry(
    relative_path: String,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<(), String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.delete_entry(&relative_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_scratchpad_entry(
    relative_path: String,
    new_name: String,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<ScratchpadEntry, String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.rename_entry(&relative_path, &new_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_scratchpad_file(
    relative_path: String,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<String, String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.read_file(&relative_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_scratchpad_file(
    relative_path: String,
    content: String,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<(), String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.save_file(&relative_path, &content).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_external_file(
    source_path: String,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<ScratchpadEntry, String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    let source = PathBuf::from(&source_path);
    scratchpad.import_external_file(&source).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_external_reference(
    alias: String,
    path: String,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<ExternalReference, String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    let ref_path = PathBuf::from(&path);
    scratchpad.add_external_reference(alias, ref_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_external_reference(
    alias: String,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<(), String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.remove_external_reference(&alias).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_scratchpad_in_explorer(
    path: String,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<(), String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    let target = PathBuf::from(&path);
    scratchpad.open_in_system_explorer(&target).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_scratchpad_file_size(
    relative_path: String,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<u64, String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.check_file_size(&relative_path).await.map_err(|e| e.to_string())
}

// ==================== Analysis Commands ====================

#[tauri::command]
pub async fn get_analyzable_files(
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<Vec<AnalyzableFile>, String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.get_analyzable_files().await.map_err(|e| e.to_string())
}

// ==================== Trash Commands ====================

#[tauri::command]
pub async fn list_scratchpad_trash(
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<Vec<ScratchpadEntry>, String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.list_trash().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_scratchpad_from_trash(
    trash_name: String,
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<ScratchpadEntry, String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.restore_from_trash(&trash_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn empty_scratchpad_trash(
    scratchpad_state: State<'_, ScratchpadState>,
) -> Result<(), String> {
    let scratchpad = get_store(&scratchpad_state).await?;
    scratchpad.empty_trash().await.map_err(|e| e.to_string())
}
