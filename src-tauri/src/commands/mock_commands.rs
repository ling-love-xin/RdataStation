use crate::core::mock::{
    ColumnDef, ColumnMappingResponse, ImportSchemaInput, MockConfig, MockEngine, MockExportInput,
    MockGenerateResult, MockHistoryRecord, MockPersistAssetInput, MockPersistAssetResult,
    MockSaveToScratchpadInput, ScenarioTemplate,
};
use crate::core::models::QueryResult;
use tauri::Emitter;
use tauri::Manager;

/// 生成 Mock 数据（带进度事件）
///
/// 接收 `MockConfig` 配置，分批次在 DuckDB 内存临时表中生成数据，
/// 通过 `mock:generate-progress` 事件向前端推送进度。
/// 返回 `MockGenerateResult` 含预览数据和耗时。
#[tauri::command]
pub async fn mock_generate(
    config: MockConfig,
    app: tauri::AppHandle,
) -> Result<MockGenerateResult, String> {
    let app_clone = app.clone();
    MockEngine::generate_with_progress(config, move |current, total| {
        let _ = app_clone.emit(
            "mock:generate-progress",
            serde_json::json!({
                "current": current,
                "total": total
            }),
        );
    })
    .await
    .map_err(|e| e.to_string())
}

/// 预览临时表数据
///
/// 从 DuckDB 内存临时表读取前 `limit` 行，
/// 以 `QueryResult` 格式返回（含列名和 Arrow RecordBatch 行数据）。
#[tauri::command]
pub async fn mock_preview(table_name: String, limit: usize) -> Result<QueryResult, String> {
    MockEngine::preview(&table_name, limit).map_err(|e| e.to_string())
}

/// 导出临时表到文件
///
/// 支持 CSV / Parquet / Xlsx / SQL INSERT 格式。
/// 使用 DuckDB COPY 命令原生导出，高性能大文件写入。
#[tauri::command]
pub async fn mock_export(input: MockExportInput) -> Result<String, String> {
    MockEngine::export(
        &input.temp_table_name,
        &input.format,
        input.output_path.as_deref(),
        input.table_name.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// 单列名→生成器智能映射
///
/// 分析列名语义和数据类型，推荐最合适的 GeneratorConfig。
/// 返回映射结果含置信度 (high/medium/low) 和示例值。
#[tauri::command]
pub async fn mock_map_column(
    column_name: String,
    data_type: String,
) -> Result<ColumnMappingResponse, String> {
    MockEngine::map_column(&column_name, &data_type).map_err(|e| e.to_string())
}

/// 批量列名→生成器映射
///
/// 对多个列同时执行智能映射推断，返回 `Vec<ColumnMappingResponse>`。
#[tauri::command]
pub async fn mock_map_columns_batch(
    columns: Vec<(String, String)>,
) -> Result<Vec<ColumnMappingResponse>, String> {
    MockEngine::map_columns_batch(columns).map_err(|e| e.to_string())
}

/// 列出内置场景模板
///
/// 返回 6 个预定义模板（电商/HR/博客/金融/社交媒体/企业通讯录），
/// 每模板含表结构和推荐 GeneratorConfig。
#[tauri::command]
pub async fn mock_list_templates() -> Result<Vec<ScenarioTemplate>, String> {
    MockEngine::list_templates().map_err(|e| e.to_string())
}

/// 从真实数据库导入表结构
///
/// 读取指定连接中目标表的列元信息，
/// 自动推断每列的生成器配置。
#[tauri::command]
pub async fn mock_import_schema(input: ImportSchemaInput) -> Result<Vec<ColumnDef>, String> {
    MockEngine::import_schema(&input).map_err(|e| e.to_string())
}

/// 应用场景模板（获取详情）
///
/// 按模板 ID 查找并返回完整模板定义。
#[tauri::command]
pub async fn mock_apply_template(template_id: String) -> Result<ScenarioTemplate, String> {
    MockEngine::apply_template(&template_id).map_err(|e| e.to_string())
}

/// 保存临时表到草稿本
///
/// 将 DuckDB 内存临时表导出到 AppData 目录的 `scratchpad/mock/` 子目录，
/// 支持 CSV / Parquet / Xlsx / SQL INSERT 格式。
#[tauri::command]
pub async fn mock_save_to_scratchpad(
    input: MockSaveToScratchpadInput,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let scratchpad_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("scratchpad")
        .join("mock");

    std::fs::create_dir_all(&scratchpad_dir)
        .map_err(|e| format!("Failed to create scratchpad/mock dir: {}", e))?;

    let scratchpad_path = scratchpad_dir
        .to_str()
        .ok_or_else(|| "Invalid scratchpad dir path".to_string())?;

    MockEngine::save_to_scratchpad(&input.temp_table_name, &input.format, scratchpad_path)
        .map_err(|e| e.to_string())
}

/// 持久化临时表为项目资产
///
/// 将 DuckDB 内存临时表写入项目 DuckDB 文件 (`analytics/data.duckdb`)，
/// 返回表名、行数、列数。
#[tauri::command]
pub async fn mock_persist_as_asset(
    input: MockPersistAssetInput,
) -> Result<MockPersistAssetResult, String> {
    let (table_name, row_count, column_count) =
        MockEngine::persist_as_asset(&input.temp_table_name, &input.name)
            .map_err(|e| e.to_string())?;

    Ok(MockPersistAssetResult {
        table_name,
        row_count,
        column_count,
    })
}

/// 查询生成历史记录
///
/// 返回最近 N 条生成配置快照（`MockHistoryRecord`），
/// 含表名、行数、种子、配置 JSON 和时间戳。
#[tauri::command]
pub async fn mock_get_history(limit: usize) -> Result<Vec<MockHistoryRecord>, String> {
    MockEngine::get_history(limit).map_err(|e| e.to_string())
}

/// 清空生成历史记录
///
/// 删除 DuckDB 历史表全部记录，返回被清空的记录数。
#[tauri::command]
pub async fn mock_clear_history() -> Result<usize, String> {
    MockEngine::clear_history().map_err(|e| e.to_string())
}

/// 基于历史记录重新生成
///
/// 按历史记录 ID 还原生成配置，触发新的数据生成。
/// 返回 `MockGenerateResult` 含全新预览数据和耗时。
#[tauri::command]
pub async fn mock_re_generate(history_id: String) -> Result<MockGenerateResult, String> {
    MockEngine::re_generate(&history_id)
        .await
        .map_err(|e| e.to_string())
}
