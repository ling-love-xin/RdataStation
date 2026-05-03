//! SQLite 连接池实现
//!
//! 实现 DbPool trait，包装 rusqlite 连接池

use std::sync::Arc;

use crate::core::driver::traits::{DbPool, Database, PoolStatus};
use crate::core::driver::native::sqlite::SqliteDatabase;
use crate::core::error::{CoreError, ConnectionError};

/// SQLite 连接池
///
/// SQLite 是单文件数据库，使用 Arc 包装实现连接复用
pub struct SqlitePoolWrapper {
    path: String,
    closed: Arc<std::sync::atomic::AtomicBool>,
}

impl SqlitePoolWrapper {
    /// 创建 SQLite 连接池
    pub fn new(path: &str) -> Result<Self, CoreError> {
        // 验证路径是否有效
        let _db = SqliteDatabase::new(path)?;
        
        Ok(Self {
            path: path.to_string(),
            closed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }
}

#[async_trait::async_trait]
impl DbPool for SqlitePoolWrapper {
    async fn acquire(&self) -> Result<Box<dyn Database + Send + Sync>, CoreError> {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(CoreError::connection(ConnectionError::PoolClosed));
        }

        let db = SqliteDatabase::new(&self.path)?;
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
            size: 1, // SQLite 单连接
            idle: if self.is_closed() { 0 } else { 1 },
            active: 0,
            waiting: 0,
        }
    }
}
