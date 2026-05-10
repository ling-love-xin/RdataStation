use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResource {
    pub id: String,
    pub resource_type: String,
    pub name: String,
    pub alias: Option<String>,
    pub config: Value,
    pub scope: String,
    pub row_count: Option<i64>,
    pub column_count: Option<i32>,
    pub file_size: Option<i64>,
    pub version: i32,
    pub parent_version_id: Option<String>,
    pub parent_resource_id: Option<String>,
    pub source_query: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsFolder {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub parent_folder_id: Option<String>,
    pub sort_order: i32,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsTag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub scope: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsRecycleItem {
    pub id: String,
    pub resource_id: String,
    pub resource_type: String,
    pub resource_name: String,
    pub resource_data: Value,
    pub deleted_by: Option<String>,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceVersion {
    pub id: String,
    pub resource_id: String,
    pub version: i32,
    pub snapshot: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResourceRequest {
    pub resource_type: String,
    pub name: String,
    pub alias: Option<String>,
    pub config: Value,
    pub scope: String,
    pub row_count: Option<i64>,
    pub column_count: Option<i32>,
    pub file_size: Option<i64>,
    pub parent_resource_id: Option<String>,
    pub source_query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub scope: String,
    pub parent_folder_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesOutput {
    pub items: Vec<AnalyticsResource>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}