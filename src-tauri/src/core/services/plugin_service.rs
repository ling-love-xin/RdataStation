
//! Plugin Service - 插件管理服务
//!
//! 提供插件的安装、卸载、启用、禁用，以及项目级插件管理等功能。
//! 与 DriverService 风格一致，通过 GlobalDatabaseManager 处理全局存储，
//! 通过 ProjectDatabaseManager 处理项目级存储。

use crate::core::error::{CoreError, PluginError};
use crate::core::persistence::global_db::GlobalDatabaseManager;
use crate::core::persistence::plugin_store::{
    Plugin, PluginGlobalConfig, ProjectPluginConfig, ProjectUsedPlugin,
};
use crate::core::persistence::project_connection_store::ProjectConnectionStore;
use crate::core::plugin::loader::get_plugin_loader;
use crate::core::plugin::events::*;

/// 插件状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PluginStatus {
    /// 未安装
    NotInstalled,
    /// 已安装但未启用
    Installed,
    /// 已启用
    Enabled,
    /// 在项目中已启用
    ProjectEnabled,
}

/// 插件信息（包含状态）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginWithStatus {
    pub plugin: Plugin,
    pub status: PluginStatus,
}

/// 插件服务
pub struct PluginService {
    global_db: &amp;'static GlobalDatabaseManager,
}

impl PluginService {
    /// 创建插件服务实例
    pub fn new(global_db: &amp;'static GlobalDatabaseManager) -&gt; Self {
        Self { global_db }
    }

    /// 获取所有已安装的插件
    pub async fn get_installed_plugins(&amp;self) -&gt; Result&lt;Vec&lt;Plugin&gt;, CoreError&gt; {
        self.global_db.get_all_plugins().await
    }

    /// 获取带有状态的所有插件
    pub async fn get_plugins_with_status(
        &amp;self,
        project_store: Option&lt;&amp;ProjectConnectionStore&gt;,
    ) -&gt; Result&lt;Vec&lt;PluginWithStatus&gt;, CoreError&gt; {
        let plugins = self.get_installed_plugins().await?;
        
        let mut result = Vec::with_capacity(plugins.len());
        
        for plugin in plugins {
            let status = if let Some(store) = project_store {
                let project_plugins = store.project_get_plugins().await?;
                let found = project_plugins.iter().find(|p| {
                    p.plugin_code == plugin.code &amp;&amp; p.plugin_version == plugin.version
                });
                
                if let Some(p) = found {
                    if p.enabled {
                        PluginStatus::ProjectEnabled
                    } else {
                        PluginStatus::Installed
                    }
                } else if plugin.is_enabled {
                    PluginStatus::Enabled
                } else {
                    PluginStatus::Installed
                }
            } else if plugin.is_enabled {
                PluginStatus::Enabled
            } else {
                PluginStatus::Installed
            };

            result.push(PluginWithStatus { plugin, status });
        }

        Ok(result)
    }

    /// 获取单个插件
    pub async fn get_plugin(&amp;self, plugin_id: &amp;str) -&gt; Result&lt;Option&lt;Plugin&gt;, CoreError&gt; {
        self.global_db.get_plugin(plugin_id).await
    }

    /// 通过 code 和 version 获取插件
    pub async fn get_plugin_by_code_version(
        &amp;self,
        code: &amp;str,
        version: &amp;str,
    ) -&gt; Result&lt;Option&lt;Plugin&gt;, CoreError&gt; {
        self.global_db.get_plugin_by_code_version(code, version).await
    }

    /// 安装新插件
    pub async fn install_plugin(
        &amp;self,
        code: String,
        name: String,
        version: String,
        author: Option&lt;String&gt;,
        description: Option&lt;String&gt;,
        repo_url: Option&lt;String&gt;,
        plugin_type: String,
        manifest_json: Option&lt;String&gt;,
        install_path: String,
        is_builtin: Option&lt;bool&gt;,
    ) -&gt; Result&lt;Plugin, CoreError&gt; {
        // 检查是否已存在
        let existing = self.get_plugin_by_code_version(&amp;code, &amp;version).await?;
        if existing.is_some() {
            return Err(CoreError::plugin(PluginError::already_exists(code.clone(), version.clone())));
        }

        let now = chrono::Utc::now().to_rfc3339();
        let plugin = Plugin {
            id: uuid::Uuid::new_v4().to_string(),
            code: code.clone(),
            name,
            version: version.clone(),
            author,
            description,
            repo_url,
            plugin_type,
            manifest_json,
            install_path,
            is_enabled: true,
            is_builtin: is_builtin.unwrap_or(false),
            installed_at: now.clone(),
            updated_at: now,
        };

        self.global_db.register_plugin(&amp;plugin).await?;

        // 发布插件已安装事件
        emit_plugin_installed(&amp;plugin.id, &amp;code, &amp;version);

        Ok(plugin)
    }

