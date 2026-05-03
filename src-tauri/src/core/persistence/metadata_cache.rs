/**
 * 连接元数据缓存管理模块
 * 
 * 每个数据库连接都有独立的 SQLite 文件用于缓存元数据。
 * 元数据缓存的存储位置跟随连接信息：
 * - 全局连接：存储到 system/global_metadata/ 目录
 * - 项目连接：存储到 project/meta/connection_metadata/ 目录
 * 
 * 设计理由：
 * - 大型数据库（如 Oracle）可能有 10 万+ 张表，元数据记录可达数百万条
 * - 独立文件避免单文件过大，提高查询性能
 * - 跟随连接信息，简化项目迁移
 */

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rusqlite::{Connection, OptionalExtension};
use std::io::{Read, Write};

use crate::core::error::{CoreError, CommonError, StorageError};
use crate::core::migration::{MigrationManager, MigrationType};

/// 连接类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
    /// 全局连接（不跟随项目）
    Global,
    /// 项目连接（跟随项目）
    Project,
}

/// 元数据缓存管理器
/// 
/// 为每个数据库连接管理独立的元数据缓存 SQLite 文件
pub struct MetadataCacheManager {
    /// 缓存数据库路径
    db_path: PathBuf,
    /// 连接 ID
    conn_id: String,
    /// 连接类型
    connection_type: ConnectionType,
}

impl MetadataCacheManager {
    /// 创建元数据缓存管理器
    /// 
    /// # 参数
    /// * `conn_id` - 连接 ID
    /// * `connection_type` - 连接类型（全局/项目）
    /// * `project_path` - 项目路径（仅项目连接需要）
    pub fn new(
        conn_id: &str,
        connection_type: ConnectionType,
        project_path: Option<&str>,
    ) -> Result<Self, CoreError> {
        let db_path = Self::build_metadata_path(conn_id, connection_type, project_path)?;

        Ok(Self {
            db_path,
            conn_id: conn_id.to_string(),
            connection_type,
        })
    }

    /// 构建元数据缓存数据库路径
    /// 
    /// 元数据文件跟随连接信息：
    /// - 全局连接：{data_dir}/RdataStation/system/global_metadata/conn_{id}.sqlite
    /// - 项目连接：{project_path}/meta/connection_metadata/conn_{id}.sqlite
    fn build_metadata_path(
        conn_id: &str,
        connection_type: ConnectionType,
        project_path: Option<&str>,
    ) -> Result<PathBuf, CoreError> {
        let dir = match connection_type {
            ConnectionType::Global => {
                // 全局连接：存储到系统目录下的全局元数据目录
                let system_dir = crate::core::migration::get_system_dir()?;
                system_dir.join("global_metadata")
            }
            ConnectionType::Project => {
                // 项目连接：存储到项目元数据目录下的连接元数据子目录
                let project_path = project_path.ok_or_else(|| CoreError::common(
                    CommonError::General("Project path is required for project connection".to_string())
                ))?;
                PathBuf::from(project_path).join("meta/connection_metadata")
            }
        };

        // 确保目录存在
        std::fs::create_dir_all(&dir).map_err(|e| CoreError::common(
            CommonError::General(format!("Failed to create metadata directory {:?}: {}", dir, e))
        ))?;

        Ok(dir.join(format!("conn_{}.sqlite", conn_id)))
    }

