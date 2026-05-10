//! Connection Manager 测试
//!
//! 测试连接管理器的核心功能

#[cfg(test)]
mod tests {
    use crate::core::services::connection_manager::{
        create_connection_id, get_connection_manager, ConnectionInfo, ConnectionManager,
    };
    use std::sync::Arc;

    fn create_test_connection_info(id: &str) -> ConnectionInfo {
        ConnectionInfo {
            id: id.to_string(),
            name: format!("Test Connection {}", id),
            db_type: "mysql".to_string(),
            url: "mysql://localhost:3306/test".to_string(),
            connection_type: crate::core::services::connection_manager::ConnectionType::Global,
            project_id: None,
            created_at: std::time::Instant::now(),
            server_version: None,
        }
    }

    #[tokio::test]
    async fn test_has_connection() {
        let manager = ConnectionManager::new();
        let conn_id = "test-conn-1".to_string();

        // 初始状态应该不存在
        assert!(!manager.has_connection(&conn_id).await);

        // 注意：这里无法测试添加连接，因为需要实际的 Database 实例
        // 实际测试需要在集成测试中进行
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
        // 应该正常执行，不会 panic
        manager.close_all_connections().await;
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_close_connection_not_exists() {
        let manager = ConnectionManager::new();
        // 关闭不存在的连接应该返回 false
        let conn_id = "non-existent".to_string();
        let result = manager.close_connection(&conn_id).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_set_active_connection_not_exists() {
        let manager = ConnectionManager::new();
        // 设置不存在的连接为活动连接应该返回 false
        let result = manager
            .set_active_connection("non-existent".to_string())
            .await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_switch_connection_not_exists() {
        let manager = ConnectionManager::new();
        // 切换到不存在的连接应该返回错误
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
        // 生成大量 ID，检查是否有重复
        let mut ids = std::collections::HashSet::new();
        for i in 0..1000 {
            let id = create_connection_id("mysql", &format!("mysql://localhost:3306/test{}", i));
            assert!(ids.insert(id), "Duplicate ID generated");
        }
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn test_connection_id_consistency() {
        // 相同输入应该始终产生相同输出
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

        // 应该返回同一个实例
        assert!(std::ptr::eq(Arc::as_ptr(manager1), Arc::as_ptr(manager2)));
    }
}
