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
    pub server_version: Option<String>,
    pub description: Option<String>,
    pub driver_id: Option<String>,
    pub environment_id: Option<String>,
    pub auth_config_id: Option<String>,
    pub network_config_id: Option<String>,
    pub driver_properties: Option<String>,
    pub advanced_options: Option<String>,
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
                options, tags, use_duckdb_fed, metadata_path, is_active,
                server_version, description, driver_id, environment_id, auth_config_id,
                network_config_id, driver_properties, advanced_options, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)",
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
                    conn.server_version,
                    conn.description,
                    conn.driver_id,
                    conn.environment_id,
                    conn.auth_config_id,
                    conn.network_config_id,
                    conn.driver_properties,
                    conn.advanced_options,
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
                tags = ?11, use_duckdb_fed = ?12, metadata_path = ?13, is_active = ?14,
                server_version = ?15, description = ?16, driver_id = ?17, environment_id = ?18,
                auth_config_id = ?19, network_config_id = ?20, driver_properties = ?21,
                advanced_options = ?22, updated_at = ?23
            WHERE id = ?1",
            rusqlite::params![
                conn.id, conn.name, conn.driver, conn.host, conn.port, conn.database,
                conn.schema_name, conn.username, conn.password_encrypted, conn.options, conn.tags,
                conn.use_duckdb_fed, conn.metadata_path, conn.is_active,
                conn.server_version, conn.description, conn.driver_id, conn.environment_id,
                conn.auth_config_id, conn.network_config_id, conn.driver_properties,
                conn.advanced_options, conn.updated_at
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
                    options, tags, use_duckdb_fed, metadata_path, is_active,
                    server_version, description, driver_id, environment_id, auth_config_id,
                    network_config_id, driver_properties, advanced_options, created_at, updated_at
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
                server_version: row.get(14)?,
                description: row.get(15)?,
                driver_id: row.get(16)?,
                environment_id: row.get(17)?,
                auth_config_id: row.get(18)?,
                network_config_id: row.get(19)?,
                driver_properties: row.get(20)?,
                advanced_options: row.get(21)?,
                created_at: row.get(22)?,
                updated_at: row.get(23)?,
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
                    options, tags, use_duckdb_fed, metadata_path, is_active,
                    server_version, description, driver_id, environment_id, auth_config_id,
                    network_config_id, driver_properties, advanced_options, created_at, updated_at
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
                    server_version: row.get(14)?,
                    description: row.get(15)?,
                    driver_id: row.get(16)?,
                    environment_id: row.get(17)?,
                    auth_config_id: row.get(18)?,
                    network_config_id: row.get(19)?,
                    driver_properties: row.get(20)?,
                    advanced_options: row.get(21)?,
                    created_at: row.get(22)?,
                    updated_at: row.get(23)?,
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
                    options, tags, use_duckdb_fed, metadata_path, is_active,
                    server_version, description, driver_id, environment_id, auth_config_id,
                    network_config_id, driver_properties, advanced_options, created_at, updated_at
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
                    server_version: row.get(14)?,
                    description: row.get(15)?,
                    driver_id: row.get(16)?,
                    environment_id: row.get(17)?,
                    auth_config_id: row.get(18)?,
                    network_config_id: row.get(19)?,
                    driver_properties: row.get(20)?,
                    advanced_options: row.get(21)?,
                    created_at: row.get(22)?,
                    updated_at: row.get(23)?,
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
                    options, tags, use_duckdb_fed, metadata_path, is_active,
                    server_version, description, driver_id, environment_id, auth_config_id,
                    network_config_id, driver_properties, advanced_options, created_at, updated_at
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
                    server_version: row.get(14)?,
                    description: row.get(15)?,
                    driver_id: row.get(16)?,
                    environment_id: row.get(17)?,
                    auth_config_id: row.get(18)?,
                    network_config_id: row.get(19)?,
                    driver_properties: row.get(20)?,
                    advanced_options: row.get(21)?,
                    created_at: row.get(22)?,
                    updated_at: row.get(23)?,
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

    /// 为当前项目启用一个驱动（写入 project_drivers 表）
    ///
    /// 在写入前会校验驱动是否在全局目录中存在且未被全局禁用。
    pub async fn enable_driver(&self, driver_id: &str) -> Result<(), CoreError> {
        let global_db = crate::core::migration::get_global_db_manager()
            .ok_or_else(|| CoreError::from("Global database not initialized".to_string()))?;

        let driver = global_db
            .get_driver(driver_id)
            .await?
            .ok_or_else(|| {
                CoreError::from(format!("驱动 {} 不存在于全局目录中", driver_id))
            })?;

        if !driver.enabled {
            return Err(CoreError::from(format!("驱动 {} 已被全局禁用", driver_id)));
        }

        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        sqlite.inner()?.execute(
            "INSERT OR REPLACE INTO project_drivers (id, driver_id, enabled, installed_at)
             VALUES (?1, ?2, 1, CURRENT_TIMESTAMP)",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), driver_id],
        ).map_err(|e| CoreError::Storage(StorageError::Persistence {
            store: "sqlite".to_string(),
            operation: "enable_driver".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(())
    }

    /// 为当前项目禁用一个驱动（软操作：设置 enabled=0）
    pub async fn disable_driver(&self, driver_id: &str) -> Result<(), CoreError> {
        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        sqlite.inner()?.execute(
            "UPDATE project_drivers SET enabled = 0 WHERE driver_id = ?1",
            rusqlite::params![driver_id],
        ).map_err(|e| CoreError::Storage(StorageError::Persistence {
            store: "sqlite".to_string(),
            operation: "disable_driver".to_string(),
            reason: e.to_string(),
        }))?;

        Ok(())
    }

    /// 检查驱动是否在当前项目中启用
    pub async fn is_driver_enabled(&self, driver_id: &str) -> Result<bool, CoreError> {
        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        let enabled: bool = sqlite.inner()?.query_row(
            "SELECT enabled FROM project_drivers WHERE driver_id = ?1",
            rusqlite::params![driver_id],
            |row| row.get::<_, i32>(0).map(|v| v != 0),
        ).unwrap_or(false);

        Ok(enabled)
    }

    /// 列出当前项目中所有已启用的驱动 ID
    pub async fn list_enabled_drivers(&self) -> Result<Vec<String>, CoreError> {
        let sqlite = self.db_manager.sqlite_pool().acquire().await?;

        let mut stmt = sqlite.inner()?.prepare(
            "SELECT driver_id FROM project_drivers WHERE enabled = 1"
        ).map_err(|e| CoreError::Storage(StorageError::Persistence {
            store: "sqlite".to_string(),
            operation: "prepare_list_enabled_drivers".to_string(),
            reason: e.to_string(),
        }))?;

        let drivers: Vec<String> = stmt.query_map([], |row| row.get(0))
            .map_err(|e| CoreError::Storage(StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "query_enabled_drivers".to_string(),
                reason: e.to_string(),
            }))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(drivers)
    }

    /// 为新项目种子 4 个内置 Native 驱动
    pub async fn seed_default_drivers(&self) -> Result<(), CoreError> {
        let defaults = [
            "mysql-native",
            "postgres-native",
            "sqlite-native",
            "duckdb-native",
        ];
        for driver_id in defaults {
            if !self.is_driver_enabled(driver_id).await? {
                self.enable_driver(driver_id).await?;
            }
        }
        Ok(())
    }
}
