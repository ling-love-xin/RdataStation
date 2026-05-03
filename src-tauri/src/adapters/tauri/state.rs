/// Tauri 状态管理模块

use std::sync::Arc;

use crate::core::ConnectionManager;

/// 应用状态
pub struct AppState {
    /// 连接管理器
    pub connection_manager: Arc<ConnectionManager>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self {
            connection_manager,
        }
    }
}

/// 状态扩展trait
trait StateExt {
    /// 获取连接管理器
    fn connection_manager(&self) -> &Arc<ConnectionManager>;
}

/// 为tauri::State实现扩展trait
impl<'a> StateExt for tauri::State<'a, AppState> {
    /// 获取连接管理器
    fn connection_manager(&self) -> &Arc<ConnectionManager> {
        &self.inner().connection_manager
    }
}
