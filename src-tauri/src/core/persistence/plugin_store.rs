
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::core::error::{CoreError, StorageError};

/// 插件信息结构（全局存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub code: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub repo_url: Option<String>,
    pub plugin_type: String,
    pub manifest_json: Option<String>,
    pub install_path: String,
    pub is_enabled: bool,
    pub is_builtin: bool,
    pub installed_at: String,
    pub updated_at: String,
}

/// 插件依赖项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub plugin_id: String,
    pub dep_code: String,
    pub dep_version_range: String,
    pub is_optional: bool,
}

/// 插件全局配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginGlobalConfig {
    pub plugin_id: String,
    pub key: String,
    pub value: Option<String>,
    pub updated_at: String,
}

/// 项目使用的插件（项目级存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUsedPlugin {
    pub plugin_code: String,
    pub plugin_version: String,
    pub enabled: bool,
    pub required: bool,
}

/// 项目插件配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPluginConfig {
    pub plugin_code: String,
    pub plugin_version: String,
    pub key: String,
    pub value: Option<String>,
    pub updated_at: String,
}

fn storage_err(operation: &amp;str, reason: String) -&gt; CoreError {
    CoreError::Storage(StorageError::Persistence {
        store: "plugin_store".to_string(),
        operation: operation.to_string(),
        reason,
    })
}

// ==================== 全局插件存储函数 ====================

/// 注册插件到全局插件中心
pub fn register_plugin(conn: &amp;Connection, plugin: &amp;Plugin) -&gt; Result&lt;(), CoreError&gt; {
    conn.execute(
        "INSERT OR REPLACE INTO plugins 
         (id, code, name, version, author, description, repo_url, plugin_type, manifest_json, install_path, is_enabled, is_builtin, installed_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            plugin.id,
            plugin.code,
            plugin.name,
            plugin.version,
            plugin.author,
            plugin.description,
            plugin.repo_url,
            plugin.plugin_type,
            plugin.manifest_json,
            plugin.install_path,
            plugin.is_enabled as i32,
            plugin.is_builtin as i32,
            plugin.installed_at,
            plugin.updated_at,
        ],
    )
    .map_err(|e| storage_err("register_plugin", e.to_string()))?;

    Ok(())
}

