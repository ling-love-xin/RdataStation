use std::collections::HashMap;

use crate::core::error::{CommonError, CoreError};
use crate::core::insight;
use crate::core::insight::RuleExecutor;
use crate::core::services::duckdb_service::{
    duckdb_value_to_json, is_datetime_type, is_numeric_type, DuckDbService,
};
use crate::core::services::result_service::{
    ColumnInsightFull, ColumnStats, ColumnStatsDetail, DateTimeStats, DistributionBin,
    NumericStats, TextFrequency, TextStats,
};

pub(crate) fn get_or_create_duckdb() -> Result<
    std::sync::Arc<std::sync::Mutex<duckdb::Connection>>,
    CoreError,
> {
    DuckDbService::get_or_create_duckdb()
}

pub(crate) fn get_column_insight_full(
    temp_table: &str,
    column_name: &str,
) -> Result<ColumnInsightFull, CoreError> {
    let duckdb = get_or_create_duckdb()?;
    let conn = duckdb.lock().map_err(|e| {
        CoreError::common(CommonError::General(format!("DuckDB lock error: {}", e)))
    })?;

    let stats = get_column_stats_internal(&conn, temp_table, column_name)?;
    let sample = get_column_sample_internal(&conn, temp_table, column_name)?;

    let histogram = match &stats.stats_detail {
        ColumnStatsDetail::Numeric(_) => {
            get_column_histogram_internal(&conn, temp_table, column_name).ok()
        }
        _ => None,
    };

    Ok(ColumnInsightFull {
        stats,
        sample,
        histogram,
    })
}

pub(crate) fn get_column_insights(
    temp_table: &str,
    column_name: &str,
) -> Result<ColumnStats, CoreError> {
    let duckdb = get_or_create_duckdb()?;
    let conn = duckdb.lock().map_err(|e| {
        CoreError::common(CommonError::General(format!("DuckDB lock error: {}", e)))
    })?;
    get_column_stats_internal(&conn, temp_table, column_name)
}

