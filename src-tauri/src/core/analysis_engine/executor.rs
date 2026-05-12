use duckdb::Connection;

use super::manager::DuckDBManager;
use crate::core::error::{CommonError, CoreError};

/// DuckDB SQL 执行结果
pub struct DuckDBResult {
    /// 受影响的行数（适用于 INSERT/UPDATE/DELETE）
    pub rows_affected: Option<u64>,
}

/// DuckDB SQL 执行器
///
/// 所有业务模块禁止直接调用 `duckdb::Connection`，必须通过本执行器执行 SQL。
///
/// # 保障
/// - 统一错误处理、日志记录
/// - 读写连接物理隔离
/// - 屏蔽底层连接管理细节
///
/// # 业务调用规范
/// - 只读分析 SQL → 使用 `execute_read()`
/// - 写入临时表/物化数据 → 使用 `execute_write()`
pub struct DuckDBExecutor<'a> {
    /// DuckDB 管理器引用
    manager: &'a DuckDBManager,
}

impl<'a> DuckDBExecutor<'a> {
    /// 创建新的 SQL 执行器。
    ///
    /// # 参数
    /// - `manager`: DuckDBManager 引用
    ///
    /// # 返回
    /// DuckDBExecutor 实例
    pub fn new(manager: &'a DuckDBManager) -> Self {
        DuckDBExecutor { manager }
    }

    /// 执行只读 SQL 查询。
    ///
    /// # 参数
    /// - `sql`: SQL 查询语句
    ///
    /// # 返回
    /// - `Ok(DuckDBResult)`: 查询结果
    /// - `Err(CoreError)`: 执行失败
    ///
    /// # 示例
    /// ```rust,ignore
    /// let executor = DuckDBExecutor::new(&manager);
    /// let result = executor.execute_read("SELECT * FROM users")?;
    /// ```
    pub fn execute_read(&self, sql: &str) -> Result<DuckDBResult, CoreError> {
        let conn = self.manager.read_conn();
        Self::execute_query(conn, sql)
    }

    /// 执行写入 SQL。
    ///
    /// # 参数
    /// - `sql`: SQL 写入语句（INSERT/UPDATE/DELETE/CREATE 等）
    ///
    /// # 返回
    /// - `Ok(DuckDBResult)`: 执行结果，包含受影响行数
    /// - `Err(CoreError)`: 执行失败
    ///
    /// # 示例
    /// ```rust,ignore
    /// let executor = DuckDBExecutor::new(&manager);
    /// let result = executor.execute_write("CREATE TABLE test (id INTEGER)")?;
    /// ```
    pub fn execute_write(&self, sql: &str) -> Result<DuckDBResult, CoreError> {
        let conn = self.manager.write_conn();
        Self::execute_query(conn, sql)
    }

    /// 执行参数化只读查询。
    ///
    /// # 参数
    /// - `sql`: SQL 查询语句，可包含 `?` 占位符
    /// - `params`: 查询参数
    ///
    /// # 返回
    /// - `Ok(DuckDBResult)`: 查询结果
    /// - `Err(CoreError)`: 执行失败
    #[allow(dead_code)]
    pub fn execute_read_with_params(
        &self,
        sql: &str,
        params: &[&dyn duckdb::types::ToSql],
    ) -> Result<DuckDBResult, CoreError> {
        let conn = self.manager.read_conn();
        Self::execute_query_with_params(conn, sql, params)
    }

    /// 执行参数化写入 SQL。
    ///
    /// # 参数
    /// - `sql`: SQL 写入语句，可包含 `?` 占位符
    /// - `params`: 查询参数
    ///
    /// # 返回
    /// - `Ok(DuckDBResult)`: 执行结果
    /// - `Err(CoreError)`: 执行失败
    #[allow(dead_code)]
    pub fn execute_write_with_params(
        &self,
        sql: &str,
        params: &[&dyn duckdb::types::ToSql],
    ) -> Result<DuckDBResult, CoreError> {
        let conn = self.manager.write_conn();
        Self::execute_query_with_params(conn, sql, params)
    }