    /// 打开元数据缓存数据库
    /// 
    /// 如果数据库不存在，将自动创建并执行迁移
    pub fn open(&self) -> Result<Connection, CoreError> {
        let conn = Connection::open(&self.db_path).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "open_metadata".to_string(),
                reason: e.to_string(),
            }
        ))?;

        // 启用 WAL 模式（PRAGMA journal_mode=WAL 会返回结果，使用 query_row）
        conn.query_row("PRAGMA journal_mode=WAL", [], |_| Ok(())).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "set_wal_mode".to_string(),
                reason: e.to_string(),
            }
        ))?;

        // 设置缓存大小
        conn.execute("PRAGMA cache_size=-1000", []).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "set_cache_size".to_string(),
                reason: e.to_string(),
            }
        ))?;

        // 执行迁移
        MigrationManager::new().migrate(&self.db_path, MigrationType::ConnectionMetadata)?;

        Ok(conn)
    }

    /// 获取元数据缓存数据库路径
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// 获取连接 ID
    pub fn conn_id(&self) -> &str {
        &self.conn_id
    }

    /// 获取连接类型
    pub fn connection_type(&self) -> ConnectionType {
        self.connection_type
    }

    /// 删除元数据缓存文件
    /// 
    /// 当连接被删除时调用
    pub fn delete(&self) -> Result<(), CoreError> {
        if self.db_path.exists() {
            std::fs::remove_file(&self.db_path).map_err(|e| CoreError::common(
                CommonError::General(format!("Failed to delete metadata cache {:?}: {}", self.db_path, e))
            ))?;
        }
        Ok(())
    }

    /// 检查元数据缓存是否存在
    pub fn exists(&self) -> bool {
        self.db_path.exists()
    }

    /// 获取元数据缓存文件大小（字节）
    pub fn size(&self) -> Result<u64, CoreError> {
        let metadata = std::fs::metadata(&self.db_path).map_err(|e| CoreError::common(
            CommonError::General(format!("Failed to get metadata cache size: {}", e))
        ))?;
        Ok(metadata.len())
    }
}

/// 元数据缓存操作封装
/// 
/// 提供常用的元数据缓存读写操作
pub struct MetadataCacheOps {
    conn: Connection,
    /// 是否启用压缩（默认对大于 1KB 的数据启用压缩）
    compression_threshold: usize,
}

impl MetadataCacheOps {
    /// 创建新的元数据缓存操作实例
    pub fn new(conn: Connection) -> Self {
        Self { 
            conn,
            compression_threshold: 1024, // 1KB 阈值
        }
    }

    /// 创建带压缩配置的元数据缓存操作实例
    pub fn with_compression(conn: Connection, compression_threshold: usize) -> Self {
        Self { 
            conn,
            compression_threshold,
        }
    }

