//! Driver 注册表
//!
//! 提供类似 DBeaver 的驱动管理能力，包括：
//! - 驱动描述符定义
//! - 连接配置模型
//! - 驱动发现和注册

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

use crate::core::connection::ConnectionMethod;

/// 连接配置（统一模型）
///
/// 一个结构支持所有数据库类型，前端根据 driver 字段动态渲染表单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
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
    /// 连接方式（SSL/SSH/Proxy）
    #[serde(default)]
    pub connection_method: ConnectionMethod,
    /// 额外连接选项
    pub options: HashMap<String, String>,
}

impl ConnectionConfig {
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

    /// 转换为数据库连接 URL
    ///
    /// 根据驱动类型生成对应的连接字符串
    pub fn to_url(&self) -> Result<String, String> {
        match self.driver.as_str() {
            "mysql" => self.build_mysql_url(),
            "postgres" => self.build_postgres_url(),
            "sqlite" => self.build_sqlite_url(),
            "duckdb" => self.build_duckdb_url(),
            _ => Err(format!("Unsupported driver: {}", self.driver)),
        }
    }

    /// 构建 MySQL URL
    fn build_mysql_url(&self) -> Result<String, String> {
        let host = self.host.as_ref().ok_or("Host is required for MySQL")?;
        let port = self.port.unwrap_or(3306);
        let username = self.username.as_deref().unwrap_or("root");
        let password = self.password.as_deref().unwrap_or("");

        let mut url = format!("mysql://{}:{}@{}:{}", username, password, host, port);

        if let Some(db) = &self.database {
            url.push('/');
            url.push_str(db);
        }

        // 添加额外选项
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
    fn build_postgres_url(&self) -> Result<String, String> {
        let host = self.host.as_ref().ok_or("Host is required for PostgreSQL")?;
        let port = self.port.unwrap_or(5432);
        let username = self.username.as_deref().unwrap_or("postgres");
        let password = self.password.as_deref().unwrap_or("");
        let database = self.database.as_deref().unwrap_or("postgres");

        let mut url = format!(
            "postgres://{}:{}@{}:{}/{}",
            username, password, host, port, database
        );

        // 添加额外选项
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
    fn build_sqlite_url(&self) -> Result<String, String> {
        let path = self
            .file_path
            .as_ref()
            .ok_or("File path is required for SQLite")?;
        Ok(format!("sqlite://{}", path))
    }

    /// 构建 DuckDB URL
    fn build_duckdb_url(&self) -> Result<String, String> {
        let path = self
            .file_path
            .as_ref()
            .ok_or("File path is required for DuckDB")?;
        Ok(format!("duckdb://{}", path))
    }
}

/// 驱动选项定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverOption {
    /// 选项键
    pub key: String,
    /// 显示名称
    pub label: String,
    /// 默认值
    pub default_value: String,
    /// 选项类型
    pub option_type: DriverOptionType,
    /// 是否必需
    pub required: bool,
    /// 描述
    pub description: Option<String>,
}

impl DriverOption {
    /// 创建新的驱动选项
    pub fn new(key: impl Into<String>, default_value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: String::new(),
            default_value: default_value.into(),
            option_type: DriverOptionType::String,
            required: false,
            description: None,
        }
    }

    /// 设置显示名称
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// 设置选项类型
    pub fn with_type(mut self, option_type: DriverOptionType) -> Self {
        self.option_type = option_type;
        self
    }

    /// 设置是否必需
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// 设置描述
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// 驱动选项类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriverOptionType {
    /// 字符串输入
    String,
    /// 数字输入
    Number,
    /// 布尔开关
    Boolean,
    /// 下拉选择
    Select { options: Vec<String> },
    /// 文件选择
    File,
}

/// 驱动字段定义（用于前端表单渲染）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverField {
    /// 字段键
    pub key: String,
    /// 显示标签
    pub label: String,
    /// 字段类型
    pub field_type: DriverFieldType,
    /// 是否必需
    pub required: bool,
    /// 默认值
    pub default_value: Option<String>,
    /// 占位符文本
    pub placeholder: Option<String>,
}

/// 驱动字段类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriverFieldType {
    /// 文本输入
    Text,
    /// 密码输入
    Password,
    /// 数字输入
    Number,
    /// 文件选择
    File,
    /// 下拉选择
    Select { options: Vec<(String, String)> },
}