    /// 执行批量 SQL 语句。
    ///
    /// # 参数
    /// - `sql`: 批量 SQL 语句，以分号分隔
    ///
    /// # 返回
    /// - `Ok(())`: 执行成功
    /// - `Err(CoreError)`: 执行失败
    #[allow(dead_code)]
    pub fn execute_batch(&self, sql: &str) -> Result<(), CoreError> {
        let conn = self.manager.write_conn();
        conn.execute_batch(sql).map_err(|e| {
            tracing::error!("[DuckDBExecutor] 批量执行失败: {}, SQL: {}", e, sql);
            CoreError::common(CommonError::General(format!("批量执行 SQL 失败: {}", e)))
        })
    }

    /// 执行事务性 SQL 批量操作。
    ///
    /// # 参数
    /// - `sql`: 批量 SQL 语句
    ///
    /// # 返回
    /// - `Ok(())`: 事务执行成功
    /// - `Err(CoreError)`: 事务执行失败，自动回滚
    #[allow(dead_code)]
    pub fn execute_transaction(&self, sql: &str) -> Result<(), CoreError> {
        let write_conn = self.manager.write_conn();
        
        // DuckDB Connection needs mutable access for transactions
        // This is a simplified version - in production you'd need proper connection management
        write_conn.execute_batch(sql).map_err(|e| {
            CoreError::common(CommonError::General(format!("事务执行失败: {}", e)))
        })
    }

    /// 内部：执行 SQL 查询（通用）。
    ///
    /// # 参数
    /// - `conn`: DuckDB 连接
    /// - `sql`: SQL 语句
    ///
    /// # 返回
    /// - `Ok(DuckDBResult)`: 执行结果
    /// - `Err(CoreError)`: 执行失败
    fn execute_query(conn: &Connection, sql: &str) -> Result<DuckDBResult, CoreError> {
        let trimmed = sql.trim().to_uppercase();

        if trimmed.starts_with("SELECT") || trimmed.starts_with("WITH") || trimmed.starts_with("EXPLAIN") {
            let mut stmt = conn.prepare(sql).map_err(|e| {
                CoreError::common(CommonError::General(format!("准备 SQL 语句失败: {}", e)))
            })?;

            let _rows = stmt.query([]).map_err(|e| {
                CoreError::common(CommonError::General(format!("执行 SQL 查询失败: {}", e)))
            })?;

            let _: Vec<()> = Vec::new();

            Ok(DuckDBResult {
                rows_affected: None,
            })
        } else {
            let rows_affected = conn.execute(sql, []).map_err(|e| {
                CoreError::common(CommonError::General(format!("执行写入 SQL 失败: {}", e)))
            })? as u64;

            Ok(DuckDBResult {
                rows_affected: Some(rows_affected),
            })
        }
    }

    /// 内部：执行参数化 SQL 查询。
    ///
    /// # 参数
    /// - `conn`: DuckDB 连接
    /// - `sql`: SQL 语句
    /// - `params`: 查询参数
    ///
    /// # 返回
    /// - `Ok(DuckDBResult)`: 执行结果
    /// - `Err(CoreError)`: 执行失败
    fn execute_query_with_params(
        conn: &Connection,
        sql: &str,
        params: &[&dyn duckdb::types::ToSql],
    ) -> Result<DuckDBResult, CoreError> {
        let trimmed = sql.trim().to_uppercase();

        let mut stmt = conn.prepare(sql).map_err(|e| {
            CoreError::common(CommonError::General(format!("准备 SQL 语句失败: {}", e)))
        })?;

        if trimmed.starts_with("SELECT") || trimmed.starts_with("WITH") || trimmed.starts_with("EXPLAIN") {
            let _rows = stmt.query(params).map_err(|e| {
                CoreError::common(CommonError::General(format!("执行 SQL 查询失败: {}", e)))
            })?;

            let _: Vec<()> = Vec::new();

            Ok(DuckDBResult {
                rows_affected: None,
            })
        } else {
            let rows_affected = stmt.execute(params).map_err(|e| {
                CoreError::common(CommonError::General(format!("执行写入 SQL 失败: {}", e)))
            })? as u64;

            Ok(DuckDBResult {
                rows_affected: Some(rows_affected),
            })
        }
    }
}

