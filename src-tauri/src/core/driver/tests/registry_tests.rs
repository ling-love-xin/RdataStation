//! Driver Registry 测试
//!
//! 测试驱动注册表的核心功能

#[cfg(test)]
mod tests {
    use crate::core::driver::registry::{ConnectionConfig, DriverDescriptor, get_all_drivers, get_driver};

    #[test]
    fn test_connection_config_new() {
        let config = ConnectionConfig::new("mysql");
        assert_eq!(config.driver, "mysql");
        assert!(config.name.is_none());
        assert!(config.host.is_none());
        assert!(config.options.is_empty());
    }

    #[test]
    fn test_connection_config_builder() {
        let config = ConnectionConfig::new("mysql")
            .with_name("test-connection")
            .with_host("localhost")
            .with_port(3306)
            .with_database("testdb")
            .with_username("root")
            .with_password("secret")
            .with_option("ssl_mode", "PREFERRED");

        assert_eq!(config.name, Some("test-connection".to_string()));
        assert_eq!(config.host, Some("localhost".to_string()));
        assert_eq!(config.port, Some(3306));
        assert_eq!(config.database, Some("testdb".to_string()));
        assert_eq!(config.username, Some("root".to_string()));
        assert_eq!(config.password, Some("secret".to_string()));
        assert_eq!(config.options.get("ssl_mode"), Some(&"PREFERRED".to_string()));
    }

    #[test]
    fn test_mysql_url_building() {
        let config = ConnectionConfig::new("mysql")
            .with_host("localhost")
            .with_port(3306)
            .with_database("testdb")
            .with_username("root")
            .with_password("secret");

        let url = config.to_url().unwrap();
        assert_eq!(url, "mysql://root:secret@localhost:3306/testdb");
    }

    #[test]
    fn test_mysql_url_with_options() {
        let config = ConnectionConfig::new("mysql")
            .with_host("localhost")
            .with_port(3306)
            .with_database("testdb")
            .with_username("root")
            .with_password("secret")
            .with_option("ssl_mode", "PREFERRED")
            .with_option("charset", "utf8mb4");

        let url = config.to_url().unwrap();
        assert!(url.contains("ssl_mode=PREFERRED"));
        assert!(url.contains("charset=utf8mb4"));
        assert!(url.starts_with("mysql://root:secret@localhost:3306/testdb?"));
    }

    #[test]
    fn test_postgres_url_building() {
        let config = ConnectionConfig::new("postgres")
            .with_host("localhost")
            .with_port(5432)
            .with_database("testdb")
            .with_username("postgres")
            .with_password("secret");

        let url = config.to_url().unwrap();
        assert_eq!(url, "postgres://postgres:secret@localhost:5432/testdb");
    }

    #[test]
    fn test_sqlite_url_building() {
        let config = ConnectionConfig::new("sqlite")
            .with_file_path("/path/to/db.sqlite");

        let url = config.to_url().unwrap();
        assert_eq!(url, "sqlite:///path/to/db.sqlite");
    }

    #[test]
    fn test_duckdb_url_building() {
        let config = ConnectionConfig::new("duckdb")
            .with_file_path("/path/to/db.duckdb");

        let url = config.to_url().unwrap();
        assert_eq!(url, "duckdb:///path/to/db.duckdb");
    }

    #[test]
    fn test_mysql_url_missing_host() {
        let config = ConnectionConfig::new("mysql");
        let result = config.to_url();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Host is required"));
    }

    #[test]
    fn test_sqlite_url_missing_file_path() {
        let config = ConnectionConfig::new("sqlite");
        let result = config.to_url();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File path is required"));
    }

    #[test]
    fn test_unsupported_driver() {
        let config = ConnectionConfig::new("oracle");
        let result = config.to_url();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported driver"));
    }

    #[test]
    fn test_get_all_drivers() {
        let drivers = get_all_drivers();
        assert!(!drivers.is_empty());

        // 应该包含所有支持的驱动
        let driver_ids: Vec<_> = drivers.iter().map(|d| d.id.as_str()).collect();
        assert!(driver_ids.contains(&"mysql"));
        assert!(driver_ids.contains(&"postgres"));
        assert!(driver_ids.contains(&"sqlite"));
        assert!(driver_ids.contains(&"duckdb"));
    }

    #[test]
    fn test_get_driver_mysql() {
        let driver = get_driver("mysql");
        assert!(driver.is_some());

        let driver = driver.unwrap();
        assert_eq!(driver.id, "mysql");
        assert_eq!(driver.default_port, Some(3306));
        assert!(!driver.require_file);
    }

    #[test]
    fn test_get_driver_postgres() {
        let driver = get_driver("postgres");
        assert!(driver.is_some());

        let driver = driver.unwrap();
        assert_eq!(driver.id, "postgres");
        assert_eq!(driver.default_port, Some(5432));
    }

    #[test]
    fn test_get_driver_sqlite() {
        let driver = get_driver("sqlite");
        assert!(driver.is_some());

        let driver = driver.unwrap();
        assert_eq!(driver.id, "sqlite");
        assert!(driver.require_file);
        assert!(!driver.require_database);
    }

    #[test]
    fn test_get_driver_not_found() {
        let driver = get_driver("oracle");
        assert!(driver.is_none());
    }

    #[test]
    fn test_mysql_driver_fields() {
        let driver = get_driver("mysql").unwrap();

        // 检查必需的字段
        let field_keys: Vec<_> = driver.fields.iter().map(|f| f.key.as_str()).collect();
        assert!(field_keys.contains(&"host"));
        assert!(field_keys.contains(&"port"));
        assert!(field_keys.contains(&"username"));
        assert!(field_keys.contains(&"password"));
    }

    #[test]
    fn test_driver_descriptor_serialization() {
        let driver = get_driver("mysql").unwrap();

        // 序列化为 JSON
        let json = serde_json::to_string(&driver).unwrap();
        assert!(json.contains("mysql"));
        assert!(json.contains("3306"));

        // 反序列化
        let deserialized: DriverDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "mysql");
    }
}
