use std::sync::Arc;
use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::core::error::{CoreError, CommonError, StorageError};
use crate::core::persistence::project_db::{ProjectSqlitePool, SqlitePoolConnection};

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

#[derive(Clone)]
pub struct AnalyticsResourceStore {
    pool: Arc<ProjectSqlitePool>,
}

impl AnalyticsResourceStore {
    pub fn new(pool: Arc<ProjectSqlitePool>) -> Self {
        Self { pool }
    }

    async fn get_conn(&self) -> Result<SqlitePoolConnection, CoreError> {
        self.pool.acquire().await
    }

    pub async fn create_resource(&self, req: CreateResourceRequest) -> Result<AnalyticsResource, CoreError> {
        let conn = self.get_conn().await?;
        let id = format!("ar_{}", uuid::Uuid::new_v4().simple());
        let now = Utc::now();

        conn.inner().execute(
            r#"
            INSERT INTO analytics_resources (
                id, resource_type, name, alias, config, scope, row_count, column_count, file_size,
                version, parent_version_id, parent_resource_id, source_query, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                &id,
                &req.resource_type,
                &req.name,
                &req.alias,
                serde_json::to_string(&req.config).map_err(|e| CoreError::common(CommonError::General(e.to_string())))?,
                &req.scope,
                req.row_count,
                req.column_count,
                req.file_size,
                1,
                None::<String>,
                req.parent_resource_id,
                req.source_query,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "insert".to_string(),
            reason: e.to_string(),
        }))?;

