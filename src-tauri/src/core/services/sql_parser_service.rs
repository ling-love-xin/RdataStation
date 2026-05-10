//! SQL 解析与转译服务
//!
//! 基于 sqlglot-rust 提供 SQL 解析、验证、格式化和跨方言转译功能

use sqlglot_rust::{parse, transpile, Dialect};

/// SQL 方言枚举
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SqlDialect {
    Generic,
    Mysql,
    Postgres,
    Sqlite,
    Duckdb,
    MsSQL,
    Oracle,
    Snowflake,
    BigQuery,
    Redshift,
}

impl SqlDialect {
    pub fn to_dialect(&self) -> Dialect {
        match self {
            SqlDialect::Generic => Dialect::Ansi,
            SqlDialect::Mysql => Dialect::Mysql,
            SqlDialect::Postgres => Dialect::Postgres,
            SqlDialect::Sqlite => Dialect::Sqlite,
            SqlDialect::Duckdb => Dialect::DuckDb,
            SqlDialect::MsSQL => Dialect::Tsql,
            SqlDialect::Oracle => Dialect::Oracle,
            SqlDialect::Snowflake => Dialect::Snowflake,
            SqlDialect::BigQuery => Dialect::BigQuery,
            SqlDialect::Redshift => Dialect::Redshift,
        }
    }
}

/// SQL 解析结果
#[derive(Debug, serde::Serialize)]
pub struct ParseResult {
    pub success: bool,
    pub error: Option<String>,
    pub statements_count: usize,
}

/// SQL 格式化请求
#[derive(Debug, serde::Deserialize)]
pub struct FormatRequest {
    pub sql: String,
    pub dialect: Option<SqlDialect>,
}

/// SQL 格式化响应
#[derive(Debug, serde::Serialize)]
pub struct FormatResponse {
    pub formatted_sql: String,
    pub success: bool,
    pub error: Option<String>,
}

/// SQL 转译请求
#[derive(Debug, serde::Deserialize)]
pub struct TranspileRequest {
    pub sql: String,
    pub source_dialect: SqlDialect,
    pub target_dialect: SqlDialect,
}

/// SQL 转译响应
#[derive(Debug, serde::Serialize)]
pub struct TranspileResponse {
    pub transpiled_sql: String,
    pub success: bool,
    pub error: Option<String>,
}

/// SQL 验证请求
#[derive(Debug, serde::Deserialize)]
pub struct ValidateRequest {
    pub sql: String,
    pub dialect: Option<SqlDialect>,
}

/// SQL 验证响应
#[derive(Debug, serde::Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// 解析 SQL
pub fn parse_sql(sql: &str, dialect: Option<SqlDialect>) -> ParseResult {
    let dialect = dialect.unwrap_or(SqlDialect::Generic).to_dialect();

    match parse(sql, dialect) {
        Ok(_statement) => ParseResult {
            success: true,
            error: None,
            statements_count: 1,
        },
        Err(e) => ParseResult {
            success: false,
            error: Some(e.to_string()),
            statements_count: 0,
        },
    }
}

/// 格式化 SQL
pub fn format_sql(sql: &str, dialect: Option<SqlDialect>) -> FormatResponse {
    let dialect = dialect.unwrap_or(SqlDialect::Generic).to_dialect();

    match parse(sql, dialect) {
        Ok(statement) => {
            // 使用 Debug 格式输出
            FormatResponse {
                formatted_sql: format!("{:?}", statement),
                success: true,
                error: None,
            }
        }
        Err(_e) => {
            // 解析失败时返回原始 SQL，不报错（优雅降级）
            // 某些数据库特有语法（如 {} 占位符、变量等）不被 sqlglot-rust 支持
            FormatResponse {
                formatted_sql: sql.to_string(),
                success: true,
                error: None,
            }
        }
    }
}

/// 转译 SQL
pub fn transpile_sql(
    sql: &str,
    source_dialect: SqlDialect,
    target_dialect: SqlDialect,
) -> TranspileResponse {
    let source = source_dialect.to_dialect();
    let target = target_dialect.to_dialect();

    match transpile(sql, source, target) {
        Ok(result) => TranspileResponse {
            transpiled_sql: result,
            success: true,
            error: None,
        },
        Err(e) => TranspileResponse {
            transpiled_sql: sql.to_string(),
            success: false,
            error: Some(e.to_string()),
        },
    }
}

/// 验证 SQL
pub fn validate_sql(sql: &str, dialect: Option<SqlDialect>) -> ValidateResponse {
    let dialect = dialect.unwrap_or(SqlDialect::Generic).to_dialect();

    match parse(sql, dialect) {
        Ok(_) => ValidateResponse {
            valid: true,
            errors: vec![],
            warnings: vec![],
        },
        Err(e) => ValidateResponse {
            valid: false,
            errors: vec![e.to_string()],
            warnings: vec![],
        },
    }
}

/// 分割 SQL 语句
pub fn split_sql(sql: &str, _dialect: Option<SqlDialect>) -> Vec<String> {
    // sqlglot-rust 的 parse 返回单个 Statement，不支持多语句分割
    // 使用简单的分号分割作为主要方案
    sql.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
