//! 结果集分析命令
//!
//! 提供结果的二次分析：SQL 过滤、DuckDB 分析、列洞察、持久化存储

use crate::commands::project_commands::ProjectState;
use crate::core::persistence::{InsightStorage, InsightMetaStore};
use crate::core::services::result_service::{
    ResultService, ResultSet, ColumnStats, ColumnInsightFull,
};

/// 重新执行带过滤条件的 SQL
#[derive(serde::Deserialize, Debug)]
pub struct ReExecuteFilterInput {
    pub conn_id: String,
    pub original_sql: String,
    pub where_clause: Option<String>,
    pub order_clause: Option<String>,
}

#[tauri::command]
pub async fn re_execute_with_filter(
    input: ReExecuteFilterInput,
) -> Result<ResultSet, String> {
    let where_clause = input.where_clause.unwrap_or_default();
    let order_clause = input.order_clause.unwrap_or_default();

    ResultService::re_execute_with_filter(
        input.conn_id,
        &input.original_sql,
        &where_clause,
        &order_clause,
    )
    .await
    .map_err(|e| e.to_string())
}

/// DuckDB 分析输入
#[derive(serde::Deserialize, Debug)]
pub struct DuckDbAnalysisInput {
    pub temp_table: String,
    pub sql: String,
    pub columns: Option<Vec<String>>,
    pub rows: Option<Vec<Vec<serde_json::Value>>>,
}

#[tauri::command]
pub async fn execute_duckdb_analysis(
    input: DuckDbAnalysisInput,
) -> Result<ResultSet, String> {
    ResultService::execute_duckdb_analysis(
        &input.temp_table,
        &input.sql,
        input.columns,
        input.rows,
    )
    .map_err(|e| e.to_string())
}

/// 列洞察输入
#[derive(serde::Deserialize, Debug)]
pub struct ColumnInsightInput {
    pub temp_table: String,
    pub column_name: String,
}

/// 获取列统计信息（旧版兼容，返回 ColumnStats）
#[tauri::command]
pub async fn get_column_insights(
    input: ColumnInsightInput,
) -> Result<ColumnStats, String> {
    ResultService::get_column_insights(&input.temp_table, &input.column_name)
        .map_err(|e| e.to_string())
}

/// 获取列全量洞察（统计 + 样本 + 直方图）
#[tauri::command]
pub async fn get_column_insight_full(
    input: ColumnInsightInput,
) -> Result<ColumnInsightFull, String> {
    ResultService::get_column_insight_full(&input.temp_table, &input.column_name)
        .map_err(|e| e.to_string())
}

/// 创建 DuckDB 临时表输入
#[derive(serde::Deserialize, Debug)]
pub struct CreateTempTableInput {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

/// 从已有数据创建 DuckDB 临时表，返回表名
#[tauri::command]
pub async fn create_duckdb_temp_table(
    input: CreateTempTableInput,
) -> Result<String, String> {
    ResultService::create_duckdb_temp_table(&input.columns, &input.rows)
        .map_err(|e| e.to_string())
}

// ═══════════════════ 持久化命令 ═══════════════════

/// 保存列洞察快照输入
#[derive(serde::Deserialize, Debug)]
pub struct SaveInsightSnapshotInput {
    pub temp_table: String,
    pub column_name: String,
    pub conn_id: Option<String>,
    pub db_name: Option<String>,
    pub schema_name: Option<String>,
    pub table_name: Option<String>,
}

/// 保存列洞察快照到项目持久化存储
#[tauri::command]
pub async fn save_column_insight_snapshot(
    input: SaveInsightSnapshotInput,
    state: tauri::State<'_, ProjectState>,
) -> Result<String, String> {
    let store_guard = state.store.lock().await;
    let store = store_guard.as_ref().ok_or("No project store available")?;
    let db = &store.db_manager;

    let insight_store = InsightStorage::new(db.duckdb_conn());
    let meta_store = InsightMetaStore::new(db.sqlite_pool());

    let insight = ResultService::get_column_insight_full(&input.temp_table, &input.column_name)
        .map_err(|e| e.to_string())?;

    let row_count = insight.stats.total_count as i64;
    let start = std::time::Instant::now();
    let elapsed = start.elapsed().as_millis() as i64;

    let (_snapshot_id, version_id) = ResultService::save_column_insight_snapshot(
        &insight,
        input.conn_id.as_deref(),
        input.db_name.as_deref(),
        input.schema_name.as_deref(),
        input.table_name.as_deref(),
        Some(row_count),
        Some(elapsed),
        &insight_store,
        &meta_store,
    ).await.map_err(|e| e.to_string())?;

    Ok(version_id)
}

/// 获取列洞察历史版本输入
#[derive(serde::Deserialize, Debug)]
pub struct InsightHistoryInput {
    pub column_name: String,
}

/// 获取列洞察所有历史版本
#[tauri::command]
pub async fn get_column_insight_history(
    input: InsightHistoryInput,
    state: tauri::State<'_, ProjectState>,
) -> Result<Vec<crate::core::persistence::InsightVersionEntry>, String> {
    let store_guard = state.store.lock().await;
    let store = store_guard.as_ref().ok_or("No project store available")?;
    let db = &store.db_manager;

    let insight_store = InsightStorage::new(db.duckdb_conn());

    ResultService::get_column_insight_history(&input.column_name, &insight_store)
        .await
        .map_err(|e| e.to_string())
}