    /// 获取底层数据库连接（用于版本迁移等操作）
    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }

    /// 压缩数据
    fn compress_data(&self, data: &str) -> Result<Vec<u8>, CoreError> {
        if data.len() < self.compression_threshold {
            return Ok(data.as_bytes().to_vec())
        }

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data.as_bytes()).map_err(|e| CoreError::common(
            CommonError::General(format!("Failed to compress data: {}", e))
        ))?;
        
        let compressed = encoder.finish().map_err(|e| CoreError::common(
            CommonError::General(format!("Failed to finish compression: {}", e))
        ))?;
        
        Ok(compressed)
    }

    /// 解压数据
    fn decompress_data(&self, data: &[u8]) -> Result<String, CoreError> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).map_err(|e| CoreError::common(
            CommonError::General(format!("Failed to decompress data: {}", e))
        ))?;
        
        Ok(decompressed)
    }

    /// 检查数据是否被压缩（通过 magic bytes 检查）
    fn is_compressed(&self, data: &[u8]) -> bool {
        data.len() > 2 && data[0] == 0x1f && data[1] == 0x8b
    }

    /// 保存表元数据
    pub fn save_table_metadata(
        &self,
        id: &str,
        database_name: &str,
        schema_name: &str,
        table_name: &str,
        comment: Option<&str>,
        last_sync: i64,
    ) -> Result<(), CoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO metadata 
             (id, obj_type, database_name, schema_name, table_name, name, comment, last_sync)
             VALUES (?1, 'table', ?2, ?3, ?4, ?4, ?5, ?6)",
            rusqlite::params![id, database_name, schema_name, table_name, comment, last_sync],
        ).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "save_table_metadata".to_string(),
                reason: e.to_string(),
            }
        ))?;

        Ok(())
    }

    /// 保存列元数据
    pub fn save_column_metadata(
        &self,
        id: &str,
        database_name: &str,
        schema_name: &str,
        table_name: &str,
        column_name: &str,
        data_type: &str,
        is_nullable: bool,
        is_primary: bool,
        is_unique: bool,
        last_sync: i64,
    ) -> Result<(), CoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO metadata 
             (id, obj_type, database_name, schema_name, table_name, name, data_type, is_nullable, is_primary, is_unique, last_sync)
             VALUES (?1, 'column', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                id, database_name, schema_name, table_name, column_name,
                data_type, is_nullable as i32, is_primary as i32, is_unique as i32, last_sync
            ],
        ).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "save_column_metadata".to_string(),
                reason: e.to_string(),
            }
        ))?;

        Ok(())
    }

    /// 获取表列表
    pub fn list_tables(&self, database_name: &str, schema_name: Option<&str>) -> Result<Vec<TableInfo>, CoreError> {
        let schema_filter = schema_name.unwrap_or("%");
        let mut stmt = self.conn.prepare(
            "SELECT id, schema_name, table_name, comment, last_sync FROM metadata 
             WHERE obj_type = 'table' AND database_name = ?1 AND schema_name LIKE ?2
             ORDER BY table_name"
        ).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "list_tables".to_string(),
                reason: e.to_string(),
            }
        ))?;

        let tables = stmt.query_map(
            rusqlite::params![database_name, schema_filter],
            |row| {
                Ok(TableInfo {
                    id: row.get(0)?,
                    schema_name: row.get(1)?,
                    name: row.get(2)?,
                    comment: row.get(3).ok(),
                    last_sync: row.get(4)?,
                })
            }
        ).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "query_tables".to_string(),
                reason: e.to_string(),
            }
        ))?;

        let mut result = Vec::new();
        for table in tables {
            result.push(table.map_err(|e| CoreError::storage(
                StorageError::Persistence {
                    store: "sqlite".to_string(),
                    operation: "fetch_table".to_string(),
                    reason: e.to_string(),
                }
            ))?);
        }

        Ok(result)
    }

    /// 获取列列表
    pub fn list_columns(&self, database_name: &str, schema_name: &str, table_name: &str) -> Result<Vec<ColumnInfo>, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, data_type, is_nullable, is_primary, is_unique, comment, last_sync 
             FROM metadata 
             WHERE obj_type = 'column' AND database_name = ?1 AND schema_name = ?2 AND table_name = ?3
             ORDER BY name"
        ).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "list_columns".to_string(),
                reason: e.to_string(),
            }
        ))?;

        let columns = stmt.query_map(
            rusqlite::params![database_name, schema_name, table_name],
            |row| {
                Ok(ColumnInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    data_type: row.get(2)?,
                    is_nullable: row.get::<_, i32>(3)? != 0,
                    is_primary: row.get::<_, i32>(4)? != 0,
                    is_unique: row.get::<_, i32>(5)? != 0,
                    comment: row.get(6).ok(),
                    last_sync: row.get(7)?,
                })
            }
        ).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "query_columns".to_string(),
                reason: e.to_string(),
            }
        ))?;

        let mut result = Vec::new();
        for column in columns {
            result.push(column.map_err(|e| CoreError::storage(
                StorageError::Persistence {
                    store: "sqlite".to_string(),
                    operation: "fetch_column".to_string(),
                    reason: e.to_string(),
                }
            ))?);
        }

        Ok(result)
    }

    /// 记录同步日志
    pub fn log_sync(&self, id: &str, start_at: i64, end_at: i64, success: bool, message: Option<&str>, objects_fetched: i64) -> Result<(), CoreError> {
        self.conn.execute(
            "INSERT INTO sync_log (id, start_at, end_at, success, message, objects_fetched)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, start_at, end_at, success as i32, message.unwrap_or(""), objects_fetched],
        ).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "log_sync".to_string(),
                reason: e.to_string(),
            }
        ))?;

        Ok(())
    }

    /// 清除指定连接的元数据
    pub fn clear_metadata(&self, database_name: &str, schema_name: &str, table_name: Option<&str>) -> Result<usize, CoreError> {
        let affected = match table_name {
            Some(t) => {
                self.conn.execute(
                    "DELETE FROM metadata WHERE database_name = ?1 AND schema_name = ?2 AND table_name = ?3",
                    rusqlite::params![database_name, schema_name, t],
                ).map_err(|e| CoreError::storage(
                    StorageError::Persistence {
                        store: "sqlite".to_string(),
                        operation: "clear_metadata".to_string(),
                        reason: e.to_string(),
                    }
                ))?
            }
            None => {
                self.conn.execute(
                    "DELETE FROM metadata WHERE database_name = ?1 AND schema_name = ?2",
                    rusqlite::params![database_name, schema_name],
                ).map_err(|e| CoreError::storage(
                    StorageError::Persistence {
                        store: "sqlite".to_string(),
                        operation: "clear_metadata".to_string(),
                        reason: e.to_string(),
                    }
                ))?
            }
        };

        Ok(affected)
    }

    /// 批量保存表元数据
    pub fn save_tables_batch(
        &mut self,
        tables: Vec<(String, String, String, String, Option<String>)>,
    ) -> Result<(), CoreError> {
        let tx = self.conn.transaction().map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "begin_transaction".to_string(),
                reason: e.to_string(),
            }
        ))?;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CoreError::common(CommonError::General(format!("获取系统时间失败: {}", e))))?
            .as_secs() as i64;

        for (id, database_name, schema_name, table_name, comment) in tables {
            tx.execute(
                "INSERT OR REPLACE INTO metadata 
                 (id, obj_type, database_name, schema_name, table_name, name, comment, last_sync)
                 VALUES (?1, 'table', ?2, ?3, ?4, ?4, ?5, ?6)",
                rusqlite::params![id, database_name, schema_name, table_name, comment, current_time],
            ).map_err(|e| CoreError::storage(
                StorageError::Persistence {
                    store: "sqlite".to_string(),
                    operation: "save_table_batch".to_string(),
                    reason: e.to_string(),
                }
            ))?;
        }

        tx.commit().map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "commit_transaction".to_string(),
                reason: e.to_string(),
            }
        ))?;

        Ok(())
    }

    /// 批量保存列元数据
    pub fn save_columns_batch(
        &mut self,
        columns: Vec<(String, String, String, String, String, String, bool, bool, bool)>,
    ) -> Result<(), CoreError> {
        let tx = self.conn.transaction().map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "begin_transaction".to_string(),
                reason: e.to_string(),
            }
        ))?;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CoreError::common(CommonError::General(format!("获取系统时间失败: {}", e))))?
            .as_secs() as i64;

        for (id, database_name, schema_name, table_name, column_name, data_type, is_nullable, is_primary, is_unique) in columns {
            tx.execute(
                "INSERT OR REPLACE INTO metadata 
                 (id, obj_type, database_name, schema_name, table_name, name, data_type, is_nullable, is_primary, is_unique, last_sync)
                 VALUES (?1, 'column', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    id, database_name, schema_name, table_name, column_name,
                    data_type, is_nullable as i32, is_primary as i32, is_unique as i32, current_time
                ],
            ).map_err(|e| CoreError::storage(
                StorageError::Persistence {
                    store: "sqlite".to_string(),
                    operation: "save_column_batch".to_string(),
                    reason: e.to_string(),
                }
            ))?;
        }

        tx.commit().map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "commit_transaction".to_string(),
                reason: e.to_string(),
            }
        ))?;

        Ok(())
    }

    /// 检查缓存是否有效（默认 24 小时）
    /// 
    /// # 参数
    /// * `database_name` - 数据库名称
    /// * `schema_name` - 模式名称
    /// * `max_age_seconds` - 最大缓存时间（秒），默认 86400 秒（24 小时）
    pub fn is_cache_valid(&self, database_name: &str, schema_name: &str, max_age_seconds: Option<i64>) -> Result<bool, CoreError> {
        let max_age = max_age_seconds.unwrap_or(86400);
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CoreError::common(CommonError::General(format!("获取系统时间失败: {}", e))))?
            .as_secs() as i64;

        let last_sync: Option<i64> = self.conn.query_row(
            "SELECT MAX(last_sync) FROM metadata 
             WHERE database_name = ?1 AND schema_name = ?2",
            rusqlite::params![database_name, schema_name],
            |row| row.get(0)
        ).optional().map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "check_cache_validity".to_string(),
                reason: e.to_string(),
            }
        ))?.flatten();

        match last_sync {
            Some(last_sync_time) => Ok((current_time - last_sync_time) < max_age),
            None => Ok(false),
        }
    }

    /// 获取最后同步时间
    /// 
    /// # 参数
    /// * `database_name` - 数据库名称
    /// * `schema_name` - 模式名称
    pub fn get_last_sync_time(&self, database_name: &str, schema_name: &str) -> Result<Option<i64>, CoreError> {
        let last_sync: Option<i64> = self.conn.query_row(
            "SELECT MAX(last_sync) FROM metadata 
             WHERE database_name = ?1 AND schema_name = ?2",
            rusqlite::params![database_name, schema_name],
            |row| row.get(0)
        ).optional().map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "get_last_sync_time".to_string(),
                reason: e.to_string(),
            }
        ))?.flatten();

        Ok(last_sync)
    }

    /// 获取缓存统计信息
    /// 
    /// # 参数
    /// * `database_name` - 数据库名称
    /// * `schema_name` - 模式名称
    pub fn get_cache_stats(&self, database_name: &str, schema_name: &str) -> Result<CacheStats, CoreError> {
        let table_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM metadata 
             WHERE obj_type = 'table' AND database_name = ?1 AND schema_name = ?2",
            rusqlite::params![database_name, schema_name],
            |row| row.get(0)
        ).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "count_tables".to_string(),
                reason: e.to_string(),
            }
        ))?;

        let column_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM metadata 
             WHERE obj_type = 'column' AND database_name = ?1 AND schema_name = ?2",
            rusqlite::params![database_name, schema_name],
            |row| row.get(0)
        ).map_err(|e| CoreError::storage(
            StorageError::Persistence {
                store: "sqlite".to_string(),
                operation: "count_columns".to_string(),
                reason: e.to_string(),
            }
        ))?;

        let last_sync = self.get_last_sync_time(database_name, schema_name)?;

        Ok(CacheStats {
            table_count: table_count as usize,
            column_count: column_count as usize,
            last_sync,
        })
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub table_count: usize,
    pub column_count: usize,
    pub last_sync: Option<i64>,
}