    /// 全局启用插件
    pub async fn enable_plugin(&amp;self, plugin_id: &amp;str) -&gt; Result&lt;(), CoreError&gt; {
        // 检查插件存在
        let plugin = self.get_plugin(plugin_id).await?;
        if plugin.is_none() {
            return Err(CoreError::plugin(PluginError::not_found(plugin_id.to_string())));
        }

        self.global_db.update_plugin_enabled(plugin_id, true).await?;
        emit_plugin_enabled(plugin_id);

        // 尝试激活插件
        let loader = get_plugin_loader();
        let _ = loader.activate_plugin(plugin_id).await;

        Ok(())
    }

    /// 全局禁用插件
    pub async fn disable_plugin(&amp;self, plugin_id: &amp;str) -&gt; Result&lt;(), CoreError&gt; {
        let plugin = self.get_plugin(plugin_id).await?;
        if plugin.is_none() {
            return Err(CoreError::plugin(PluginError::not_found(plugin_id.to_string())));
        }

        // 先停用插件
        let loader = get_plugin_loader();
        let _ = loader.deactivate_plugin(plugin_id).await;

        self.global_db.update_plugin_enabled(plugin_id, false).await?;
        emit_plugin_disabled(plugin_id);

        Ok(())
    }

    /// 卸载插件
    pub async fn uninstall_plugin(&amp;self, plugin_id: &amp;str) -&gt; Result&lt;(), CoreError&gt; {
        let plugin = self.get_plugin(plugin_id).await?;
        if plugin.is_none() {
            return Err(CoreError::plugin(PluginError::not_found(plugin_id.to_string())));
        }

        // 先停用和卸载
        let loader = get_plugin_loader();
        let _ = loader.deactivate_plugin(plugin_id).await;
        let _ = loader.unload_plugin(plugin_id).await;

        self.global_db.delete_plugin(plugin_id).await?;
        emit_plugin_uninstalled(plugin_id);

        Ok(())
    }

    // ==================== 项目级插件管理 ====================

    /// 在项目中启用插件
    pub async fn enable_plugin_in_project(
        &amp;self,
        project_store: &amp;ProjectConnectionStore,
        plugin_code: String,
        plugin_version: String,
        required: Option&lt;bool&gt;,
    ) -&gt; Result&lt;(), CoreError&gt; {
        // 检查全局是否有这个插件
        let global_plugin = self.get_plugin_by_code_version(&amp;plugin_code, &amp;plugin_version).await?;
        
        if global_plugin.is_none() {
            return Err(CoreError::plugin(PluginError::not_found_by_code(
                plugin_code,
                plugin_version,
            )));
        }

        let used_plugin = ProjectUsedPlugin {
            plugin_code,
            plugin_version,
            enabled: true,
            required: required.unwrap_or(false),
        };

        project_store.project_add_plugin(&amp;used_plugin).await?;
        Ok(())
    }

    /// 在项目中禁用插件
    pub async fn disable_plugin_in_project(
        &amp;self,
        project_store: &amp;ProjectConnectionStore,
        plugin_code: String,
        plugin_version: String,
    ) -&gt; Result&lt;(), CoreError&gt; {
        project_store
            .project_update_plugin_enabled(&amp;plugin_code, &amp;plugin_version, false)
            .await?;
        Ok(())
    }

    /// 从项目中移除插件
    pub async fn remove_plugin_from_project(
        &amp;self,
        project_store: &amp;ProjectConnectionStore,
        plugin_code: String,
        plugin_version: String,
    ) -&gt; Result&lt;(), CoreError&gt; {
        project_store
            .project_remove_plugin(&amp;plugin_code, &amp;plugin_version)
            .await?;
        Ok(())
    }