fn get_column_stats_internal(
    conn: &duckdb::Connection,
    temp_table: &str,
    column_name: &str,
) -> Result<ColumnStats, CoreError> {
    let count_sql = format!(
        "SELECT COUNT(*), COUNT(\"{}\"), COUNT(DISTINCT \"{}\") FROM \"{}\"",
        column_name, column_name, temp_table
    );
    let (total, non_null, unique_cnt): (i64, i64, i64) = conn
        .query_row(&count_sql, [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| {
            CoreError::common(CommonError::General(format!(
                "DuckDB count failed for '{}': {}",
                column_name, e
            )))
        })?;

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
    let data_type: String = conn
        .query_row(&type_sql, [], |row| row.get(0))
        .unwrap_or_else(|_| "VARCHAR".to_string());

    let dt_lower = data_type.to_lowercase();

    let stats_detail = if is_numeric_type(&dt_lower) {
        compute_numeric_stats(conn, temp_table, column_name)?
    } else if is_datetime_type(&dt_lower) {
        compute_datetime_stats(conn, temp_table, column_name)?
    } else if dt_lower == "boolean" {
        compute_boolean_stats(conn, temp_table, column_name)?
    } else {
        compute_text_stats(conn, temp_table, column_name)?
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
    let registry = insight::global_registry().read().map_err(|e| {
        CoreError::common(CommonError::General(format!("Registry lock error: {}", e)))
    })?;
    let mut params = HashMap::new();
    params.insert("table".to_string(), temp_table.to_string());
    params.insert("col".to_string(), column_name.to_string());

    let rule = registry.get("numeric-stats").ok_or_else(|| {
        CoreError::common(CommonError::General(
            "Rule 'numeric-stats' not found".to_string(),
        ))
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
            let is_extreme =
                crate::core::services::duckdb_service::detect_extremes(
                    min_v,
                    max_v,
                    stddev_v.unwrap_or(0.0),
                );

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
    let registry = insight::global_registry().read().map_err(|e| {
        CoreError::common(CommonError::General(format!("Registry lock error: {}", e)))
    })?;
    let mut params = HashMap::new();
    params.insert("table".to_string(), temp_table.to_string());
    params.insert("col".to_string(), column_name.to_string());

    let freq_rule = registry.get("text-frequency").ok_or_else(|| {
        CoreError::common(CommonError::General(
            "Rule 'text-frequency' not found".to_string(),
        ))
    })?;

    let top_values: Vec<TextFrequency> = match RuleExecutor::execute(freq_rule, conn, &params) {
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

    let len_rule = registry.get("text-length").ok_or_else(|| {
        CoreError::common(CommonError::General(
            "Rule 'text-length' not found".to_string(),
        ))
    })?;
    let (min_len, max_len): (usize, usize) =
        match RuleExecutor::execute(len_rule, conn, &params) {
            Ok(serde_json::Value::Object(map)) => (
                map.get("min_length")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as usize,
                map.get("max_length")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as usize,
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
    let registry = insight::global_registry().read().map_err(|e| {
        CoreError::common(CommonError::General(format!("Registry lock error: {}", e)))
    })?;
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
            map.get("span_days")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
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
    let registry = insight::global_registry().read().map_err(|e| {
        CoreError::common(CommonError::General(format!("Registry lock error: {}", e)))
    })?;
    let mut params = HashMap::new();
    params.insert("table".to_string(), temp_table.to_string());
    params.insert("col".to_string(), column_name.to_string());

    let rule = registry.get("boolean-ratio").ok_or_else(|| {
        CoreError::common(CommonError::General(
            "Rule 'boolean-ratio' not found".to_string(),
        ))
    })?;

    match RuleExecutor::execute(rule, conn, &params) {
        Ok(serde_json::Value::Object(map)) => {
            let true_count =
                map.get("true_count").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
            let false_count = map
                .get("false_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as usize;
            let total = true_count + false_count;
            let true_ratio = if total > 0 {
                true_count as f64 / total as f64
            } else {
                0.0
            };
            Ok(ColumnStatsDetail::Boolean(
                crate::core::services::result_service::BooleanStats {
                    true_count,
                    false_count,
                    true_ratio,
                },
            ))
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
    let mut stmt = conn.prepare(&sql).map_err(|e| {
        CoreError::common(CommonError::General(format!(
            "DuckDB sample prepare failed: {}",
            e
        )))
    })?;
    let samples: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            let v: duckdb::types::Value = row.get_unwrap(0);
            Ok(duckdb_value_to_json(&v))
        })
        .map_err(|e| {
            CoreError::common(CommonError::General(format!(
                "DuckDB sample query failed: {}",
                e
            )))
        })?
        .filter_map(|r| r.ok())
        .collect();

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
    let count: i64 = conn
        .query_row(&count_sql, [], |row| row.get(0))
        .unwrap_or(0);

    if count < 10 {
        return Ok(vec![]);
    }

    let registry = insight::global_registry().read().map_err(|e| {
        CoreError::common(CommonError::General(format!("Registry lock error: {}", e)))
    })?;
    let mut params = HashMap::new();
    params.insert("table".to_string(), temp_table.to_string());
    params.insert("col".to_string(), column_name.to_string());

    let rule = registry.get("histogram").ok_or_else(|| {
        CoreError::common(CommonError::General(
            "Rule 'histogram' not found".to_string(),
        ))
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

pub(crate) fn execute_insight_rule(
    rule_id: &str,
    conn: &duckdb::Connection,
    params: &HashMap<String, String>,
) -> Result<insight::ExecutionResult, CoreError> {
    let registry = insight::global_registry().read().map_err(|e| {
        CoreError::common(CommonError::General(format!("Registry lock error: {}", e)))
    })?;
    let rule = registry.get(rule_id).ok_or_else(|| {
        CoreError::common(CommonError::General(format!(
            "Rule '{}' not found",
            rule_id
        )))
    })?;
    RuleExecutor::execute_qualified(rule, conn, params)
}

pub(crate) fn list_insight_rules(
    category: Option<&str>,
) -> Result<Vec<serde_json::Value>, CoreError> {
    let registry = insight::global_registry().read().map_err(|e| {
        CoreError::common(CommonError::General(format!(
            "Failed to lock insight registry: {}",
            e
        )))
    })?;
    let rules: Vec<&crate::core::insight::RuleFile> = match category {
        Some(cat) => registry.list_by_category(cat),
        None => registry.all_rules(),
    };
    let result: Vec<serde_json::Value> = rules
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
        .collect();
    Ok(result)
}

pub(crate) fn list_rules_for_column(
    column_type: &str,
) -> Result<Vec<serde_json::Value>, CoreError> {
    let registry = insight::global_registry().read().map_err(|e| {
        CoreError::common(CommonError::General(format!(
            "Failed to lock insight registry: {}",
            e
        )))
    })?;
    let result: Vec<serde_json::Value> = registry
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
        .collect();
    Ok(result)
}
