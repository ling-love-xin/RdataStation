use crate::api::dto::QueryResult;
use crate::core::error::CoreError;
use crate::core::services::plugin_bridge::get_plugin_bridge;

#[tauri::command]
pub async fn plugin_db_query(
    plugin_id: String,
    conn_id: String,
    sql: String,
    timeout: Option<u64>,
) -> Result<QueryResult, CoreError> {
    let _ = timeout;
    let bridge = get_plugin_bridge();
    let result = bridge.query(&plugin_id, &conn_id, &sql).await?;
    Ok(result)
}

#[tauri::command]
pub async fn plugin_db_metadata(
    plugin_id: String,
    conn_id: String,
    catalog: String,
    schema: String,
    kind: String,
) -> Result<serde_json::Value, CoreError> {
    let bridge = get_plugin_bridge();
    let result = bridge
        .metadata(&plugin_id, &conn_id, &catalog, &schema, &kind)
        .await?;
    Ok(result)
}