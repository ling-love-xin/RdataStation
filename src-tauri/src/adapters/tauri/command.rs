//! Tauri Adapter - Command 层
//!
//! 负责将 Tauri 前端的请求转换为 Core 业务调用。
//! 这是 Tauri 特定的适配层，其他适配器（CLI/HTTP/WASM）会有各自的实现。

use crate::core::DataSourceMeta;
use crate::api::dto::QueryResult;
use crate::core::services::sql_service::SqlExecuteResult;

// ==================== Connection Commands ====================

/// 创建数据库连接请求参数
#[derive(serde::Deserialize, Debug)]
pub struct ConnectDatabaseInput {
    pub db_type: String,
    pub url: String,
    pub name: Option<String>,
}

/// 连接响应
#[derive(serde::Serialize, Debug)]
pub struct ConnectDatabaseResponse {
    pub conn_id: String,
    pub name: String,
    pub db_type: String,
    pub url: String,
    pub meta: DataSourceMetaResponse,
}

/// 数据源元数据响应
#[derive(serde::Serialize, Debug)]
pub struct DataSourceMetaResponse {
    pub supports_transaction: bool,
    pub supports_streaming: bool,
    pub supports_arrow: bool,
    pub supports_federated: bool,
    pub supports_concurrent_write: bool,
    pub is_in_memory: bool,
}

impl From<DataSourceMeta> for DataSourceMetaResponse {
    fn from(meta: DataSourceMeta) -> Self {
        Self {
            supports_transaction: meta.supports_transaction,
            supports_streaming: meta.supports_streaming,
            supports_arrow: meta.supports_arrow,
            supports_federated: meta.supports_federated,
            supports_concurrent_write: meta.supports_concurrent_write,
            is_in_memory: meta.is_in_memory,
        }
    }
}



/// 连接信息响应
#[derive(serde::Serialize, Debug)]
pub struct ConnectionInfoResponse {
    pub id: String,
    pub name: String,
    pub db_type: String,
    pub url: String,
    pub is_active: bool,
    pub created_at_ms: u64,
}



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



// ==================== Persistence Commands ====================

/// 最近连接记录响应
#[derive(serde::Serialize, Debug)]
pub struct RecentConnectionResponse {
    pub name: String,
    pub db_type: String,
    pub url: String,
    pub last_used_at: String,
}



/// SQL 历史记录响应
#[derive(serde::Serialize, Debug)]
pub struct SqlHistoryResponse {
    pub id: String,
    pub sql: String,
    pub conn_id: Option<String>,
    pub executed_at: String,
}



// ==================== Schema Explorer Commands ====================

use crate::core::{SchemaObject, SchemaObjectKind};

/// Schema 对象响应（前端友好格式）
#[derive(serde::Serialize, Debug)]
pub struct SchemaObjectResponse {
    pub name: String,
    pub kind: String,
    pub children: Option<Vec<SchemaObjectResponse>>,
}

impl From<SchemaObject> for SchemaObjectResponse {
    fn from(obj: SchemaObject) -> Self {
        Self {
            name: obj.name,
            kind: match obj.kind {
                SchemaObjectKind::Database => "database".to_string(),
                SchemaObjectKind::Schema => "schema".to_string(),
                SchemaObjectKind::Table => "table".to_string(),
                SchemaObjectKind::View => "view".to_string(),
                SchemaObjectKind::Column => "column".to_string(),
                SchemaObjectKind::Index => "index".to_string(),
                SchemaObjectKind::PrimaryKey => "primary_key".to_string(),
                SchemaObjectKind::ForeignKey => "foreign_key".to_string(),
            },
            children: obj.children.map(|c| c.into_iter().map(|child| child.into()).collect()),
        }
    }
}















// ==================== Project Connection Commands ====================

use crate::core::persistence::project_connection_store::{
    ProjectConnection,
};

/// 项目连接响应
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProjectConnectionResponse {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub database: Option<String>,
    pub schema_name: Option<String>,
    pub username: Option<String>,
    pub options: Option<String>,
    pub tags: Option<String>,
    pub use_duckdb_fed: bool,
    pub metadata_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ProjectConnection> for ProjectConnectionResponse {
    fn from(conn: ProjectConnection) -> Self {
        Self {
            id: conn.id,
            name: conn.name,
            driver: conn.driver,
            host: conn.host,
            port: conn.port,
            database: conn.database,
            schema_name: conn.schema_name,
            username: conn.username,
            options: conn.options,
            tags: conn.tags,
            use_duckdb_fed: conn.use_duckdb_fed,
            metadata_path: conn.metadata_path,
            created_at: conn.created_at,
            updated_at: conn.updated_at,
        }
    }
}




