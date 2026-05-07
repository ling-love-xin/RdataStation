/**
 * 结果集服务 + 洞察计算引擎
 *
 * 提供：
 * - SQL 过滤（拼接 WHERE 重新查询）
 * - DuckDB 深度分析（针对临时表）
 * - 列洞察全量统计（统计 + 样本 + 直方图）
 * - 规则引擎 API（统一入口，SQL 模板与 Rust 代码分离）
 */

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::get_connection_manager;
use crate::core::insight::{self, RuleExecutor};
use crate::core::services::sql_service::SqlExecuteOptions;
use crate::core::services::SqlService;
use crate::core::error::{CoreError, CommonError};

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

// ==================== ResultService ====================

pub struct ResultService;

impl ResultService {
    // ═══════════════════ SQL 过滤 ═══════════════════

    pub async fn re_execute_with_filter(
        conn_id: String,
        original_sql: &str,
        where_clause: &str,
        order_clause: &str,
    ) -> Result<ResultSet, CoreError> {
        let start = std::time::Instant::now();
        let manager = get_connection_manager().clone();
        let service = SqlService::new(manager);

        let base_sql = original_sql.trim().trim_end_matches(';');
        let mut filtered_sql = format!("SELECT * FROM ({}) AS _result", base_sql);
        if !where_clause.trim().is_empty() {
            filtered_sql.push_str(&format!(" WHERE {}", where_clause));
        }
        if !order_clause.trim().is_empty() {
            filtered_sql.push_str(&format!(" ORDER BY {}", order_clause));
        }

        let options = SqlExecuteOptions {
            record_history: false,
            use_transaction: false,
            timeout_ms: None,
            use_cache: false,
        };

        let result = service.execute(Some(conn_id), &filtered_sql, options).await?;
        let elapsed = start.elapsed().as_millis() as u64;

        let json_value = serde_json::to_value(&result.result).map_err(|e| CoreError::common(
            CommonError::General(format!("Serialize error: {}", e))
        ))?;

        let columns = json_value["columns"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<String>>())
            .unwrap_or_default();

        let rows = extract_rows_from_serialized(&json_value);
        let temp_table = Self::create_duckdb_temp_table(&columns, &rows)?;

        Ok(ResultSet {
            row_count: rows.len(),
            columns,
            rows,
            elapsed_ms: elapsed,
            temp_table,
        })
    }

    // ═══════════════════ DuckDB 分析 ═══════════════════

    pub fn execute_duckdb_analysis(
        temp_table: &str,
        sql: &str,
        columns: Option<Vec<String>>,
        rows: Option<Vec<Vec<serde_json::Value>>>,
    ) -> Result<ResultSet, CoreError> {
        let start = std::time::Instant::now();
        let duckdb = Self::get_or_create_duckdb()?;
        let mut conn = duckdb.lock().map_err(|e| CoreError::common(
            CommonError::General(format!("DuckDB lock error: {}", e))
        ))?;

        let actual_table = if temp_table.is_empty() {
            if let (Some(cols), Some(rws)) = (columns, rows) {
                Self::create_temp_table_internal(&mut conn, &cols, &rws)?
            } else {
                return Err(CoreError::common(CommonError::General(
                    "No temp table or data provided".to_string()
                )));
            }
        } else {
            temp_table.to_string()
        };

        let analysis_sql = sql
            .replace("{table}", &actual_table)
            .replace("result_temp", &actual_table);

        let (cols_out, rws_out) = Self::query_duckdb(&mut conn, &analysis_sql)?;
        let elapsed = start.elapsed().as_millis() as u64;
        let row_count = rws_out.len();

        Ok(ResultSet {
            columns: cols_out,
            rows: rws_out,
            row_count,
            elapsed_ms: elapsed,
            temp_table: actual_table,
        })
    }

    // ═══════════════════ 洞察计算引擎 ═══════════════════

