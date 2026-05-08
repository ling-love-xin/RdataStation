//! 结果集分析命令
//!
//! 提供结果的二次分析：SQL 过滤、DuckDB 分析、列洞察、持久化存储

use crate::commands::project_commands::ProjectState;
use crate::core::persistence::{InsightStorage, InsightMetaStore};
use crate::core::services::result_service::{
    ResultService, ResultSet, ColumnStats, ColumnInsightFull, TableProfile,
    QualityScore, TableQuality,
};
use crate::core::insight::schema_analyzer::{SchemaAnalyzer, SchemaInsightReport};

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

// ═══════════════ 单元格编辑持久化 ═══════════════

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CellUpdateResult {
    pub success: bool,
    pub affected_rows: usize,
    pub message: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct CellUpdateInput {
    pub conn_id: String,
    pub table_name: String,
    pub column_name: String,
    pub new_value: serde_json::Value,
    pub row_identity: std::collections::HashMap<String, serde_json::Value>,
}

#[tauri::command]
pub async fn save_cell_update(
    input: CellUpdateInput,
) -> Result<CellUpdateResult, String> {
    use crate::core::get_connection_manager;
    use crate::core::services::sql_service::SqlExecuteOptions;
    use crate::core::services::SqlService;

    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    let set_clause = format!(
        "`{}` = {}",
        input.column_name,
        value_to_sql(&input.new_value)
    );

    let where_parts: Vec<String> = input
        .row_identity
        .iter()
        .filter(|(k, _)| *k != &input.column_name)
        .map(|(col, val)| format!("`{}` = {}", col, value_to_sql(val)))
        .collect();

    if where_parts.is_empty() {
        return Err("无法构建 WHERE 条件：行标识数据为空".to_string());
    }

    let sql = format!(
        "UPDATE `{}` SET {} WHERE {}",
        input.table_name,
        set_clause,
        where_parts.join(" AND ")
    );

    let opts = SqlExecuteOptions {
        record_history: false,
        use_transaction: true,
        timeout_ms: Some(10000),
        use_cache: false,
    };

    match service.execute(Some(input.conn_id), &sql, opts).await {
        Ok(result) => {
            let affected = result.result.affected_rows.unwrap_or(0) as usize;
            Ok(CellUpdateResult {
                success: true,
                affected_rows: affected,
                message: format!("更新成功，影响 {} 行", affected),
            })
        }
        Err(e) => Ok(CellUpdateResult {
            success: false,
            affected_rows: 0,
            message: format!("更新失败: {}", e),
        }),
    }
}

fn value_to_sql(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => {
            if *b { "TRUE".to_string() } else { "FALSE".to_string() }
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                f.to_string()
            } else {
                format!("'{}'", n)
            }
        }
        serde_json::Value::String(s) => {
            format!("'{}'", s.replace('\'', "''"))
        }
        _ => format!("'{}'", val),
    }
}

/// Schema 洞察输入
#[derive(serde::Deserialize, Debug)]
pub struct SchemaInsightInput {
    pub conn_id: String,
    pub database: String,
    pub schema: String,
}

/// Schema 级洞察分析（外键推断、类型一致性、孤立表检测）
#[tauri::command]
pub async fn get_schema_insight(
    input: SchemaInsightInput,
) -> Result<SchemaInsightReport, String> {
    SchemaAnalyzer::analyze(input.conn_id, &input.database, &input.schema)
        .await
        .map_err(|e| e.to_string())
}

/// 列质量评分输入
#[derive(serde::Deserialize, Debug)]
pub struct ColumnQualityInput {
    pub column_name: String,
    pub temp_table: String,
}

/// 计算列数据质量评分 (0-100)
#[tauri::command]
pub async fn get_column_quality(
    input: ColumnQualityInput,
) -> Result<QualityScore, String> {
    let stats = ResultService::get_column_insight_full(&input.temp_table, &input.column_name)
        .map_err(|e| e.to_string())?;
    Ok(ResultService::compute_column_quality(&stats))
}

