//! 持久化存储模块
//!
//! 负责系统数据的持久化存储，包括：
//! - 最近使用的数据库连接信息
//! - SQL 执行历史记录
//! - 项目级数据（SQLite + DuckDB）
//!
//! 所有 IO 错误都在此模块转换为 CoreError，确保上下文不丢失。

pub mod cache_version_migration;
pub mod connection_store;
pub mod global_db;
pub mod history_store;
pub mod metadata_cache;
pub mod project_connection_store;
pub mod project_db;
pub mod project_store;
pub mod sql_template_store;
pub mod workbench_context_store;
pub mod analytics_resource_store;
pub mod insight_store;
pub mod insight_meta_store;

pub use cache_version_migration::{CacheVersionManager, CURRENT_CACHE_VERSION};
pub use global_db::{GlobalDatabaseManager, GlobalDuckdbConnection, GlobalSqlitePool};
pub use metadata_cache::{ConnectionType, MetadataCacheManager, MetadataCacheOps};
pub use project_db::{ProjectDatabaseManager, ProjectDuckdbConnection, ProjectSqlitePool};
pub use sql_template_store::{SqlTemplate, SqlTemplateStore};
pub use workbench_context_store::{WorkbenchContextStore, WorkbenchLayout, EditorContext};
pub use analytics_resource_store::{
    AnalyticsResourceStore, AnalyticsResource, AnalyticsFolder, AnalyticsTag, AnalyticsRecycleItem,
    CreateResourceRequest, CreateFolderRequest, CreateTagRequest, ListResourcesOutput,
    ResourceVersion,
};
pub use insight_store::{InsightStorage, InsightColumnStore, InsightTableReportStore, InsightSchemaReportStore, InsightVersionEntry, InsightStorageStats};
pub use insight_meta_store::InsightMetaStore;

use crate::core::error::{CoreError, StorageError};
use std::path::Path;

/// 将 IO 错误转换为 CoreError（显式转换，保留完整上下文）
///
/// # Arguments
///
/// * `err` - IO 错误
/// * `path` - 操作路径
/// * `operation` - 操作描述（如 "read", "write", "create"）
///
/// # Returns
///
/// 返回包含完整上下文的 CoreError
pub fn io_to_core_error(err: std::io::Error, path: &Path, operation: &str) -> CoreError {
    CoreError::storage(StorageError::io(
        path.display().to_string(),
        operation.to_string(),
        err.to_string(),
    ))
}

/// 将序列化错误转换为 CoreError
pub fn serialize_to_core_error(format: &str, reason: &str) -> CoreError {
    CoreError::storage(StorageError::Serialization {
        format: format.to_string(),
        reason: reason.to_string(),
    })
}

/// 将反序列化错误转换为 CoreError
pub fn deserialize_to_core_error(format: &str, data: &str, reason: &str) -> CoreError {
    CoreError::storage(StorageError::Deserialization {
        format: format.to_string(),
        data: data.to_string(),
        reason: reason.to_string(),
    })
}

/// 将持久化错误转换为 CoreError
pub fn persistence_to_core_error(store: &str, operation: &str, reason: &str) -> CoreError {
    CoreError::storage(StorageError::Persistence {
        store: store.to_string(),
        operation: operation.to_string(),
        reason: reason.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_io_to_core_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let path = Path::new("/test/path.json");
        let core_err = io_to_core_error(io_err, path, "read");

        assert!(matches!(core_err, CoreError::Storage(_)));
        assert_eq!(core_err.code(), "STORE_IO");
    }

    #[test]
    fn test_serialize_to_core_error() {
        let err = serialize_to_core_error("JSON", "invalid type");
        assert!(matches!(err, CoreError::Storage(StorageError::Serialization { .. })));
        assert_eq!(err.code(), "STORE_SERIALIZE");
    }

    #[test]
    fn test_deserialize_to_core_error() {
        let err = deserialize_to_core_error("JSON", "{invalid}", "unexpected token");
        assert!(matches!(err, CoreError::Storage(StorageError::Deserialization { .. })));
        assert_eq!(err.code(), "STORE_DESERIALIZE");
    }
}
