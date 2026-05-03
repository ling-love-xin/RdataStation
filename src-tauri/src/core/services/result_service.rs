/**
 * 结果集服务
 *
 * 提供结果集的二次分析功能：
 * - SQL 过滤（拼接 WHERE 重新查询）
 * - DuckDB 深度分析（针对临时表）
 * - 列洞察统计
 */

use std::sync::Arc;

use crate::core::get_connection_manager;
use crate::core::services::sql_service::SqlExecuteOptions;
use crate::core::services::SqlService;
use crate::core::error::{CoreError, CommonError};

/// 结果集响应
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ResultSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub elapsed_ms: u64,
    pub temp_table: String,
}

/// 列统计信息
#[derive(Debug, serde::Serialize)]
pub struct ColumnStats {
    pub column_name: String,
    pub data_type: String,
    pub total_count: usize,
    pub null_count: usize,
    pub unique_count: Option<usize>,
    pub numeric_stats: Option<NumericStats>,
    pub text_stats: Option<TextStats>,
}

#[derive(Debug, serde::Serialize)]
pub struct NumericStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub median: f64,
    pub sum: f64,
    pub stddev: Option<f64>,
}

#[derive(Debug, serde::Serialize)]
pub struct TextStats {
    pub min_length: usize,
    pub max_length: usize,
    pub top_values: Vec<(String, usize)>,
}

/// 结果集服务
pub struct ResultService;

impl ResultService {
    /// SQL 过滤：拼接 WHERE 重新查询
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

        // 通过 serde 序列化获取标准化的行列结构
        let json_value = serde_json::to_value(&result.result).map_err(|e| CoreError::common(
            CommonError::General(format!("Serialize error: {}", e))
        ))?;

        let columns = json_value["columns"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<String>>())
            .unwrap_or(vec![]);

        // 从序列化后的 result.batches 中提取行数据
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

    /// DuckDB 分析：对临时表执行 SQL
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

    /// 获取列洞察统计
    pub fn get_column_insights(
        temp_table: &str,
        column_name: &str,
    ) -> Result<ColumnStats, CoreError> {
        let duckdb = Self::get_or_create_duckdb()?;
        let conn = duckdb.lock().map_err(|e| CoreError::common(
            CommonError::General(format!("DuckDB lock error: {}", e))
        ))?;

        let count_sql = format!(
            "SELECT COUNT(*), COUNT(\"{}\"), COUNT(DISTINCT \"{}\") FROM \"{}\"",
            column_name, column_name, temp_table
        );
        let (total, non_null, unique_cnt): (i64, i64, i64) = conn.query_row(&count_sql, [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).map_err(|e| CoreError::common(CommonError::General(
            format!("DuckDB count failed: {}", e)
        )))?;

        let null_count = (total - non_null) as usize;

        let type_sql = format!(
            "SELECT typeof(\"{}\") FROM \"{}\" WHERE \"{}\" IS NOT NULL LIMIT 1",
            column_name, temp_table, column_name
        );
        let data_type: String = conn.query_row(&type_sql, [], |row| row.get(0))
            .unwrap_or_else(|_| "VARCHAR".to_string());

        let is_numeric = matches!(data_type.to_lowercase().as_str(),
            "bigint" | "integer" | "smallint" | "tinyint" | "double" | "float" | "hugeint" | "decimal" | "numeric"
        );

        if is_numeric {
            let stats_sql = format!(
                "SELECT MIN(\"{}\")::DOUBLE, MAX(\"{}\")::DOUBLE, AVG(\"{}\")::DOUBLE, \
                 MEDIAN(\"{}\")::DOUBLE, SUM(\"{}\")::DOUBLE, STDDEV_SAMP(\"{}\")::DOUBLE \
                 FROM \"{}\"",
                column_name, column_name, column_name, column_name, column_name, column_name, temp_table
            );
            let (min_v, max_v, avg_v, median_v, sum_v, stddev_v): (f64, f64, f64, f64, f64, Option<f64>) =
                conn.query_row(&stats_sql, [], |row| Ok((
                    row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5).ok(),
                ))).map_err(|e| CoreError::common(CommonError::General(
                    format!("DuckDB numeric stats failed: {}", e)
                )))?;

            Ok(ColumnStats {
                column_name: column_name.to_string(),
                data_type,
                total_count: total as usize,
                null_count,
                unique_count: Some(unique_cnt as usize),
                numeric_stats: Some(NumericStats { min: min_v, max: max_v, avg: avg_v, median: median_v, sum: sum_v, stddev: stddev_v }),
                text_stats: None,
            })
        } else {
            let top_sql = format!(
                "SELECT \"{}\"::VARCHAR, COUNT(*) FROM \"{}\" \
                 WHERE \"{}\" IS NOT NULL GROUP BY 1 ORDER BY 2 DESC LIMIT 10",
                column_name, temp_table, column_name
            );
            let mut stmt = conn.prepare(&top_sql).map_err(|e| CoreError::common(
                CommonError::General(format!("DuckDB prepare failed: {}", e))
            ))?;
            let top_values: Vec<(String, usize)> = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
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

            Ok(ColumnStats {
                column_name: column_name.to_string(),
                data_type,
                total_count: total as usize,
                null_count,
                unique_count: Some(unique_cnt as usize),
                numeric_stats: None,
                text_stats: Some(TextStats {
                    min_length: min_len as usize,
                    max_length: max_len as usize,
                    top_values,
                }),
            })
        }
    }

    // ==================== 辅助方法 ====================

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
                match v {
                    duckdb::types::Value::Null => serde_json::Value::Null,
                    duckdb::types::Value::Boolean(b) => serde_json::json!(b),
                    duckdb::types::Value::TinyInt(n) => serde_json::json!(n),
                    duckdb::types::Value::SmallInt(n) => serde_json::json!(n),
                    duckdb::types::Value::Int(n) => serde_json::json!(n),
                    duckdb::types::Value::BigInt(n) => serde_json::json!(n),
                    duckdb::types::Value::Float(f) => serde_json::json!(f),
                    duckdb::types::Value::Double(f) => serde_json::json!(f),
                    duckdb::types::Value::Text(s) => serde_json::Value::String(s),
                    _ => serde_json::Value::Null,
                }
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

/// 从 serde 序列化后的 JSON 中提取行数据
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
            // data 是 { "col_name": [v1, v2, ...], ... } 格式
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
