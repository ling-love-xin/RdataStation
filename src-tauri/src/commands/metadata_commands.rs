use arrow::array::{Array, StringArray};
use serde::{Deserialize, Serialize};

use crate::core::get_connection_manager;
use crate::core::models::QueryResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMeta {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMeta {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMeta {
    pub name: String,
    #[serde(rename = "type")]
    pub table_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewMeta {
    pub name: String,
    #[serde(rename = "type")]
    pub view_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMeta {
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: String,
    #[serde(rename = "isNullable")]
    pub is_nullable: bool,
    #[serde(rename = "defaultValue")]
    pub default_value: Option<String>,
    #[serde(rename = "isPrimaryKey")]
    pub is_primary_key: bool,
}

#[tauri::command]
pub async fn load_databases(conn_id: String) -> Result<Vec<DatabaseMeta>, String> {
    let manager = get_connection_manager().clone();
    let db = manager.get_connection(&conn_id).await
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    let names = db.list_databases().await
        .map_err(|e| e.to_string())?;

    Ok(names.into_iter().map(|name| DatabaseMeta { name }).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogMeta {
    pub name: String,
}

/// ANSI SQL 标准：Catalog（目录）是顶层容器，包含多个 Schema
/// 内部委托给 Database trait 的 list_databases() 方法
#[tauri::command]
pub async fn load_catalogs(conn_id: String) -> Result<Vec<CatalogMeta>, String> {
    let manager = get_connection_manager().clone();
    let db = manager.get_connection(&conn_id).await
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    let databases = db.list_databases().await
        .map_err(|e| e.to_string())?;

    Ok(databases.into_iter().map(|name| CatalogMeta { name }).collect())
}

#[tauri::command]
pub async fn load_schemas(conn_id: String, db_name: String) -> Result<Vec<SchemaMeta>, String> {
    let manager = get_connection_manager().clone();
    let db = manager.get_connection(&conn_id).await
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    let names = db.list_schemas(&db_name).await
        .map_err(|e| e.to_string())?;

    Ok(names.into_iter().map(|name| SchemaMeta { name }).collect())
}

#[tauri::command]
pub async fn load_tables(conn_id: String, db_name: String, schema_name: String) -> Result<Vec<TableMeta>, String> {
    let manager = get_connection_manager().clone();
    let db = manager.get_connection(&conn_id).await
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    let objects = db.list_tables(&db_name, Some(&schema_name)).await
        .map_err(|e| e.to_string())?;

    Ok(objects.into_iter().map(|obj| TableMeta {
        name: obj.name,
        table_type: "TABLE".to_string(),
    }).collect())
}

#[tauri::command]
pub async fn load_views(conn_id: String, db_name: String, schema_name: String) -> Result<Vec<ViewMeta>, String> {
    let manager = get_connection_manager().clone();
    let db = manager.get_connection(&conn_id).await
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    let objects = db.list_tables(&db_name, Some(&schema_name)).await
        .map_err(|e| e.to_string())?;

    Ok(objects.into_iter()
        .filter(|obj| matches!(obj.kind, crate::core::driver::SchemaObjectKind::View))
        .map(|obj| ViewMeta {
            name: obj.name,
            view_type: "VIEW".to_string(),
        }).collect())
}

#[tauri::command]
pub async fn load_columns(
    conn_id: String,
    db_name: String,
    schema_name: String,
    table_name: String,
) -> Result<Vec<ColumnMeta>, String> {
    let manager = get_connection_manager().clone();
    let db = manager.get_connection(&conn_id).await
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    let columns = db.list_columns(&db_name, Some(&schema_name), &table_name).await
        .map_err(|e| e.to_string())?;

    Ok(columns.into_iter().map(|col| ColumnMeta {
        name: col.name,
        data_type: col.data_type,
        is_nullable: col.nullable,
        default_value: col.default_value,
        is_primary_key: col.is_primary_key,
    }).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureMeta {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMeta {
    pub name: String,
}

fn build_procedures_sql(db_type: &str, schema_name: &str) -> String {
    let safe_schema = schema_name.replace('\'', "''");
    match db_type {
        "mysql" => format!(
            "SELECT ROUTINE_NAME as name FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_TYPE = 'PROCEDURE' AND ROUTINE_SCHEMA = '{}' ORDER BY ROUTINE_NAME",
            safe_schema
        ),
        "postgres" => format!(
            "SELECT proname as name FROM pg_proc JOIN pg_namespace ON pg_proc.pronamespace = pg_namespace.oid WHERE pg_namespace.nspname = '{}' AND pg_proc.prokind = 'p' ORDER BY proname",
            safe_schema
        ),
        _ => format!(
            "SELECT ROUTINE_NAME as name FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_TYPE = 'PROCEDURE' AND ROUTINE_SCHEMA = '{}' ORDER BY ROUTINE_NAME",
            safe_schema
        ),
    }
}

fn build_functions_sql(db_type: &str, schema_name: &str) -> String {
    let safe_schema = schema_name.replace('\'', "''");
    match db_type {
        "mysql" => format!(
            "SELECT ROUTINE_NAME as name FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_TYPE = 'FUNCTION' AND ROUTINE_SCHEMA = '{}' ORDER BY ROUTINE_NAME",
            safe_schema
        ),
        "postgres" => format!(
            "SELECT proname as name FROM pg_proc JOIN pg_namespace ON pg_proc.pronamespace = pg_namespace.oid WHERE pg_namespace.nspname = '{}' AND pg_proc.prokind IN ('f', 'a') ORDER BY proname",
            safe_schema
        ),
        _ => format!(
            "SELECT ROUTINE_NAME as name FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_TYPE = 'FUNCTION' AND ROUTINE_SCHEMA = '{}' ORDER BY ROUTINE_NAME",
            safe_schema
        ),
    }
}

fn extract_string_column(result: &QueryResult, col_idx: usize) -> Vec<String> {
    let mut values = Vec::new();
    for batch in &result.batches {
        if let Some(arr) = batch.column(col_idx).as_any().downcast_ref::<StringArray>() {
            for i in 0..arr.len() {
                values.push(arr.value(i).to_string());
            }
        }
    }
    values
}

#[tauri::command]
pub async fn load_procedures(conn_id: String, db_type: String, schema_name: String) -> Result<Vec<ProcedureMeta>, String> {
    if db_type == "sqlite" || db_type == "duckdb" {
        return Ok(vec![]);
    }

    let manager = get_connection_manager().clone();
    let db = manager.get_connection(&conn_id).await
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    let sql = build_procedures_sql(&db_type, &schema_name);
    let result = db.query(&sql).await.map_err(|e| e.to_string())?;

    let names = extract_string_column(&result, 0);
    Ok(names.into_iter().map(|name| ProcedureMeta { name }).collect())
}

#[tauri::command]
pub async fn load_functions(conn_id: String, db_type: String, schema_name: String) -> Result<Vec<FunctionMeta>, String> {
    if db_type == "sqlite" || db_type == "duckdb" {
        return Ok(vec![]);
    }

    let manager = get_connection_manager().clone();
    let db = manager.get_connection(&conn_id).await
        .ok_or_else(|| format!("Connection not found: {}", conn_id))?;

    let sql = build_functions_sql(&db_type, &schema_name);
    let result = db.query(&sql).await.map_err(|e| e.to_string())?;

    let names = extract_string_column(&result, 0);
    Ok(names.into_iter().map(|name| FunctionMeta { name }).collect())
}