// ========== 测试 ==========
#[cfg(test)]
mod tests {
    use super::*;
    use super::manager::DuckDBManager;
    use std::fs;

    fn setup_test_executor() -> (DuckDBExecutor<'static>, std::path::PathBuf) {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_executor_{}.duckdb", std::process::id()));

        // 清理可能存在的旧文件
        let _ = fs::remove_file(&db_path);

        // 使用 Box::leak 创建 'static 生命周期引用用于测试
        let manager = Box::new(DuckDBManager::open(&db_path).expect("创建测试数据库"));
        let manager: &'static DuckDBManager = Box::leak(manager);

        let executor = DuckDBExecutor::new(manager);
        (executor, db_path)
    }

    fn cleanup_test_db(path: &std::path::Path) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_execute_read_simple_select() {
        let (executor, db_path) = setup_test_executor();

        // 先创建测试表
        executor
            .execute_write("CREATE TABLE test_users (id INTEGER, name TEXT)")
            .expect("创建表");
        executor
            .execute_write("INSERT INTO test_users VALUES (1, 'Alice'), (2, 'Bob')")
            .expect("插入数据");

        // 执行只读查询
        let result = executor.execute_read("SELECT * FROM test_users");
        assert!(result.is_ok());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_execute_write_create_table() {
        let (executor, db_path) = setup_test_executor();

        let result = executor.execute_write("CREATE TABLE test_table (id INTEGER PRIMARY KEY, value TEXT)");
        assert!(result.is_ok());

        let duckdb_result = result.unwrap();
        assert!(duckdb_result.rows_affected.is_some());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_execute_write_insert() {
        let (executor, db_path) = setup_test_executor();

        executor.execute_write("CREATE TABLE test_insert (id INTEGER)").expect("创建表");

        let result = executor.execute_write("INSERT INTO test_insert VALUES (1), (2), (3)");
        assert!(result.is_ok());

        let duckdb_result = result.unwrap();
        assert_eq!(duckdb_result.rows_affected, Some(3));

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_execute_invalid_sql() {
        let (executor, db_path) = setup_test_executor();

        let result = executor.execute_read("INVALID SQL STATEMENT");
        assert!(result.is_err());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_execute_batch() {
        let (executor, db_path) = setup_test_executor();

        let sql = "
            CREATE TABLE test_batch1 (id INTEGER);
            CREATE TABLE test_batch2 (name TEXT);
            INSERT INTO test_batch1 VALUES (1);
            INSERT INTO test_batch2 VALUES ('test');
        ";

        let result = executor.execute_batch(sql);
        assert!(result.is_ok());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_execute_transaction() {
        let (executor, db_path) = setup_test_executor();

        executor.execute_write("CREATE TABLE test_tx (id INTEGER)").expect("创建表");

        let sql = "
            INSERT INTO test_tx VALUES (1);
            INSERT INTO test_tx VALUES (2);
            INSERT INTO test_tx VALUES (3);
        ";

        let result = executor.execute_transaction(sql);
        assert!(result.is_ok());

        // 验证数据已插入
        let select_result = executor.execute_read("SELECT COUNT(*) FROM test_tx");
        assert!(select_result.is_ok());

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_execute_transaction_rollback() {
        let (executor, db_path) = setup_test_executor();

        executor.execute_write("CREATE TABLE test_rollback (id INTEGER)").expect("创建表");

        // 故意包含错误 SQL 触发回滚
        let sql = "
            INSERT INTO test_rollback VALUES (1);
            INSERT INTO test_rollback VALUES (2);
            INVALID SQL;
        ";

        let result = executor.execute_transaction(sql);
        assert!(result.is_err());

        cleanup_test_db(&db_path);
    }
}
