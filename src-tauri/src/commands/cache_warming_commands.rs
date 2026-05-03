/**
 * 缓存预热相关命令
 * 
 * 处理缓存预热的启动、取消、进度查询等操作
 */

use serde::{Deserialize, Serialize};

use crate::core::persistence::metadata_cache::{MetadataCacheManager, MetadataCacheOps, ConnectionType};
use crate::core::persistence::cache_version_migration::{CacheVersionManager, CURRENT_CACHE_VERSION};

/// 预热进度响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmingProgressResponse {
    pub connection_id: String,
    pub is_warming: bool,
    pub current_step: String,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub progress_percentage: f64,
    pub current_database: Option<String>,
    pub current_schema: Option<String>,
    pub current_table: Option<String>,
}

/// 预热请求
#[derive(Debug, Clone, Deserialize)]
pub struct WarmCacheInput {
    pub connection_id: String,
    pub connection_type: String,
    pub project_path: Option<String>,
    pub databases: Vec<String>,
}

/// 取消预热请求
#[derive(Debug, Clone, Deserialize)]
pub struct CancelWarmingInput {
    pub connection_id: String,
}

/// 版本迁移响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResponse {
    pub from_version: u32,
    pub to_version: u32,
    pub success: bool,
    pub duration_ms: Option<u64>,
    pub message: String,
}

/// 启动缓存预热
#[tauri::command]
pub async fn start_cache_warming(
    input: WarmCacheInput,
) -> Result<WarmingProgressResponse, String> {
    let connection_type = if input.connection_type == "global" {
        ConnectionType::Global
    } else {
        ConnectionType::Project
    };

    let cache_manager = MetadataCacheManager::new(
        &input.connection_id,
        connection_type,
        input.project_path.as_deref(),
    ).map_err(|e| e.to_string())?;

    let conn = cache_manager.open().map_err(|e| e.to_string())?;
    let ops = MetadataCacheOps::new(conn);

    // 检查是否需要版本迁移
    let version_manager = CacheVersionManager::new();
    if version_manager.needs_upgrade(ops.get_connection()).map_err(|e| e.to_string())? {
        let records = version_manager.migrate(ops.get_connection()).map_err(|e| e.to_string())?;
        
        if !records.is_empty() {
            tracing::info!(
                connection_id = %input.connection_id,
                "缓存版本迁移完成: {} -> {}",
                records[0].from_version,
                records[0].to_version
            );
        }
    }

    let total_steps = input.databases.len() * 3;

    Ok(WarmingProgressResponse {
        connection_id: input.connection_id,
        is_warming: true,
        current_step: "开始预热".to_string(),
        total_steps,
        completed_steps: 0,
        progress_percentage: 0.0,
        current_database: None,
        current_schema: None,
        current_table: None,
    })
}

/// 取消缓存预热
#[tauri::command]
pub async fn cancel_cache_warming(
    input: CancelWarmingInput,
) -> Result<(), String> {
    tracing::info!(
        connection_id = %input.connection_id,
        "取消缓存预热"
    );

    // 预热取消逻辑由前端状态管理器处理
    // 后端只需要记录取消事件
    Ok(())
}

/// 获取预热进度
#[tauri::command]
pub async fn get_warming_progress(
    connection_id: String,
) -> Result<WarmingProgressResponse, String> {
    // 返回当前预热进度
    // 实际进度由前端状态管理器维护
    Ok(WarmingProgressResponse {
        connection_id,
        is_warming: false,
        current_step: "空闲".to_string(),
        total_steps: 0,
        completed_steps: 0,
        progress_percentage: 0.0,
        current_database: None,
        current_schema: None,
        current_table: None,
    })
}

/// 检查缓存版本
#[tauri::command]
pub async fn check_cache_version(
    connection_id: String,
    connection_type: String,
    project_path: Option<String>,
) -> Result<u32, String> {
    let cache_manager = MetadataCacheManager::new(
        &connection_id,
        if connection_type == "global" {
            ConnectionType::Global
        } else {
            ConnectionType::Project
        },
        project_path.as_deref(),
    ).map_err(|e| e.to_string())?;

    if !cache_manager.exists() {
        return Ok(0);
    }

    let conn = cache_manager.open().map_err(|e| e.to_string())?;
    let version_manager = CacheVersionManager::new();
    
    version_manager.get_current_version(&conn).map_err(|e| e.to_string())
}

/// 执行缓存版本迁移
#[tauri::command]
pub async fn execute_cache_migration(
    connection_id: String,
    connection_type: String,
    project_path: Option<String>,
) -> Result<MigrationResponse, String> {
    let cache_manager = MetadataCacheManager::new(
        &connection_id,
        if connection_type == "global" {
            ConnectionType::Global
        } else {
            ConnectionType::Project
        },
        project_path.as_deref(),
    ).map_err(|e| e.to_string())?;

    let conn = cache_manager.open().map_err(|e| e.to_string())?;
    let version_manager = CacheVersionManager::new();

    let from_version = version_manager.get_current_version(&conn).map_err(|e| e.to_string())?;

    if from_version >= CURRENT_CACHE_VERSION {
        return Ok(MigrationResponse {
            from_version,
            to_version: from_version,
            success: true,
            duration_ms: None,
            message: "缓存已是最新版本".to_string(),
        });
    }

    let start_time = std::time::Instant::now();
    let records = version_manager.migrate(&conn).map_err(|e| e.to_string())?;
    let duration = start_time.elapsed();

    if records.is_empty() {
        Ok(MigrationResponse {
            from_version,
            to_version: from_version,
            success: true,
            duration_ms: None,
            message: "无需迁移".to_string(),
        })
    } else {
        let record = &records[0];
        Ok(MigrationResponse {
            from_version: record.from_version,
            to_version: record.to_version,
            success: record.success,
            duration_ms: Some(duration.as_millis() as u64),
            message: if record.success {
                format!("迁移成功: {} -> {}", record.from_version, record.to_version)
            } else {
                format!("迁移失败: {} -> {}", record.from_version, record.to_version)
            },
        })
    }
}

/// 获取缓存版本迁移历史
#[tauri::command]
pub async fn get_cache_migration_history(
    connection_id: String,
    connection_type: String,
    project_path: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let cache_manager = MetadataCacheManager::new(
        &connection_id,
        if connection_type == "global" {
            ConnectionType::Global
        } else {
            ConnectionType::Project
        },
        project_path.as_deref(),
    ).map_err(|e| e.to_string())?;

    if !cache_manager.exists() {
        return Ok(vec![]);
    }

    let conn = cache_manager.open().map_err(|e| e.to_string())?;
    let version_manager = CacheVersionManager::new();

    let history = version_manager.get_migration_history(&conn).map_err(|e| e.to_string())?;

    let result: Vec<serde_json::Value> = history.iter().map(|record| {
        serde_json::json!({
            "from_version": record.from_version,
            "to_version": record.to_version,
            "migrated_at": record.migrated_at,
            "reason": record.reason,
            "duration_ms": record.duration_ms,
            "success": record.success,
        })
    }).collect();

    Ok(result)
}
