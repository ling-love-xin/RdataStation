use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::core::error::{CoreError, StorageError};

/// 认证配置（用户名/密码、密钥文件、Kerberos、OAuth2 等），密码字段已 AES-256-GCM 加密
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub id: String,
    pub name: Option<String>,
    pub auth_type: String,
    pub auth_data: String,
    pub origin: Option<String>,
    pub source_id: Option<String>,
    pub snapshot_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn storage_err(op: &str, reason: String) -> CoreError {
    CoreError::storage(StorageError::Persistence {
        store: "auth_store".to_string(),
        operation: op.to_string(),
        reason,
    })
}

/// 创建认证配置
pub fn create_auth_config(conn: &Connection, ac: &AuthConfig) -> Result<(), CoreError> {
    conn.execute(
        "INSERT INTO auth_configs (id, name, auth_type, auth_data, origin, source_id, snapshot_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            ac.id,
            ac.name,
            ac.auth_type,
            ac.auth_data,
            ac.origin,
            ac.source_id,
            ac.snapshot_at,
            ac.created_at,
            ac.updated_at
        ],
    )
    .map_err(|e| storage_err("create_auth_config", e.to_string()))?;
    Ok(())
}

/// 列出认证配置，可按认证类型过滤
pub fn list_auth_configs(
    conn: &Connection,
    auth_type: Option<&str>,
) -> Result<Vec<AuthConfig>, CoreError> {
    let (sql, param): (String, Option<String>) = if let Some(t) = auth_type {
        (
            "SELECT id, name, auth_type, auth_data, origin, source_id, snapshot_at, created_at, updated_at
             FROM auth_configs WHERE auth_type = ?1 ORDER BY name"
                .to_string(),
            Some(t.to_string()),
        )
    } else {
        (
            "SELECT id, name, auth_type, auth_data, origin, source_id, snapshot_at, created_at, updated_at
             FROM auth_configs ORDER BY auth_type, name"
                .to_string(),
            None,
        )
    };

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| storage_err("prepare_list_auth_configs", e.to_string()))?;

    let items = if let Some(ref p) = param {
        stmt.query_map(params![p], |row| {
            Ok(AuthConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                auth_type: row.get(2)?,
                auth_data: row.get(3)?,
                origin: row.get(4)?,
                source_id: row.get(5)?,
                snapshot_at: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| storage_err("query_auth_configs", e.to_string()))?
        .filter_map(|r| r.ok())
        .collect()
    } else {
        stmt.query_map([], |row| {
            Ok(AuthConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                auth_type: row.get(2)?,
                auth_data: row.get(3)?,
                origin: row.get(4)?,
                source_id: row.get(5)?,
                snapshot_at: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| storage_err("query_auth_configs", e.to_string()))?
        .filter_map(|r| r.ok())
        .collect()
    };

    Ok(items)
}

/// 根据 ID 获取认证配置
pub fn get_auth_config(conn: &Connection, id: &str) -> Result<Option<AuthConfig>, CoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, auth_type, auth_data, origin, source_id, snapshot_at, created_at, updated_at
             FROM auth_configs WHERE id = ?1",
        )
        .map_err(|e| storage_err("prepare_get_auth_config", e.to_string()))?;

    stmt.query_row(params![id], |row| {
        Ok(AuthConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            auth_type: row.get(2)?,
            auth_data: row.get(3)?,
            origin: row.get(4)?,
            source_id: row.get(5)?,
            snapshot_at: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    })
    .optional()
    .map_err(|e| storage_err("get_auth_config", e.to_string()))
}

/// 删除认证配置
pub fn delete_auth_config(conn: &Connection, id: &str) -> Result<(), CoreError> {
    conn.execute("DELETE FROM auth_configs WHERE id = ?1", params![id])
        .map_err(|e| storage_err("delete_auth_config", e.to_string()))?;
    Ok(())
}

/// 更新认证配置的名称和认证数据
pub fn update_auth_config(conn: &Connection, ac: &AuthConfig) -> Result<(), CoreError> {
    let rows = conn
        .execute(
            "UPDATE auth_configs SET name = ?1, auth_type = ?2, auth_data = ?3, updated_at = ?4 WHERE id = ?5",
            params![ac.name, ac.auth_type, ac.auth_data, ac.updated_at, ac.id],
        )
        .map_err(|e| storage_err("update_auth_config", e.to_string()))?;

    if rows == 0 {
        return Err(CoreError::storage(StorageError::Persistence {
            store: "auth_store".to_string(),
            operation: "update_auth_config".to_string(),
            reason: format!("auth_config not found: {}", ac.id),
        }));
    }
    Ok(())
}
