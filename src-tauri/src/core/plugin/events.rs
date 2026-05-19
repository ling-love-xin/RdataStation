
//! 插件事件系统
//!
//! 提供插件生命周期事件的发布和订阅机制

use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// 插件事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum PluginEvent {
    /// 插件已加载
    PluginLoaded {
        plugin_id: String,
    },
    /// 插件已激活
    PluginActivated {
        plugin_id: String,
    },
    /// 插件已停用
    PluginDeactivated {
        plugin_id: String,
    },
    /// 插件已卸载
    PluginUnloaded {
        plugin_id: String,
    },
    /// 插件已安装
    PluginInstalled {
        plugin_id: String,
        code: String,
        version: String,
    },
    /// 插件已卸载（从系统中删除）
    PluginUninstalled {
        plugin_id: String,
    },
    /// 插件已启用
    PluginEnabled {
        plugin_id: String,
    },
    /// 插件已禁用
    PluginDisabled {
        plugin_id: String,
    },
    /// 自定义事件
    Custom {
        plugin_id: String,
        event_name: String,
        payload: serde_json::Value,
    },
}

/// 事件管理器
pub struct PluginEventManager {
    sender: broadcast::Sender&lt;PluginEvent&gt;,
}

impl PluginEventManager {
    /// 创建新的事件管理器
    pub fn new(capacity: usize) -&gt; Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// 发布事件
    pub fn emit(&amp;self, event: PluginEvent) {
        let _ = self.sender.send(event);
    }

    /// 订阅事件
    pub fn subscribe(&amp;self) -&gt; broadcast::Receiver&lt;PluginEvent&gt; {
        self.sender.subscribe()
    }
}

impl Default for PluginEventManager {
    fn default() -&gt; Self {
        Self::new(100)
    }
}

/// 全局事件管理器
static EVENT_MANAGER: std::sync::OnceLock&lt;Arc&lt;PluginEventManager&gt;&gt; = std::sync::OnceLock::new();

/// 获取全局事件管理器
pub fn get_event_manager() -&gt; Arc&lt;PluginEventManager&gt; {
    EVENT_MANAGER.get_or_init(|| Arc::new(PluginEventManager::default())).clone()
}

/// 初始化事件管理器
pub fn init_event_manager() -&gt; Arc&lt;PluginEventManager&gt; {
    get_event_manager()
}

/// 发布插件已加载事件
pub fn emit_plugin_loaded(plugin_id: &amp;str) {
    get_event_manager().emit(PluginEvent::PluginLoaded {
        plugin_id: plugin_id.to_string(),
    });
}

/// 发布插件已激活事件
pub fn emit_plugin_activated(plugin_id: &amp;str) {
    get_event_manager().emit(PluginEvent::PluginActivated {
        plugin_id: plugin_id.to_string(),
    });
}

/// 发布插件已停用事件
pub fn emit_plugin_deactivated(plugin_id: &amp;str) {
    get_event_manager().emit(PluginEvent::PluginDeactivated {
        plugin_id: plugin_id.to_string(),
    });
}

/// 发布插件已卸载事件
pub fn emit_plugin_unloaded(plugin_id: &amp;str) {
    get_event_manager().emit(PluginEvent::PluginUnloaded {
        plugin_id: plugin_id.to_string(),
    });
}

/// 发布插件已安装事件
pub fn emit_plugin_installed(plugin_id: &amp;str, code: &amp;str, version: &amp;str) {
    get_event_manager().emit(PluginEvent::PluginInstalled {
        plugin_id: plugin_id.to_string(),
        code: code.to_string(),
        version: version.to_string(),
    });
}

/// 发布插件已卸载（从系统删除）事件
pub fn emit_plugin_uninstalled(plugin_id: &amp;str) {
    get_event_manager().emit(PluginEvent::PluginUninstalled {
        plugin_id: plugin_id.to_string(),
    });
}

/// 发布插件已启用事件
pub fn emit_plugin_enabled(plugin_id: &amp;str) {
    get_event_manager().emit(PluginEvent::PluginEnabled {
        plugin_id: plugin_id.to_string(),
    });
}

/// 发布插件已禁用事件
pub fn emit_plugin_disabled(plugin_id: &amp;str) {
    get_event_manager().emit(PluginEvent::PluginDisabled {
        plugin_id: plugin_id.to_string(),
    });
}

/// 发布自定义事件
pub fn emit_custom_event(plugin_id: &amp;str, event_name: &amp;str, payload: serde_json::Value) {
    get_event_manager().emit(PluginEvent::Custom {
        plugin_id: plugin_id.to_string(),
        event_name: event_name.to_string(),
        payload,
    });
}
