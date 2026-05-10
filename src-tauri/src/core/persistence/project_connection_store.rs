//! 项目级连接存储模块
//!
//! 负责在项目级别存储和管理数据库连接配置。
//! 表结构与 global_connections 保持一致。

use crate::core::error::{CoreError, StorageError};
use crate::core::persistence::project_db::ProjectDatabaseManager;

/// 连接配置（支持 DuckDB 联邦分析）
#[derive(Debug, Clone)]
pub struct ProjectConnection {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub database: Option<String>,
    pub schema_name: Option<String>,
    pub username: Option<String>,
    pub password_encrypted: Option<String>,
    pub options: Option<String>,
    pub tags: Option<String>,
    pub use_duckdb_fed: bool,
    pub metadata_path: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 项目连接存储
pub struct ProjectConnectionStore {
    db_manager: std::sync::Arc<ProjectDatabaseManager>,
}

impl ProjectConnectionStore {
    pub fn new(db_manager: std::sync::Arc<ProjectDatabaseManager>) -> Self {
        Self { db_manager }
    }

    pub async fn create_connection(&self, conn: &ProjectConnection) -> Result<(), CoreError> {
        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        sqlite
            .inner()?
            .execute(
                "INSERT INTO connections (
                id, name, driver, host, port, database, schema_name, username, password_encrypted,
                options, tags, use_duckdb_fed, metadata_path, is_active, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                rusqlite::params![
                    conn.id,
                    conn.name,
                    conn.driver,
                    conn.host,
                    conn.port,
                    conn.database,
                    conn.schema_name,
                    conn.username,
                    conn.password_encrypted,
                    conn.options,
                    conn.tags,
                    conn.use_duckdb_fed,
                    conn.metadata_path,
                    conn.is_active,
                    conn.created_at,
                    conn.updated_at
                ],
            )
            .map_err(|e| {
                CoreError::Storage(StorageError::Persistence {
                    store: "sqlite".to_string(),
                    operation: "insert_connection".to_string(),
                    reason: e.to_string(),
                })
            })?;

        Ok(())
    }

    pub async fn update_connection(&self, conn: &ProjectConnection) -> Result<(), CoreError> {
        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        sqlite.inner()?.execute(
            "UPDATE connections SET
                name = ?2, driver = ?3, host = ?4, port = ?5, database = ?6,
                schema_name = ?7, username = ?8, password_encrypted = ?9, options = ?10,
                tags = ?11, use_duckdb_fed = ?12, metadata_path = ?13, is_active = ?14, updated_at = ?15
            WHERE id = ?1",
            rusqlite::params![
                conn.id, conn.name, conn.driver, conn.host, conn.port, conn.database,
                conn.schema_name, conn.username, conn.password_encrypted, conn.options, conn.tags,
                conn.use_duckdb_fed, conn.metadata_path, conn.is_active, conn.updated_at
            ],
        ).map_err(|e| CoreError::Storage(StorageError::Persistence { 
            store: "sqlite".to_string(), 
            operation: "update_connection".to_string(), 
            reason: e.to_string() 
        }))?;

        Ok(())
    }

    pub async fn delete_connection(&self, id: &str) -> Result<(), CoreError> {
        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        sqlite
            .inner()?
            .execute("DELETE FROM connections WHERE id = ?1", [id])
            .map_err(|e| {
                CoreError::Storage(StorageError::Persistence {
                    store: "sqlite".to_string(),
                    operation: "delete_connection".to_string(),
                    reason: e.to_string(),
                })
            })?;

        Ok(())
    }

    pub async fn get_connection(&self, id: &str) -> Result<Option<ProjectConnection>, CoreError> {
        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        let mut stmt = sqlite.inner()?.prepare(
            "SELECT id, name, driver, host, port, database, schema_name, username, password_encrypted,
                    options, tags, use_duckdb_fed, metadata_path, is_active, created_at, updated_at
             FROM connections WHERE id = ?1"
        ).map_err(|e| CoreError::Storage(StorageError::Persistence { 
            store: "sqlite".to_string(), 
            operation: "prepare_get_connection".to_string(), 
            reason: e.to_string() 
        }))?;

        let result = stmt.query_row([id], |row| {
            Ok(ProjectConnection {
                id: row.get(0)?,
                name: row.get(1)?,
                driver: row.get(2)?,
                host: row.get(3)?,
                port: row.get(4)?,
                database: row.get(5)?,
                schema_name: row.get(6)?,
                username: row.get(7)?,
                password_encrypted: row.get(8)?,
                options: row.get(9)?,
                tags: row.get(10)?,
                use_duckdb_fed: row.get(11)?,
                metadata_path: row.get(12)?,
                is_active: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        });

        match result {
            Ok(conn) => Ok(Some(conn)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CoreError::Storage(StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "get_connection".to_string(),
                reason: e.to_string(),
            })),
        }
    }

    pub async fn get_all_connections(&self) -> Result<Vec<ProjectConnection>, CoreError> {
        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        let mut stmt = sqlite.inner()?.prepare(
            "SELECT id, name, driver, host, port, database, schema_name, username, password_encrypted,
                    options, tags, use_duckdb_fed, metadata_path, is_active, created_at, updated_at
             FROM connections ORDER BY updated_at DESC"
        ).map_err(|e| CoreError::Storage(StorageError::Persistence { 
            store: "sqlite".to_string(), 
            operation: "prepare_get_all_connections".to_string(), 
            reason: e.to_string() 
        }))?;

        let connections = stmt
            .query_map([], |row| {
                Ok(ProjectConnection {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    driver: row.get(2)?,
                    host: row.get(3)?,
                    port: row.get(4)?,
                    database: row.get(5)?,
                    schema_name: row.get(6)?,
                    username: row.get(7)?,
                    password_encrypted: row.get(8)?,
                    options: row.get(9)?,
                    tags: row.get(10)?,
                    use_duckdb_fed: row.get(11)?,
                    metadata_path: row.get(12)?,
                    is_active: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: row.get(15)?,
                })
            })
            .map_err(|e| {
                CoreError::Storage(StorageError::Persistence {
                    store: "sqlite".to_string(),
                    operation: "query_connections".to_string(),
                    reason: e.to_string(),
                })
            })?;

        connections.collect::<Result<Vec<_>, _>>().map_err(|e| {
            CoreError::Storage(StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "collect_connections".to_string(),
                reason: e.to_string(),
            })
        })
    }

