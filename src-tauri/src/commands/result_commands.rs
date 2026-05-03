//! 结果集分析命令
//!
//! 提供结果的二次分析：SQL 过滤、DuckDB 分析、列洞察

use crate::core::services::result_service::{ResultService, ResultSet, ColumnStats};

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
pub struct ColumnInsightsInput {
    pub temp_table: String,
    pub column_name: String,
}

#[tauri::command]
pub async fn get_column_insights(
    input: ColumnInsightsInput,
) -> Result<ColumnStats, String> {
    ResultService::get_column_insights(&input.temp_table, &input.column_name)
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
