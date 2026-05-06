//! 洞察持久化存储（DuckDB 侧）
//!
//! 管理洞察快照的 JSON 序列化存储。
//! 数据存储在项目的 `analytics.duckdb` 中。
//!
//! 版本化：每条记录包含 version_id / parent_version_id / checksum，
//! 支持历史版本链和快照对比。

use std::sync::Arc;
use uuid::Uuid;

use crate::core::error::{CoreError, CommonError, StorageError};
use crate::core::persistence::project_db::ProjectDuckdbConnection;

use super::super::services::result_service::ColumnInsightFull;

// ==================== 列洞察快照存储 ====================

pub struct InsightColumnStore {
    duckdb: Arc<ProjectDuckdbConnection>,
}

impl InsightColumnStore {
    pub fn new(duckdb: Arc<ProjectDuckdbConnection>) -> Self {
        Self { duckdb }
    }

    /// 保存列洞察快照到 DuckDB
    ///
    /// # 参数
    /// - `insight`: 洞察全量数据
    /// - `parent_version_id`: 父版本 ID（None 表示首个版本）
    ///
    /// # 返回
    /// - `snapshot_id`: 生成的快照 ID
    pub async fn save_snapshot(
        &self,
        insight: &ColumnInsightFull,
        parent_version_id: Option<&str>,
    ) -> Result<(String, String), CoreError> {
        let snapshot_id = Uuid::new_v4().to_string();
        let version_id = Uuid::new_v4().to_string();
        let stats_json = serde_json::to_string(insight).map_err(|e| CoreError::common(
            CommonError::General(format!("Serialize insight failed: {}", e))
        ))?;
        let checksum = sha256_hex(&stats_json);
        let column_name = insight.stats.column_name.clone();
        let data_type = insight.stats.data_type.clone();

        let duckdb_conn = self.duckdb.acquire().await?;
        let mut guard = duckdb_conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| CoreError::common(
            CommonError::General("DuckDB connection is closed".to_string())
        ))?;

        conn.execute(
            "INSERT INTO insight_column_snapshots (snapshot_id, column_name, data_type, stats_json, version_id, parent_version_id, checksum, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
            duckdb::params![
                &snapshot_id,
                &column_name,
                &data_type,
                &stats_json,
                &version_id,
                &parent_version_id.map(|s| s.to_string()),
                &checksum,
            ],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "duckdb".to_string(),
            operation: "save_insight_snapshot".to_string(),
            reason: e.to_string(),
        }))?;

        Ok((snapshot_id.clone(), version_id))
    }

    /// 获取列洞察最新快照
    pub async fn get_latest_snapshot(
        &self,
        column_name: &str,
    ) -> Result<Option<ColumnInsightFull>, CoreError> {
        let duckdb_conn = self.duckdb.acquire().await?;
        let mut guard = duckdb_conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| CoreError::common(
            CommonError::General("DuckDB connection is closed".to_string())
        ))?;

        let result: Option<String> = conn.query_row(
            "SELECT stats_json FROM insight_column_snapshots WHERE column_name = ? ORDER BY created_at DESC LIMIT 1",
            duckdb::params![column_name],
            |row| row.get(0),
        ).ok().flatten();

        match result {
            Some(json) => {
                let insight: ColumnInsightFull = serde_json::from_str(&json).map_err(|e| CoreError::common(
                    CommonError::General(format!("Deserialize insight failed: {}", e))
                ))?;
                Ok(Some(insight))
            }
            None => Ok(None),
        }
    }

    /// 获取列洞察所有历史版本（从新到旧）
    pub async fn get_history(
        &self,
        column_name: &str,
        limit: Option<usize>,
    ) -> Result<Vec<InsightVersionEntry>, CoreError> {
        let duckdb_conn = self.duckdb.acquire().await?;
        let mut guard = duckdb_conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| CoreError::common(
            CommonError::General("DuckDB connection is closed".to_string())
        ))?;

        let limit_val = limit.unwrap_or(20) as i64;
        let mut stmt = conn.prepare(
            "SELECT snapshot_id, column_name, data_type, stats_json, version_id, parent_version_id, checksum, created_at
             FROM insight_column_snapshots WHERE column_name = ? ORDER BY created_at DESC LIMIT ?"
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "duckdb".to_string(),
            operation: "get_insight_history".to_string(),
            reason: e.to_string(),
        }))?;

        let entries: Vec<InsightVersionEntry> = stmt.query_map(
            duckdb::params![column_name, limit_val],
            |row| {
                Ok(InsightVersionEntry {
                    snapshot_id: row.get(0)?,
                    column_name: row.get(1)?,
                    data_type: row.get(2)?,
                    stats_json: row.get::<_, String>(3)?,
                    version_id: row.get(4)?,
                    parent_version_id: row.get(5)?,
                    checksum: row.get(6)?,
                    created_at: row.get::<_, String>(7)?,
                })
            }
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "duckdb".to_string(),
            operation: "query_insight_history".to_string(),
            reason: e.to_string(),
        }))?.filter_map(|r| r.ok()).collect();

        Ok(entries)
    }

    /// 按版本 ID 获取特定快照
    pub async fn get_snapshot_by_version(
        &self,
        version_id: &str,
    ) -> Result<Option<ColumnInsightFull>, CoreError> {
        let duckdb_conn = self.duckdb.acquire().await?;
        let mut guard = duckdb_conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| CoreError::common(
            CommonError::General("DuckDB connection is closed".to_string())
        ))?;

        let result: Option<String> = conn.query_row(
            "SELECT stats_json FROM insight_column_snapshots WHERE version_id = ?",
            duckdb::params![version_id],
            |row| row.get(0),
        ).ok().flatten();

        match result {
            Some(json) => {
                let insight: ColumnInsightFull = serde_json::from_str(&json).map_err(|e| CoreError::common(
                    CommonError::General(format!("Deserialize insight failed: {}", e))
                ))?;
                Ok(Some(insight))
            }
            None => Ok(None),
        }
    }

    /// 删除指定快照（标记删除，保留数据用于版本链）
    pub async fn delete_snapshot(&self, snapshot_id: &str) -> Result<(), CoreError> {
        let duckdb_conn = self.duckdb.acquire().await?;
        let mut guard = duckdb_conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| CoreError::common(
            CommonError::General("DuckDB connection is closed".to_string())
        ))?;

        conn.execute(
            "DELETE FROM insight_column_snapshots WHERE snapshot_id = ?",
            duckdb::params![snapshot_id],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "duckdb".to_string(),
            operation: "delete_insight_snapshot".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(())
    }
}