    pub async fn get_connections_by_type(
        &self,
        driver: &str,
    ) -> Result<Vec<ProjectConnection>, CoreError> {
        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        let mut stmt = sqlite.inner()?.prepare(
            "SELECT id, name, driver, host, port, database, schema_name, username, password_encrypted,
                    options, tags, use_duckdb_fed, metadata_path, is_active, created_at, updated_at
             FROM connections WHERE driver = ?1 ORDER BY updated_at DESC"
        ).map_err(|e| CoreError::Storage(StorageError::Persistence { 
            store: "sqlite".to_string(), 
            operation: "prepare_get_connections_by_type".to_string(), 
            reason: e.to_string() 
        }))?;

        let connections = stmt
            .query_map([driver], |row| {
                Ok(ProjectConnection {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    driver: row.get(2)?,
                    host: row.get(3)?,
                    port: row.get(4)?,
                    database: row.get(5)?,
                    schema_name: row.get(6)?,
                    username: row.get(7)?,
                    password_encrypted: row.get(8)?,
                    options: row.get(9)?,
                    tags: row.get(10)?,
                    use_duckdb_fed: row.get(11)?,
                    metadata_path: row.get(12)?,
                    is_active: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: row.get(15)?,
                })
            })
            .map_err(|e| {
                CoreError::Storage(StorageError::Persistence {
                    store: "sqlite".to_string(),
                    operation: "query_connections_by_type".to_string(),
                    reason: e.to_string(),
                })
            })?;

        connections.collect::<Result<Vec<_>, _>>().map_err(|e| {
            CoreError::Storage(StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "collect_connections_by_type".to_string(),
                reason: e.to_string(),
            })
        })
    }

    pub async fn update_connection_status(
        &self,
        id: &str,
        is_active: bool,
    ) -> Result<(), CoreError> {
        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        sqlite.inner()?.execute(
            "UPDATE connections SET is_active = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            rusqlite::params![is_active, id],
        ).map_err(|e| CoreError::Storage(StorageError::Persistence { 
            store: "sqlite".to_string(), 
            operation: "update_connection_status".to_string(), 
            reason: e.to_string() 
        }))?;

        Ok(())
    }

    pub async fn search_connections(
        &self,
        query: &str,
    ) -> Result<Vec<ProjectConnection>, CoreError> {
        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        let search_pattern = format!("%{}%", query);

        let mut stmt = sqlite.inner()?.prepare(
            "SELECT id, name, driver, host, port, database, schema_name, username, password_encrypted,
                    options, tags, use_duckdb_fed, metadata_path, is_active, created_at, updated_at
             FROM connections 
             WHERE name LIKE ?1 OR host LIKE ?1 OR database LIKE ?1
             ORDER BY updated_at DESC"
        ).map_err(|e| CoreError::Storage(StorageError::Persistence { 
            store: "sqlite".to_string(), 
            operation: "prepare_search_connections".to_string(), 
            reason: e.to_string() 
        }))?;

        let connections = stmt
            .query_map([&search_pattern], |row| {
                Ok(ProjectConnection {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    driver: row.get(2)?,
                    host: row.get(3)?,
                    port: row.get(4)?,
                    database: row.get(5)?,
                    schema_name: row.get(6)?,
                    username: row.get(7)?,
                    password_encrypted: row.get(8)?,
                    options: row.get(9)?,
                    tags: row.get(10)?,
                    use_duckdb_fed: row.get(11)?,
                    metadata_path: row.get(12)?,
                    is_active: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: row.get(15)?,
                })
            })
            .map_err(|e| {
                CoreError::Storage(StorageError::Persistence {
                    store: "sqlite".to_string(),
                    operation: "query_search_connections".to_string(),
                    reason: e.to_string(),
                })
            })?;

        connections.collect::<Result<Vec<_>, _>>().map_err(|e| {
            CoreError::Storage(StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "collect_search_connections".to_string(),
                reason: e.to_string(),
            })
        })
    }
}