/// 驱动描述符（类似 DBeaver 的驱动定义）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverDescriptor {
    /// 驱动 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 驱动描述
    pub description: String,
    /// 默认端口
    pub default_port: Option<u16>,
    /// 是否需要数据库名
    pub require_database: bool,
    /// 是否需要文件路径
    pub require_file: bool,
    /// 是否支持 SSL/TLS
    pub supports_ssl: bool,
    /// 是否支持 SSH 隧道
    pub supports_ssh_tunnel: bool,
    /// 是否支持 HTTP 代理
    pub supports_http_proxy: bool,
    /// 是否支持 SOCKS 代理
    pub supports_socks_proxy: bool,
    /// 表单字段定义
    pub fields: Vec<DriverField>,
    /// 额外选项
    pub extra_options: Vec<DriverOption>,
}

impl DriverDescriptor {
    /// 创建新的驱动描述符
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            default_port: None,
            require_database: false,
            require_file: false,
            supports_ssl: false,
            supports_ssh_tunnel: false,
            supports_http_proxy: false,
            supports_socks_proxy: false,
            fields: Vec::new(),
            extra_options: Vec::new(),
        }
    }

    /// 设置描述
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// 设置默认端口
    pub fn with_default_port(mut self, port: u16) -> Self {
        self.default_port = Some(port);
        self
    }

    /// 设置需要数据库名
    pub fn requires_database(mut self) -> Self {
        self.require_database = true;
        self
    }

    /// 设置需要文件路径
    pub fn requires_file(mut self) -> Self {
        self.require_file = true;
        self
    }

    /// 设置支持 SSL
    pub fn with_ssl_support(mut self) -> Self {
        self.supports_ssl = true;
        self
    }

    /// 设置支持 SSH 隧道
    pub fn with_ssh_tunnel_support(mut self) -> Self {
        self.supports_ssh_tunnel = true;
        self
    }

    /// 设置支持 HTTP 代理
    pub fn with_http_proxy_support(mut self) -> Self {
        self.supports_http_proxy = true;
        self
    }

    /// 设置支持 SOCKS 代理
    pub fn with_socks_proxy_support(mut self) -> Self {
        self.supports_socks_proxy = true;
        self
    }

    /// 添加字段
    pub fn with_field(mut self, field: DriverField) -> Self {
        self.fields.push(field);
        self
    }

    /// 添加额外选项
    pub fn with_extra_option(mut self, option: DriverOption) -> Self {
        self.extra_options.push(option);
        self
    }
}

/// MySQL 驱动描述符
pub fn mysql_driver() -> DriverDescriptor {
    DriverDescriptor::new("mysql", "MySQL")
        .with_description("MySQL 关系型数据库")
        .with_default_port(3306)
        .requires_database()
        .with_ssl_support()
        .with_ssh_tunnel_support()
        .with_http_proxy_support()
        .with_socks_proxy_support()
        .with_field(DriverField {
            key: "host".to_string(),
            label: "主机".to_string(),
            field_type: DriverFieldType::Text,
            required: true,
            default_value: Some("localhost".to_string()),
            placeholder: Some("localhost 或 IP 地址".to_string()),
        })
        .with_field(DriverField {
            key: "port".to_string(),
            label: "端口".to_string(),
            field_type: DriverFieldType::Number,
            required: true,
            default_value: Some("3306".to_string()),
            placeholder: None,
        })
        .with_field(DriverField {
            key: "database".to_string(),
            label: "数据库".to_string(),
            field_type: DriverFieldType::Text,
            required: false,
            default_value: None,
            placeholder: Some("可选，留空显示所有数据库".to_string()),
        })
        .with_field(DriverField {
            key: "username".to_string(),
            label: "用户名".to_string(),
            field_type: DriverFieldType::Text,
            required: true,
            default_value: Some("root".to_string()),
            placeholder: None,
        })
        .with_field(DriverField {
            key: "password".to_string(),
            label: "密码".to_string(),
            field_type: DriverFieldType::Password,
            required: false,
            default_value: None,
            placeholder: Some("可选".to_string()),
        })
        .with_extra_option(
            DriverOption::new("ssl_mode", "PREFERRED")
                .with_label("SSL 模式")
                .with_type(DriverOptionType::Select {
                    options: vec![
                        "DISABLED".to_string(),
                        "PREFERRED".to_string(),
                        "REQUIRED".to_string(),
                        "VERIFY_CA".to_string(),
                        "VERIFY_IDENTITY".to_string(),
                    ],
                })
                .with_description("SSL 连接模式"),
        )
}

