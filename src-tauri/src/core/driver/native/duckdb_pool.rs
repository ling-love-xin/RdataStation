//! DuckDB 连接池实现
//!
//! 实现 DbPool trait，包装 duckdb 连接池

use std::sync::Arc;

use crate::core::driver::native::duckdb::DuckDbDatabase;
use crate::core::driver::traits::{Database, DbPool, PoolStatus};
use crate::core::error::{ConnectionError, CoreError};

/// DuckDB 连接池
///
/// DuckDB 支持并发读取，但写入需要互斥
pub struct DuckDbPoolWrapper {
    path: String,
    closed: Arc<std::sync::atomic::AtomicBool>,
}

impl DuckDbPoolWrapper {
    /// 创建 DuckDB 连接池
    pub fn new(path: &str) -> Result<Self, CoreError> {
        // 验证路径是否有效
        let _db = DuckDbDatabase::new(path)?;

        Ok(Self {
            path: path.to_string(),
            closed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }
}

#[async_trait::async_trait]
impl DbPool for DuckDbPoolWrapper {
    async fn acquire(&self) -> Result<Box<dyn Database + Send + Sync>, CoreError> {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(CoreError::connection(ConnectionError::PoolClosed));
        }

        let db = DuckDbDatabase::new(&self.path)?;
        Ok(Box::new(db))
    }

    async fn close(&self) -> Result<(), CoreError> {
        self.closed.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    fn is_closed(&self) -> bool {
        self.closed.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn status(&self) -> PoolStatus {
        PoolStatus {
            size: 1, // DuckDB 单连接（后续可改为连接池）
            idle: if self.is_closed() { 0 } else { 1 },
            active: 0,
            waiting: 0,
            max_connections: 1,
            min_connections: 1,
        }
    }
}