/// 批量评估表质量 — 一次调用完成全表所有列的质量评分
#[derive(serde::Deserialize, Debug)]
pub struct BatchEvaluateInput {
    pub conn_id: String,
    pub database: String,
    pub schema: String,
    pub table: String,
}

#[tauri::command]
pub async fn batch_evaluate_columns(
    input: BatchEvaluateInput,
) -> Result<TableQuality, String> {
    ResultService::batch_evaluate_columns(
        input.conn_id,
        &input.database,
        &input.schema,
        &input.table,
    )
    .await
    .map_err(|e| e.to_string())
}

// ═══════════════ 从表直接探查列命令 ═══════════════

#[derive(serde::Deserialize, Debug)]
pub struct ProfileColumnFromTableInput {
    pub conn_id: String,
    pub database: String,
    pub schema: String,
    pub table: String,
    pub column_name: String,
}

#[tauri::command]
pub async fn profile_column_from_table(
    input: ProfileColumnFromTableInput,
) -> Result<ColumnInsightFull, String> {
    ResultService::profile_column_from_table(
        input.conn_id,
        &input.database,
        &input.schema,
        &input.table,
        &input.column_name,
    )
    .await
    .map_err(|e| e.to_string())
}

// ═══════════════ 版本详情命令 ═══════════════

#[derive(serde::Deserialize, Debug)]
pub struct InsightVersionDetailInput {
    pub version_id: String,
}

