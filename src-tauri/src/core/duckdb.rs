use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};

use crate::core::error::{CommonError, CoreError};

const DEFAULT_POOL_SIZE: usize = 4;
const MIN_POOL_SIZE: usize = 1;
const MAX_POOL_SIZE: usize = 32;
const MAX_TEMP_TABLES: usize = 50;
const TEMP_TABLE_TTL_SECS: u64 = 1800;
const TEMP_TABLE_PREFIX: &str = "rs_";

/// DuckDBManager manages a pool of in-memory DuckDB connections for parallel
/// query execution. It handles temporary table registration and cleanup,
/// and provides a persistent database connection option.
///
/// # Bottleneck
/// DuckDB's single-writer limitation means that true concurrent writes
/// are not possible; this pool only distributes read operations across
/// multiple connections to reduce contention.
pub struct DuckDBManager {
    in_memory_pool: RwLock<Option<Vec<Arc<Mutex<duckdb::Connection>>>>>,
    preferred_pool_size: AtomicUsize,
    next_conn: AtomicUsize,
    persistent: Mutex<Option<(String, Arc<Mutex<duckdb::Connection>>)>>,
    table_registry: Mutex<HashMap<String, Instant>>,
    max_temp_tables: usize,
}

impl DuckDBManager {
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<DuckDBManager> = OnceLock::new();
        INSTANCE.get_or_init(|| Self {
            in_memory_pool: RwLock::new(None),
            preferred_pool_size: AtomicUsize::new(DEFAULT_POOL_SIZE),
            next_conn: AtomicUsize::new(0),
            persistent: Mutex::new(None),
            table_registry: Mutex::new(HashMap::new()),
            max_temp_tables: MAX_TEMP_TABLES,
        })
    }

    pub fn get_or_create_in_memory(&self) -> Result<Arc<Mutex<duckdb::Connection>>, CoreError> {
        {
            let pool_guard = self.in_memory_pool.read().map_err(|e| {
                CoreError::common(CommonError::General(format!("DuckDB lock error: {}", e)))
            })?;
            if let Some(ref pool) = *pool_guard {
                if !pool.is_empty() {
                    let idx = self.next_conn.fetch_add(1, Ordering::Relaxed) % pool.len();
                    return Ok(pool[idx].clone());
                }
            }
        }

        let size = self
            .preferred_pool_size
            .load(Ordering::Relaxed)
            .clamp(MIN_POOL_SIZE, MAX_POOL_SIZE);
        let mut pool = Vec::with_capacity(size);
        for _ in 0..size {
            let conn = Self::open_in_memory_conn()?;
            pool.push(Arc::new(Mutex::new(conn)));
        }

        {
            let mut pool_guard = self.in_memory_pool.write().map_err(|e| {
                CoreError::common(CommonError::General(format!("DuckDB lock error: {}", e)))
            })?;
            *pool_guard = Some(pool);
        }

        let pool_guard = self.in_memory_pool.read().map_err(|e| {
            CoreError::common(CommonError::General(format!("DuckDB lock error: {}", e)))
        })?;
        let pool = pool_guard.as_ref().ok_or_else(|| {
            CoreError::common(CommonError::General("DuckDB 连接池初始化失败".to_string()))
        })?;
        let idx = self.next_conn.fetch_add(1, Ordering::Relaxed) % pool.len();
        Ok(pool[idx].clone())
    }

    pub fn register_temp_table(&self, table_name: &str) {
        let mut registry = self
            .table_registry
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        registry.insert(table_name.to_string(), Instant::now());
        drop(registry);

        self.evict_oldest_tables();
        self.cleanup_expired_tables();
    }

    pub fn evict_oldest_tables(&self) {
        let mut registry = self
            .table_registry
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        while registry.len() > self.max_temp_tables {
            if let Some(oldest_name) = registry
                .iter()
                .min_by_key(|(_, t)| **t)
                .map(|(name, _)| name.clone())
            {
                registry.remove(&oldest_name);
                if let Ok(pool_guard) = self.in_memory_pool.read() {
                    if let Some(pool) = pool_guard.as_ref() {
                        if let Some(conn) = pool.first() {
                            if let Ok(guard) = conn.lock() {
                                let drop_sql = format!("DROP TABLE IF EXISTS \"{}\"", oldest_name);
                                let _ = guard.execute_batch(&drop_sql);
                            }
                        }
                    }
                }
                tracing::info!(
                    "[DuckDBManager] 计数淘汰临时表: {} (registry > {})",
                    oldest_name,
                    self.max_temp_tables
                );
            } else {
                break;
            }
        }
    }

    pub fn cleanup_expired_tables(&self) {
        let ttl = Duration::from_secs(TEMP_TABLE_TTL_SECS);
        let now = Instant::now();
        let expired: Vec<String>;

        {
            let mut registry = self
                .table_registry
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            expired = registry
                .iter()
                .filter(|(_, created_at)| now.duration_since(**created_at) >= ttl)
                .map(|(name, _)| name.clone())
                .collect();
            for name in &expired {
                registry.remove(name);
            }
        }

        if expired.is_empty() {
            return;
        }

        if let Ok(pool_guard) = self.in_memory_pool.read() {
            if let Some(pool) = pool_guard.as_ref() {
                if let Some(conn) = pool.first() {
                    if let Ok(guard) = conn.lock() {
                        for name in &expired {
                            let drop_sql = format!("DROP TABLE IF EXISTS \"{}\"", name);
                            let _ = guard.execute_batch(&drop_sql);
                            tracing::info!(
                                "[DuckDBManager] TTL 淘汰临时表: {} ({:.0}s 过期)",
                                name,
                                TEMP_TABLE_TTL_SECS
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn cleanup_temp_table(&self, table_name: &str) {
        if let Ok(mut registry) = self.table_registry.lock() {
            registry.remove(table_name);
        }
    }

    pub fn temp_table_count(&self) -> usize {
        self.table_registry.lock().map(|r| r.len()).unwrap_or(0)
    }

    pub fn validate_analysis_sql(sql: &str) -> Result<(), CoreError> {
        let trimmed = sql.trim().to_uppercase();

        let dangerous_keywords = ["ATTACH", "DETACH", "INSTALL", "LOAD", "EXPORT DATABASE"];
        for kw in &dangerous_keywords {
            if trimmed.contains(kw) {
                return Err(CoreError::common(CommonError::General(format!(
                    "禁止的操作: {} 在分析SQL中不被允许",
                    kw
                ))));
            }
        }

        if trimmed.contains("DROP") {
            let prefix_upper = TEMP_TABLE_PREFIX.to_uppercase();
            if !trimmed.contains(&format!("DROP TABLE IF EXISTS \"{}", prefix_upper))
                && !trimmed.contains(&format!("DROP TABLE \"{}", prefix_upper))
            {
                return Err(CoreError::common(CommonError::General(format!(
                    "DROP 操作仅允许删除临时表 ({}前缀)",
                    TEMP_TABLE_PREFIX
                ))));
            }
        }

        if trimmed.contains("CREATE TABLE") {
            let prefix_upper = TEMP_TABLE_PREFIX.to_uppercase();
            if !trimmed.contains(&format!("CREATE TABLE \"{}", prefix_upper))
                && !trimmed.contains(&format!("CREATE TABLE {}", prefix_upper))
            {
                return Err(CoreError::common(CommonError::General(format!(
                    "CREATE TABLE 仅允许创建临时表 ({}前缀)",
                    TEMP_TABLE_PREFIX
                ))));
            }
        }

        Ok(())
    }

    pub fn ensure_connection() -> Result<duckdb::Connection, CoreError> {
        Self::open_in_memory_conn()
    }

    fn open_in_memory_conn() -> Result<duckdb::Connection, CoreError> {
        duckdb::Connection::open_in_memory().map_err(|e| {
            CoreError::common(CommonError::General(format!(
                "Failed to create in-memory DuckDB connection: {}",
                e
            )))
        })
    }

    pub fn open_file_with_retry(path: &str) -> Result<duckdb::Connection, CoreError> {
        let path_buf = std::path::PathBuf::from(path);

        if let Some(parent) = path_buf.parent() {
            if parent.as_os_str() != "" {
                std::fs::create_dir_all(parent).map_err(|e| {
                    CoreError::common(CommonError::General(format!(
                        "Failed to create directory {:?}: {}",
                        parent, e
                    )))
                })?;
            }
        }

        match duckdb::Connection::open(&path_buf) {
            Ok(conn) => Ok(conn),
            Err(e) => {
                tracing::warn!(
                    "Failed to open DuckDB at {}: {}, attempting to recreate...",
                    path_buf.display(),
                    e
                );
                if path_buf.exists() {
                    if let Err(remove_err) = std::fs::remove_file(&path_buf) {
                        tracing::error!(
                            "Failed to remove corrupted DuckDB file: {}",
                            remove_err
                        );
                        return Err(CoreError::common(CommonError::General(format!(
                            "Failed to open persistent DuckDB at {}: {}",
                            path, e
                        ))));
                    }
                    tracing::info!(
                        "Removed corrupted DuckDB file: {}",
                        path_buf.display()
                    );
                }
                duckdb::Connection::open(&path_buf).map_err(|_| {
                    CoreError::common(CommonError::General(format!(
                        "Failed to open persistent DuckDB at {}: {}",
                        path, e
                    )))
                })
            }
        }
    }

    pub fn pool_size(&self) -> usize {
        self.in_memory_pool
            .read()
            .ok()
            .and_then(|g| g.as_ref().map(|p| p.len()))
            .unwrap_or(0)
    }

    pub fn set_pool_size(&self, size: usize) -> usize {
        let clamped = size.clamp(MIN_POOL_SIZE, MAX_POOL_SIZE);
        self.preferred_pool_size.store(clamped, Ordering::Relaxed);
        clamped
    }

    pub fn preferred_pool_size(&self) -> usize {
        self.preferred_pool_size.load(Ordering::Relaxed)
    }

    pub fn restart_pool(&self) -> Result<usize, CoreError> {
        let size = self.preferred_pool_size.load(Ordering::Relaxed);
        let mut pool = Vec::with_capacity(size);
        for _ in 0..size {
            let conn = Self::open_in_memory_conn()?;
            pool.push(Arc::new(Mutex::new(conn)));
        }

        {
            let mut pool_guard = self.in_memory_pool.write().map_err(|e| {
                CoreError::common(CommonError::General(format!("DuckDB lock error: {}", e)))
            })?;
            *pool_guard = Some(pool);
        }

        {
            let mut registry = self
                .table_registry
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            registry.clear();
        }

        self.next_conn.store(0, Ordering::Relaxed);
        Ok(size)
    }

    pub fn get_persistent(&self) -> Result<Option<Arc<Mutex<duckdb::Connection>>>, CoreError> {
        let guard = self.persistent.lock().map_err(|e| {
            CoreError::common(CommonError::General(format!("DuckDB lock error: {}", e)))
        })?;
        Ok(guard.as_ref().map(|(_, conn)| conn.clone()))
    }

    pub fn set_persistent(&self, path: &str) -> Result<Arc<Mutex<duckdb::Connection>>, CoreError> {
        let mut guard = self.persistent.lock().map_err(|e| {
            CoreError::common(CommonError::General(format!("DuckDB lock error: {}", e)))
        })?;

        if let Some((ref existing_path, ref conn)) = *guard {
            if existing_path == path {
                return Ok(conn.clone());
            }
        }

        let conn = Self::open_file_with_retry(path)?;
        let conn = Arc::new(Mutex::new(conn));
        *guard = Some((path.to_string(), conn.clone()));
        Ok(conn)
    }

    pub fn clear_persistent(&self) -> Result<(), CoreError> {
        let mut guard = self.persistent.lock().map_err(|e| {
            CoreError::common(CommonError::General(format!("DuckDB lock error: {}", e)))
        })?;
        *guard = None;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn insert_test_timestamp(&self, table_name: &str, timestamp: Instant) {
        let mut registry = self
            .table_registry
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        registry.insert(table_name.to_string(), timestamp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_size_default_4() {
        let mgr = DuckDBManager::global();
        let _ = mgr.get_or_create_in_memory().expect("create pool");
        assert_eq!(mgr.pool_size(), DEFAULT_POOL_SIZE);
    }

    #[test]
    fn test_round_robin_distribution() {
        let mgr = DuckDBManager::global();
        let mut pointers = Vec::new();
        for _ in 0..(DEFAULT_POOL_SIZE * 3) {
            let conn = mgr.get_or_create_in_memory().expect("get conn");
            pointers.push(Arc::as_ptr(&conn) as usize);
        }
        let unique: std::collections::HashSet<_> = pointers.iter().collect();
        assert_eq!(unique.len(), DEFAULT_POOL_SIZE);
    }

    #[test]
    fn test_temp_table_registry() {
        let mgr = DuckDBManager::global();
        let initial = mgr.temp_table_count();
        mgr.register_temp_table("rs_test_1");
        mgr.register_temp_table("rs_test_2");
        assert_eq!(mgr.temp_table_count(), initial + 2);
        mgr.cleanup_temp_table("rs_test_1");
        assert_eq!(mgr.temp_table_count(), initial + 1);
    }

    #[test]
    fn test_validate_analysis_sql_allows_select() {
        assert!(DuckDBManager::validate_analysis_sql("SELECT * FROM rs_test").is_ok());
        assert!(DuckDBManager::validate_analysis_sql("SELECT COUNT(*) FROM rs_test").is_ok());
    }

    #[test]
    fn test_validate_analysis_sql_blocks_attach() {
        assert!(DuckDBManager::validate_analysis_sql("ATTACH 'file.db'").is_err());
        assert!(DuckDBManager::validate_analysis_sql("INSTALL httpfs").is_err());
    }

    #[test]
    fn test_validate_analysis_sql_blocks_drop_non_rs() {
        assert!(DuckDBManager::validate_analysis_sql("DROP TABLE users").is_err());
        assert!(
            DuckDBManager::validate_analysis_sql("DROP TABLE IF EXISTS \"rs_temp_123\"").is_ok()
        );
    }

    #[test]
    fn test_ttl_cleanup_expired_tables() {
        let mgr = DuckDBManager::global();
        let _ = mgr.get_or_create_in_memory().expect("create pool");
        let initial = mgr.temp_table_count();

        let long_ago = Instant::now()
            .checked_sub(Duration::from_secs(TEMP_TABLE_TTL_SECS + 60))
            .expect("checked_sub should work for reasonable durations");
        mgr.insert_test_timestamp("rs_ttl_expired", long_ago);
        assert_eq!(mgr.temp_table_count(), initial + 1);

        mgr.cleanup_expired_tables();
        assert_eq!(
            mgr.temp_table_count(),
            initial,
            "TTL 过期表应被清理，expired = {}s old, TTL = {}s",
            TEMP_TABLE_TTL_SECS + 60,
            TEMP_TABLE_TTL_SECS
        );
    }

    #[test]
    fn test_ttl_fresh_table_not_cleaned() {
        let mgr = DuckDBManager::global();
        let _ = mgr.get_or_create_in_memory().expect("create pool");
        let initial = mgr.temp_table_count();

        mgr.register_temp_table("rs_ttl_fresh");
        assert_eq!(mgr.temp_table_count(), initial + 1);

        mgr.cleanup_expired_tables();
        assert_eq!(
            mgr.temp_table_count(),
            initial + 1,
            "30分钟内新建的表不应被 TTL 清理"
        );

        mgr.cleanup_temp_table("rs_ttl_fresh");
    }
}
