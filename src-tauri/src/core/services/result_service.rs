/**
 * 结果集服务 + 洞察计算引擎
 *
 * 提供：
 * - SQL 过滤（拼接 WHERE 重新查询）
 * - DuckDB 深度分析（针对临时表）
 * - 列洞察全量统计（统计 + 样本 + 直方图）
 */

use std::sync::Arc;

use crate::core::get_connection_manager;
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
        let stats_sql = format!(
            "SELECT \
             MIN(\"{}\")::DOUBLE, MAX(\"{}\")::DOUBLE, AVG(\"{}\")::DOUBLE, \
             MEDIAN(\"{}\")::DOUBLE, \
             PERCENTILE_DISC(0.25) WITHIN GROUP (ORDER BY \"{}\")::DOUBLE, \
             PERCENTILE_DISC(0.75) WITHIN GROUP (ORDER BY \"{}\")::DOUBLE, \
             SUM(\"{}\")::DOUBLE, \
             STDDEV_SAMP(\"{}\")::DOUBLE, \
             SKEWNESS(\"{}\")::DOUBLE, \
             KURTOSIS(\"{}\")::DOUBLE \
             FROM \"{}\"",
            column_name, column_name, column_name,
            column_name,
            column_name, column_name,
            column_name,
            column_name,
            column_name, column_name,
            temp_table
        );

        let row_result: Result<(f64, f64, f64, f64, f64, f64, f64, Option<f64>, Option<f64>, Option<f64>), _> =
            conn.query_row(&stats_sql, [], |row| Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get::<_, Option<f64>>(7)?,
                row.get::<_, Option<f64>>(8)?,
                row.get::<_, Option<f64>>(9)?,
            )));

        match row_result {
            Ok((min_v, max_v, avg_v, median_v, p25, p75, sum_v, stddev_v, skewness_v, kurtosis_v)) => {
                let is_extreme = detect_extremes(min_v, max_v, stddev_v.unwrap_or(0.0));

                Ok(ColumnStatsDetail::Numeric(NumericStats {
                    min: min_v,
                    max: max_v,
                    avg: avg_v,
                    median: median_v,
                    p25,
                    p75,
                    sum: sum_v,
                    stddev: stddev_v,
                    skewness: skewness_v,
                    kurtosis: kurtosis_v,
                    is_extreme,
                }))
            }
            Err(e) => {
                let fallback_sql = format!(
                    "SELECT \
                     MIN(\"{}\")::DOUBLE, MAX(\"{}\")::DOUBLE, AVG(\"{}\")::DOUBLE, \
                     MEDIAN(\"{}\")::DOUBLE, \
                     SUM(\"{}\")::DOUBLE, \
                     STDDEV_SAMP(\"{}\")::DOUBLE \
                     FROM \"{}\"",
                    column_name, column_name, column_name,
                    column_name,
                    column_name,
                    column_name,
                    temp_table
                );
                let (min_v, max_v, avg_v, median_v, sum_v, stddev_v): (f64, f64, f64, f64, f64, Option<f64>) =
                    conn.query_row(&fallback_sql, [], |row| Ok((
                        row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get::<_, Option<f64>>(5)?,
                    ))).map_err(|_| CoreError::common(CommonError::General(
                        format!("DuckDB numeric fallback failed: {}", e)
                    )))?;

                Ok(ColumnStatsDetail::Numeric(NumericStats {
                    min: min_v,
                    max: max_v,
                    avg: avg_v,
                    median: median_v,
                    p25: 0.0,
                    p75: 0.0,
                    sum: sum_v,
                    stddev: stddev_v,
                    skewness: None,
                    kurtosis: None,
                    is_extreme: vec![],
                }))
            }
        }
    }

    fn compute_text_stats(
        conn: &duckdb::Connection,
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnStatsDetail, CoreError> {
        let top_sql = format!(
            "SELECT \"{}\"::VARCHAR, COUNT(*), COUNT(*) * 1.0 / SUM(COUNT(*)) OVER() \
             FROM \"{}\" WHERE \"{}\" IS NOT NULL GROUP BY 1 ORDER BY 2 DESC LIMIT 10",
            column_name, temp_table, column_name
        );
        let mut stmt = conn.prepare(&top_sql).map_err(|e| CoreError::common(
            CommonError::General(format!("DuckDB prepare failed: {}", e))
        ))?;
        let top_values: Vec<TextFrequency> = stmt.query_map([], |row| {
            Ok(TextFrequency {
                value: row.get::<_, String>(0)?,
                count: row.get::<_, i64>(1)? as usize,
                ratio: row.get::<_, f64>(2)?,
            })
        }).map_err(|e| CoreError::common(CommonError::General(
            format!("DuckDB top values query failed: {}", e)
        )))?.filter_map(|r| r.ok()).collect();

        let len_sql = format!(
            "SELECT MIN(LENGTH(\"{}\"::VARCHAR)), MAX(LENGTH(\"{}\"::VARCHAR)) \
             FROM \"{}\" WHERE \"{}\" IS NOT NULL",
            column_name, column_name, temp_table, column_name
        );
        let (min_len, max_len): (i64, i64) = conn.query_row(&len_sql, [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).unwrap_or((0, 0));

        Ok(ColumnStatsDetail::Text(TextStats {
            min_length: min_len as usize,
            max_length: max_len as usize,
            top_values,
        }))
    }

    fn compute_datetime_stats(
        conn: &duckdb::Connection,
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnStatsDetail, CoreError> {
        let range_sql = format!(
            "SELECT \
             MIN(\"{}\")::VARCHAR, MAX(\"{}\")::VARCHAR, \
             DATEDIFF('day', MIN(\"{}\"), MAX(\"{}\")) \
             FROM \"{}\" WHERE \"{}\" IS NOT NULL",
            column_name, column_name,
            column_name, column_name,
            temp_table, column_name
        );
        let (earliest, latest, span): (String, String, i64) = conn.query_row(&range_sql, [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).unwrap_or_else(|_| ("N/A".to_string(), "N/A".to_string(), 0));

        let monthly_distribution = Self::compute_monthly_distribution(
            conn, temp_table, column_name
        ).unwrap_or_default();

        Ok(ColumnStatsDetail::DateTime(DateTimeStats {
            earliest,
            latest,
            span_days: span,
            monthly_distribution,
        }))
    }

    fn compute_monthly_distribution(
        conn: &duckdb::Connection,
        temp_table: &str,
        column_name: &str,
    ) -> Result<Vec<TextFrequency>, CoreError> {
        let sql = format!(
            "SELECT \
             STRFTIME(\"{}\", '%Y-%m') AS month, \
             COUNT(*) AS cnt, \
             COUNT(*) * 1.0 / SUM(COUNT(*)) OVER() AS ratio \
             FROM \"{}\" WHERE \"{}\" IS NOT NULL \
             GROUP BY 1 ORDER BY 1",
            column_name, temp_table, column_name
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| CoreError::common(
            CommonError::General(format!("DuckDB prepare monthly failed: {}", e))
        ))?;
        let results: Vec<TextFrequency> = stmt.query_map([], |row| {
            Ok(TextFrequency {
                value: row.get::<_, String>(0)?,
                count: row.get::<_, i64>(1)? as usize,
                ratio: row.get::<_, f64>(2)?,
            })
        }).map_err(|e| CoreError::common(CommonError::General(
            format!("DuckDB monthly query failed: {}", e)
        )))?.filter_map(|r| r.ok()).collect();

        Ok(results)
    }

    fn compute_boolean_stats(
        conn: &duckdb::Connection,
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnStatsDetail, CoreError> {
        let sql = format!(
            "SELECT \
             COUNT(*) FILTER (WHERE \"{}\"), \
             COUNT(*) FILTER (WHERE NOT \"{}\") \
             FROM \"{}\"",
            column_name, column_name, temp_table
        );
        let (true_count, false_count): (i64, i64) = conn.query_row(&sql, [], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        }).map_err(|e| CoreError::common(CommonError::General(
            format!("DuckDB boolean stats failed: {}", e)
        )))?;

        let total = true_count + false_count;
        let true_ratio = if total > 0 {
            true_count as f64 / total as f64
        } else {
            0.0
        };

        Ok(ColumnStatsDetail::Boolean(BooleanStats {
            true_count: true_count as usize,
            false_count: false_count as usize,
            true_ratio,
        }))
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

        let bins = if count < 10 {
            vec![]
        } else {
            let bin_count = 10;
            let sql = format!(
                "WITH bounds AS ( \
                 SELECT MIN(\"{}\")::DOUBLE AS lo, MAX(\"{}\")::DOUBLE AS hi \
                 FROM \"{}\" WHERE \"{}\" IS NOT NULL \
                 ), \
                 bins AS (SELECT UNNEST(GENERATE_SERIES(1, {})) AS r) \
                 SELECT \
                 CASE \
                   WHEN r = {} THEN CAST(> AS VARCHAR) || ' ' || CAST(lo + ({}-1.0) * (hi - lo) / {} AS VARCHAR) \
                   ELSE CAST(lo + (r - 1) * (hi - lo) / {} AS VARCHAR) \
                   || ' ~ ' || CAST(lo + r * (hi - lo) / {} AS VARCHAR) \
                 END AS label, \
                 COUNT(\"{}\") AS cnt \
                 FROM \"{}\", bounds, bins \
                 WHERE \"{}\" IS NOT NULL \
                   AND \"{}\" >= lo + (r - 1) * (hi - lo) / {} \
                   AND (\"{}\" < lo + r * (hi - lo) / {} OR r = {}) \
                 GROUP BY r, lo, hi ORDER BY r",
                column_name, column_name, temp_table, column_name,
                bin_count,
                bin_count, bin_count, bin_count as f64,
                bin_count as f64, bin_count as f64,
                column_name, temp_table,
                column_name,
                column_name, bin_count as f64,
                column_name, bin_count as f64, bin_count,
            );
            let mut stmt = match conn.prepare(&sql) {
                Ok(s) => s,
                Err(_) => return Ok(vec![]),
            };
            let raw: Vec<(String, i64)> = match stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            }) {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
                Err(_) => vec![],
            };

            raw.into_iter().map(|(label, cnt)| DistributionBin {
                label,
                count: cnt as usize,
                ratio: if count > 0 { cnt as f64 / count as f64 } else { 0.0 },
            }).collect()
        };

        Ok(bins)
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

    fn get_or_create_duckdb() -> Result<Arc<std::sync::Mutex<duckdb::Connection>>, CoreError> {
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
}

fn sha256_hex(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}