    /// 获取列的全量洞察（统计 + 样本 + 直方图）
    pub fn get_column_insight_full(
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnInsightFull, CoreError> {
        let duckdb = Self::get_or_create_duckdb()?;
        let conn = duckdb.lock().map_err(|e| CoreError::common(
            CommonError::General(format!("DuckDB lock error: {}", e))
        ))?;

        let stats = Self::get_column_stats_internal(&conn, temp_table, column_name)?;
        let sample = Self::get_column_sample_internal(&conn, temp_table, column_name)?;

        let histogram = match &stats.stats_detail {
            ColumnStatsDetail::Numeric(_) => {
                Self::get_column_histogram_internal(&conn, temp_table, column_name).ok()
            }
            _ => None,
        };

        Ok(ColumnInsightFull {
            stats,
            sample,
            histogram,
        })
    }

    // ═══════════════════ 内部统计计算 ═══════════════════

    fn get_column_stats_internal(
        conn: &duckdb::Connection,
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnStats, CoreError> {
        let count_sql = format!(
            "SELECT COUNT(*), COUNT(\"{}\"), COUNT(DISTINCT \"{}\") FROM \"{}\"",
            column_name, column_name, temp_table
        );
        let (total, non_null, unique_cnt): (i64, i64, i64) = conn.query_row(&count_sql, [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).map_err(|e| CoreError::common(CommonError::General(
            format!("DuckDB count failed for '{}': {}", column_name, e)
        )))?;

        let null_count = (total - non_null) as usize;
        let null_rate = if total > 0 {
            null_count as f64 / total as f64
        } else {
            0.0
        };

        let type_sql = format!(
            "SELECT typeof(\"{}\") FROM \"{}\" WHERE \"{}\" IS NOT NULL LIMIT 1",
            column_name, temp_table, column_name
        );
        let data_type: String = conn.query_row(&type_sql, [], |row| row.get(0))
            .unwrap_or_else(|_| "VARCHAR".to_string());

        let dt_lower = data_type.to_lowercase();

        let stats_detail = if is_numeric_type(&dt_lower) {
            Self::compute_numeric_stats(conn, temp_table, column_name)?
        } else if is_datetime_type(&dt_lower) {
            Self::compute_datetime_stats(conn, temp_table, column_name)?
        } else if dt_lower == "boolean" {
            Self::compute_boolean_stats(conn, temp_table, column_name)?
        } else {
            Self::compute_text_stats(conn, temp_table, column_name)?
        };

        Ok(ColumnStats {
            column_name: column_name.to_string(),
            data_type,
            total_count: total as usize,
            null_count,
            null_rate,
            unique_count: Some(unique_cnt as usize),
            stats_detail,
        })
    }

    fn compute_numeric_stats(
        conn: &duckdb::Connection,
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnStatsDetail, CoreError> {
        let registry = insight::global_registry()
            .read()
            .map_err(|e| CoreError::common(CommonError::General(format!("Registry lock error: {}", e))))?;
        let mut params = HashMap::new();
        params.insert("table".to_string(), temp_table.to_string());
        params.insert("col".to_string(), column_name.to_string());

        let rule = registry.get("numeric-stats").ok_or_else(|| {
            CoreError::common(CommonError::General("Rule 'numeric-stats' not found".to_string()))
        })?;

        match RuleExecutor::execute(rule, conn, &params) {
            Ok(serde_json::Value::Object(map)) => {
                let extract = |key: &str| -> f64 {
                    map.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0)
                };
                let extract_opt = |key: &str| -> Option<f64> {
                    map.get(key).and_then(|v| v.as_f64())
                };
                let min_v = extract("min");
                let max_v = extract("max");
                let stddev_v = extract_opt("stddev");
                let is_extreme = detect_extremes(min_v, max_v, stddev_v.unwrap_or(0.0));

                Ok(ColumnStatsDetail::Numeric(NumericStats {
                    min: min_v,
                    max: max_v,
                    avg: extract("avg"),
                    median: extract("median"),
                    p25: extract("p25"),
                    p75: extract("p75"),
                    sum: extract("sum"),
                    stddev: stddev_v,
                    skewness: extract_opt("skewness"),
                    kurtosis: extract_opt("kurtosis"),
                    is_extreme,
                }))
            }
            Ok(_) => Err(CoreError::common(CommonError::General(
                "numeric-stats rule returned unexpected result type".to_string(),
            ))),
            Err(e) => {
                let basic_rule = registry.get("numeric-basic").ok_or_else(|| {
                    CoreError::common(CommonError::General(
                        "Rule 'numeric-basic' not found".to_string(),
                    ))
                })?;
                match RuleExecutor::execute(basic_rule, conn, &params) {
                    Ok(serde_json::Value::Object(map)) => {
                        let extract = |key: &str| -> f64 {
                            map.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0)
                        };
                        Ok(ColumnStatsDetail::Numeric(NumericStats {
                            min: extract("min"),
                            max: extract("max"),
                            avg: extract("avg"),
                            median: extract("median"),
                            p25: 0.0,
                            p75: 0.0,
                            sum: extract("sum"),
                            stddev: map.get("stddev").and_then(|v| v.as_f64()),
                            skewness: None,
                            kurtosis: None,
                            is_extreme: vec![],
                        }))
                    }
                    Ok(_) => Err(CoreError::common(CommonError::General(
                        "numeric-basic rule returned unexpected result type".to_string(),
                    ))),
                    Err(e2) => Err(CoreError::common(CommonError::General(format!(
                        "DuckDB numeric stats failed: {} / fallback: {}",
                        e, e2
                    )))),
                }
            }
        }
    }

    fn compute_text_stats(
        conn: &duckdb::Connection,
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnStatsDetail, CoreError> {
        let registry = insight::global_registry()
            .read()
            .map_err(|e| CoreError::common(CommonError::General(format!("Registry lock error: {}", e))))?;
        let mut params = HashMap::new();
        params.insert("table".to_string(), temp_table.to_string());
        params.insert("col".to_string(), column_name.to_string());

        let freq_rule = registry.get("text-frequency").ok_or_else(|| {
            CoreError::common(CommonError::General("Rule 'text-frequency' not found".to_string()))
        })?;

        let top_values: Vec<TextFrequency> = match RuleExecutor::execute(freq_rule, conn, &params) {
            Ok(serde_json::Value::Array(arr)) => arr.iter().filter_map(|item| {
                let obj = item.as_object()?;
                Some(TextFrequency {
                    value: obj.get("value")?.as_str()?.to_string(),
                    count: obj.get("count")?.as_u64()? as usize,
                    ratio: obj.get("ratio")?.as_f64()?,
                })
            }).collect(),
            _ => vec![],
        };

        let len_rule = registry.get("text-length").ok_or_else(|| {
            CoreError::common(CommonError::General("Rule 'text-length' not found".to_string()))
        })?;
        let (min_len, max_len): (usize, usize) = match RuleExecutor::execute(len_rule, conn, &params) {
            Ok(serde_json::Value::Object(map)) => (
                map.get("min_length").and_then(|v| v.as_i64()).unwrap_or(0) as usize,
                map.get("max_length").and_then(|v| v.as_i64()).unwrap_or(0) as usize,
            ),
            _ => (0, 0),
        };

        Ok(ColumnStatsDetail::Text(TextStats {
            min_length: min_len,
            max_length: max_len,
            top_values,
        }))
    }

    fn compute_datetime_stats(
        conn: &duckdb::Connection,
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnStatsDetail, CoreError> {
        let registry = insight::global_registry()
            .read()
            .map_err(|e| CoreError::common(CommonError::General(format!("Registry lock error: {}", e))))?;
        let mut params = HashMap::new();
        params.insert("table".to_string(), temp_table.to_string());
        params.insert("col".to_string(), column_name.to_string());

        let range_rule = registry.get("datetime-range").ok_or_else(|| {
            CoreError::common(CommonError::General(
                "Rule 'datetime-range' not found".to_string(),
            ))
        })?;

        let (earliest, latest, span) = match RuleExecutor::execute(range_rule, conn, &params) {
            Ok(serde_json::Value::Object(map)) => (
                map.get("earliest")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A")
                    .to_string(),
                map.get("latest")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A")
                    .to_string(),
                map.get("span_days").and_then(|v| v.as_i64()).unwrap_or(0),
            ),
            _ => ("N/A".to_string(), "N/A".to_string(), 0),
        };

        let monthly_rule = registry.get("datetime-monthly").ok_or_else(|| {
            CoreError::common(CommonError::General(
                "Rule 'datetime-monthly' not found".to_string(),
            ))
        })?;
        let monthly_distribution: Vec<TextFrequency> =
            match RuleExecutor::execute(monthly_rule, conn, &params) {
                Ok(serde_json::Value::Array(arr)) => arr
                    .iter()
                    .filter_map(|item| {
                        let obj = item.as_object()?;
                        Some(TextFrequency {
                            value: obj.get("value")?.as_str()?.to_string(),
                            count: obj.get("count")?.as_u64()? as usize,
                            ratio: obj.get("ratio")?.as_f64()?,
                        })
                    })
                    .collect(),
                _ => vec![],
            };

        Ok(ColumnStatsDetail::DateTime(DateTimeStats {
            earliest,
            latest,
            span_days: span,
            monthly_distribution,
        }))
    }

    fn compute_boolean_stats(
        conn: &duckdb::Connection,
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnStatsDetail, CoreError> {
        let registry = insight::global_registry()
            .read()
            .map_err(|e| CoreError::common(CommonError::General(format!("Registry lock error: {}", e))))?;
        let mut params = HashMap::new();
        params.insert("table".to_string(), temp_table.to_string());
        params.insert("col".to_string(), column_name.to_string());

        let rule = registry.get("boolean-ratio").ok_or_else(|| {
            CoreError::common(CommonError::General("Rule 'boolean-ratio' not found".to_string()))
        })?;

        match RuleExecutor::execute(rule, conn, &params) {
            Ok(serde_json::Value::Object(map)) => {
                let true_count = map.get("true_count").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
                let false_count = map.get("false_count").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
                let total = true_count + false_count;
                let true_ratio = if total > 0 {
                    true_count as f64 / total as f64
                } else {
                    0.0
                };
                Ok(ColumnStatsDetail::Boolean(BooleanStats {
                    true_count,
                    false_count,
                    true_ratio,
                }))
            }
            Ok(_) => Err(CoreError::common(CommonError::General(
                "boolean-ratio rule returned unexpected result type".to_string(),
            ))),
            Err(e) => Err(CoreError::common(CommonError::General(format!(
                "DuckDB boolean stats failed: {}",
                e
            )))),
        }
    }

    fn get_column_sample_internal(
        conn: &duckdb::Connection,
        temp_table: &str,
        column_name: &str,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let sql = format!(
            "SELECT \"{}\" FROM \"{}\" LIMIT 5",
            column_name, temp_table
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| CoreError::common(
            CommonError::General(format!("DuckDB sample prepare failed: {}", e))
        ))?;
        let samples: Vec<serde_json::Value> = stmt.query_map([], |row| {
            let v: duckdb::types::Value = row.get_unwrap(0);
            Ok(duckdb_value_to_json(&v))
        }).map_err(|e| CoreError::common(CommonError::General(
            format!("DuckDB sample query failed: {}", e)
        )))?.filter_map(|r| r.ok()).collect();

        Ok(samples)
    }

    fn get_column_histogram_internal(
        conn: &duckdb::Connection,
        temp_table: &str,
        column_name: &str,
    ) -> Result<Vec<DistributionBin>, CoreError> {
        let count_sql = format!(
            "SELECT COUNT(*) FROM \"{}\" WHERE \"{}\" IS NOT NULL",
            temp_table, column_name
        );
        let count: i64 = conn.query_row(&count_sql, [], |row| row.get(0)).unwrap_or(0);

        if count < 10 {
            return Ok(vec![]);
        }

        let registry = insight::global_registry()
            .read()
            .map_err(|e| CoreError::common(CommonError::General(format!("Registry lock error: {}", e))))?;
        let mut params = HashMap::new();
        params.insert("table".to_string(), temp_table.to_string());
        params.insert("col".to_string(), column_name.to_string());

        let rule = registry.get("histogram").ok_or_else(|| {
            CoreError::common(CommonError::General("Rule 'histogram' not found".to_string()))
        })?;

        match RuleExecutor::execute(rule, conn, &params) {
            Ok(serde_json::Value::Array(arr)) => {
                let bins: Vec<DistributionBin> = arr
                    .iter()
                    .filter_map(|item| {
                        let obj = item.as_object()?;
                        Some(DistributionBin {
                            label: obj.get("label")?.as_str()?.to_string(),
                            count: obj.get("count")?.as_u64()? as usize,
                            ratio: obj.get("ratio")?.as_f64()?,
                        })
                    })
                    .collect();
                Ok(bins)
            }
            _ => Ok(vec![]),
        }
    }

    // ═══════════════════ 旧版兼容 API ═══════════════════

    /// 获取列洞察统计（旧版，保留兼容）
    pub fn get_column_insights(
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnStats, CoreError> {
        let duckdb = Self::get_or_create_duckdb()?;
        let conn = duckdb.lock().map_err(|e| CoreError::common(
            CommonError::General(format!("DuckDB lock error: {}", e))
        ))?;
        Self::get_column_stats_internal(&conn, temp_table, column_name)
    }

    // ═══════════════════ DuckDB 管理 ═══════════════════

    pub fn get_or_create_duckdb() -> Result<Arc<std::sync::Mutex<duckdb::Connection>>, CoreError> {
        static DUCKDB: std::sync::OnceLock<Arc<std::sync::Mutex<duckdb::Connection>>> = std::sync::OnceLock::new();
        Ok(DUCKDB.get_or_init(|| {
            let conn = duckdb::Connection::open_in_memory()
                .expect("Failed to create in-memory DuckDB");
            Arc::new(std::sync::Mutex::new(conn))
        }).clone())
    }

    pub fn create_duckdb_temp_table(
        columns: &[String],
        rows: &[Vec<serde_json::Value>],
    ) -> Result<String, CoreError> {
        let duckdb = Self::get_or_create_duckdb()?;
        let mut conn = duckdb.lock().map_err(|e| CoreError::common(
            CommonError::General(format!("DuckDB lock error: {}", e))
        ))?;
        Self::create_temp_table_internal(&mut conn, columns, rows)
    }

    fn create_temp_table_internal(
        conn: &mut duckdb::Connection,
        columns: &[String],
        rows: &[Vec<serde_json::Value>],
    ) -> Result<String, CoreError> {
        let table_name = format!("rs_{}", uuid::Uuid::new_v4().to_string().replace('-', "_"));
        let col_defs: Vec<String> = columns.iter().enumerate().map(|(i, col)| {
            let dtype = if rows.is_empty() { "VARCHAR" } else { infer_type(rows, i) };
            format!("\"{}\" {}", col, dtype)
        }).collect();

        conn.execute_batch(&format!("CREATE TABLE \"{}\" ({})", table_name, col_defs.join(", ")))
            .map_err(|e| CoreError::common(CommonError::General(
                format!("Failed to create DuckDB table: {}", e)
            )))?;

        if !rows.is_empty() {
            let placeholders: Vec<String> = (0..columns.len()).map(|_| "?".to_string()).collect();
            let insert_sql = format!("INSERT INTO \"{}\" VALUES ({})", table_name, placeholders.join(", "));
            let mut stmt = conn.prepare(&insert_sql).map_err(|e| CoreError::common(
                CommonError::General(format!("Prepare insert failed: {}", e))
            ))?;

            for row in rows {
                let params: Vec<duckdb::types::Value> = row.iter().map(json_to_duckdb_value).collect();
                let params_refs: Vec<&dyn duckdb::types::ToSql> = params.iter()
                    .map(|p| p as &dyn duckdb::types::ToSql).collect();
                stmt.execute(&params_refs[..]).map_err(|e| CoreError::common(
                    CommonError::General(format!("Insert row failed: {}", e))
                ))?;
            }
        }

        Ok(table_name)
    }

    fn query_duckdb(
        conn: &mut duckdb::Connection,
        sql: &str,
    ) -> Result<(Vec<String>, Vec<Vec<serde_json::Value>>), CoreError> {
        let mut stmt = conn.prepare(sql).map_err(|e| CoreError::common(
            CommonError::General(format!("DuckDB prepare failed: {}", e))
        ))?;
        let col_count = stmt.column_count();
        let col_names: Vec<String> = (0..col_count)
            .map(|i| stmt.column_name(i).unwrap_or(&format!("c{}", i)).to_string())
            .collect();

        let rows_result = stmt.query_map([], |row| {
            Ok((0..col_count).map(|i| {
                let v: duckdb::types::Value = row.get_unwrap(i);
                duckdb_value_to_json(&v)
            }).collect())
        }).map_err(|e| CoreError::common(CommonError::General(
            format!("DuckDB query failed: {}", e)
        )))?;

        let mut rows = Vec::new();
        for r in rows_result {
            rows.push(r.map_err(|e| CoreError::common(
                CommonError::General(format!("DuckDB row error: {}", e))
            ))?);
        }

        Ok((col_names, rows))
    }

    // ═══════════════════ 规则引擎公开 API ═══════════════════

    /// 执行指定规则并返回原始 JSON 结果
    pub fn execute_insight_rule(
        rule_id: &str,
        conn: &duckdb::Connection,
        params: &HashMap<String, String>,
    ) -> Result<serde_json::Value, CoreError> {
        let registry = insight::global_registry()
            .read()
            .map_err(|e| CoreError::common(CommonError::General(format!("Registry lock error: {}", e))))?;
        let rule = registry.get(rule_id).ok_or_else(|| {
            CoreError::common(CommonError::General(format!(
                "Rule '{}' not found",
                rule_id
            )))
        })?;
        RuleExecutor::execute(rule, conn, params)
    }

    /// 列出所有可用规则的元数据
    pub fn list_insight_rules(category: Option<&str>) -> Vec<serde_json::Value> {
        let registry = insight::global_registry()
            .read()
            .expect("failed to lock insight registry");
        let rules: Vec<&crate::core::insight::RuleFile> = match category {
            Some(cat) => registry.list_by_category(cat),
            None => registry.all_rules(),
        };
        rules
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.meta.id,
                    "name": r.meta.name,
                    "description": r.meta.description,
                    "version": r.meta.version,
                    "category": r.meta.category,
                    "applies_to": r.meta.applies_to,
                    "builtin": r.meta.builtin,
                    "parameters": r.query.parameters,
                    "result_type": r.query.result_type,
                })
            })
            .collect()
    }

    /// 获取适用于指定列类型的规则列表
    pub fn list_rules_for_column(column_type: &str) -> Vec<serde_json::Value> {
        let registry = insight::global_registry()
            .read()
            .expect("failed to lock insight registry");
        registry
            .rules_for_column_type(column_type)
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.meta.id,
                    "name": r.meta.name,
                    "category": r.meta.category,
                    "applies_to": r.meta.applies_to,
                    "parameters": r.query.parameters,
                })
            })
            .collect()
    }
}

