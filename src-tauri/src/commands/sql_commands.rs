//! SQL 执行相关命令
//!
//! 处理 SQL 查询、事务执行、历史记录等操作

use crate::core::get_connection_manager;
use crate::core::services::{SqlService, sql_service::{SqlExecuteOptions, SqlExecuteResult}};
use crate::core::dbi::engine::duckdb_engine::DuckDBEngine;
use crate::adapters::tauri::state::AppState;
use crate::api::dto::QueryResult;

// ==================== SQL Query Commands ====================

/// 执行 SQL 请求参数
#[derive(serde::Deserialize, Debug)]
pub struct ExecuteSqlInput {
    pub conn_id: Option<String>,
    pub sql: String,
    pub timeout_ms: Option<u64>,
}

/// 执行 SQL 响应
#[derive(serde::Serialize, Debug)]
pub struct ExecuteSqlResponse {
    pub result: QueryResult,
    pub elapsed_ms: u64,
    pub affected_rows: Option<usize>,
}

impl From<SqlExecuteResult> for ExecuteSqlResponse {
    fn from(result: SqlExecuteResult) -> Self {
        Self {
            result: result.result,
            elapsed_ms: result.elapsed_ms,
            affected_rows: result.affected_rows,
        }
    }
}

/// 执行 SQL 查询
#[tauri::command]
pub async fn execute_sql(input: ExecuteSqlInput) -> Result<ExecuteSqlResponse, String> {
    if input.sql.trim().is_empty() {
        return Err("SQL statement cannot be empty".into());
    }

    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    let options = SqlExecuteOptions {
        record_history: true,
        use_transaction: false,
        timeout_ms: input.timeout_ms,
        use_cache: true,
    };

    let result = service
        .execute(input.conn_id, &input.sql, options)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.into())
}

/// 在事务中执行多个 SQL
#[derive(serde::Deserialize, Debug)]
pub struct ExecuteTransactionInput {
    pub conn_id: Option<String>,
    pub sqls: Vec<String>,
}

#[derive(serde::Serialize, Debug)]
pub struct ExecuteTransactionResponse {
    pub results: Vec<ExecuteSqlResponse>,
}

#[tauri::command]
pub async fn execute_transaction(
    input: ExecuteTransactionInput,
) -> Result<ExecuteTransactionResponse, String> {
    if input.sqls.is_empty() {
        return Err("SQL statements cannot be empty".into());
    }

    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    let results = service
        .execute_in_transaction(input.conn_id, input.sqls)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ExecuteTransactionResponse {
        results: results.into_iter().map(|r| r.into()).collect(),
    })
}

// ==================== Transaction Commands ====================

/// 事务状态响应
#[derive(serde::Serialize, Debug)]
pub struct TransactionStatusResponse {
    pub conn_id: String,
    pub is_in_transaction: bool,
    pub transaction_start_time_ms: Option<u64>,
    pub transaction_duration_ms: Option<u64>,
}

/// 开始事务
#[tauri::command]
pub async fn begin_transaction(conn_id: Option<String>) -> Result<TransactionStatusResponse, String> {
    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    let result = service.begin_transaction(conn_id).await.map_err(|e| e.to_string())?;

    Ok(TransactionStatusResponse {
        conn_id: result.conn_id,
        is_in_transaction: result.is_in_transaction,
        transaction_start_time_ms: result.transaction_start_time_ms,
        transaction_duration_ms: result.transaction_duration_ms,
    })
}

/// 提交事务
#[tauri::command]
pub async fn commit_transaction(conn_id: Option<String>) -> Result<TransactionStatusResponse, String> {
    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    let result = service.commit_transaction(conn_id).await.map_err(|e| e.to_string())?;

    Ok(TransactionStatusResponse {
        conn_id: result.conn_id,
        is_in_transaction: result.is_in_transaction,
        transaction_start_time_ms: result.transaction_start_time_ms,
        transaction_duration_ms: result.transaction_duration_ms,
    })
}

/// 回滚事务
#[tauri::command]
pub async fn rollback_transaction(conn_id: Option<String>) -> Result<TransactionStatusResponse, String> {
    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    let result = service.rollback_transaction(conn_id).await.map_err(|e| e.to_string())?;

    Ok(TransactionStatusResponse {
        conn_id: result.conn_id,
        is_in_transaction: result.is_in_transaction,
        transaction_start_time_ms: result.transaction_start_time_ms,
        transaction_duration_ms: result.transaction_duration_ms,
    })
}

/// 取消正在执行的 SQL 查询
#[tauri::command]
pub async fn cancel_sql_query(conn_id: Option<String>) -> Result<bool, String> {
    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    service.cancel_query(conn_id).await.map_err(|e| e.to_string())
}

