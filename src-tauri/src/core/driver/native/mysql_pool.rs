//! MySQL 连接池实现
//!
//! 实现 DbPool trait，包装 sqlx::MySqlPool

use std::sync::Arc;

use sqlx::mysql::MySqlPool;

use crate::core::driver::traits::{DbPool, Database, PoolStatus};
use crate::core::driver::native::mysql::MySqlDatabase;
use crate::core::error::{CoreError, ConnectionError};

/// MySQL 连接池
pub struct MySqlPoolWrapper {
    pool: MySqlPool,
    closed: Arc<std::sync::atomic::AtomicBool>,
}

impl MySqlPoolWrapper {
    /// 从 URL 创建连接池
    pub async fn new(url: &str) -> Result<Self, CoreError> {
        let pool = MySqlPool::connect(url)
            .await
            .map_err(|e| CoreError::connection(ConnectionError::Refused {
                conn_id: "mysql".to_string(),
                reason: e.to_string(),
            }))?;
        
        Ok(Self {
            pool,
            closed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// 从现有 Pool 创建
    pub fn from_pool(pool: MySqlPool) -> Self {
        Self {
            pool,
            closed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

#[async_trait::async_trait]
impl DbPool for MySqlPoolWrapper {
    async fn acquire(&self) -> Result<Box<dyn Database + Send + Sync>, CoreError> {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(CoreError::connection(ConnectionError::PoolClosed));
        }

        let db = MySqlDatabase::from_pool(self.pool.clone());
        Ok(Box::new(db))
    }

    async fn close(&self) -> Result<(), CoreError> {
        self.closed.store(true, std::sync::atomic::Ordering::SeqCst);
        self.pool.close().await;
        Ok(())
    }

    fn is_closed(&self) -> bool {
        self.closed.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn status(&self) -> PoolStatus {
        let size = self.pool.size() as usize;
        let idle = self.pool.num_idle() as usize;
        let active = size - idle;
        
        PoolStatus {
            size,
            idle,
            active,
            waiting: 0, // sqlx 不直接暴露等待数
        }
    }
}