// ==================== 工具函数 ====================

fn extract_rows_from_serialized(result_json: &serde_json::Value) -> Vec<Vec<serde_json::Value>> {
    let columns = match result_json["columns"].as_array() {
        Some(c) => c.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>(),
        None => return vec![],
    };

    let batches = match result_json["batches"].as_array() {
        Some(b) => b,
        None => return vec![],
    };

    let mut rows = Vec::new();
    for batch in batches {
        if let Some(data) = batch["data"].as_object() {
            let num_rows = columns.first()
                .and_then(|c| data.get(c))
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);

            for ri in 0..num_rows {
                let mut row = Vec::with_capacity(columns.len());
                for col in &columns {
                    let val = data.get(col)
                        .and_then(|v| v.as_array())
                        .and_then(|a| a.get(ri))
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    row.push(val);
                }
                rows.push(row);
            }
        }
    }

    rows
}

fn infer_type(rows: &[Vec<serde_json::Value>], col_idx: usize) -> &str {
    for row in rows {
        if col_idx < row.len() {
            match &row[col_idx] {
                serde_json::Value::Null => continue,
                serde_json::Value::Number(n) => {
                    return if n.is_f64() { "DOUBLE" } else { "BIGINT" };
                }
                serde_json::Value::Bool(_) => return "BOOLEAN",
                _ => return "VARCHAR",
            }
        }
    }
    "VARCHAR"
}

