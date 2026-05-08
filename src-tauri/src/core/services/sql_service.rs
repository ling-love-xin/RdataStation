use std::sync::Arc;

use crate::core::services::connection_manager::ConnectionManager;
use crate::core::driver::traits::DynDatabase;
use crate::core::error::{CoreError, DatabaseError};
use crate::core::models::QueryResult;
use crate::core::persistence::history_store;
use crate::core::cache::get_query_cache;

/// 事务状态结果
#[derive(Debug)]
pub struct TransactionStatusResult {
    pub conn_id: String,
    pub is_in_transaction: bool,
    pub transaction_start_time_ms: Option<u64>,
    pub transaction_duration_ms: Option<u64>,
}

/// SQL 执行服务
///
/// 负责 SQL 查询的执行和管理，包括：
/// - 执行查询
/// - 执行事务
/// - 取消查询
/// - SQL 历史记录
pub struct SqlService {
    manager: Arc<ConnectionManager>,
}

/// SQL 执行结果
#[derive(Debug)]
pub struct SqlExecuteResult {
    /// 查询结果
    pub result: QueryResult,
    /// 执行耗时（毫秒）
    pub elapsed_ms: u64,
    /// 影响的行数（如果有）
    pub affected_rows: Option<usize>,
}

/// SQL 执行选项
#[derive(Debug, Default)]
pub struct SqlExecuteOptions {
    /// 是否记录到历史
    pub record_history: bool,
    /// 是否使用事务
    pub use_transaction: bool,
    /// 查询超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    /// 是否使用查询缓存
    pub use_cache: bool,
}

impl SqlService {
    /// 创建新的 SQL 服务
    pub fn new(manager: Arc<ConnectionManager>) -> Self {
        Self { manager }
    }

