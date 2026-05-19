
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::sync::OnceLock;

use super::manifest::*;
use crate::adapters::wasm::{ExtismPluginManager, PluginState};
use crate::core::error::{CommonError, CoreError};

/// 插件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginKind {
    /// WASM 插件
    Wasm,
    /// Go Sidecar 插件
    Sidecar,
}

/// 插件信息
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: PluginKind,
    pub metadata: PluginMetadata,
    pub state: PluginState,
}

/// 插件管理器核心
pub struct PluginManager {
    wasm_manager: Arc&lt;ExtismPluginManager&gt;,
    sidecar_manager: Arc&lt;crate::adapters::sidecar::SidecarManager&gt;,
    plugin_index: Arc&lt;RwLock&lt;HashMap&lt;String, PluginKind&gt;&gt;&gt;,
    plugin_dirs: Vec&lt;PathBuf&gt;,
}

impl Default for PluginManager {
    fn default() -&gt; Self {
        Self::new()
    }
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -&gt; Self {
        Self {
            wasm_manager: Arc::new(ExtismPluginManager::new()),
            sidecar_manager: Arc::new(crate::adapters::sidecar::SidecarManager::new()),
            plugin_index: Arc::new(RwLock::new(HashMap::new())),
            plugin_dirs: Vec::new(),
        }
    }

    /// 添加插件目录
    pub fn add_plugin_dir(&amp;mut self, path: PathBuf) {
        self.plugin_dirs.push(path);
    }

    /// 扫描并发现插件
    pub fn scan_plugins(&amp;self) -&gt; Result&lt;Vec&lt;PluginInfo&gt;, CoreError&gt; {
        let mut discovered_plugins = Vec::new();

        for dir in &amp;self.plugin_dirs {
            if !dir.exists() {
                continue;
            }

            let entries = std::fs::read_dir(dir)
                .map_err(|e| CoreError::common(CommonError::Internal(format!("Failed to read plugin dir: {}", e))))?;

            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(info) = self.try_discover_plugin(&amp;path)? {
                    discovered_plugins.push(info);
                }
            }
        }