/// PostgreSQL 驱动描述符
pub fn postgres_driver() -> DriverDescriptor {
    DriverDescriptor::new("postgres", "PostgreSQL")
        .with_description("PostgreSQL 关系型数据库")
        .with_default_port(5432)
        .requires_database()
        .with_ssl_support()
        .with_ssh_tunnel_support()
        .with_http_proxy_support()
        .with_socks_proxy_support()
        .with_field(DriverField {
            key: "host".to_string(),
            label: "主机".to_string(),
            field_type: DriverFieldType::Text,
            required: true,
            default_value: Some("localhost".to_string()),
            placeholder: Some("localhost 或 IP 地址".to_string()),
        })
        .with_field(DriverField {
            key: "port".to_string(),
            label: "端口".to_string(),
            field_type: DriverFieldType::Number,
            required: true,
            default_value: Some("5432".to_string()),
            placeholder: None,
        })
        .with_field(DriverField {
            key: "database".to_string(),
            label: "数据库".to_string(),
            field_type: DriverFieldType::Text,
            required: true,
            default_value: Some("postgres".to_string()),
            placeholder: None,
        })
        .with_field(DriverField {
            key: "username".to_string(),
            label: "用户名".to_string(),
            field_type: DriverFieldType::Text,
            required: true,
            default_value: Some("postgres".to_string()),
            placeholder: None,
        })
        .with_field(DriverField {
            key: "password".to_string(),
            label: "密码".to_string(),
            field_type: DriverFieldType::Password,
            required: false,
            default_value: None,
            placeholder: Some("可选".to_string()),
        })
        .with_extra_option(
            DriverOption::new("ssl_mode", "prefer")
                .with_label("SSL 模式")
                .with_type(DriverOptionType::Select {
                    options: vec![
                        "disable".to_string(),
                        "allow".to_string(),
                        "prefer".to_string(),
                        "require".to_string(),
                        "verify-ca".to_string(),
                        "verify-full".to_string(),
                    ],
                })
                .with_description("SSL 连接模式"),
        )
}

/// SQLite 驱动描述符
pub fn sqlite_driver() -> DriverDescriptor {
    DriverDescriptor::new("sqlite", "SQLite")
        .with_description("SQLite 嵌入式数据库")
        .requires_file()
        .with_field(DriverField {
            key: "file_path".to_string(),
            label: "数据库文件".to_string(),
            field_type: DriverFieldType::File,
            required: true,
            default_value: None,
            placeholder: Some("选择 .db 或 .sqlite 文件".to_string()),
        })
        .with_extra_option(
            DriverOption::new("mode", "rwc")
                .with_label("打开模式")
                .with_type(DriverOptionType::Select {
                    options: vec![
                        "ro".to_string(),
                        "rw".to_string(),
                        "rwc".to_string(),
                    ],
                })
                .with_description("数据库文件打开模式：只读/读写/读写创建"),
        )
}

/// DuckDB 驱动描述符
pub fn duckdb_driver() -> DriverDescriptor {
    DriverDescriptor::new("duckdb", "DuckDB")
        .with_description("DuckDB 分析型数据库")
        .requires_file()
        .with_field(DriverField {
            key: "file_path".to_string(),
            label: "数据库文件".to_string(),
            field_type: DriverFieldType::File,
            required: true,
            default_value: None,
            placeholder: Some("选择 .duckdb 文件或 :memory:".to_string()),
        })
        .with_extra_option(
            DriverOption::new("memory_limit", "")
                .with_label("内存限制")
                .with_type(DriverOptionType::String)
                .with_description("例如: 1GB, 512MB（留空表示无限制）"),
        )
}

/// 获取所有支持的驱动
pub fn get_all_drivers() -> Vec<DriverDescriptor> {
    DriverRegistry::all_descriptors()
}

/// 根据 ID 获取驱动描述符
pub fn get_driver(id: &str) -> Option<DriverDescriptor> {
    DriverRegistry::get(id).map(|f| f.descriptor())
}

// =============================================================================
// 真正的 Driver Registry（可扩展、可注册）
// =============================================================================

use crate::core::error::CoreError;
use crate::core::driver::traits::Database;

/// 动态数据库类型
pub type DynDatabase = Arc<dyn Database + Send + Sync>;

/// Driver 工厂 Trait
///
/// 每个数据库驱动需要实现此 trait，并在启动时注册到 DriverRegistry
///
/// # 示例
///
/// ```rust
/// pub struct MySqlDriverFactory;
///
/// impl DriverFactory for MySqlDriverFactory {
///     fn descriptor(&self) -> DriverDescriptor {
///         mysql_driver()
///     }
///
///     fn create(&self, config: ConnectionConfig) -> Pin<Box<dyn Future<Output = Result<DynDatabase, CoreError>> + Send>> {
///         Box::pin(async move {
///             let url = config.to_url()?;
///             let db = MySqlDatabase::new(&url).await?;
///             Ok(Arc::new(db))
///         })
///     }
/// }
/// ```
pub trait DriverFactory: Send + Sync {
    /// 获取驱动描述符（用于前端渲染表单）
    fn descriptor(&self) -> DriverDescriptor;