/// 获取事务状态
#[tauri::command]
pub async fn get_transaction_status(conn_id: Option<String>) -> Result<TransactionStatusResponse, String> {
    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    let result = service.get_transaction_status(conn_id).await.map_err(|e| e.to_string())?;

    Ok(TransactionStatusResponse {
        conn_id: result.conn_id,
        is_in_transaction: result.is_in_transaction,
        transaction_start_time_ms: result.transaction_start_time_ms,
        transaction_duration_ms: result.transaction_duration_ms,
    })
}

/// SQL 历史记录响应
#[derive(serde::Serialize, Debug)]
pub struct SqlHistoryResponse {
    pub id: String,
    pub sql: String,
    pub conn_id: Option<String>,
    pub executed_at: String,
}

/// 获取 SQL 执行历史
#[tauri::command]
pub async fn get_sql_history(limit: Option<usize>) -> Result<Vec<SqlHistoryResponse>, String> {
    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    let history = service
        .get_sql_history(limit.unwrap_or(100))
        .map_err(|e| e.to_string())?;

    Ok(history
        .into_iter()
        .map(|h| SqlHistoryResponse {
            id: h.id,
            sql: h.sql,
            conn_id: h.conn_id,
            executed_at: h.executed_at.to_rfc3339(),
        })
        .collect())
}

/// 搜索 SQL 历史
#[tauri::command]
pub async fn search_sql_history(
    keyword: String,
    limit: Option<usize>,
) -> Result<Vec<SqlHistoryResponse>, String> {
    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    let history = service
        .search_sql_history(&keyword, limit.unwrap_or(100))
        .map_err(|e| e.to_string())?;

    Ok(history
        .into_iter()
        .map(|h| SqlHistoryResponse {
            id: h.id,
            sql: h.sql,
            conn_id: h.conn_id,
            executed_at: h.executed_at.to_rfc3339(),
        })
        .collect())
}

/// 清空 SQL 历史
#[tauri::command]
pub async fn clear_sql_history() -> Result<(), String> {
    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    service.clear_sql_history().map_err(|e| e.to_string())
}

/// 删除单条 SQL 历史
#[tauri::command]
pub async fn remove_sql_history(id: String) -> Result<(), String> {
    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    service.remove_sql_history(&id).map_err(|e| e.to_string())
}

// ==================== Federated Query Commands ====================

/// 注册外部数据库连接请求参数
#[derive(serde::Deserialize, Debug)]
pub struct RegisterExternalDatabaseInput {
    pub conn_id: Option<String>,
    pub name: String,
    pub driver: String,
    pub connection_string: String,
}

/// 注册外部数据库连接
#[tauri::command]
pub async fn register_external_database(
    input: RegisterExternalDatabaseInput
) -> Result<(), String> {
    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    service.register_external_database(
        input.conn_id,
        &input.name,
        &input.driver,
        &input.connection_string
    )
    .await
    .map_err(|e| e.to_string())
}

/// DuckDB 加速执行输入
#[derive(Debug, serde::Deserialize)]
pub struct DuckDBAcceleratedInput {
    /// SQL 查询
    pub sql: String,
    /// 源数据库连接 ID
    pub conn_id: String,
    /// 数据库类型
    pub db_type: Option<String>,
    /// APP 数据目录（扩展存储）
    pub data_dir: Option<String>,
}