    /// 获取项目使用的所有插件
    pub async fn get_project_plugins(
        &amp;self,
        project_store: &amp;ProjectConnectionStore,
    ) -&gt; Result&lt;Vec&lt;ProjectUsedPlugin&gt;, CoreError&gt; {
        project_store.project_get_plugins().await
    }

    /// 设置项目插件配置
    pub async fn set_project_plugin_config(
        &amp;self,
        project_store: &amp;ProjectConnectionStore,
        plugin_code: String,
        plugin_version: String,
        key: String,
        value: Option&lt;String&gt;,
    ) -&gt; Result&lt;(), CoreError&gt; {
        let config = ProjectPluginConfig {
            plugin_code,
            plugin_version,
            key,
            value,
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        project_store.project_set_plugin_config(&amp;config).await?;
        Ok(())
    }

    /// 获取项目插件配置
    pub async fn get_project_plugin_configs(
        &amp;self,
        project_store: &amp;ProjectConnectionStore,
        plugin_code: String,
        plugin_version: String,
    ) -&gt; Result&lt;Vec&lt;ProjectPluginConfig&gt;, CoreError&gt; {
        project_store
            .project_get_plugin_configs(&amp;plugin_code, &amp;plugin_version)
            .await
    }

    // ==================== 全局配置 ====================

    /// 设置插件全局配置
    pub async fn set_global_config(
        &amp;self,
        plugin_id: String,
        key: String,
        value: Option&lt;String&gt;,
    ) -&gt; Result&lt;(), CoreError&gt; {
        let config = PluginGlobalConfig {
            plugin_id,
            key,
            value,
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        self.global_db.set_plugin_global_config(&amp;config).await?;
        Ok(())
    }

    /// 获取插件全局配置
    pub async fn get_global_configs(
        &amp;self,
        plugin_id: &amp;str,
    ) -&gt; Result&lt;Vec&lt;PluginGlobalConfig&gt;, CoreError&gt; {
        self.global_db.get_plugin_global_configs(plugin_id).await
    }

    // ==================== 启动流程 ====================

    /// 应用启动时加载所有启用的插件
    pub async fn load_enabled_plugins_on_startup(&amp;self) -&gt; Result&lt;Vec&lt;Plugin&gt;, CoreError&gt; {
        let all_plugins = self.get_installed_plugins().await?;
        let enabled_plugins = all_plugins
            .into_iter()
            .filter(|p| p.is_enabled)
            .collect::&lt;Vec&lt;_&gt;&gt;();

        // 扫描并加载插件到 PluginManager
        let loader = get_plugin_loader();
        let loaded = loader.scan_and_load_plugins().await;

        tracing::info!(
            "Loaded {} enabled plugins on startup",
            enabled_plugins.len()
        );

        // 尝试激活已加载的插件
        for loaded_plugin in loaded.unwrap_or_default() {
            let _ = loader.activate_plugin(&amp;loaded_plugin.id).await;
        }

        Ok(enabled_plugins)
    }

    /// 项目打开时加载项目指定的插件
    pub async fn load_project_plugins_on_open(
        &amp;self,
        project_store: &amp;ProjectConnectionStore,
    ) -&gt; Result&lt;Vec&lt;Plugin&gt;, CoreError&gt; {
        let project_plugins = self.get_project_plugins(project_store).await?;
        let mut result = Vec::with_capacity(project_plugins.len());

        let loader = get_plugin_loader();

        for used_plugin in project_plugins {
            if used_plugin.enabled {
                if let Some(plugin) = self
                    .get_plugin_by_code_version(&amp;used_plugin.plugin_code, &amp;used_plugin.plugin_version)
                    .await?
                {
                    result.push(plugin.clone());

                    // 尝试加载并激活插件
                    let install_path = std::path::PathBuf::from(&amp;plugin.install_path);
                    if install_path.exists() {
                        if let Ok(loaded) = loader.load_plugin_from_dir(&amp;install_path).await {
                            let _ = loader.activate_plugin(&amp;loaded.id).await;
                        }
                    }
                }
            }
        }

        tracing::info!(
            "Loaded {} project plugins on open",
            result.len()
        );

        Ok(result)
    }
}