    /// 创建数据库连接
    ///
    /// # Arguments
    ///
    /// * `config` - 连接配置
    ///
    /// # Returns
    ///
    /// 返回动态数据库实例的 Future
    fn create(&self, config: ConnectionConfig) -> Pin<Box<dyn std::future::Future<Output = Result<DynDatabase, CoreError>> + Send>>;
}

use std::pin::Pin;

/// 全局 Driver Registry 存储
static DRIVER_REGISTRY: OnceLock<RwLock<HashMap<String, Arc<dyn DriverFactory>>>> = OnceLock::new();

/// Driver Registry
///
/// 管理所有已注册的驱动工厂，提供驱动发现和创建功能
///
/// # 使用示例
///
/// ```rust
/// // 注册驱动（在 lib.rs 或 main.rs 中）
/// DriverRegistry::register(MySqlDriverFactory);
/// DriverRegistry::register(PostgresDriverFactory);
///
/// // 获取驱动
/// if let Some(factory) = DriverRegistry::get("mysql") {
///     let db = factory.create(config).await?;
/// }
/// ```
pub struct DriverRegistry;

impl DriverRegistry {
    /// 注册驱动工厂
    ///
    /// # Arguments
    ///
    /// * `factory` - 实现 DriverFactory 的驱动工厂实例
    ///
    /// # Example
    ///
    /// ```rust
    /// DriverRegistry::register(MySqlDriverFactory);
    /// ```
    pub fn register<F>(factory: F)
    where
        F: DriverFactory + 'static,
    {
        let registry = DRIVER_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()));
        let descriptor = factory.descriptor();
        let id = descriptor.id.clone();

        if let Ok(mut reg) = registry.write() {
            reg.insert(id, Arc::new(factory));
        }
    }

    /// 根据 ID 获取驱动工厂
    ///
    /// # Arguments
    ///
    /// * `id` - 驱动 ID，如 "mysql", "postgres"
    ///
    /// # Returns
    ///
    /// 返回驱动工厂的 Arc 引用，如果不存在返回 None
    pub fn get(id: &str) -> Option<Arc<dyn DriverFactory>> {
        DRIVER_REGISTRY
            .get()
            .and_then(|registry| {
                registry.read().ok().and_then(|reg| reg.get(id).cloned())
            })
    }

    /// 获取所有已注册驱动的描述符
    ///
    /// # Returns
    ///
    /// 返回所有驱动描述符列表
    pub fn all_descriptors() -> Vec<DriverDescriptor> {
        DRIVER_REGISTRY
            .get()
            .map(|registry| {
                registry
                    .read()
                    .ok()
                    .map(|reg| reg.values().map(|f| f.descriptor()).collect())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    /// 获取所有已注册的驱动 ID
    ///
    /// # Returns
    ///
    /// 返回驱动 ID 列表
    pub fn all_driver_ids() -> Vec<String> {
        DRIVER_REGISTRY
            .get()
            .map(|registry| {
                registry
                    .read()
                    .ok()
                    .map(|reg| reg.keys().cloned().collect())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    /// 检查驱动是否已注册
    ///
    /// # Arguments
    ///
    /// * `id` - 驱动 ID
    ///
    /// # Returns
    ///
    /// 如果驱动已注册返回 true
    pub fn is_registered(id: &str) -> bool {
        DRIVER_REGISTRY
            .get()
            .map(|registry| {
                registry.read().ok().map(|reg| reg.contains_key(id)).unwrap_or(false)
            })
            .unwrap_or(false)
    }

    /// 注销驱动
    ///
    /// # Arguments
    ///
    /// * `id` - 驱动 ID
    ///
    /// # Returns
    ///
    /// 如果成功注销返回 true
    pub fn unregister(id: &str) -> bool {
        DRIVER_REGISTRY
            .get()
            .and_then(|registry| {
                registry.write().ok().map(|mut reg| reg.remove(id).is_some())
            })
            .unwrap_or(false)
    }

    /// 清空所有注册的驱动
    pub fn clear() {
        if let Some(registry) = DRIVER_REGISTRY.get() {
            if let Ok(mut reg) = registry.write() {
                reg.clear();
            }
        }
    }
}