/// DuckDB 加速执行响应
#[derive(Debug, serde::Serialize)]
pub struct DuckDBAcceleratedResponse {
    pub success: bool,
    pub columns: Option<Vec<String>>,
    pub rows: Option<Vec<Vec<serde_json::Value>>>,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

/// 使用 DuckDB 加速引擎执行 SQL（含自动 ATTACH / DETACH）
#[tauri::command]
pub async fn execute_duckdb_accelerated(
    state: tauri::State<'_, AppState>,
    input: DuckDBAcceleratedInput,
) -> Result<DuckDBAcceleratedResponse, String> {
    use crate::core::dbi::engine::{ExecutionEngine, ExecutionMode};
    use crate::core::dbi::context::QueryContext;

    let manager = get_connection_manager().clone();
    let conn_info = manager
        .get_connection_info(&input.conn_id)
        .await
        .ok_or_else(|| format!("Connection not found: {}", input.conn_id))?;

    let attach_type = match conn_info.db_type.as_str() {
        "mysql" => "mysql",
        "postgresql" | "postgres" => "postgres",
        "sqlite" => "sqlite",
        other => {
            return Err(format!(
                "Unsupported database type for DuckDB acceleration: {}",
                other
            ))
        }
    };

    let sanitized = conn_info
        .name
        .replace(
            |c: char| !c.is_alphanumeric() && c != '_',
            "_",
        );
    let attach_name = format!("ext_{}", sanitized);
    let attach_sql = format!(
        "ATTACH '{}' AS {} (TYPE {})",
        conn_info.url, attach_name, attach_type
    );

    let engine = state.duckdb_engine.lock().await;

    // Step 1: Init extensions + ATTACH source database
    {
        let conn = engine.conn().map_err(|e| e.to_string())?;
        if let Some(ref data_dir) = input.data_dir {
            DuckDBEngine::init_extensions(&conn, data_dir).map_err(|e| e.to_string())?;
        }
        conn.execute_batch(&attach_sql)
            .map_err(|e| format!("Failed to ATTACH source database: {}", e))?;
    }

    // Step 2: Execute user SQL via DuckDB engine
    let ctx = QueryContext::new(Some(input.conn_id.clone()), ExecutionMode::DuckDB);
    let result = engine
        .execute(&input.sql, &ctx)
        .await
        .map_err(|e| e.to_string())?;

    // Step 3: DETACH (best-effort, don't fail on error)
    {
        let conn = engine.conn().map_err(|e| e.to_string())?;
        let _ = conn.execute_batch(&format!("DETACH IF EXISTS {}", attach_name));
    }

    let columns: Vec<String> = result
        .batches
        .first()
        .map(|b| {
            b.schema()
                .fields()
                .iter()
                .map(|f| f.name().clone())
                .collect()
        })
        .unwrap_or_default();

    let rows: Vec<Vec<serde_json::Value>> = result.batches.iter()
        .flat_map(|batch| {
            let mut batch_rows = Vec::new();
            for row_idx in 0..batch.num_rows() {
                let row: Vec<serde_json::Value> = (0..batch.num_columns())
                    .map(|col_idx| {
                        let col = batch.column(col_idx);
                        format_arrow_value(col, row_idx)
                    })
                    .collect();
                batch_rows.push(row);
            }
            batch_rows
        })
        .collect();

    Ok(DuckDBAcceleratedResponse {
        success: true,
        columns: if columns.is_empty() { None } else { Some(columns) },
        rows: if rows.is_empty() { None } else { Some(rows) },
        elapsed_ms: 0,
        error: None,
    })
}

fn format_arrow_value(col: &dyn arrow::array::Array, row_idx: usize) -> serde_json::Value {
    use arrow::array::*;
    use serde_json::Value;

    if col.is_null(row_idx) {
        return Value::Null;
    }

    macro_rules! as_array {
        ($col:expr, $ty:ty) => {
            $col.as_any().downcast_ref::<$ty>().map(|a| a.value(row_idx))
        };
    }

    if let Some(v) = as_array!(col, Int8Array) { return Value::Number(serde_json::Number::from(v)); }
    if let Some(v) = as_array!(col, Int16Array) { return Value::Number(serde_json::Number::from(v)); }
    if let Some(v) = as_array!(col, Int32Array) { return Value::Number(serde_json::Number::from(v)); }
    if let Some(v) = as_array!(col, Int64Array) { return Value::Number(serde_json::Number::from(v)); }
    if let Some(v) = as_array!(col, UInt8Array) { return Value::Number(serde_json::Number::from(v)); }
    if let Some(v) = as_array!(col, UInt16Array) { return Value::Number(serde_json::Number::from(v)); }
    if let Some(v) = as_array!(col, UInt32Array) { return Value::Number(serde_json::Number::from(v)); }
    if let Some(v) = as_array!(col, UInt64Array) { return Value::Number(serde_json::Number::from(v)); }
    if let Some(v) = as_array!(col, Float32Array) { return serde_json::Number::from_f64(v as f64).map(Value::Number).unwrap_or(Value::Null); }
    if let Some(v) = as_array!(col, Float64Array) { return serde_json::Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null); }

    if let Some(arr) = col.as_any().downcast_ref::<StringArray>() {
        return Value::String(arr.value(row_idx).to_string());
    }

    Value::String(format!("{}", arrow::util::display::array_value_to_string(col, row_idx).unwrap_or_default()))
}

/// 创建外部表请求参数
#[derive(serde::Deserialize, Debug)]
pub struct CreateExternalTableInput {
    pub conn_id: Option<String>,
    pub external_db_name: String,
    pub schema_name: String,
    pub table_name: String,
    pub external_table_name: String,
}

/// 创建外部表
#[tauri::command]
pub async fn create_external_table(
    input: CreateExternalTableInput
) -> Result<(), String> {
    let manager = get_connection_manager().clone();
    let service = SqlService::new(manager);

    service.create_external_table(
        input.conn_id,
        &input.external_db_name,
        &input.schema_name,
        &input.table_name,
        &input.external_table_name
    )
    .await
    .map_err(|e| e.to_string())
}
