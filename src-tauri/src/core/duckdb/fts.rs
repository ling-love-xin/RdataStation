use crate::core::error::{CommonError, CoreError};

/// 全文搜索管理器
///
/// 负责 FTS 索引维护与查询。
///
/// # 使用方式
/// 1. 为表创建 FTS 索引
/// 2. 执行全文搜索查询
/// 3. 更新/删除索引
///
/// # DuckDB FTS 扩展
/// 需要加载 `fts` 扩展以支持全文搜索功能。
pub struct FTSManager;

impl FTSManager {
    /// 生成创建 FTS 索引的 SQL 语句。
    ///
    /// # 参数
    /// - `index_name`: 索引名称
    /// - `table_name`: 表名
    /// - `columns`: 需要索引的列名列表
    ///
    /// # 返回
    /// 创建 FTS 索引的 SQL 语句
    ///
    /// # 示例
    /// ```rust,ignore
    /// let sql = FTSManager::generate_create_index_sql(
    ///     "users_fts",
    ///     "users",
    ///     &["name", "description"]
    /// );
    /// ```
    pub fn generate_create_index_sql(
        index_name: &str,
        table_name: &str,
        columns: &[&str],
    ) -> String {
        let cols = columns.join(", ");
        format!(
            "CREATE INDEX {} ON {} USING FTS ({})",
            index_name, table_name, cols
        )
    }

    /// 生成删除 FTS 索引的 SQL 语句。
    ///
    /// # 参数
    /// - `index_name`: 索引名称
    ///
    /// # 返回
    /// 删除 FTS 索引的 SQL 语句
    pub fn generate_drop_index_sql(index_name: &str) -> String {
        format!("DROP INDEX IF EXISTS {}", index_name)
    }

    /// 生成全文搜索查询 SQL。
    ///
    /// # 参数
    /// - `table_name`: 表名
    /// - `search_term`: 搜索词
    /// - `columns`: 搜索的列名列表
    ///
    /// # 返回
    /// 全文搜索查询 SQL
    ///
    /// # 示例
    /// ```rust,ignore
    /// let sql = FTSManager::generate_search_sql(
    ///     "users",
    ///     "John Doe",
    ///     &["name", "description"]
    /// );
    /// ```
    pub fn generate_search_sql(
        table_name: &str,
        _search_term: &str,
        columns: &[&str],
    ) -> String {
        let search_conditions: Vec<String> = columns
            .iter()
            .map(|col| format!("{} MATCH ?", col))
            .collect();

        let where_clause = search_conditions.join(" OR ");

        format!(
            "SELECT * FROM {} WHERE {}",
            table_name, where_clause
        )
    }

    /// 生成重建 FTS 索引的 SQL 语句。
    ///
    /// # 参数
    /// - `index_name`: 索引名称
    /// - `table_name`: 表名
    /// - `columns`: 需要索引的列名列表
    ///
    /// # 返回
    /// 重建 FTS 索引的 SQL 语句（DROP + CREATE）
    pub fn generate_rebuild_index_sql(
        index_name: &str,
        table_name: &str,
        columns: &[&str],
    ) -> String {
        let drop_sql = Self::generate_drop_index_sql(index_name);
        let create_sql = Self::generate_create_index_sql(index_name, table_name, columns);
        format!("{}; {}", drop_sql, create_sql)
    }

    /// 生成检查索引是否存在的 SQL 查询。
    ///
    /// # 参数
    /// - `index_name`: 索引名称
    ///
    /// # 返回
    /// 检查索引是否存在的查询 SQL
    pub fn generate_check_index_sql(index_name: &str) -> String {
        format!(
            "SELECT COUNT(*) FROM duckdb_indexes WHERE index_name = '{}'",
            index_name
        )
    }

    /// 验证搜索词是否合法。
    ///
    /// # 参数
    /// - `search_term`: 搜索词
    ///
    /// # 返回
    /// - `Ok(())`: 搜索词合法
    /// - `Err(CoreError)`: 搜索词不合法
    pub fn validate_search_term(search_term: &str) -> Result<(), CoreError> {
        if search_term.trim().is_empty() {
            return Err(CoreError::common(CommonError::General(
                "搜索词不能为空".to_string(),
            )));
        }

        if search_term.len() > 200 {
            return Err(CoreError::common(CommonError::General(
                "搜索词长度不能超过200字符".to_string(),
            )));
        }

        Ok(())
    }
}

// ========== 测试 ==========
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_create_index_sql() {
        let sql = FTSManager::generate_create_index_sql(
            "users_fts",
            "users",
            &["name", "description"],
        );
        assert!(sql.contains("CREATE INDEX users_fts"));
        assert!(sql.contains("ON users USING FTS"));
        assert!(sql.contains("name, description"));
    }

    #[test]
    fn test_generate_drop_index_sql() {
        let sql = FTSManager::generate_drop_index_sql("users_fts");
        assert_eq!(sql, "DROP INDEX IF EXISTS users_fts");
    }

    #[test]
    fn test_generate_search_sql() {
        let sql = FTSManager::generate_search_sql(
            "users",
            "John Doe",
            &["name", "description"],
        );
        assert!(sql.contains("SELECT * FROM users"));
        assert!(sql.contains("name MATCH ?"));
        assert!(sql.contains("description MATCH ?"));
    }

    #[test]
    fn test_generate_rebuild_index_sql() {
        let sql = FTSManager::generate_rebuild_index_sql(
            "users_fts",
            "users",
            &["name"],
        );
        assert!(sql.contains("DROP INDEX"));
        assert!(sql.contains("CREATE INDEX"));
    }

    #[test]
    fn test_generate_check_index_sql() {
        let sql = FTSManager::generate_check_index_sql("users_fts");
        assert!(sql.contains("duckdb_indexes"));
        assert!(sql.contains("users_fts"));
    }

    #[test]
    fn test_validate_search_term_empty() {
        assert!(FTSManager::validate_search_term("").is_err());
        assert!(FTSManager::validate_search_term("   ").is_err());
    }

    #[test]
    fn test_validate_search_term_too_long() {
        let long_term = "a".repeat(201);
        assert!(FTSManager::validate_search_term(&long_term).is_err());
    }

    #[test]
    fn test_validate_search_term_valid() {
        assert!(FTSManager::validate_search_term("hello").is_ok());
        assert!(FTSManager::validate_search_term("John Doe").is_ok());
    }
}