    /// 执行 SQL 查询
    ///
    /// # Arguments
    ///
    /// * `conn_id` - 连接 ID（使用活动连接如果为 None）
    /// * `sql` - SQL 语句
    /// * `options` - 执行选项
    ///
    /// # Returns
    ///
    /// 返回执行结果
    pub async fn execute(
        &self,
        conn_id: Option<String>,
        sql: &str,
        options: SqlExecuteOptions,
    ) -> Result<SqlExecuteResult, CoreError> {
        let start_time = std::time::Instant::now();

        // 参数校验
        if sql.trim().is_empty() {
            return Err(CoreError::database(DatabaseError::query(
                sql,
                "SQL statement cannot be empty".to_string(),
            )));
        }

        // 获取查询缓存
        let query_cache = get_query_cache();

        // 检查缓存（仅当启用缓存且不是事务操作时）
        if options.use_cache && !options.use_transaction {
            let connection_id = conn_id.as_deref().unwrap_or("active");
            if let Some(cached_result) = query_cache.get(connection_id, sql).await {
                // 缓存命中，直接返回
                let elapsed_ms = start_time.elapsed().as_millis() as u64;
                
                // 计算影响行数
                let affected_rows = if cached_result.total_rows() == 1 && cached_result.columns.len() == 1 {
                    None
                } else {
                    Some(cached_result.total_rows())
                };
                
                return Ok(SqlExecuteResult {
                    result: cached_result,
                    elapsed_ms,
                    affected_rows,
                });
            }
        }

        // 获取数据库连接
        let db = self.get_database(conn_id.clone()).await?;

        // 创建取消令牌（支持前端 cancel_query）
        let conn_key = conn_id.clone().unwrap_or_else(|| "active".to_string());
        let cancel_token = self.manager.create_cancel_token(&conn_key).await;

        // 执行查询（支持取消和超时）
        let result = if let Some(timeout_ms) = options.timeout_ms {
            let token = cancel_token.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(timeout_ms)).await;
                token.cancel();
            });

            db.query_with_cancel(sql, cancel_token.clone()).await?
        } else {
            db.query_with_cancel(sql, cancel_token.clone()).await?
        };

        // 清理取消令牌
        self.manager.remove_cancel_token(&conn_key).await;

        // 将结果存入缓存（仅当启用缓存且不是事务操作时）
        if options.use_cache && !options.use_transaction {
            let connection_id = conn_id.as_deref().unwrap_or("active");
            let _ = query_cache.set(connection_id, sql, result.clone(), None).await;
        }

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        // 使用 QueryResult 中的 affected_rows（驱动层已正确计算）
        let affected_rows = result.affected_rows;

        // 记录到历史
        if options.record_history {
            tracing::info!(sql = %sql, elapsed_ms, "SQL executed");
            if let Err(e) = history_store::save_sql_history(sql, conn_id.as_deref()) {
                tracing::error!(error = %e, "Failed to save SQL history");
            }
        }

        Ok(SqlExecuteResult {
            result,
            elapsed_ms,
            affected_rows,
        })
    }

    /// 在事务中执行多个 SQL
    ///
    /// # Arguments
    ///
    /// * `conn_id` - 连接 ID
    /// * `sqls` - SQL 语句列表
    ///
    /// # Returns
    ///
    /// 返回每个 SQL 的执行结果
    pub async fn execute_in_transaction(
        &self,
        conn_id: Option<String>,
        sqls: Vec<String>,
    ) -> Result<Vec<SqlExecuteResult>, CoreError> {
        if sqls.is_empty() {
            return Ok(Vec::new());
        }

        // 获取数据库连接
        let db = self.get_database(conn_id).await?;

        // 开始事务
        let mut tx = db.begin_transaction().await?;

        let mut results = Vec::with_capacity(sqls.len());
        let mut failed = false;
        let mut error = None;

        for sql in sqls {
            let start_time = std::time::Instant::now();

            match tx.query(&sql).await {
                Ok(result) => {
                    let elapsed_ms = start_time.elapsed().as_millis() as u64;
                    results.push(SqlExecuteResult {
                        result,
                        elapsed_ms,
                        affected_rows: None,
                    });
                }
                Err(e) => {
                    failed = true;
                    error = Some(e);
                    break;
                }
            }
        }

        if failed {
            // 执行失败，回滚事务
            let _ = tx.rollback().await;
            return Err(error.unwrap_or_else(|| CoreError::database(DatabaseError::Query {
                sql: "unknown".to_string(),
                reason: "Transaction failed with unknown error".to_string(),
                position: None,
            })));
        }

        // 提交事务
        tx.commit().await?;

        Ok(results)
    }

    /// 执行查询（简化版，不带选项）
    pub async fn query(
        &self,
        conn_id: Option<String>,
        sql: &str,
    ) -> Result<QueryResult, CoreError> {
        let result = self
            .execute(
                conn_id,
                sql,
                SqlExecuteOptions {
                    record_history: true,
                    use_cache: true, // 默认启用缓存
                    ..Default::default()
                },
            )
            .await?;
        Ok(result.result)
    }

    /// 获取数据库连接
    async fn get_database(&self, conn_id: Option<String>) -> Result<DynDatabase, CoreError> {
        match conn_id {
            Some(id) => self
                .manager
                .get_connection(&id)
                .await
                .ok_or_else(|| CoreError::connection(crate::core::error::ConnectionError::NotFound(id))),
            None => self
                .manager
                .get_active_connection()
                .await
                .map(|(_, db)| db)
                .ok_or_else(|| CoreError::connection(crate::core::error::ConnectionError::NoActiveConnection)),
        }
    }

    /// 获取 SQL 执行历史
    pub fn get_sql_history(&self, limit: usize) -> Result<Vec<history_store::SqlHistoryRecord>, CoreError> {
        history_store::get_sql_history(limit)
            .map_err(|e| CoreError::storage(crate::core::error::StorageError::read(
                "sql_history",
                e.to_string(),
            )))
    }

    /// 搜索 SQL 历史
    pub fn search_sql_history(
        &self,
        keyword: &str,
        limit: usize,
    ) -> Result<Vec<history_store::SqlHistoryRecord>, CoreError> {
        history_store::search_sql_history(keyword, limit)
            .map_err(|e| CoreError::storage(crate::core::error::StorageError::read(
                "sql_history",
                e.to_string(),
            )))
    }

    /// 清空 SQL 历史
    pub fn clear_sql_history(&self) -> Result<(), CoreError> {
        history_store::clear_sql_history()
            .map_err(|e| CoreError::storage(crate::core::error::StorageError::write(
                "sql_history",
                e.to_string(),
            )))
    }

    /// 删除单条 SQL 历史
    pub fn remove_sql_history(&self, id: &str) -> Result<(), CoreError> {
        history_store::remove_sql_history(id)
            .map_err(|e| CoreError::storage(crate::core::error::StorageError::write(
                "sql_history",
                e.to_string(),
            )))
    }

    /// 注册外部数据库连接
    pub async fn register_external_database(
        &self,
        conn_id: Option<String>,
        name: &str,
        driver: &str,
        connection_string: &str
    ) -> Result<(), CoreError> {
        // 获取数据库连接
        let db = self.get_database(conn_id).await?;

        // 检查是否支持联邦查询
        if !db.meta().supports_federated {
            return Err(CoreError::database(DatabaseError::Driver {
                db_type: "generic".to_string(),
                operation: "register_external_database".to_string(),
                source: "Federated queries not supported".to_string(),
            }));
        }

        // 注册外部数据库
        db.register_external_database(name, driver, connection_string).await
    }

    /// 创建外部表
    pub async fn create_external_table(
        &self,
        conn_id: Option<String>,
        external_db_name: &str,
        schema_name: &str,
        table_name: &str,
        external_table_name: &str
    ) -> Result<(), CoreError> {
        // 获取数据库连接
        let db = self.get_database(conn_id).await?;

        // 检查是否支持联邦查询
        if !db.meta().supports_federated {
            return Err(CoreError::database(DatabaseError::Driver {
                db_type: "generic".to_string(),
                operation: "create_external_table".to_string(),
                source: "Federated queries not supported".to_string(),
            }));
        }

        // 创建外部表
        db.create_external_table(external_db_name, schema_name, table_name, external_table_name).await
    }

    /// 开始事务
    pub async fn begin_transaction(&self, conn_id: Option<String>) -> Result<TransactionStatusResult, CoreError> {
        let db = self.get_database(conn_id.clone()).await?;
        
        // 获取连接 ID
        let conn_id_str = match conn_id {
            Some(id) => id,
            None => self.manager.get_active_conn_id().await.unwrap_or_default(),
        };

        // 执行 BEGIN TRANSACTION
        db.query("BEGIN TRANSACTION").await?;

        // 返回事务状态
        Ok(TransactionStatusResult {
            conn_id: conn_id_str,
            is_in_transaction: true,
            transaction_start_time_ms: Some(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64),
            transaction_duration_ms: Some(0),
        })
    }

    /// 提交事务
    pub async fn commit_transaction(&self, conn_id: Option<String>) -> Result<TransactionStatusResult, CoreError> {
        let db = self.get_database(conn_id.clone()).await?;
        
        // 获取连接 ID
        let conn_id_str = match conn_id {
            Some(id) => id,
            None => self.manager.get_active_conn_id().await.unwrap_or_default(),
        };

        // 执行 COMMIT
        db.query("COMMIT").await?;

        // 返回事务状态
        Ok(TransactionStatusResult {
            conn_id: conn_id_str,
            is_in_transaction: false,
            transaction_start_time_ms: None,
            transaction_duration_ms: None,
        })
    }

    /// 回滚事务
    pub async fn rollback_transaction(&self, conn_id: Option<String>) -> Result<TransactionStatusResult, CoreError> {
        let db = self.get_database(conn_id.clone()).await?;
        
        // 获取连接 ID
        let conn_id_str = match conn_id {
            Some(id) => id,
            None => self.manager.get_active_conn_id().await.unwrap_or_default(),
        };

        // 执行 ROLLBACK
        db.query("ROLLBACK").await?;

        // 返回事务状态
        Ok(TransactionStatusResult {
            conn_id: conn_id_str,
            is_in_transaction: false,
            transaction_start_time_ms: None,
            transaction_duration_ms: None,
        })
    }

    /// 获取事务状态
    pub async fn get_transaction_status(&self, conn_id: Option<String>) -> Result<TransactionStatusResult, CoreError> {
        // 获取连接 ID
        let conn_id_str = match &conn_id {
            Some(id) => id.clone(),
            None => self.manager.get_active_conn_id().await.unwrap_or_default(),
        };

        // 检查连接是否存在
        if conn_id_str.is_empty() {
            return Err(CoreError::connection(crate::core::error::ConnectionError::NoActiveConnection));
        }

        // 检查事务状态（简化实现，实际应从 session 获取）
        Ok(TransactionStatusResult {
            conn_id: conn_id_str,
            is_in_transaction: false,  // 实际应从 session 查询
            transaction_start_time_ms: None,
            transaction_duration_ms: None,
        })
    }

    /// 取消指定连接正在执行的查询
    pub async fn cancel_query(&self, conn_id: Option<String>) -> Result<bool, CoreError> {
        let conn_id_str = match &conn_id {
            Some(id) => id.clone(),
            None => self.manager.get_active_conn_id().await.unwrap_or_default(),
        };
        if conn_id_str.is_empty() {
            return Err(CoreError::connection(crate::core::error::ConnectionError::NoActiveConnection));
        }
        Ok(self.manager.cancel_query(&conn_id_str).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_empty_sql() {
        let manager = Arc::new(ConnectionManager::new());
        let service = SqlService::new(manager);

        let result = service.execute(None, "", Default::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_no_active_connection() {
        let manager = Arc::new(ConnectionManager::new());
        let service = SqlService::new(manager);

        let result = service.execute(None, "SELECT 1", Default::default()).await;
        assert!(result.is_err());
    }
}