// ==================== 历史版本条目 ====================

#[derive(Debug, Clone, serde::Serialize)]
pub struct InsightVersionEntry {
    pub snapshot_id: String,
    pub column_name: String,
    pub data_type: Option<String>,
    pub stats_json: String,
    pub version_id: String,
    pub parent_version_id: Option<String>,
    pub checksum: String,
    pub created_at: String,
}

impl InsightVersionEntry {
    /// 反序列化 stats_json 为 ColumnInsightFull
    pub fn parse_insight(&self) -> Result<ColumnInsightFull, CoreError> {
        serde_json::from_str(&self.stats_json).map_err(|e| CoreError::common(
            CommonError::General(format!("Parse insight from version entry failed: {}", e))
        ))
    }
}

// ==================== 表探查报告存储（Phase 2 预留） ====================

pub struct InsightTableReportStore {
    duckdb: Arc<ProjectDuckdbConnection>,
}

impl InsightTableReportStore {
    pub fn new(duckdb: Arc<ProjectDuckdbConnection>) -> Self {
        Self { duckdb }
    }
}

// ==================== Schema 洞察报告存储（Phase 3 预留） ====================

pub struct InsightSchemaReportStore {
    duckdb: Arc<ProjectDuckdbConnection>,
}

impl InsightSchemaReportStore {
    pub fn new(duckdb: Arc<ProjectDuckdbConnection>) -> Self {
        Self { duckdb }
    }
}

// ==================== 统一洞察存储门面 ====================

pub struct InsightStorage {
    pub columns: InsightColumnStore,
    pub tables: InsightTableReportStore,
    pub schemas: InsightSchemaReportStore,
}

impl InsightStorage {
    pub fn new(duckdb: Arc<ProjectDuckdbConnection>) -> Self {
        Self {
            columns: InsightColumnStore::new(duckdb.clone()),
            tables: InsightTableReportStore::new(duckdb.clone()),
            schemas: InsightSchemaReportStore::new(duckdb),
        }
    }
}

// ==================== 工具函数 ====================

fn sha256_hex(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}
