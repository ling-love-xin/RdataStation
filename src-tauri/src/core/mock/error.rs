use crate::core::error::{CommonError, CoreError};

pub type MockResult<T> = Result<T, MockError>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum MockError {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("数据生成失败: {0}")]
    Generation(String),

    #[error("无效的行数: {0}")]
    InvalidRowCount(usize),

    #[error("无效的列定义: {0}")]
    InvalidColumn(String),

    #[error("导出失败: format={format}, reason={reason}")]
    Export { format: String, reason: String },

    #[error("模板未找到: {0}")]
    TemplateNotFound(String),

    #[error("预览失败: {0}")]
    Preview(String),
}

impl From<MockError> for CoreError {
    fn from(e: MockError) -> Self {
        CoreError::Common(CommonError::General(e.to_string()))
    }
}

impl From<CoreError> for MockError {
    fn from(e: CoreError) -> Self {
        MockError::Generation(e.to_string())
    }
}

impl From<duckdb::Error> for MockError {
    fn from(e: duckdb::Error) -> Self {
        MockError::Generation(format!("DuckDB error: {}", e))
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, duckdb::Connection>>> for MockError {
    fn from(e: std::sync::PoisonError<std::sync::MutexGuard<'_, duckdb::Connection>>) -> Self {
        MockError::Generation(format!("DuckDB lock error: {}", e))
    }
}