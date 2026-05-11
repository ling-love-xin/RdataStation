//! Driver 注册表
//!
//! 提供类似 DBeaver 的驱动管理能力，包括：
//! - 驱动工厂 trait（DriverFactory）
//! - 全局注册表（DriverRegistry）
//! - 连接配置模型（config 子模块）
//! - 驱动描述符定义（descriptors 子模块）

pub mod config;
pub mod descriptors;

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, OnceLock, RwLock};

use crate::core::driver::traits::Database;
use crate::core::error::CoreError;

pub use config::DriverConnectionConfig;
pub use descriptors::{
    duckdb_driver, get_all_drivers, get_driver, mysql_driver, postgres_driver, sqlite_driver,
    DriverDescriptor, DriverField, DriverFieldType, DriverKind, DriverOption, DriverOptionType,
};

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
///     fn create(&self, config: DriverConnectionConfig) -> Pin<Box<dyn Future<Output = Result<DynDatabase, CoreError>> + Send>> {
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
    fn create(
        &self,
        config: DriverConnectionConfig,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<DynDatabase, CoreError>> + Send>>;
}

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
            .and_then(|registry| registry.read().ok().and_then(|reg| reg.get(id).cloned()))
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
                registry
                    .read()
                    .ok()
                    .map(|reg| reg.contains_key(id))
                    .unwrap_or(false)
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
                registry
                    .write()
                    .ok()
                    .map(|mut reg| reg.remove(id).is_some())
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

// ========== 测试 ==========
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::driver::auto_register::AutoDriverRegistrar;

    fn ensure_registry_initialized() {
        AutoDriverRegistrar::register_builtin_drivers();
    }

    #[test]
    fn test_get_all_drivers() {
        ensure_registry_initialized();
        let drivers = descriptors::get_all_drivers();
        assert!(!drivers.is_empty());

        let driver_ids: Vec<_> = drivers.iter().map(|d| d.id.as_str()).collect();
        assert!(driver_ids.contains(&"mysql"));
        assert!(driver_ids.contains(&"postgres"));
        assert!(driver_ids.contains(&"sqlite"));
        assert!(driver_ids.contains(&"duckdb"));
    }

    #[test]
    fn test_get_driver_mysql() {
        ensure_registry_initialized();
        let driver = descriptors::get_driver("mysql");
        assert!(driver.is_some());

        let driver = driver.unwrap();
        assert_eq!(driver.id, "mysql");
        assert_eq!(driver.default_port, Some(3306));
        assert!(!driver.require_file);
    }

    #[test]
    fn test_get_driver_postgres() {
        ensure_registry_initialized();
        let driver = descriptors::get_driver("postgres");
        assert!(driver.is_some());

        let driver = driver.unwrap();
        assert_eq!(driver.id, "postgres");
        assert_eq!(driver.default_port, Some(5432));
    }

    #[test]
    fn test_get_driver_sqlite() {
        ensure_registry_initialized();
        let driver = descriptors::get_driver("sqlite");
        assert!(driver.is_some());

        let driver = driver.unwrap();
        assert_eq!(driver.id, "sqlite");
        assert!(driver.require_file);
        assert!(!driver.require_database);
    }

    #[test]
    fn test_get_driver_not_found() {
        ensure_registry_initialized();
        let driver = descriptors::get_driver("oracle");
        assert!(driver.is_none());
    }

    #[test]
    fn test_mysql_driver_fields() {
        ensure_registry_initialized();
        let driver = descriptors::get_driver("mysql").unwrap();

        let field_keys: Vec<_> = driver.fields.iter().map(|f| f.key.as_str()).collect();
        assert!(field_keys.contains(&"host"));
        assert!(field_keys.contains(&"port"));
        assert!(field_keys.contains(&"username"));
        assert!(field_keys.contains(&"password"));
    }

    #[test]
    fn test_driver_descriptor_serialization() {
        ensure_registry_initialized();
        let driver = descriptors::get_driver("mysql").unwrap();

        let json = serde_json::to_string(&driver).unwrap();
        assert!(json.contains("mysql"));
        assert!(json.contains("3306"));

        let deserialized: DriverDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "mysql");
    }
}
