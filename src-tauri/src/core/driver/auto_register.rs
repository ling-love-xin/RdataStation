//! 驱动自动注册模块
//!
//! 提供从配置文件或自动扫描注册驱动的功能

use crate::core::driver::{
    DriverRegistry, DuckDbDriverFactory, MySqlDriverFactory, PostgresDriverFactory,
    SqliteDriverFactory,
};

/// 驱动自动注册器
///
/// 支持多种注册方式：
/// 1. 内置驱动自动注册
/// 2. 配置文件驱动注册
/// 3. 自动扫描驱动注册
pub struct AutoDriverRegistrar;

impl AutoDriverRegistrar {
    /// 注册所有内置驱动
    ///
    /// 在应用启动时调用，自动发现和注册所有可用的内置驱动
    pub fn register_builtin_drivers() {
        // 注册 MySQL 驱动
        DriverRegistry::register(MySqlDriverFactory);

        // 注册 PostgreSQL 驱动
        DriverRegistry::register(PostgresDriverFactory);

        // 注册 SQLite 驱动
        DriverRegistry::register(SqliteDriverFactory);

        // 注册 DuckDB 驱动
        DriverRegistry::register(DuckDbDriverFactory);

        #[cfg(debug_assertions)]
        tracing::debug!(
            "Builtin drivers registered: {:?}",
            DriverRegistry::all_driver_ids()
        );
    }

    /// 自动注册所有驱动
    ///
    /// 注册所有内置驱动
    pub fn auto_register() {
        // 注册内置驱动
        Self::register_builtin_drivers();

        #[cfg(debug_assertions)]
        tracing::debug!(
            "Auto registration complete. Registered drivers: {:?}",
            DriverRegistry::all_driver_ids()
        );
    }
}

/// 驱动注册宏
///
/// 简化驱动注册代码
#[macro_export]
macro_rules! register_driver {
    ($factory:ty) => {
        $crate::core::driver::DriverRegistry::register($factory);
    };
}

/// 批量注册驱动宏
#[macro_export]
macro_rules! register_drivers {
    ($($factory:ty),+ $(,)?) => {
        $(
            $crate::core::driver::DriverRegistry::register($factory);
        )+
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_register() {
        // 确保测试时不会 panic
        AutoDriverRegistrar::register_builtin_drivers();

        let drivers = DriverRegistry::all_driver_ids();
        assert!(!drivers.is_empty());
    }
}
