//! SQL 执行相关命令
//!
//! 处理 SQL 查询、事务执行、历史记录等操作

use crate::core::get_connection_manager;
use crate::core::services::{SqlService, sql_service::{SqlExecuteOptions, SqlExecuteResult}};
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