fn json_to_duckdb_value(v: &serde_json::Value) -> duckdb::types::Value {
    match v {
        serde_json::Value::Null => duckdb::types::Value::Null,
        serde_json::Value::Bool(b) => duckdb::types::Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            n.as_f64().map(duckdb::types::Value::Double)
                .or_else(|| n.as_i64().map(duckdb::types::Value::BigInt))
                .unwrap_or(duckdb::types::Value::Text(n.to_string()))
        }
        serde_json::Value::String(s) => duckdb::types::Value::Text(s.clone()),
        _ => duckdb::types::Value::Text(v.to_string()),
    }
}

fn duckdb_value_to_json(v: &duckdb::types::Value) -> serde_json::Value {
    match v {
        duckdb::types::Value::Null => serde_json::Value::Null,
        duckdb::types::Value::Boolean(b) => serde_json::json!(b),
        duckdb::types::Value::TinyInt(n) => serde_json::json!(n),
        duckdb::types::Value::SmallInt(n) => serde_json::json!(n),
        duckdb::types::Value::Int(n) => serde_json::json!(n),
        duckdb::types::Value::BigInt(n) => serde_json::json!(n),
        duckdb::types::Value::Float(f) => serde_json::json!(f),
        duckdb::types::Value::Double(f) => serde_json::json!(f),
        duckdb::types::Value::Text(s) => serde_json::Value::String(s.clone()),
        _ => serde_json::Value::Null,
    }
}