        self.get_resource_by_id(&id).await
    }

    pub async fn update_resource(&self, id: &str, req: CreateResourceRequest) -> Result<AnalyticsResource, CoreError> {
        let conn = self.get_conn().await?;
        let now = Utc::now();

        let current = self.get_resource_by_id(id).await?;

        let snapshot = serde_json::to_string(&current).map_err(|e| CoreError::common(CommonError::General(e.to_string())))?;
        self.save_resource_version(id, current.version, &snapshot).await?;

        conn.inner().execute(
            r#"
            UPDATE analytics_resources
            SET name = ?, alias = ?, config = ?, scope = ?, row_count = ?, column_count = ?, file_size = ?,
                version = version + 1, parent_version_id = ?, updated_at = ?
            WHERE id = ? AND deleted_at IS NULL
            "#,
            rusqlite::params![
                &req.name,
                &req.alias,
                serde_json::to_string(&req.config).map_err(|e| CoreError::common(CommonError::General(e.to_string())))?,
                &req.scope,
                req.row_count,
                req.column_count,
                req.file_size,
                Some(&current.id),
                now.to_rfc3339(),
                id,
            ],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "update".to_string(),
            reason: e.to_string(),
        }))?;

        self.get_resource_by_id(id).await
    }

    pub async fn get_resource_by_id(&self, id: &str) -> Result<AnalyticsResource, CoreError> {
        let conn = self.get_conn().await?;

        let mut stmt = conn.inner().prepare(
            r#"
            SELECT id, resource_type, name, alias, config, scope, row_count, column_count, file_size,
                   version, parent_version_id, parent_resource_id, source_query, created_at, updated_at,
                   created_by, deleted_at
            FROM analytics_resources
            WHERE id = ?
            "#,
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let resource = stmt.query_row(rusqlite::params![id], |row| {
            let config_str: String = row.get(4)?;
            let config: Value = serde_json::from_str(&config_str).unwrap_or(Value::Null);

            Ok(AnalyticsResource {
                id: row.get(0)?,
                resource_type: row.get(1)?,
                name: row.get(2)?,
                alias: row.get(3)?,
                config,
                scope: row.get(5)?,
                row_count: row.get(6)?,
                column_count: row.get(7)?,
                file_size: row.get(8)?,
                version: row.get(9)?,
                parent_version_id: row.get(10)?,
                parent_resource_id: row.get(11)?,
                source_query: row.get(12)?,
                created_at: Self::parse_datetime_sqlite(row.get(13)?)?,
                updated_at: Self::parse_datetime_sqlite(row.get(14)?)?,
                created_by: row.get(15)?,
                deleted_at: row.get(16).ok().and_then(|s| Self::parse_datetime(s).ok()),
            })
        }).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(resource)
    }

    pub async fn list_resources(
        &self,
        scope: Option<&str>,
        resource_type: Option<&str>,
        folder_id: Option<&str>,
    ) -> Result<Vec<AnalyticsResource>, CoreError> {
        let conn = self.get_conn().await?;

        let mut sql = String::from(
            r#"
            SELECT id, resource_type, name, alias, config, scope, row_count, column_count, file_size,
                   version, parent_version_id, parent_resource_id, source_query, created_at, updated_at,
                   created_by, deleted_at
            FROM analytics_resources
            WHERE deleted_at IS NULL
            "#,
        );

        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(s) = scope {
            sql.push_str(" AND scope = ?");
            params.push(rusqlite::types::Value::Text(s.to_string()));
        }

        if let Some(t) = resource_type {
            sql.push_str(" AND resource_type = ?");
            params.push(rusqlite::types::Value::Text(t.to_string()));
        }

        if let Some(f) = folder_id {
            sql.push_str(
                r#"
                AND id IN (
                    SELECT resource_id FROM analytics_resource_folder WHERE folder_id = ?
                )
                "#,
            );
            params.push(rusqlite::types::Value::Text(f.to_string()));
        }

        sql.push_str(" ORDER BY created_at DESC");

        let mut stmt = conn.inner().prepare(&sql).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let resources = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            let config_str: String = row.get(4)?;
            let config: Value = serde_json::from_str(&config_str).unwrap_or(Value::Null);

            Ok(AnalyticsResource {
                id: row.get(0)?,
                resource_type: row.get(1)?,
                name: row.get(2)?,
                alias: row.get(3)?,
                config,
                scope: row.get(5)?,
                row_count: row.get(6)?,
                column_count: row.get(7)?,
                file_size: row.get(8)?,
                version: row.get(9)?,
                parent_version_id: row.get(10)?,
                parent_resource_id: row.get(11)?,
                source_query: row.get(12)?,
                created_at: Self::parse_datetime_sqlite(row.get(13)?)?,
                updated_at: Self::parse_datetime_sqlite(row.get(14)?)?,
                created_by: row.get(15)?,
                deleted_at: row.get(16).ok().and_then(|s| Self::parse_datetime(s).ok()),
            })
        }).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        resources.collect::<Result<Vec<_>, _>>().map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))
    }

    pub async fn delete_resource(&self, id: &str) -> Result<(), CoreError> {
        let conn = self.get_conn().await?;
        let now = Utc::now();

        let resource = self.get_resource_by_id(id).await?;

        conn.inner().execute(
            r#"
            INSERT INTO analytics_recycle_bin (
                id, resource_id, resource_type, resource_name, resource_data, deleted_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                format!("rb_{}", uuid::Uuid::new_v4().simple()),
                &resource.id,
                &resource.resource_type,
                &resource.name,
                serde_json::to_string(&resource).map_err(|e| CoreError::common(CommonError::General(e.to_string())))?,
                now.to_rfc3339(),
            ],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_recycle_bin".to_string(),
            operation: "insert".to_string(),
            reason: e.to_string(),
        }))?;

        conn.inner().execute(
            r#"
            UPDATE analytics_resources
            SET deleted_at = ?
            WHERE id = ?
            "#,
            rusqlite::params![now.to_rfc3339(), id],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "update".to_string(),
            reason: e.to_string(),
        }))?;

        conn.inner().execute(
            r#"
            DELETE FROM analytics_resource_folder
            WHERE resource_id = ?
            "#,
            rusqlite::params![id],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_folder".to_string(),
            operation: "delete".to_string(),
            reason: e.to_string(),
        }))?;

        conn.inner().execute(
            r#"
            DELETE FROM analytics_resource_tags
            WHERE resource_id = ?
            "#,
            rusqlite::params![id],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_tags".to_string(),
            operation: "delete".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(())
    }

    pub async fn batch_delete_resources(&self, ids: &[String]) -> Result<(), CoreError> {
        let conn = self.get_conn().await?;
        let now = Utc::now();

        conn.inner().execute("BEGIN TRANSACTION", []).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "begin_transaction".to_string(),
            reason: e.to_string(),
        }))?;

        let result = (|| -> Result<(), CoreError> {
            for id in ids {
                let resource = {
                    let mut stmt = conn.inner().prepare(
                        r#"
                        SELECT id, resource_type, name, alias, config, scope, row_count, column_count, file_size,
                               version, parent_version_id, parent_resource_id, source_query, created_at, updated_at,
                               created_by, deleted_at
                        FROM analytics_resources
                        WHERE id = ?
                        "#,
                    ).map_err(|e| CoreError::storage(StorageError::Persistence {
                        store: "analytics_resources".to_string(),
                        operation: "select".to_string(),
                        reason: e.to_string(),
                    }))?;

                    stmt.query_row(rusqlite::params![id], |row| {
                        let config_str: String = row.get(4)?;
                        let config: Value = serde_json::from_str(&config_str).unwrap_or(Value::Null);

                        Ok(AnalyticsResource {
                            id: row.get(0)?,
                            resource_type: row.get(1)?,
                            name: row.get(2)?,
                            alias: row.get(3)?,
                            config,
                            scope: row.get(5)?,
                            row_count: row.get(6)?,
                            column_count: row.get(7)?,
                            file_size: row.get(8)?,
                            version: row.get(9)?,
                            parent_version_id: row.get(10)?,
                            parent_resource_id: row.get(11)?,
                            source_query: row.get(12)?,
                            created_at: Self::parse_datetime_sqlite(row.get(13)?)?,
                            updated_at: Self::parse_datetime_sqlite(row.get(14)?)?,
                            created_by: row.get(15)?,
                            deleted_at: row.get(16).ok().and_then(|s| Self::parse_datetime(s).ok()),
                        })
                    }).map_err(|e| CoreError::storage(StorageError::Persistence {
                        store: "analytics_resources".to_string(),
                        operation: "select".to_string(),
                        reason: e.to_string(),
                    }))?
                };

                conn.inner().execute(
                    r#"
                    INSERT INTO analytics_recycle_bin (
                        id, resource_id, resource_type, resource_name, resource_data, deleted_at
                    ) VALUES (?, ?, ?, ?, ?, ?)
                    "#,
                    rusqlite::params![
                        format!("rb_{}", uuid::Uuid::new_v4().simple()),
                        &resource.id,
                        &resource.resource_type,
                        &resource.name,
                        serde_json::to_string(&resource).map_err(|e| CoreError::common(CommonError::General(e.to_string())))?,
                        now.to_rfc3339(),
                    ],
                ).map_err(|e| CoreError::storage(StorageError::Persistence {
                    store: "analytics_recycle_bin".to_string(),
                    operation: "insert".to_string(),
                    reason: e.to_string(),
                }))?;

                conn.inner().execute(
                    r#"
                    UPDATE analytics_resources
                    SET deleted_at = ?
                    WHERE id = ?
                    "#,
                    rusqlite::params![now.to_rfc3339(), id],
                ).map_err(|e| CoreError::storage(StorageError::Persistence {
                    store: "analytics_resources".to_string(),
                    operation: "update".to_string(),
                    reason: e.to_string(),
                }))?;

                conn.inner().execute(
                    r#"
                    DELETE FROM analytics_resource_folder
                    WHERE resource_id = ?
                    "#,
                    rusqlite::params![id],
                ).map_err(|e| CoreError::storage(StorageError::Persistence {
                    store: "analytics_resource_folder".to_string(),
                    operation: "delete".to_string(),
                    reason: e.to_string(),
                }))?;

                conn.inner().execute(
                    r#"
                    DELETE FROM analytics_resource_tags
                    WHERE resource_id = ?
                    "#,
                    rusqlite::params![id],
                ).map_err(|e| CoreError::storage(StorageError::Persistence {
                    store: "analytics_resource_tags".to_string(),
                    operation: "delete".to_string(),
                    reason: e.to_string(),
                }))?;
            }
            Ok(())
        })();

        match result {
            Ok(()) => {
                conn.inner().execute("COMMIT", []).map_err(|e| CoreError::storage(StorageError::Persistence {
                    store: "analytics_resources".to_string(),
                    operation: "commit".to_string(),
                    reason: e.to_string(),
                }))?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.inner().execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    pub async fn create_folder(&self, req: CreateFolderRequest) -> Result<AnalyticsFolder, CoreError> {
        let conn = self.get_conn().await?;
        let id = format!("af_{}", uuid::Uuid::new_v4().simple());
        let now = Utc::now();

        conn.inner().execute(
            r#"
            INSERT INTO analytics_folders (
                id, name, scope, parent_folder_id, sort_order, color, icon, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                &id,
                &req.name,
                &req.scope,
                req.parent_folder_id,
                0,
                req.color,
                req.icon,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_folders".to_string(),
            operation: "insert".to_string(),
            reason: e.to_string(),
        }))?;

        self.get_folder_by_id(&id).await
    }

    pub async fn get_folder_by_id(&self, id: &str) -> Result<AnalyticsFolder, CoreError> {
        let conn = self.get_conn().await?;

        let mut stmt = conn.inner().prepare(
            r#"
            SELECT id, name, scope, parent_folder_id, sort_order, color, icon, created_at, updated_at, deleted_at
            FROM analytics_folders
            WHERE id = ?
            "#,
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_folders".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let folder = stmt.query_row(rusqlite::params![id], |row| {
            Ok(AnalyticsFolder {
                id: row.get(0)?,
                name: row.get(1)?,
                scope: row.get(2)?,
                parent_folder_id: row.get(3)?,
                sort_order: row.get(4)?,
                color: row.get(5)?,
                icon: row.get(6)?,
                created_at: Self::parse_datetime_sqlite(row.get(7)?)?,
                updated_at: Self::parse_datetime_sqlite(row.get(8)?)?,
                deleted_at: row.get(9).ok().and_then(|s| Self::parse_datetime(s).ok()),
            })
        }).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_folders".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(folder)
    }

    pub async fn list_folders(&self, scope: Option<&str>, parent_folder_id: Option<&str>) -> Result<Vec<AnalyticsFolder>, CoreError> {
        let conn = self.get_conn().await?;

        let mut sql = String::from(
            r#"
            SELECT id, name, scope, parent_folder_id, sort_order, color, icon, created_at, updated_at, deleted_at
            FROM analytics_folders
            WHERE deleted_at IS NULL
            "#,
        );

        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(s) = scope {
            sql.push_str(" AND scope = ?");
            params.push(rusqlite::types::Value::Text(s.to_string()));
        }

        if let Some(p) = parent_folder_id {
            sql.push_str(" AND parent_folder_id = ?");
            params.push(rusqlite::types::Value::Text(p.to_string()));
        } else {
            sql.push_str(" AND parent_folder_id IS NULL");
        }

        sql.push_str(" ORDER BY sort_order ASC, name ASC");

        let mut stmt = conn.inner().prepare(&sql).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_folders".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let folders = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(AnalyticsFolder {
                id: row.get(0)?,
                name: row.get(1)?,
                scope: row.get(2)?,
                parent_folder_id: row.get(3)?,
                sort_order: row.get(4)?,
                color: row.get(5)?,
                icon: row.get(6)?,
                created_at: Self::parse_datetime_sqlite(row.get(7)?)?,
                updated_at: Self::parse_datetime_sqlite(row.get(8)?)?,
                deleted_at: row.get(9).ok().and_then(|s| Self::parse_datetime(s).ok()),
            })
        }).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_folders".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        folders.collect::<Result<Vec<_>, _>>().map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_folders".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))
    }

    pub async fn add_resource_to_folder(&self, resource_id: &str, folder_id: &str) -> Result<(), CoreError> {
        let conn = self.get_conn().await?;

        conn.inner().execute(
            r#"
            INSERT OR REPLACE INTO analytics_resource_folder (resource_id, folder_id, sort_order)
            VALUES (?, ?, 0)
            "#,
            rusqlite::params![resource_id, folder_id],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_folder".to_string(),
            operation: "insert".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(())
    }

    pub async fn remove_resource_from_folder(&self, resource_id: &str, folder_id: &str) -> Result<(), CoreError> {
        let conn = self.get_conn().await?;

        conn.inner().execute(
            r#"
            DELETE FROM analytics_resource_folder
            WHERE resource_id = ? AND folder_id = ?
            "#,
            rusqlite::params![resource_id, folder_id],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_folder".to_string(),
            operation: "delete".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(())
    }

    pub async fn create_tag(&self, req: CreateTagRequest) -> Result<AnalyticsTag, CoreError> {
        let conn = self.get_conn().await?;
        let id = format!("at_{}", uuid::Uuid::new_v4().simple());
        let now = Utc::now();

        conn.inner().execute(
            r#"
            INSERT INTO analytics_tags (id, name, color, icon, scope, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![&id, &req.name, req.color, req.icon, &req.scope, now.to_rfc3339()],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_tags".to_string(),
            operation: "insert".to_string(),
            reason: e.to_string(),
        }))?;

        self.get_tag_by_id(&id).await
    }

    pub async fn get_tag_by_id(&self, id: &str) -> Result<AnalyticsTag, CoreError> {
        let conn = self.get_conn().await?;

        let mut stmt = conn.inner().prepare(
            r#"
            SELECT id, name, color, icon, scope, created_at, deleted_at
            FROM analytics_tags
            WHERE id = ?
            "#,
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_tags".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let tag = stmt.query_row(rusqlite::params![id], |row| {
            Ok(AnalyticsTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                icon: row.get(3)?,
                scope: row.get(4)?,
                created_at: Self::parse_datetime_sqlite(row.get(5)?)?,
                deleted_at: row.get(6).ok().and_then(|s| Self::parse_datetime(s).ok()),
            })
        }).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_tags".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(tag)
    }

    pub async fn list_tags(&self, scope: Option<&str>) -> Result<Vec<AnalyticsTag>, CoreError> {
        let conn = self.get_conn().await?;

        let mut sql = String::from(
            r#"
            SELECT id, name, color, icon, scope, created_at, deleted_at
            FROM analytics_tags
            WHERE deleted_at IS NULL
            "#,
        );

        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(s) = scope {
            sql.push_str(" AND scope = ?");
            params.push(rusqlite::types::Value::Text(s.to_string()));
        }

        sql.push_str(" ORDER BY name ASC");

        let mut stmt = conn.inner().prepare(&sql).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_tags".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let tags = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(AnalyticsTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                icon: row.get(3)?,
                scope: row.get(4)?,
                created_at: Self::parse_datetime_sqlite(row.get(5)?)?,
                deleted_at: row.get(6).ok().and_then(|s| Self::parse_datetime(s).ok()),
            })
        }).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_tags".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        tags.collect::<Result<Vec<_>, _>>().map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_tags".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))
    }

    pub async fn add_tag_to_resource(&self, resource_id: &str, tag_id: &str) -> Result<(), CoreError> {
        let conn = self.get_conn().await?;

        conn.inner().execute(
            r#"
            INSERT OR REPLACE INTO analytics_resource_tags (resource_id, tag_id)
            VALUES (?, ?)
            "#,
            rusqlite::params![resource_id, tag_id],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_tags".to_string(),
            operation: "insert".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(())
    }

    pub async fn remove_tag_from_resource(&self, resource_id: &str, tag_id: &str) -> Result<(), CoreError> {
        let conn = self.get_conn().await?;

        conn.inner().execute(
            r#"
            DELETE FROM analytics_resource_tags
            WHERE resource_id = ? AND tag_id = ?
            "#,
            rusqlite::params![resource_id, tag_id],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_tags".to_string(),
            operation: "delete".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(())
    }

    pub async fn get_recycle_items(&self) -> Result<Vec<AnalyticsRecycleItem>, CoreError> {
        let conn = self.get_conn().await?;

        let mut stmt = conn.inner().prepare(
            r#"
            SELECT id, resource_id, resource_type, resource_name, resource_data, deleted_by, deleted_at
            FROM analytics_recycle_bin
            ORDER BY deleted_at DESC
            "#,
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_recycle_bin".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let items = stmt.query_map([], |row| {
            let data_str: String = row.get(4)?;
            let resource_data: Value = serde_json::from_str(&data_str).unwrap_or(Value::Null);

            Ok(AnalyticsRecycleItem {
                id: row.get(0)?,
                resource_id: row.get(1)?,
                resource_type: row.get(2)?,
                resource_name: row.get(3)?,
                resource_data,
                deleted_by: row.get(5)?,
                deleted_at: Self::parse_datetime_sqlite(row.get(6)?)?,
            })
        }).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_recycle_bin".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        items.collect::<Result<Vec<_>, _>>().map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_recycle_bin".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))
    }

    pub async fn restore_from_recycle(&self, recycle_id: &str) -> Result<AnalyticsResource, CoreError> {
        let conn = self.get_conn().await?;

        let resource_id = {
            let mut stmt = conn.inner().prepare(
                r#"
                SELECT resource_data FROM analytics_recycle_bin WHERE id = ?
                "#,
            ).map_err(|e| CoreError::storage(StorageError::Persistence {
                store: "analytics_recycle_bin".to_string(),
                operation: "select".to_string(),
                reason: e.to_string(),
            }))?;

            let data_str: String = stmt.query_row(rusqlite::params![recycle_id], |row| row.get(0))
                .map_err(|e| CoreError::storage(StorageError::Persistence {
                    store: "analytics_recycle_bin".to_string(),
                    operation: "select".to_string(),
                    reason: e.to_string(),
                }))?;

            let resource: AnalyticsResource = serde_json::from_str(&data_str)
                .map_err(|e| CoreError::common(CommonError::General(e.to_string())))?;

            conn.inner().execute("BEGIN TRANSACTION", []).map_err(|e| CoreError::storage(StorageError::Persistence {
                store: "analytics_resources".to_string(),
                operation: "begin_transaction".to_string(),
                reason: e.to_string(),
            }))?;

            let restore_result = (|| -> Result<(), CoreError> {
                conn.inner().execute(
                    r#"
                    UPDATE analytics_resources
                    SET deleted_at = NULL
                    WHERE id = ?
                    "#,
                    rusqlite::params![&resource.id],
                ).map_err(|e| CoreError::storage(StorageError::Persistence {
                    store: "analytics_resources".to_string(),
                    operation: "update".to_string(),
                    reason: e.to_string(),
                }))?;

                conn.inner().execute(
                    r#"
                    DELETE FROM analytics_recycle_bin WHERE id = ?
                    "#,
                    rusqlite::params![recycle_id],
                ).map_err(|e| CoreError::storage(StorageError::Persistence {
                    store: "analytics_recycle_bin".to_string(),
                    operation: "delete".to_string(),
                    reason: e.to_string(),
                }))?;

                Ok(())
            })();

            match restore_result {
                Ok(()) => {
                    conn.inner().execute("COMMIT", []).map_err(|e| CoreError::storage(StorageError::Persistence {
                        store: "analytics_resources".to_string(),
                        operation: "commit".to_string(),
                        reason: e.to_string(),
                    }))?;
                }
                Err(e) => {
                    let _ = conn.inner().execute("ROLLBACK", []);
                    return Err(e);
                }
            }

            resource.id
        };

        self.get_resource_by_id(&resource_id).await
    }

    pub async fn permanent_delete(&self, recycle_id: &str) -> Result<(), CoreError> {
        let conn = self.get_conn().await?;

        conn.inner().execute(
            r#"
            DELETE FROM analytics_recycle_bin WHERE id = ?
            "#,
            rusqlite::params![recycle_id],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_recycle_bin".to_string(),
            operation: "delete".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(())
    }

    pub async fn clone_resource(&self, id: &str, new_name: Option<&str>) -> Result<AnalyticsResource, CoreError> {
        let conn = self.get_conn().await?;

        let original = self.get_resource_by_id(id).await?;

        let cloned_id = format!("ar_{}", uuid::Uuid::new_v4().simple());
        let now = Utc::now();
        let default_name = format!("{} (副本)", original.name);
        let cloned_name = new_name.unwrap_or(&default_name);

        conn.inner().execute(
            r#"
            INSERT INTO analytics_resources (
                id, resource_type, name, alias, config, scope, row_count, column_count, file_size,
                version, parent_version_id, parent_resource_id, source_query, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                &cloned_id,
                &original.resource_type,
                cloned_name,
                original.alias,
                serde_json::to_string(&original.config).map_err(|e| CoreError::common(CommonError::General(e.to_string())))?,
                &original.scope,
                original.row_count,
                original.column_count,
                original.file_size,
                1,
                None::<String>,
                Some(&original.id),
                original.source_query,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "insert".to_string(),
            reason: e.to_string(),
        }))?;

        self.get_resource_by_id(&cloned_id).await
    }

    pub async fn list_resources_paginated(
        &self,
        scope: Option<&str>,
        resource_type: Option<&str>,
        folder_id: Option<&str>,
        search: Option<&str>,
        page: i64,
        page_size: i64,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<ListResourcesOutput, CoreError> {
        let conn = self.get_conn().await?;

        let mut where_clauses = vec!["deleted_at IS NULL".to_string()];
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(s) = scope {
            where_clauses.push("scope = ?".to_string());
            params.push(rusqlite::types::Value::Text(s.to_string()));
        }

        if let Some(t) = resource_type {
            where_clauses.push("resource_type = ?".to_string());
            params.push(rusqlite::types::Value::Text(t.to_string()));
        }

        if let Some(f) = folder_id {
            where_clauses.push(
                "id IN (SELECT resource_id FROM analytics_resource_folder WHERE folder_id = ?)".to_string()
            );
            params.push(rusqlite::types::Value::Text(f.to_string()));
        }

        if let Some(search_term) = search {
            where_clauses.push("(name LIKE ? OR alias LIKE ?)".to_string());
            let search_pattern = format!("%{}%", search_term);
            params.push(rusqlite::types::Value::Text(search_pattern.clone()));
            params.push(rusqlite::types::Value::Text(search_pattern));
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!(
            "SELECT COUNT(*) FROM analytics_resources {}",
            where_sql
        );

        let total: i64 = conn.inner()
            .query_row(&count_sql, rusqlite::params_from_iter(params.iter()), |row| row.get(0))
            .map_err(|e| CoreError::storage(StorageError::Persistence {
                store: "analytics_resources".to_string(),
                operation: "count".to_string(),
                reason: e.to_string(),
            }))?;

        let sort_field = match sort_by {
            Some("name") => "name",
            Some("created_at") => "created_at",
            Some("updated_at") => "updated_at",
            Some("row_count") => "row_count",
            Some("file_size") => "file_size",
            _ => "created_at",
        };

        let sort_dir = match sort_order {
            Some("desc") => "DESC",
            _ => "ASC",
        };

        let offset = (page - 1) * page_size;

        let sql = format!(
            r#"
            SELECT id, resource_type, name, alias, config, scope, row_count, column_count, file_size,
                   version, parent_version_id, parent_resource_id, source_query, created_at, updated_at,
                   created_by, deleted_at
            FROM analytics_resources
            {}
            ORDER BY {} {}
            LIMIT ? OFFSET ?
            "#,
            where_sql, sort_field, sort_dir
        );

        params.push(rusqlite::types::Value::Integer(page_size));
        params.push(rusqlite::types::Value::Integer(offset));

        let mut stmt = conn.inner().prepare(&sql).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let resources = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            let config_str: String = row.get(4)?;
            let config: Value = serde_json::from_str(&config_str).unwrap_or(Value::Null);

            Ok(AnalyticsResource {
                id: row.get(0)?,
                resource_type: row.get(1)?,
                name: row.get(2)?,
                alias: row.get(3)?,
                config,
                scope: row.get(5)?,
                row_count: row.get(6)?,
                column_count: row.get(7)?,
                file_size: row.get(8)?,
                version: row.get(9)?,
                parent_version_id: row.get(10)?,
                parent_resource_id: row.get(11)?,
                source_query: row.get(12)?,
                created_at: Self::parse_datetime_sqlite(row.get(13)?)?,
                updated_at: Self::parse_datetime_sqlite(row.get(14)?)?,
                created_by: row.get(15)?,
                deleted_at: row.get(16).ok().and_then(|s| Self::parse_datetime(s).ok()),
            })
        }).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resources".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let items: Vec<AnalyticsResource> = resources.collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::storage(StorageError::Persistence {
                store: "analytics_resources".to_string(),
                operation: "select".to_string(),
                reason: e.to_string(),
            }))?;

        let total_pages = if total == 0 { 1 } else { (total + page_size - 1) / page_size };

        Ok(ListResourcesOutput {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    fn parse_datetime(s: String) -> Result<DateTime<Utc>, CoreError> {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| CoreError::common(CommonError::General(format!("Invalid datetime: {}", e))))
    }

    fn parse_datetime_sqlite(s: String) -> Result<DateTime<Utc>, rusqlite::Error> {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid datetime: {}", e)))
    }

    // ==================== 版本历史 ====================

    pub async fn get_resource_versions(&self, resource_id: &str) -> Result<Vec<ResourceVersion>, CoreError> {
        let conn = self.get_conn().await?;

        let mut stmt = conn.inner().prepare(
            r#"
            SELECT id, resource_id, version, snapshot, created_at
            FROM analytics_resource_versions
            WHERE resource_id = ?
            ORDER BY version DESC
            "#,
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_versions".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let versions = stmt.query_map(rusqlite::params![resource_id], |row| {
            let snapshot_str: String = row.get(3)?;
            Ok(ResourceVersion {
                id: row.get(0)?,
                resource_id: row.get(1)?,
                version: row.get(2)?,
                snapshot: serde_json::from_str(&snapshot_str).unwrap_or(Value::Null),
                created_at: Self::parse_datetime_sqlite(row.get(4)?)?,
            })
        }).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_versions".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        versions.collect::<Result<Vec<_>, _>>().map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_versions".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))
    }

    pub async fn save_resource_version(&self, resource_id: &str, version: i32, snapshot: &str) -> Result<(), CoreError> {
        let conn = self.get_conn().await?;
        let id = format!("arv_{}", uuid::Uuid::new_v4().simple());

        conn.inner().execute(
            r#"
            INSERT OR IGNORE INTO analytics_resource_versions (id, resource_id, version, snapshot, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            rusqlite::params![&id, resource_id, version, snapshot, Utc::now().to_rfc3339()],
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_versions".to_string(),
            operation: "insert".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(())
    }

    // ==================== 标签双向查询 ====================

    pub async fn get_tags_for_resource(&self, resource_id: &str) -> Result<Vec<AnalyticsTag>, CoreError> {
        let conn = self.get_conn().await?;

        let mut stmt = conn.inner().prepare(
            r#"
            SELECT t.id, t.name, t.color, t.icon, t.scope, t.created_at, t.deleted_at
            FROM analytics_tags t
            INNER JOIN analytics_resource_tags rt ON t.id = rt.tag_id
            WHERE rt.resource_id = ? AND t.deleted_at IS NULL
            ORDER BY t.name ASC
            "#,
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_tags".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let tags = stmt.query_map(rusqlite::params![resource_id], |row| {
            Ok(AnalyticsTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                icon: row.get(3)?,
                scope: row.get(4)?,
                created_at: Self::parse_datetime_sqlite(row.get(5)?)?,
                deleted_at: row.get(6).ok().and_then(|s| Self::parse_datetime(s).ok()),
            })
        }).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_tags".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        tags.collect::<Result<Vec<_>, _>>().map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_tags".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))
    }

    pub async fn get_resources_by_tag(&self, tag_id: &str) -> Result<Vec<AnalyticsResource>, CoreError> {
        let conn = self.get_conn().await?;

        let mut stmt = conn.inner().prepare(
            r#"
            SELECT r.id, r.resource_type, r.name, r.alias, r.config, r.scope,
                   r.row_count, r.column_count, r.file_size, r.version,
                   r.parent_version_id, r.parent_resource_id, r.source_query,
                   r.created_at, r.updated_at, r.created_by, r.deleted_at
            FROM analytics_resources r
            INNER JOIN analytics_resource_tags rt ON r.id = rt.resource_id
            WHERE rt.tag_id = ? AND r.deleted_at IS NULL
            ORDER BY r.created_at DESC
            "#,
        ).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_tags".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        let resources = stmt.query_map(rusqlite::params![tag_id], |row| {
            let config_str: String = row.get(4)?;
            let config: Value = serde_json::from_str(&config_str).unwrap_or(Value::Null);

            Ok(AnalyticsResource {
                id: row.get(0)?,
                resource_type: row.get(1)?,
                name: row.get(2)?,
                alias: row.get(3)?,
                config,
                scope: row.get(5)?,
                row_count: row.get(6)?,
                column_count: row.get(7)?,
                file_size: row.get(8)?,
                version: row.get(9)?,
                parent_version_id: row.get(10)?,
                parent_resource_id: row.get(11)?,
                source_query: row.get(12)?,
                created_at: Self::parse_datetime_sqlite(row.get(13)?)?,
                updated_at: Self::parse_datetime_sqlite(row.get(14)?)?,
                created_by: row.get(15)?,
                deleted_at: row.get(16).ok().and_then(|s| Self::parse_datetime(s).ok()),
            })
        }).map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_tags".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))?;

        resources.collect::<Result<Vec<_>, _>>().map_err(|e| CoreError::storage(StorageError::Persistence {
            store: "analytics_resource_tags".to_string(),
            operation: "select".to_string(),
            reason: e.to_string(),
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesOutput {
    pub items: Vec<AnalyticsResource>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