/// 根据 ID 获取插件
pub fn get_plugin(conn: &amp;Connection, id: &amp;str) -&gt; Result&lt;Option&lt;Plugin&gt;, CoreError&gt; {
    let mut stmt = conn
        .prepare(
            "SELECT id, code, name, version, author, description, repo_url, plugin_type, manifest_json, install_path, is_enabled, is_builtin, installed_at, updated_at
             FROM plugins WHERE id = ?1",
        )
        .map_err(|e| storage_err("prepare_get_plugin", e.to_string()))?;

    stmt.query_row(params![id], |row| {
        Ok(Plugin {
            id: row.get(0)?,
            code: row.get(1)?,
            name: row.get(2)?,
            version: row.get(3)?,
            author: row.get(4)?,
            description: row.get(5)?,
            repo_url: row.get(6)?,
            plugin_type: row.get(7)?,
            manifest_json: row.get(8)?,
            install_path: row.get(9)?,
            is_enabled: row.get::&lt;_, i32&gt;(10)? != 0,
            is_builtin: row.get::&lt;_, i32&gt;(11)? != 0,
            installed_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })
    .optional()
    .map_err(|e| storage_err("get_plugin", e.to_string()))
}

/// 根据 code 和 version 获取插件
pub fn get_plugin_by_code_version(
    conn: &amp;Connection,
    code: &amp;str,
    version: &amp;str,
) -&gt; Result&lt;Option&lt;Plugin&gt;, CoreError&gt; {
    let mut stmt = conn
        .prepare(
            "SELECT id, code, name, version, author, description, repo_url, plugin_type, manifest_json, install_path, is_enabled, is_builtin, installed_at, updated_at
             FROM plugins WHERE code = ?1 AND version = ?2",
        )
        .map_err(|e| storage_err("prepare_get_plugin_by_code_version", e.to_string()))?;

    stmt.query_row(params![code, version], |row| {
        Ok(Plugin {
            id: row.get(0)?,
            code: row.get(1)?,
            name: row.get(2)?,
            version: row.get(3)?,
            author: row.get(4)?,
            description: row.get(5)?,
            repo_url: row.get(6)?,
            plugin_type: row.get(7)?,
            manifest_json: row.get(8)?,
            install_path: row.get(9)?,
            is_enabled: row.get::&lt;_, i32&gt;(10)? != 0,
            is_builtin: row.get::&lt;_, i32&gt;(11)? != 0,
            installed_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })
    .optional()
    .map_err(|e| storage_err("get_plugin_by_code_version", e.to_string()))
}

/// 获取所有已安装插件
pub fn get_all_plugins(conn: &amp;Connection) -&gt; Result&lt;Vec&lt;Plugin&gt;, CoreError&gt; {
    let mut stmt = conn
        .prepare(
            "SELECT id, code, name, version, author, description, repo_url, plugin_type, manifest_json, install_path, is_enabled, is_builtin, installed_at, updated_at
             FROM plugins ORDER BY code, updated_at DESC",
        )
        .map_err(|e| storage_err("prepare_get_all_plugins", e.to_string()))?;

    let plugins = stmt
        .query_map([], |row| {
            Ok(Plugin {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                version: row.get(3)?,
                author: row.get(4)?,
                description: row.get(5)?,
                repo_url: row.get(6)?,
                plugin_type: row.get(7)?,
                manifest_json: row.get(8)?,
                install_path: row.get(9)?,
                is_enabled: row.get::&lt;_, i32&gt;(10)? != 0,
                is_builtin: row.get::&lt;_, i32&gt;(11)? != 0,
                installed_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })
        .map_err(|e| storage_err("query_all_plugins", e.to_string()))?;

    plugins
        .collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()
        .map_err(|e| storage_err("collect_plugins", e.to_string()))
}

/// 更新插件启用状态
pub fn update_plugin_enabled(
    conn: &amp;Connection,
    id: &amp;str,
    is_enabled: bool,
) -&gt; Result&lt;(), CoreError&gt; {
    conn.execute(
        "UPDATE plugins SET is_enabled = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![is_enabled as i32, id],
    )
    .map_err(|e| storage_err("update_plugin_enabled", e.to_string()))?;

    Ok(())
}

/// 删除插件
pub fn delete_plugin(conn: &amp;Connection, id: &amp;str) -&gt; Result&lt;(), CoreError&gt; {
    conn.execute("DELETE FROM plugins WHERE id = ?1", params![id])
        .map_err(|e| storage_err("delete_plugin", e.to_string()))?;
    Ok(())
}

/// 注册插件依赖
pub fn register_plugin_dependency(
    conn: &amp;Connection,
    dep: &amp;PluginDependency,
) -&gt; Result&lt;(), CoreError&gt; {
    conn.execute(
        "INSERT OR REPLACE INTO plugin_dependencies 
         (plugin_id, dep_code, dep_version_range, is_optional)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            dep.plugin_id,
            dep.dep_code,
            dep.dep_version_range,
            dep.is_optional as i32,
        ],
    )
    .map_err(|e| storage_err("register_plugin_dependency", e.to_string()))?;

    Ok(())
}

/// 获取插件的所有依赖
pub fn get_plugin_dependencies(
    conn: &amp;Connection,
    plugin_id: &amp;str,
) -&gt; Result&lt;Vec&lt;PluginDependency&gt;, CoreError&gt; {
    let mut stmt = conn
        .prepare(
            "SELECT plugin_id, dep_code, dep_version_range, is_optional
             FROM plugin_dependencies WHERE plugin_id = ?1",
        )
        .map_err(|e| storage_err("prepare_get_plugin_dependencies", e.to_string()))?;

    let deps = stmt
        .query_map(params![plugin_id], |row| {
            Ok(PluginDependency {
                plugin_id: row.get(0)?,
                dep_code: row.get(1)?,
                dep_version_range: row.get(2)?,
                is_optional: row.get::&lt;_, i32&gt;(3)? != 0,
            })
        })
        .map_err(|e| storage_err("query_plugin_dependencies", e.to_string()))?;

    deps.collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()
        .map_err(|e| storage_err("collect_dependencies", e.to_string()))
}

/// 设置插件全局配置
pub fn set_plugin_global_config(
    conn: &amp;Connection,
    config: &amp;PluginGlobalConfig,
) -&gt; Result&lt;(), CoreError&gt; {
    conn.execute(
        "INSERT OR REPLACE INTO plugin_global_config 
         (plugin_id, key, value, updated_at)
         VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
        params![config.plugin_id, config.key, config.value,],
    )
    .map_err(|e| storage_err("set_plugin_global_config", e.to_string()))?;

    Ok(())
}

/// 获取插件全局配置
pub fn get_plugin_global_configs(
    conn: &amp;Connection,
    plugin_id: &amp;str,
) -&gt; Result&lt;Vec&lt;PluginGlobalConfig&gt;, CoreError&gt; {
    let mut stmt = conn
        .prepare(
            "SELECT plugin_id, key, value, updated_at
             FROM plugin_global_config WHERE plugin_id = ?1",
        )
        .map_err(|e| storage_err("prepare_get_plugin_global_configs", e.to_string()))?;

    let configs = stmt
        .query_map(params![plugin_id], |row| {
            Ok(PluginGlobalConfig {
                plugin_id: row.get(0)?,
                key: row.get(1)?,
                value: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| storage_err("query_plugin_global_configs", e.to_string()))?;

    configs
        .collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()
        .map_err(|e| storage_err("collect_global_configs", e.to_string()))
}

// ==================== 项目级插件存储函数 ====================

/// 添加插件到项目（启用）
pub fn project_add_plugin(
    conn: &amp;Connection,
    used_plugin: &amp;ProjectUsedPlugin,
) -&gt; Result&lt;(), CoreError&gt; {
    conn.execute(
        "INSERT OR REPLACE INTO project_used_plugins 
         (plugin_code, plugin_version, enabled, required)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            used_plugin.plugin_code,
            used_plugin.plugin_version,
            used_plugin.enabled as i32,
            used_plugin.required as i32,
        ],
    )
    .map_err(|e| storage_err("project_add_plugin", e.to_string()))?;

    Ok(())
}

/// 从项目移除插件
pub fn project_remove_plugin(conn: &amp;Connection, code: &amp;str, version: &amp;str) -&gt; Result&lt;(), CoreError&gt; {
    conn.execute(
        "DELETE FROM project_used_plugins WHERE plugin_code = ?1 AND plugin_version = ?2",
        params![code, version],
    )
    .map_err(|e| storage_err("project_remove_plugin", e.to_string()))?;
    Ok(())
}

/// 获取项目使用的所有插件
pub fn project_get_plugins(conn: &amp;Connection) -&gt; Result&lt;Vec&lt;ProjectUsedPlugin&gt;, CoreError&gt; {
    let mut stmt = conn
        .prepare(
            "SELECT plugin_code, plugin_version, enabled, required
             FROM project_used_plugins ORDER BY plugin_code",
        )
        .map_err(|e| storage_err("prepare_project_get_plugins", e.to_string()))?;

    let plugins = stmt
        .query_map([], |row| {
            Ok(ProjectUsedPlugin {
                plugin_code: row.get(0)?,
                plugin_version: row.get(1)?,
                enabled: row.get::&lt;_, i32&gt;(2)? != 0,
                required: row.get::&lt;_, i32&gt;(3)? != 0,
            })
        })
        .map_err(|e| storage_err("query_project_plugins", e.to_string()))?;

    plugins
        .collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()
        .map_err(|e| storage_err("collect_project_plugins", e.to_string()))
}

/// 更新项目插件启用状态
pub fn project_update_plugin_enabled(
    conn: &amp;Connection,
    code: &amp;str,
    version: &amp;str,
    enabled: bool,
) -&gt; Result&lt;(), CoreError&gt; {
    conn.execute(
        "UPDATE project_used_plugins SET enabled = ?1 WHERE plugin_code = ?2 AND plugin_version = ?3",
        params![enabled as i32, code, version],
    )
    .map_err(|e| storage_err("project_update_plugin_enabled", e.to_string()))?;

    Ok(())
}

/// 设置项目插件配置
pub fn project_set_plugin_config(
    conn: &amp;Connection,
    config: &amp;ProjectPluginConfig,
) -&gt; Result&lt;(), CoreError&gt; {
    conn.execute(
        "INSERT OR REPLACE INTO project_plugin_config 
         (plugin_code, plugin_version, key, value, updated_at)
         VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
        params![
            config.plugin_code,
            config.plugin_version,
            config.key,
            config.value,
        ],
    )
    .map_err(|e| storage_err("project_set_plugin_config", e.to_string()))?;

    Ok(())
}

/// 获取项目插件配置
pub fn project_get_plugin_configs(
    conn: &amp;Connection,
    code: &amp;str,
    version: &amp;str,
) -&gt; Result&lt;Vec&lt;ProjectPluginConfig&gt;, CoreError&gt; {
    let mut stmt = conn
        .prepare(
            "SELECT plugin_code, plugin_version, key, value, updated_at
             FROM project_plugin_config WHERE plugin_code = ?1 AND plugin_version = ?2",
        )
        .map_err(|e| storage_err("prepare_project_get_plugin_configs", e.to_string()))?;

    let configs = stmt
        .query_map(params![code, version], |row| {
            Ok(ProjectPluginConfig {
                plugin_code: row.get(0)?,
                plugin_version: row.get(1)?,
                key: row.get(2)?,
                value: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| storage_err("query_project_plugin_configs", e.to_string()))?;

    configs
        .collect::&lt;Result&lt;Vec&lt;_&gt;, _&gt;&gt;()
        .map_err(|e| storage_err("collect_project_configs", e.to_string()))
}