fn is_numeric_type(dt_lower: &str) -> bool {
    matches!(dt_lower,
        "bigint" | "integer" | "int" | "smallint" | "tinyint" |
        "double" | "float" | "hugeint" | "decimal" | "numeric" | "real"
    )
}

fn is_datetime_type(dt_lower: &str) -> bool {
    matches!(dt_lower,
        "date" | "timestamp" | "datetime" | "time" | "timestamp with time zone" | "timestamptz"
    )
}

fn detect_extremes(_min: f64, _max: f64, _stddev: f64) -> Vec<ExtremeValue> {
    let mut results = Vec::new();
    if _stddev > 0.0 && _max > 0.0 {
        let range = _max - _min;
        if range > 10.0 * _stddev && range > 1000.0 {
            results.push(ExtremeValue {
                value: _max,
                kind: "outlier_high".to_string(),
            });
        }
    }
    results
}

// ==================== 持久化 API ====================

impl ResultService {
    /// 保存列洞察快照到持久化存储
    ///
    /// 同时写入 DuckDB（JSON 数据）和 SQLite（元数据）
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
        let parent_version_id = meta_store
            .get_latest_meta("column", &insight.stats.column_name)
            .await
            .ok()
            .flatten()
            .map(|m| m.version_id);

        let (snapshot_id, version_id) = insight_store.columns
            .save_snapshot(insight, parent_version_id.as_deref())
            .await?;

        let entity_source = {
            let mut parts = Vec::new();
            if let Some(c) = conn_id { parts.push(format!("conn={}", c)); }
            if let Some(d) = db_name { parts.push(format!("db={}", d)); }
            if let Some(s) = schema_name { parts.push(format!("schema={}", s)); }
            if let Some(t) = table_name { parts.push(format!("table={}", t)); }
            if parts.is_empty() { None } else { Some(parts.join(",")) }
        };

