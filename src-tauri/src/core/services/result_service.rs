/**
 * 结果集服务 + 洞察计算引擎
 *
 * 提供：
 * - SQL 过滤（拼接 WHERE 重新查询）
 * - DuckDB 深度分析（针对临时表）
 * - 列洞察全量统计（统计 + 样本 + 直方图）
 * - 规则引擎 API（统一入口，SQL 模板与 Rust 代码分离）
 */

// ==================== 结果集响应 ====================

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ResultSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub elapsed_ms: u64,
    pub temp_table: String,
}

// ==================== 洞察体系 — 顶层结构 ====================

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ColumnInsightFull {
    pub stats: ColumnStats,
    pub sample: Vec<serde_json::Value>,
    pub histogram: Option<Vec<DistributionBin>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ColumnStats {
    pub column_name: String,
    pub data_type: String,
    pub total_count: usize,
    pub null_count: usize,
    pub null_rate: f64,
    pub unique_count: Option<usize>,
    pub stats_detail: ColumnStatsDetail,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind")]
pub enum ColumnStatsDetail {
    Numeric(NumericStats),
    Text(TextStats),
    DateTime(DateTimeStats),
    Boolean(BooleanStats),
    Unknown,
}

// ==================== 数值列统计 ====================

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NumericStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub median: f64,
    pub p25: f64,
    pub p75: f64,
    pub sum: f64,
    pub stddev: Option<f64>,
    pub skewness: Option<f64>,
    pub kurtosis: Option<f64>,
    pub is_extreme: Vec<ExtremeValue>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExtremeValue {
    pub value: f64,
    pub kind: String,
}

// ==================== 文本列统计 ====================

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TextStats {
    pub min_length: usize,
    pub max_length: usize,
    pub top_values: Vec<TextFrequency>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TextFrequency {
    pub value: String,
    pub count: usize,
    pub ratio: f64,
}

// ==================== 日期时间列统计 ====================

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DateTimeStats {
    pub earliest: String,
    pub latest: String,
    pub span_days: i64,
    pub monthly_distribution: Vec<TextFrequency>,
}

// ==================== 布尔列统计 ====================

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BooleanStats {
    pub true_count: usize,
    pub false_count: usize,
    pub true_ratio: f64,
}

// ==================== 分箱直方图 ====================

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DistributionBin {
    pub label: String,
    pub count: usize,
    pub ratio: f64,
}

// ==================== 表探查 ====================

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TableProfile {
    pub table_name: String,
    pub db_type: String,
    pub columns: Vec<TableColumnMeta>,
    pub row_count: Option<i64>,
    pub schema_name: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TableColumnMeta {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub ordinal_position: i32,
}

// ==================== 质量评分 ====================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QualityScore {
    pub column_name: String,
    pub overall_score: f64,
    pub level: String,
    pub dimensions: Vec<QualityDimension>,
    pub summary: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QualityDimension {
    pub name: String,
    pub score: f64,
    pub weight: f64,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableQuality {
    pub table_name: String,
    pub overall_score: f64,
    pub level: String,
    pub column_scores: Vec<ColumnQualityEntry>,
    pub summary: String,
    pub scored_count: usize,
    pub total_columns: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColumnQualityEntry {
    pub column_name: String,
    pub quality_score: f64,
    pub level: String,
    pub null_rate: f64,
}

// ==================== ResultService（外观 / Facade）====================

use crate::core::error::CoreError;

pub struct ResultService;

impl ResultService {
    pub async fn re_execute_with_filter(
        conn_id: String,
        original_sql: &str,
        where_clause: &str,
        order_clause: &str,
    ) -> Result<ResultSet, CoreError> {
        crate::core::services::execution_service::re_execute_with_filter(
            conn_id,
            original_sql,
            where_clause,
            order_clause,
        )
        .await
    }

    pub fn execute_duckdb_analysis(
        temp_table: &str,
        sql: &str,
        columns: Option<Vec<String>>,
        rows: Option<Vec<Vec<serde_json::Value>>>,
    ) -> Result<ResultSet, CoreError> {
        crate::core::services::execution_service::execute_duckdb_analysis(
            temp_table,
            sql,
            columns,
            rows,
        )
    }

    pub fn get_or_create_duckdb(
    ) -> Result<std::sync::Arc<std::sync::Mutex<duckdb::Connection>>, CoreError> {
        crate::core::services::duckdb_service::DuckDbService::get_or_create_duckdb()
    }

    pub fn create_duckdb_temp_table(
        columns: &[String],
        rows: &[Vec<serde_json::Value>],
    ) -> Result<String, CoreError> {
        crate::core::services::duckdb_service::DuckDbService::create_duckdb_temp_table(
            columns, rows,
        )
    }

    pub fn get_column_insight_full(
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnInsightFull, CoreError> {
        crate::core::services::insight_engine::get_column_insight_full(
            temp_table,
            column_name,
        )
    }

    pub fn get_column_insights(
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnStats, CoreError> {
        crate::core::services::insight_engine::get_column_insights(
            temp_table,
            column_name,
        )
    }

    pub fn execute_insight_rule(
        rule_id: &str,
        conn: &duckdb::Connection,
        params: &std::collections::HashMap<String, String>,
    ) -> Result<crate::core::insight::ExecutionResult, CoreError> {
        crate::core::services::insight_engine::execute_insight_rule(
            rule_id, conn, params,
        )
    }

    pub fn list_insight_rules(
        category: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        crate::core::services::insight_engine::list_insight_rules(category)
    }

    pub fn list_rules_for_column(
        column_type: &str,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        crate::core::services::insight_engine::list_rules_for_column(column_type)
    }

    pub fn compute_column_quality(stats: &ColumnInsightFull) -> QualityScore {
        crate::core::services::quality_scorer::compute_column_quality(stats)
    }

    pub fn compute_table_quality(
        table_name: &str,
        stats_list: &[ColumnInsightFull],
    ) -> TableQuality {
        crate::core::services::quality_scorer::compute_table_quality(
            table_name,
            stats_list,
        )
    }

    pub async fn save_column_insight_snapshot(
        insight: &ColumnInsightFull,
        conn_id: Option<&str>,
        db_name: Option<&str>,
        schema_name: Option<&str>,
        table_name: Option<&str>,
        row_count: Option<i64>,
        elapsed_ms: Option<i64>,
        insight_store: &crate::core::persistence::InsightStorage,
        meta_store: &crate::core::persistence::InsightMetaStore,
    ) -> Result<(String, String), CoreError> {
        crate::core::services::persistence_service::save_column_insight_snapshot(
            insight,
            conn_id,
            db_name,
            schema_name,
            table_name,
            row_count,
            elapsed_ms,
            insight_store,
            meta_store,
        )
        .await
    }

    pub async fn get_column_insight_history(
        column_name: &str,
        insight_store: &crate::core::persistence::InsightStorage,
    ) -> Result<Vec<crate::core::persistence::InsightVersionEntry>, CoreError> {
        crate::core::services::persistence_service::get_column_insight_history(
            column_name,
            insight_store,
        )
        .await
    }

    pub async fn cleanup_old_insight_snapshots(
        days: i64,
        insight_store: &crate::core::persistence::InsightStorage,
        meta_store: &crate::core::persistence::InsightMetaStore,
    ) -> Result<(i64, usize), CoreError> {
        crate::core::services::persistence_service::cleanup_old_insight_snapshots(
            days,
            insight_store,
            meta_store,
        )
        .await
    }

    pub async fn get_table_profile(
        conn_id: String,
        db_type: String,
        database: &str,
        schema: &str,
        table: &str,
    ) -> Result<TableProfile, CoreError> {
        crate::core::services::table_profile_service::get_table_profile(
            conn_id, db_type, database, schema, table,
        )
        .await
    }

    pub async fn get_insight_storage_stats(
        insight_store: &crate::core::persistence::InsightStorage,
    ) -> Result<crate::core::persistence::InsightStorageStats, CoreError> {
        crate::core::services::persistence_service::get_insight_storage_stats(
            insight_store,
        )
        .await
    }

    pub async fn get_insight_version_detail(
        version_id: &str,
        insight_store: &crate::core::persistence::InsightStorage,
    ) -> Result<Option<ColumnInsightFull>, CoreError> {
        crate::core::services::persistence_service::get_insight_version_detail(
            version_id,
            insight_store,
        )
        .await
    }

    pub async fn profile_column_from_table(
        conn_id: String,
        database: &str,
        schema: &str,
        table: &str,
        column_name: &str,
    ) -> Result<ColumnInsightFull, CoreError> {
        crate::core::services::persistence_service::profile_column_from_table(
            conn_id, database, schema, table, column_name,
        )
        .await
    }

    pub async fn batch_evaluate_columns(
        conn_id: String,
        database: &str,
        schema: &str,
        table: &str,
    ) -> Result<TableQuality, CoreError> {
        crate::core::services::persistence_service::batch_evaluate_columns(
            conn_id, database, schema, table,
        )
        .await
    }
}
