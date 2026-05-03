//! 数据源路由层
//!
//! 职责：数据源注册、路由、能力发现
//! 不存放具体驱动实现，实现统一在 driver/ 中
//!
//! # 架构设计
//!
//! ```
//! 前端请求
//!     │
//!     ▼
//! DataSourceRouter (路由)
//!     │ 根据 driver_id 查找
//!     ▼
//! DriverRegistry (注册表)
//!     │ 返回 DriverFactory
//!     ▼
//! DriverFactory.create() (创建连接)
//!     │
//!     ▼
//! driver/native/mysql.rs (具体实现)
//! ```

use crate::core::driver::{DriverRegistry, DynDatabase};
use crate::core::driver::registry::ConnectionConfig;
use crate::core::error::{CoreError, ConnectionError};

/// 数据源路由器
///
/// 根据驱动 ID 路由到对应的驱动工厂
pub struct DataSourceRouter;

impl DataSourceRouter {
    /// 根据驱动配置创建数据库连接
    ///
    /// # Arguments
    ///
    /// * `config` - 驱动连接配置
    ///
    /// # Returns
    ///
    /// 返回动态数据库实例
    pub async fn route(config: ConnectionConfig) -> Result<DynDatabase, CoreError> {
        let driver_id = &config.driver;

        // 从注册表获取驱动工厂
        let factory = DriverRegistry::get(driver_id)
            .ok_or_else(|| CoreError::connection(ConnectionError::DriverNotFound {
                driver: driver_id.clone(),
            }))?;

        // 使用工厂创建连接
        factory.create(config).await
    }

    /// 检查驱动是否已注册
    pub fn is_driver_registered(driver_id: &str) -> bool {
        DriverRegistry::get(driver_id).is_some()
    }

    /// 获取所有已注册的驱动 ID
    pub fn list_registered_drivers() -> Vec<String> {
        DriverRegistry::all_driver_ids()
    }

    /// 获取驱动描述符
    pub fn get_driver_descriptor(driver_id: &str) -> Option<crate::core::driver::DriverDescriptor> {
        DriverRegistry::get(driver_id).map(|f| f.descriptor())
    }
}