        let checksum = sha256_hex(&serde_json::to_string(insight).unwrap_or_default());

        meta_store.save_meta(
            "column",
            &insight.stats.column_name,
            entity_source.as_deref(),
            &snapshot_id,
            row_count,
            elapsed_ms,
            &version_id,
            parent_version_id.as_deref(),
            &checksum,
        ).await?;

        Ok((snapshot_id, version_id))
    }

    /// 获取列洞察历史版本
    pub async fn get_column_insight_history(
        column_name: &str,
        insight_store: &crate::core::persistence::InsightStorage,
    ) -> Result<Vec<crate::core::persistence::InsightVersionEntry>, CoreError> {
        insight_store.columns.get_history(column_name, Some(10)).await
    }

    /// 清理 N 天前的洞察快照（DuckDB + SQLite 双写清理）
    pub async fn cleanup_old_insight_snapshots(
        days: i64,
        insight_store: &crate::core::persistence::InsightStorage,
        meta_store: &crate::core::persistence::InsightMetaStore,
    ) -> Result<(i64, usize), CoreError> {
        let duckdb_deleted = insight_store.columns.cleanup_older_than(days).await?;
        let sqlite_deleted = meta_store.cleanup_older_than(days).await?;
        Ok((duckdb_deleted, sqlite_deleted))
    }

    /// 获取表探查概要（列元数据 + 行数估算）
    pub async fn get_table_profile(
        conn_id: String,
        db_type: String,
        database: &str,
        schema: &str,
        table: &str,
    ) -> Result<TableProfile, CoreError> {
        let manager = get_connection_manager().clone();
        let service = SqlService::new(manager);
        let conn_id_opt = Some(conn_id.clone());

        let columns = Self::fetch_table_columns(&service, conn_id_opt.clone(), database, schema, table).await?;

        let row_count = match Self::fetch_row_count(&service, conn_id_opt, database, schema, table).await {
            Ok(count) => Some(count),
            Err(_) => None,
        };

        Ok(TableProfile {
            table_name: table.to_string(),
            db_type,
            columns,
            row_count,
            schema_name: Some(schema.to_string()),
        })
    }

    async fn fetch_table_columns(
        service: &SqlService,
        conn_id: Option<String>,
        _database: &str,
        schema: &str,
        table: &str,
    ) -> Result<Vec<TableColumnMeta>, CoreError> {
        use crate::core::services::sql_service::SqlExecuteOptions;

        let sql = format!(
            "SELECT column_name, data_type, is_nullable, ordinal_position, column_key \
             FROM information_schema.columns \
             WHERE table_schema = '{}' AND table_name = '{}' \
             ORDER BY ordinal_position",
            schema, table
        );

        let opts = SqlExecuteOptions {
            record_history: false,
            use_transaction: false,
            timeout_ms: Some(15000),
            use_cache: false,
        };

        let result = service.execute(conn_id, &sql, opts).await?;
        let json = serde_json::to_value(&result.result).map_err(|e| CoreError::common(
            CommonError::General(format!("Serialize error: {}", e))
        ))?;

        let rows = json["batches"]
            .as_array()
            .and_then(|batches| {
                batches.first()?.get("columns").and_then(|c| c.as_array()).and_then(|columns_arr| {
                    batches.first()?.get("rows").and_then(|r| r.as_array()).map(|rows_arr| {
                        (columns_arr.clone(), rows_arr.clone())
                    })
                })
            });

        let columns: Vec<TableColumnMeta> = match rows {
            Some((col_names, row_data)) => {
                let col_idx = |name: &str| -> Option<usize> {
                    col_names.iter().position(|c| c.as_str() == Some(name))
                };
                row_data.iter().filter_map(|row| {
                    let arr = row.as_array()?;
                    let name = arr.get(col_idx("column_name")?)?.as_str()?.to_string();
                    let dtype = arr.get(col_idx("data_type")?)?.as_str()?.to_string();
                    let nullable = arr.get(col_idx("is_nullable")?)
                        .and_then(|v| v.as_str())
                        .map(|s| s == "YES")
                        .unwrap_or(false);
                    let pos = arr.get(col_idx("ordinal_position")?)
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32;
                    let is_pk = arr.get(col_idx("column_key")?)
                        .and_then(|v| v.as_str())
                        .map(|s| s == "PRI")
                        .unwrap_or(false);
                    Some(TableColumnMeta {
                        column_name: name,
                        data_type: dtype,
                        is_nullable: nullable,
                        is_primary_key: is_pk,
                        ordinal_position: pos,
                    })
                }).collect()
            }
            None => vec![],
        };

        Ok(columns)
    }

    async fn fetch_row_count(
        service: &SqlService,
        conn_id: Option<String>,
        database: &str,
        schema: &str,
        table: &str,
    ) -> Result<i64, CoreError> {
        use crate::core::services::sql_service::SqlExecuteOptions;

        let sql = format!("SELECT COUNT(*) AS cnt FROM `{}`.`{}`.`{}`", database, schema, table);

        let opts = SqlExecuteOptions {
            record_history: false,
            use_transaction: false,
            timeout_ms: Some(30000),
            use_cache: false,
        };

        let result = service.execute(conn_id, &sql, opts).await?;
        let json = serde_json::to_value(&result.result).map_err(|e| CoreError::common(
            CommonError::General(format!("Serialize error: {}", e))
        ))?;

        let count = json["batches"]
            .as_array()
            .and_then(|batches| batches.first())
            .and_then(|batch| batch["rows"].as_array())
            .and_then(|rows| rows.first())
            .and_then(|row| row.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(count)
    }

    /// 获取洞察存储用量统计
    pub async fn get_insight_storage_stats(
        insight_store: &crate::core::persistence::InsightStorage,
    ) -> Result<crate::core::persistence::InsightStorageStats, CoreError> {
        insight_store.columns.get_storage_stats().await
    }

    /// 获取指定版本的完整洞察数据
    pub async fn get_insight_version_detail(
        version_id: &str,
        insight_store: &crate::core::persistence::InsightStorage,
    ) -> Result<Option<ColumnInsightFull>, CoreError> {
        insight_store.columns.get_snapshot_by_version(version_id).await
    }

    /// 从真实表取样并生成列洞察（端到端：取样→DuckDB→洞察）
    pub async fn profile_column_from_table(
        conn_id: String,
        database: &str,
        schema: &str,
        table: &str,
        column_name: &str,
    ) -> Result<ColumnInsightFull, CoreError> {
        use crate::core::services::sql_service::SqlExecuteOptions;

        let manager = get_connection_manager().clone();
        let service = SqlService::new(manager);

        let sample_sql = format!(
            "SELECT * FROM `{}`.`{}`.`{}` LIMIT 500",
            database, schema, table
        );

        let opts = SqlExecuteOptions {
            record_history: false,
            use_transaction: false,
            timeout_ms: Some(15000),
            use_cache: false,
        };

        let result = service.execute(Some(conn_id.clone()), &sample_sql, opts).await?;
        let json = serde_json::to_value(&result.result).map_err(|e| CoreError::common(
            CommonError::General(format!("Serialize error: {}", e))
        ))?;

        let (columns, rows) = match json["batches"].as_array().and_then(|batches| batches.first()) {
            Some(batch) => {
                let cols: Vec<String> = batch["columns"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|c| c.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                let rows_data: Vec<Vec<serde_json::Value>> = batch["rows"]
                    .as_array()
                    .map(|arr| {
                        arr.iter().map(|row| {
                            row.as_array().cloned().unwrap_or_default()
                        }).collect()
                    })
                    .unwrap_or_default();

                (cols, rows_data)
            }
            None => (vec![], vec![]),
        };

        if columns.is_empty() {
            return Err(CoreError::common(CommonError::General(
                "无法从表中读取数据".to_string()
            )));
        }

        let temp_table = Self::create_duckdb_temp_table(&columns, &rows)?;

        let stats = Self::get_column_insight_full(&temp_table, column_name)?;

        Ok(stats)
    }

    pub async fn batch_evaluate_columns(
        conn_id: String,
        database: &str,
        schema: &str,
        table: &str,
    ) -> Result<TableQuality, CoreError> {
        use crate::core::services::sql_service::SqlExecuteOptions;

        let manager = get_connection_manager().clone();
        let service = SqlService::new(manager);

        let sample_sql = format!(
            "SELECT * FROM `{}`.`{}`.`{}` LIMIT 500",
            database, schema, table
        );

        let opts = SqlExecuteOptions {
            record_history: false,
            use_transaction: false,
            timeout_ms: Some(15000),
            use_cache: false,
        };

        let result = service.execute(Some(conn_id), &sample_sql, opts).await?;
        let json = serde_json::to_value(&result.result).map_err(|e| CoreError::common(
            CommonError::General(format!("Serialize error: {}", e))
        ))?;

        let (col_names, rows_data) = match json["batches"].as_array().and_then(|b| b.first()) {
            Some(batch) => {
                let cols: Vec<String> = batch["columns"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|c| c.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                let rows: Vec<Vec<serde_json::Value>> = batch["rows"]
                    .as_array()
                    .map(|arr| {
                        arr.iter().map(|row| {
                            row.as_array().cloned().unwrap_or_default()
                        }).collect()
                    })
                    .unwrap_or_default();

                (cols, rows)
            }
            None => (vec![], vec![]),
        };

        if col_names.is_empty() {
            return Ok(TableQuality {
                table_name: table.into(),
                overall_score: 0.0,
                level: "无数据".into(),
                column_scores: vec![],
                summary: "表为空或无数据".into(),
                scored_count: 0,
                total_columns: 0,
            });
        }

        let temp_table = Self::create_duckdb_temp_table(&col_names, &rows_data)?;

        let mut stats_list: Vec<ColumnInsightFull> = Vec::new();
        for col_name in &col_names {
            match Self::get_column_insight_full(&temp_table, col_name) {
                Ok(stats) => stats_list.push(stats),
                Err(_) => continue,
            }
        }

        Ok(Self::compute_table_quality(table, &stats_list))
    }

    pub fn compute_column_quality(stats: &ColumnInsightFull) -> QualityScore {
        let null_rate = stats.stats.null_rate;
        let total = stats.stats.total_count as f64;
        let unique = stats.stats.unique_count.unwrap_or(0) as f64;
        let non_null = total * (1.0 - null_rate);

        let completeness = if total > 0.0 {
            (1.0 - null_rate) * 100.0
        } else {
            0.0
        };

        let uniqueness = if non_null > 0.0 {
            let ratio = unique / non_null;
            if ratio > 0.9 {
                100.0
            } else if ratio > 0.5 {
                80.0
            } else if ratio > 0.2 {
                60.0
            } else if ratio > 0.05 {
                40.0
            } else if ratio > 0.01 {
                20.0
            } else {
                10.0
            }
        } else {
            0.0
        };

        let type_consistency = match stats.stats.stats_detail {
            ColumnStatsDetail::Numeric(_) => {
                if null_rate > 0.5 {
                    40.0
                } else {
                    90.0
                }
            }
            ColumnStatsDetail::Text(_) => {
                if unique < 2.0 {
                    30.0
                } else if null_rate > 0.6 {
                    40.0
                } else {
                    75.0
                }
            }
            ColumnStatsDetail::DateTime(_) => {
                let has_range = stats.histogram.as_ref()
                    .map_or(false, |h| h.len() > 1);
                if has_range { 85.0 } else { 60.0 }
            }
            ColumnStatsDetail::Boolean(_) => 95.0,
            ColumnStatsDetail::Unknown => 50.0,
        };

        fn detail_variant_name(detail: &ColumnStatsDetail) -> &str {
            match detail {
                ColumnStatsDetail::Numeric(_) => "Numeric",
                ColumnStatsDetail::Text(_) => "Text",
                ColumnStatsDetail::DateTime(_) => "DateTime",
                ColumnStatsDetail::Boolean(_) => "Boolean",
                ColumnStatsDetail::Unknown => "Unknown",
            }
        }

        let distribution = if let Some(ref hist) = stats.histogram {
            let bins = hist.len() as f64;
            if bins > 0.0 {
                let values: Vec<f64> = hist.iter().map(|b| b.count as f64).collect();
                let sum: f64 = values.iter().sum();
                if sum > 0.0 {
                    let avg = sum / bins;
                    let variance: f64 = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / bins;
                    let cv = variance.sqrt() / avg.max(1.0);
                    if cv < 0.3 {
                        90.0
                    } else if cv < 0.7 {
                        75.0
                    } else if cv < 1.5 {
                        50.0
                    } else {
                        30.0
                    }
                } else {
                    50.0
                }
            } else {
                50.0
            }
        } else {
            50.0
        };

        let unique_display = stats.stats.unique_count.unwrap_or(0);

        let dimensions = vec![
            QualityDimension {
                name: "完整性".into(),
                score: completeness,
                weight: 0.35,
                detail: format!("空值率 {:.1}%", null_rate * 100.0),
            },
            QualityDimension {
                name: "唯一性".into(),
                score: uniqueness,
                weight: 0.25,
                detail: format!("去重 {}/{}", unique_display, stats.stats.total_count),
            },
            QualityDimension {
                name: "类型一致".into(),
                score: type_consistency,
                weight: 0.20,
                detail: detail_variant_name(&stats.stats.stats_detail).into(),
            },
            QualityDimension {
                name: "分布均匀".into(),
                score: distribution,
                weight: 0.20,
                detail: "直方图分布评估".into(),
            },
        ];

        let overall: f64 = dimensions
            .iter()
            .map(|d| d.score * d.weight)
            .sum();

        let level = if overall >= 85.0 {
            "优秀"
        } else if overall >= 70.0 {
            "良好"
        } else if overall >= 50.0 {
            "一般"
        } else if overall >= 30.0 {
            "较差"
        } else {
            "差"
        };

        let summary = if overall >= 85.0 {
            format!("数据质量优秀 ({:.0}分)，可直接用于分析", overall)
        } else if overall >= 70.0 {
            format!("数据质量良好 ({:.0}分)，建议关注空值", overall)
        } else if overall >= 50.0 {
            format!("数据质量一般 ({:.0}分)，存在明显质量问题", overall)
        } else {
            format!("数据质量较差 ({:.0}分)，建议清洗后使用", overall)
        };

        QualityScore {
            column_name: stats.stats.column_name.clone(),
            overall_score: overall,
            level: level.into(),
            dimensions,
            summary,
        }
    }

    pub fn compute_table_quality(
        table_name: &str,
        stats_list: &[ColumnInsightFull],
    ) -> TableQuality {
        let mut entries: Vec<ColumnQualityEntry> = stats_list
            .iter()
            .map(|s| {
                let qs = Self::compute_column_quality(s);
                ColumnQualityEntry {
                    column_name: s.stats.column_name.clone(),
                    quality_score: qs.overall_score,
                    level: qs.level,
                    null_rate: s.stats.null_rate,
                }
            })
            .collect();

        entries.sort_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap_or(std::cmp::Ordering::Equal));

        let scored_count = entries.len();
        let total_columns = scored_count;
        let overall = if scored_count > 0 {
            entries.iter().map(|e| e.quality_score).sum::<f64>() / scored_count as f64
        } else {
            0.0
        };

        let level = if overall >= 85.0 {
            "优秀"
        } else if overall >= 70.0 {
            "良好"
        } else if overall >= 50.0 {
            "一般"
        } else if overall >= 30.0 {
            "较差"
        } else {
            "差"
        };

        let problem_columns = entries.iter().filter(|e| e.quality_score < 50.0).count();
        let summary = if scored_count == 0 {
            "无数据".into()
        } else if overall >= 85.0 {
            format!("表质量优秀 ({:.0}分)，{} 列均健康", overall, scored_count)
        } else if problem_columns > 0 {
            format!(
                "表质量{} ({:.0}分)，{} 列需关注 ({}风险列)",
                level, overall, scored_count, problem_columns
            )
        } else {
            format!("表质量{} ({:.0}分)，{} 列已评估", level, overall, scored_count)
        };

        TableQuality {
            table_name: table_name.into(),
            overall_score: overall,
            level: level.into(),
            column_scores: entries,
            summary,
            scored_count,
            total_columns,
        }
    }
}

fn sha256_hex(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}
