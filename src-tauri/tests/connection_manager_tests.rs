//! Connection Manager 测试
//!
//! 测试连接管理器的核心功能
//!
//! 本文件位于 src-tauri/tests/（集成测试），
//! 遵循 RdataStation 测试代码组织铁律。

use std::collections::HashSet;
use std::sync::Arc;

use rdata_station_lib::core::services::connection_manager::{
    create_connection_id, get_connection_manager, ConnectionInfo, ConnectionManager, ConnectionType,
};

fn create_test_connection_info(id: &str) -> ConnectionInfo {
    ConnectionInfo {
        id: id.to_string(),
        name: format!("Test Connection {}", id),
        db_type: "mysql".to_string(),
        url: "mysql://localhost:3306/test".to_string(),
        connection_type: ConnectionType::Global,
        project_id: None,
        created_at: std::time::Instant::now(),
        server_version: None,
    }
}

#[tokio::test]
async fn test_has_connection() {
    let manager = ConnectionManager::new();
    let conn_id = "test-conn-1".to_string();

    assert!(!manager.has_connection(&conn_id).await);
}

#[tokio::test]
async fn test_get_all_connection_ids_empty() {
    let manager = ConnectionManager::new();
    let ids = manager.get_all_connection_ids().await;
    assert!(ids.is_empty());
}

#[tokio::test]
async fn test_get_all_connection_info_empty() {
    let manager = ConnectionManager::new();
    let infos = manager.get_all_connection_info().await;
    assert!(infos.is_empty());
}

#[tokio::test]
async fn test_get_active_connection_id_initially_none() {
    let manager = ConnectionManager::new();
    let active_id = manager.get_active_connection_id().await;
    assert!(active_id.is_none());
}

#[tokio::test]
async fn test_close_all_connections_empty() {
    let manager = ConnectionManager::new();
    manager.close_all_connections().await;
    assert_eq!(manager.connection_count().await, 0);
}

#[tokio::test]
async fn test_close_connection_not_exists() {
    let manager = ConnectionManager::new();
    let conn_id = "non-existent".to_string();
    let result = manager.close_connection(&conn_id).await;
    assert!(!result);
}

#[tokio::test]
async fn test_set_active_connection_not_exists() {
    let manager = ConnectionManager::new();
    let result = manager
        .set_active_connection("non-existent".to_string())
        .await;
    assert!(!result);
}

#[tokio::test]
async fn test_switch_connection_not_exists() {
    let manager = ConnectionManager::new();
    let conn_id = "non-existent".to_string();
    let result = manager.switch_connection(&conn_id).await;
    assert!(result.is_err());
}

#[test]
fn test_connection_info_creation() {
    let info = create_test_connection_info("test-1");
    assert_eq!(info.id, "test-1");
    assert_eq!(info.name, "Test Connection test-1");
    assert_eq!(info.db_type, "mysql");
    assert_eq!(info.url, "mysql://localhost:3306/test");
}

#[test]
fn test_connection_id_uniqueness() {
    let mut ids = HashSet::new();
    for i in 0..1000 {
        let id = create_connection_id("mysql", &format!("mysql://localhost:3306/test{}", i));
        assert!(ids.insert(id), "Duplicate ID generated");
    }
    assert_eq!(ids.len(), 1000);
}

#[test]
fn test_connection_id_consistency() {
    for _ in 0..100 {
        let id1 = create_connection_id("mysql", "mysql://localhost:3306/test");
        let id2 = create_connection_id("mysql", "mysql://localhost:3306/test");
        assert_eq!(id1, id2);
    }
}

#[tokio::test]
async fn test_get_connection_manager_singleton() {
    let manager1 = get_connection_manager();
    let manager2 = get_connection_manager();

    assert!(std::ptr::eq(Arc::as_ptr(manager1), Arc::as_ptr(manager2)));
}