#[tauri::command]
pub async fn get_insight_version_detail(
    input: InsightVersionDetailInput,
    state: tauri::State<'_, ProjectState>,
) -> Result<Option<crate::core::services::result_service::ColumnInsightFull>, String> {
    let store_guard = state.store.lock().await;
    let store = store_guard.as_ref().ok_or("No project store available")?;
    let db = &store.db_manager;

    let insight_store = InsightStorage::new(db.duckdb_conn());

    ResultService::get_insight_version_detail(&input.version_id, &insight_store)
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

/// 清理洞察快照输入
#[derive(serde::Deserialize, Debug)]
pub struct CleanupInsightInput {
    pub days: i64,
}

/// 清理 N 天前的洞察快照（DuckDB + SQLite 双写清理）
#[tauri::command]
pub async fn cleanup_insight_snapshots(
    input: CleanupInsightInput,
    state: tauri::State<'_, ProjectState>,
) -> Result<CleanupResult, String> {
    let store_guard = state.store.lock().await;
    let store = store_guard.as_ref().ok_or("No project store available")?;
    let db = &store.db_manager;

    let insight_store = InsightStorage::new(db.duckdb_conn());
    let meta_store = InsightMetaStore::new(db.sqlite_pool());

    let (duckdb_deleted, sqlite_deleted) = ResultService::cleanup_old_insight_snapshots(
        input.days,
        &insight_store,
        &meta_store,
    ).await.map_err(|e| e.to_string())?;

    Ok(CleanupResult {
        duckdb_deleted,
        sqlite_deleted: sqlite_deleted as i64,
    })
}

#[derive(serde::Serialize)]
pub struct CleanupResult {
    pub duckdb_deleted: i64,
    pub sqlite_deleted: i64,
}

/// 获取洞察存储用量统计
#[tauri::command]
pub async fn get_insight_storage_stats(
    state: tauri::State<'_, ProjectState>,
) -> Result<crate::core::persistence::InsightStorageStats, String> {
    let store_guard = state.store.lock().await;
    let store = store_guard.as_ref().ok_or("No project store available")?;
    let db = &store.db_manager;

    let insight_store = InsightStorage::new(db.duckdb_conn());

    ResultService::get_insight_storage_stats(&insight_store)
        .await
        .map_err(|e| e.to_string())
}

// ═══════════════ 规则引擎公开命令 ═══════════════

#[derive(serde::Deserialize, Debug)]
pub struct ExecuteRuleInput {
    pub rule_id: String,
    pub params: std::collections::HashMap<String, String>,
    pub temp_table: String,
}

#[tauri::command]
pub async fn execute_insight_rule(
    input: ExecuteRuleInput,
) -> Result<serde_json::Value, String> {
    let duckdb = ResultService::get_or_create_duckdb()
        .map_err(|e| e.to_string())?;
    let conn = duckdb.lock().map_err(|e| e.to_string())?;

    let exec_result = ResultService::execute_insight_rule(&input.rule_id, &conn, &input.params)
        .map_err(|e| e.to_string())?;
    serde_json::to_value(exec_result).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_insight_rules(
    category: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    ResultService::list_insight_rules(category.as_deref())
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, Debug)]
pub struct RulesForColumnInput {
    pub column_type: String,
}

#[tauri::command]
pub async fn list_rules_for_column(
    input: RulesForColumnInput,
) -> Result<Vec<serde_json::Value>, String> {
    ResultService::list_rules_for_column(&input.column_type)
        .map_err(|e| e.to_string())
}

// ═══════════════ 表探查命令 ═══════════════

#[derive(serde::Deserialize, Debug)]
pub struct TableProfileInput {
    pub conn_id: String,
    pub db_type: String,
    pub database: String,
    pub schema: String,
    pub table: String,
}

#[tauri::command]
pub async fn get_table_profile(
    input: TableProfileInput,
) -> Result<TableProfile, String> {
    ResultService::get_table_profile(
        input.conn_id,
        input.db_type,
        &input.database,
        &input.schema,
        &input.table,
    )
    .await
    .map_err(|e| e.to_string())
}

// ═══════════════ DuckDB 连接池配置命令 ═══════════════

// ═══════════════ 数据导出命令 ═══════════════

#[derive(serde::Deserialize, Debug)]
pub struct ExportInput {
    pub temp_table: String,
    pub file_path: String,
    pub format: String,
}

#[tauri::command]
pub fn export_result_to_file(input: ExportInput) -> Result<String, String> {
    use crate::core::services::duckdb_service::{DuckDbService, ExportFormat};

    let format = match input.format.as_str() {
        "csv" => ExportFormat::Csv,
        "parquet" => ExportFormat::Parquet,
        "xlsx" => ExportFormat::Xlsx,
        other => return Err(format!("不支持的导出格式: {}", other)),
    };

    DuckDbService::export_temp_table(&input.temp_table, &input.file_path, format)
        .map_err(|e| e.to_string())
}

// ═══════════════ DuckDB 连接池配置命令 ═══════════════

#[derive(serde::Deserialize, Debug)]
pub struct SetPoolSizeInput {
    pub size: usize,
    pub restart: bool,
}

#[derive(serde::Serialize)]
pub struct PoolSizeInfo {
    pub current: usize,
    pub preferred: usize,
    pub min: usize,
    pub max: usize,
}

#[tauri::command]
pub fn get_duckdb_pool_info() -> PoolSizeInfo {
    let mgr = crate::core::duckdb::DuckDBManager::global();
    PoolSizeInfo {
        current: mgr.pool_size(),
        preferred: mgr.preferred_pool_size(),
        min: 1,
        max: 32,
    }
}

#[tauri::command]
pub fn set_duckdb_pool_size(input: SetPoolSizeInput) -> Result<PoolSizeInfo, String> {
    let mgr = crate::core::duckdb::DuckDBManager::global();
    let clamped = mgr.set_pool_size(input.size);

    if input.restart {
        mgr.restart_pool().map_err(|e| e.to_string())?;
    }

    Ok(PoolSizeInfo {
        current: mgr.pool_size(),
        preferred: clamped,
        min: 1,
        max: 32,
    })
}

#[tauri::command]
pub fn restart_duckdb_pool() -> Result<PoolSizeInfo, String> {
    let mgr = crate::core::duckdb::DuckDBManager::global();
    let size = mgr.restart_pool().map_err(|e| e.to_string())?;

    Ok(PoolSizeInfo {
        current: size,
        preferred: mgr.preferred_pool_size(),
        min: 1,
        max: 32,
    })
}
