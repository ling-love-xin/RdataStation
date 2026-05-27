//! 连接配置模型
//!
//! 一个结构支持所有数据库类型，前端根据 driver 字段动态渲染表单。

use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;

use crate::core::driver::connection::config::ConnectionMethod;
use crate::core::error::{CommonError, CoreError};

/// 连接配置（统一模型）
///
/// 一个结构支持所有数据库类型，前端根据 driver 字段动态渲染表单
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DriverConnectionConfig {
    /// 驱动类型: "mysql" | "postgres" | "sqlite" | "duckdb"
    pub driver: String,
    /// 连接名称（显示用）
    pub name: Option<String>,
    /// 主机地址（网络数据库）
    pub host: Option<String>,
    /// 端口（网络数据库）
    pub port: Option<u16>,
    /// 数据库名
    pub database: Option<String>,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 文件路径（SQLite/DuckDB 等文件型数据库）
    pub file_path: Option<String>,
    /// 连接方式覆盖（避免重新构建 URL）
    pub url_override: Option<String>,
    /// 连接方式（SSL/SSH/Proxy）
    #[serde(default)]
    pub connection_method: ConnectionMethod,
    /// 额外连接选项
    pub options: HashMap<String, String>,
}

impl DriverConnectionConfig {
    /// 创建新的连接配置
    pub fn new(driver: impl Into<String>) -> Self {
        Self {
            driver: driver.into(),
            name: None,
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
            file_path: None,
            url_override: None,
            connection_method: ConnectionMethod::Direct,
            options: HashMap::new(),
        }
    }

    /// 设置连接名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 设置主机
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// 设置端口
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// 设置数据库
    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    /// 设置用户名
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// 设置密码
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// 设置文件路径
    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// 添加额外选项
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// 设置连接方式（SSL/SSH/Proxy）
    pub fn with_connection_method(mut self, method: ConnectionMethod) -> Self {
        self.connection_method = method;
        self
    }

    /// 设置预构建的 URL（绕过 to_url 自动构建）
    pub fn with_url_override(mut self, url: impl Into<String>) -> Self {
        self.url_override = Some(url.into());
        self
    }

    /// 转换为数据库连接 URL
    ///
    /// 如果设置了 url_override，直接返回；
    /// 否则根据驱动类型生成对应的连接字符串
    pub fn to_url(&self) -> Result<String, CoreError> {
        if let Some(ref url) = self.url_override {
            return Ok(url.clone());
        }
        match self.driver.as_str() {
            "mysql" | "mysql_native" => self.build_mysql_url(),
            "postgres" | "postgres_native" => self.build_postgres_url(),
            "sqlite" => self.build_sqlite_url(),
            "duckdb" => self.build_duckdb_url(),
            _ => Err(CoreError::common(CommonError::Internal(format!(
                "Unsupported driver: {}",
                self.driver
            )))),
        }
    }

    fn build_mysql_url(&self) -> Result<String, CoreError> {
        let host = self.host.as_ref().ok_or("Host is required for MySQL")?;
        let port = self.port.unwrap_or(3306);
        let username = self.username.as_deref().unwrap_or("root");
        let password = self.password.as_deref().unwrap_or("");

        let mut url = format!("mysql://{}:{}@{}:{}", username, password, host, port);

        if let Some(db) = &self.database {
            url.push('/');
            url.push_str(db);
        }

        if !self.options.is_empty() {
            url.push('?');
            let params: Vec<String> = self
                .options
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push_str(&params.join("&"));
        }

        Ok(url)
    }

    /// 构建 PostgreSQL URL
    fn build_postgres_url(&self) -> Result<String, CoreError> {
        let host = self
            .host
            .as_ref()
            .ok_or("Host is required for PostgreSQL")?;
        let port = self.port.unwrap_or(5432);
        let username = self.username.as_deref().unwrap_or("postgres");
        let password = self.password.as_deref().unwrap_or("");
        let database = self.database.as_deref().unwrap_or("postgres");

        let mut url = format!(
            "postgres://{}:{}@{}:{}/{}",
            username, password, host, port, database
        );

        if !self.options.is_empty() {
            url.push('?');
            let params: Vec<String> = self
                .options
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push_str(&params.join("&"));
        }

        Ok(url)
    }

    /// 构建 SQLite URL
    fn build_sqlite_url(&self) -> Result<String, CoreError> {
        let path = self
            .file_path
            .as_ref()
            .ok_or("File path is required for SQLite")?;
        Ok(format!("sqlite://{}", path))
    }

    /// 构建 DuckDB URL
    fn build_duckdb_url(&self) -> Result<String, CoreError> {
        let path = self
            .file_path
            .as_ref()
            .ok_or("File path is required for DuckDB")?;
        Ok(format!("duckdb://{}", path))
    }
}
