//! 内存管理相关命令
//!
//! 处理内存监控、缓存清理、内存统计等操作

use crate::core::cache::{get_memory_guard, CacheManager};

/// 内存统计响应
#[derive(serde::Serialize, Debug)]
pub struct MemoryStatsResponse {
    pub cache_entries: usize,
    pub pool_size: usize,
    pub estimated_memory_mb: f64,
    pub last_eviction_secs_ago: Option<u64>,
    pub total_evictions: u64,
    pub pressure_level: String,
    pub cache_hit_rate: f64,
    pub cache_size: usize,
}

/// 获取内存统计信息
#[tauri::command]
pub async fn get_memory_stats() -> Result<MemoryStatsResponse, String> {
    let guard = get_memory_guard().ok_or_else(|| "Memory guard not initialized".to_string())?;

    let stats = guard.get_stats().await;

    let last_eviction_secs_ago = stats
        .last_eviction
        .map(|instant| instant.elapsed().as_secs());

    // 获取缓存命中率
    let cache_manager = CacheManager::instance();
    let cache_hit_rate = {
        let manager = cache_manager
            .lock()
            .map_err(|e| format!("Failed to lock cache: {}", e))?;
        manager.stats().metadata.hit_rate()
    };

    let cache_size = {
        let manager = cache_manager
            .lock()
            .map_err(|e| format!("Failed to lock cache: {}", e))?;
        manager.stats().metadata.entry_count
    };

    Ok(MemoryStatsResponse {
        cache_entries: stats.cache_entries,
        pool_size: stats.pool_size,
        estimated_memory_mb: stats.estimated_memory_mb,
        last_eviction_secs_ago,
        total_evictions: stats.total_evictions,
        pressure_level: stats.pressure_level,
        cache_hit_rate,
        cache_size,
    })
}

/// 清理过期缓存
#[tauri::command]
pub async fn cleanup_expired_cache() -> Result<usize, String> {
    let cache_manager = CacheManager::instance();
    let manager = cache_manager
        .lock()
        .map_err(|e| format!("Failed to lock cache: {}", e))?;

    let cleaned = manager.cleanup_expired();
    Ok(cleaned)
}

/// 强制淘汰缓存
#[tauri::command]
pub async fn force_evict_cache(_ratio: f64) -> Result<usize, String> {
    let guard = get_memory_guard().ok_or_else(|| "Memory guard not initialized".to_string())?;

    let evicted = guard.check_and_evict().await?;
    Ok(evicted)
}

/// 清除指定连接的所有缓存
#[tauri::command]
pub async fn clear_connection_cache(conn_id: String) -> Result<(), String> {
    let cache_manager = CacheManager::instance();
    let manager = cache_manager
        .lock()
        .map_err(|e| format!("Failed to lock cache: {}", e))?;

    manager.invalidate_connection(&conn_id);
    Ok(())
}

/// 清除所有缓存
#[tauri::command]
pub async fn clear_all_cache() -> Result<(), String> {
    let cache_manager = CacheManager::instance();
    let manager = cache_manager
        .lock()
        .map_err(|e| format!("Failed to lock cache: {}", e))?;

    manager.clear_all();
    Ok(())
}
