// RdataStation Core Library
//
// 架构分层：
// - api/: 对外 DTO / Error（前后端共享）
// - core/: 纯业务逻辑（无框架依赖）
// - adapters/: 适配器层（Tauri/CLI/HTTP/WASM）
// - commands/: Tauri 命令（统一入口）

pub mod api;
pub mod core;
pub mod adapters;
pub mod commands;

// 重新导出常用类型
pub use api::{ErrorResponse, QueryResult, Row, Value};

// 统一从 commands 模块导入所有 Tauri 命令
// 注意：不再从 adapters/tauri/command.rs 导入命令，避免命令名称冲突
use commands::*;

// 项目状态管理
use commands::project_commands::ProjectState;
// 分析资源状态管理
use commands::analytics_resource_commands::AnalyticsResourceState;

/// 注册所有数据库驱动
///
/// 使用自动注册机制，支持：
/// 1. 内置驱动自动注册
/// 2. 配置文件驱动注册
/// 3. 自动扫描驱动注册
fn register_drivers() {
    use core::driver::auto_register::AutoDriverRegistrar;
    
    // 自动注册所有驱动（内置 + 配置 + 扫描）
    AutoDriverRegistrar::auto_register();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 注册数据库驱动
    register_drivers();

    // 初始化全局驱动管理器 + 全局系统数据库（异步初始化，在同步上下文中使用 block_on）
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    
    // 初始化全局系统数据库（SQLite 连接池 + DuckDB 长连接）
    if let Err(e) = rt.block_on(core::migration::initialize_global_system()) {
        tracing::error!("Global system database initialization failed: {}", e);
        eprintln!("ERROR: Global system database initialization failed: {}", e);
        // 不阻塞启动，但记录详细错误
    }
    
    // 初始化全局驱动管理器
    if let Err(e) = rt.block_on(core::driver::init_driver_manager()) {
        tracing::error!("Driver manager initialization failed: {}", e);
        eprintln!("ERROR: Driver manager initialization failed: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ProjectState::new())
        .manage(AnalyticsResourceState::new())
        .invoke_handler(tauri::generate_handler![
            // 连接命令
            connect_database,
            get_connections,
            switch_connection,
            close_connection,
            close_all_connections,
            get_active_connection,
            get_recent_connections,
            remove_recent_connection,
            test_connection,
            test_connection_config,
            create_database_file,
            convert_connection_type,
            detect_global_connections_in_project,
            get_global_connections,
            
            // SQL 命令
            execute_sql,
            execute_transaction,
            get_sql_history,
            search_sql_history,
            clear_sql_history,
            remove_sql_history,
            
            // 驱动命令
            get_drivers,
            get_driver_info,
            create_connection,
            create_connection_with_config,
            update_connection,
            
            // 导航器状态命令
            save_navigator_state,
            load_navigator_state,
            
            // 项目命令
            create_project,
            get_project_config,
            update_project_config,
            get_recent_projects,
            add_recent_project,
            open_project_by_id,
            open_project_by_path,
            create_and_save_project,
            validate_project,
            validate_project_full,
            delete_project,
            update_project,
            rename_project,
            get_all_projects,
            
            // 端口协商命令
            negotiate_port,
            negotiate_local_port,
            is_port_available,
            release_port,
            negotiate_multiple_ports,
            negotiate_port_range,
            get_common_db_ports,
            get_port_range_info,
            
            // 项目连接命令
            create_project_connection,
            get_project_connections,
            get_project_connection,
            update_project_connection,
            update_project_connection_status,
            delete_project_connection,
            search_project_connections,
            
            // 项目存储命令
            init_project_store,
            close_project_store,
            save_project_store_connection,
            get_project_store_connections,
            get_project_store_connection,
            delete_project_store_connection,
            save_project_store_sql_history,
            get_project_store_sql_history,
            save_project_store_workbench_state,
            get_project_store_workbench_state,
            
            // 联邦查询命令
            register_external_database,
            create_external_table,
            
            // 元数据缓存命令
            metadata_cache_commands::get_metadata_cache_status,
            metadata_cache_commands::refresh_metadata_cache,
            metadata_cache_commands::clear_metadata_cache,
            metadata_cache_commands::save_table_metadata_to_cache,
            metadata_cache_commands::save_column_metadata_to_cache,
            metadata_cache_commands::save_tables_batch_to_cache,
            metadata_cache_commands::save_columns_batch_to_cache,
            metadata_cache_commands::get_tables_from_cache,
            metadata_cache_commands::get_columns_from_cache,
            
            // 缓存预热命令
            cache_warming_commands::start_cache_warming,
            cache_warming_commands::cancel_cache_warming,
            cache_warming_commands::get_warming_progress,
            cache_warming_commands::check_cache_version,
            cache_warming_commands::execute_cache_migration,
            cache_warming_commands::get_cache_migration_history,
            cache_warming_commands::get_introspect_level_suggestion,
            cache_warming_commands::get_schema_object_counts,
            cache_warming_commands::build_cache_index,

            // SQL 解析与转译命令
            parse_sql,
            format_sql,
            transpile_sql,
            validate_sql,
            split_sql,
            
            // 结果集分析命令
            re_execute_with_filter,
            execute_duckdb_analysis,
            get_column_insights,
            get_column_insight_full,
            create_duckdb_temp_table,
            save_column_insight_snapshot,
            get_column_insight_history,
            
            // 项目管理命令（已合并到 project_commands）
            // 旧的 project_management_commands 模块已删除
            // save_project_info_to_system, get_all_projects, update_project_last_opened, delete_project_info
            // 现在使用: save_project_info_to_system, get_all_projects, add_recent_project, delete_project
            
            // 分析资源管理命令
            create_analytics_resource,
            update_analytics_resource,
            get_analytics_resource,
            list_analytics_resources,
            list_analytics_resources_paginated,
            delete_analytics_resource,
            batch_delete_analytics_resources,
            clone_analytics_resource,
            create_analytics_folder,
            get_analytics_folder,
            list_analytics_folders,
            add_analytics_resource_to_folder,
            remove_analytics_resource_from_folder,
            create_analytics_tag,
            list_analytics_tags,
            add_analytics_tag_to_resource,
            remove_analytics_tag_from_resource,
            get_analytics_recycle_bin,
            restore_analytics_resource_from_recycle,
            permanent_delete_analytics_resource,
            init_analytics_resource_store,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