/// 表信息
#[derive(Debug, Clone)]
pub struct TableInfo {
    pub id: String,
    pub name: String,
    pub schema_name: String,
    pub comment: Option<String>,
    pub last_sync: i64,
}

/// 列信息
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub id: String,
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary: bool,
    pub is_unique: bool,
    pub comment: Option<String>,
    pub last_sync: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rdata_test_metadata_cache_{}", name));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_metadata_cache_manager_global() {
        let conn_id = "test_mysql_001";

        let manager = MetadataCacheManager::new(conn_id, ConnectionType::Global, None).unwrap();
        assert!(manager.db_path().to_string_lossy().contains("metadata/global"));
        assert!(manager.db_path().to_string_lossy().contains(conn_id));
    }

    #[test]
    fn test_metadata_cache_manager_project() {
        let project_path = test_temp_dir("project").to_str().unwrap().to_string();
        let conn_id = "test_pg_001";

        let manager = MetadataCacheManager::new(conn_id, ConnectionType::Project, Some(&project_path)).unwrap();
        assert!(manager.db_path().to_string_lossy().contains("meta/connection_metadata"));
        assert!(manager.db_path().to_string_lossy().contains(conn_id));
    }

    #[test]
    fn test_metadata_cache_ops() {
        let db_path = test_temp_dir("ops").join("test_metadata.sqlite");

        let conn = Connection::open(&db_path).unwrap();
        conn.execute("PRAGMA journal_mode=WAL", []).unwrap();

        let ops = MetadataCacheOps::new(conn);

        let result = ops.list_tables("test_db", None);
        assert!(result.is_err());
    }
}