        Ok(discovered_plugins)
    }

    /// 尝试发现单个插件
    fn try_discover_plugin(&amp;self, path: &amp;PathBuf) -&gt; Result&lt;Option&lt;PluginInfo&gt;, CoreError&gt; {
        let manifest_path = path.join("plugin.toml");
        
        if !manifest_path.exists() {
            return Ok(None);
        }

        let manifest_str = std::fs::read_to_string(&amp;manifest_path)
            .map_err(|e| CoreError::common(CommonError::Internal(format!("Failed to read manifest: {}", e))))?;

        let manifest: PluginManifest = toml::from_str(&amp;manifest_str)
            .map_err(|e| CoreError::common(CommonError::Internal(format!("Failed to parse manifest: {}", e))))?;

        let kind = self.detect_plugin_kind(&amp;manifest, path)?;
        let state = PluginState::Inactive;

        Ok(Some(PluginInfo {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            kind,
            metadata: manifest.into(),
            state,
        }))
    }

    /// 检测插件类型
    fn detect_plugin_kind(&amp;self, manifest: &amp;PluginManifest, path: &amp;PathBuf) -&gt; Result&lt;PluginKind, CoreError&gt; {
        // 优先看 manifest 指定的类型
        match &amp;manifest.plugin_type {
            PluginType::Wasm =&gt; return Ok(PluginKind::Wasm),
            PluginType::Sidecar =&gt; return Ok(PluginKind::Sidecar),
            _ =&gt; {}
        }

        // 自动检测
        let wasm_files = glob::glob(path.join("*.wasm").to_str().unwrap()).unwrap().count();
        let sidecar_files = glob::glob(path.join("*.wasm").to_str().unwrap()).unwrap().count();
        let has_sidecar_bin = path.join("sidecar").exists() || path.join("sidecar.exe").exists();

        if wasm_files &gt; 0 {
            Ok(PluginKind::Wasm)
        } else if has_sidecar_bin {
            Ok(PluginKind::Sidecar)
        } else {
            Err(CoreError::common(CommonError::Internal(
                format!("Could not detect plugin type for {}", path.display())
            )))
        }
    }

    /// 加载插件
    pub fn load_plugin(&amp;self, id: &amp;str, path: &amp;PathBuf) -&gt; Result&lt;PluginInfo, CoreError&gt; {
        let info = self.try_discover_plugin(path)?
            .ok_or_else(|| CoreError::common(CommonError::Internal("Plugin not found or invalid".to_string())))?;

        match info.kind {
            PluginKind::Wasm =&gt; self.load_wasm_plugin(id, path)?,
            PluginKind::Sidecar =&gt; self.load_sidecar_plugin(id, path)?,
        }

        self.plugin_index
            .write()
            .map_err(|_| CoreError::common(CommonError::Internal("Failed to acquire lock".to_string())))?
            .insert(id.to_string(), info.kind);

        Ok(info)
    }

    /// 加载 WASM 插件
    fn load_wasm_plugin(&amp;self, id: &amp;str, path: &amp;PathBuf) -&gt; Result&lt;(), CoreError&gt; {
        let wasm_path = path.join("plugin.wasm");
        let wasm_bytes = std::fs::read(wasm_path)
            .map_err(|e| CoreError::common(CommonError::Internal(format!("Failed to read wasm: {}", e))))?;

        let manifest_path = path.join("plugin.toml");
        let manifest_str = std::fs::read_to_string(&amp;manifest_path)
            .map_err(|e| CoreError::common(CommonError::Internal(format!("Failed to read manifest: {}", e))))?;

        let manifest: PluginManifest = toml::from_str(&amp;manifest_str)
            .map_err(|e| CoreError::common(CommonError::Internal(format!("Failed to parse manifest: {}", e))))?;

        let config = manifest.config.unwrap_or_default();
        self.wasm_manager.load_plugin(id, &amp;wasm_bytes, Some(config), None)?;
        Ok(())
    }

    /// 加载 Sidecar 插件
    fn load_sidecar_plugin(&amp;self, id: &amp;str, path: &amp;PathBuf) -&gt; Result&lt;(), CoreError&gt; {
        // Sidecar 插件加载逻辑
        self.sidecar_manager.register_plugin(id, path)?;
        Ok(())
    }

    /// 激活插件
    pub fn activate_plugin(&amp;self, id: &amp;str) -&gt; Result&lt;(), CoreError&gt; {
        let index = self.plugin_index
            .read()
            .map_err(|_| CoreError::common(CommonError::Internal("Failed to acquire lock".to_string())))?;

        let kind = index.get(id)
            .ok_or_else(|| CoreError::common(CommonError::Internal(format!("Plugin {} not found", id))))?;

        match kind {
            PluginKind::Wasm =&gt; self.wasm_manager.activate_plugin(id),
            PluginKind::Sidecar =&gt; self.sidecar_manager.activate_plugin(id),
        }
    }

    /// 停用插件
    pub fn deactivate_plugin(&amp;self, id: &amp;str) -&gt; Result&lt;(), CoreError&gt; {
        let index = self.plugin_index
            .read()
            .map_err(|_| CoreError::common(CommonError::Internal("Failed to acquire lock".to_string())))?;

        let kind = index.get(id)
            .ok_or_else(|| CoreError::common(CommonError::Internal(format!("Plugin {} not found", id))))?;

        match kind {
            PluginKind::Wasm =&gt; self.wasm_manager.deactivate_plugin(id),
            PluginKind::Sidecar =&gt; self.sidecar_manager.deactivate_plugin(id),
        }
    }

    /// 卸载插件
    pub fn unload_plugin(&amp;self, id: &amp;str) -&gt; Result&lt;(), CoreError&gt; {
        let index = self.plugin_index
            .read()
            .map_err(|_| CoreError::common(CommonError::Internal("Failed to acquire lock".to_string())))?;

        let kind = index.get(id)
            .ok_or_else(|| CoreError::common(CommonError::Internal(format!("Plugin {} not found", id))))?;

        match kind {
            PluginKind::Wasm =&gt; self.wasm_manager.unload_plugin(id)?,
            PluginKind::Sidecar =&gt; self.sidecar_manager.unload_plugin(id)?,
        }

        self.plugin_index
            .write()
            .map_err(|_| CoreError::common(CommonError::Internal("Failed to acquire lock".to_string())))?
            .remove(id);

        Ok(())
    }

    /// 列出所有已加载插件
    pub fn list_plugins(&amp;self) -&gt; Vec&lt;PluginInfo&gt; {
        let wasm_plugins: Vec&lt;_&gt; = self.wasm_manager.list_plugins()
            .into_iter()
            .map(|(meta, state)| PluginInfo {
                id: meta.id.clone(),
                name: meta.name.clone(),
                version: meta.version.clone(),
                kind: PluginKind::Wasm,
                metadata: meta,
                state,
            })
            .collect();

        // TODO: Add Sidecar plugins
        wasm_plugins
    }

    /// 获取 WASM 管理器引用
    pub fn wasm_manager(&amp;self) -&gt; Arc&lt;ExtismPluginManager&gt; {
        Arc::clone(&amp;self.wasm_manager)
    }

    /// 获取 Sidecar 管理器引用
    pub fn sidecar_manager(&amp;self) -&gt; Arc&lt;crate::adapters::sidecar::SidecarManager&gt; {
        Arc::clone(&amp;self.sidecar_manager)
    }
}

/// 全局插件管理器实例
pub static PLUGIN_MANAGER: OnceLock&lt;Arc&lt;PluginManager&gt;&gt; = OnceLock::new();

/// 获取全局插件管理器实例
///
/// 如果尚未初始化，将返回 None。必须先调用 init_plugin_manager() 进行初始化。
pub fn get_plugin_manager() -&gt; Option&lt;&amp;'static Arc&lt;PluginManager&gt;&gt; {
    PLUGIN_MANAGER.get()
}

/// 初始化全局插件管理器
///
/// 必须在应用启动时调用此函数，且只能调用一次。
pub async fn init_plugin_manager() -&gt; Result&lt;(), CoreError&gt; {
    let manager = Arc::new(PluginManager::new());

    PLUGIN_MANAGER.set(manager).map_err(|_| {
        CoreError::common(crate::core::error::CommonError::General(
            "Plugin manager already initialized".to_string(),
        ))
    })?;

    Ok(())
}
