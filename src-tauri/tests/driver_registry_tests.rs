//! DriverConnectionConfig 集成测试
//!
//! 测试连接配置构建器的公共 API
//!
//! 本文件位于 src-tauri/tests/（集成测试），
//! 遵循 RdataStation 测试代码组织铁律。

use rdata_station_lib::core::driver::registry::DriverConnectionConfig;

#[test]
fn test_connection_config_new() {
    let config = DriverConnectionConfig::new("mysql");
    assert_eq!(config.driver, "mysql");
    assert!(config.name.is_none());
    assert!(config.host.is_none());
    assert!(config.options.is_empty());
}

#[test]
fn test_connection_config_builder() {
    let config = DriverConnectionConfig::new("mysql")
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
    assert_eq!(
        config.options.get("ssl_mode"),
        Some(&"PREFERRED".to_string())
    );
}

#[test]
fn test_mysql_url_building() {
    let config = DriverConnectionConfig::new("mysql")
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
    let config = DriverConnectionConfig::new("mysql")
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
    let config = DriverConnectionConfig::new("postgres")
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
    let config = DriverConnectionConfig::new("sqlite").with_file_path("/path/to/db.sqlite");

    let url = config.to_url().unwrap();
    assert_eq!(url, "sqlite:///path/to/db.sqlite");
}

#[test]
fn test_duckdb_url_building() {
    let config = DriverConnectionConfig::new("duckdb").with_file_path("/path/to/db.duckdb");

    let url = config.to_url().unwrap();
    assert_eq!(url, "duckdb:///path/to/db.duckdb");
}

#[test]
fn test_mysql_url_missing_host() {
    let config = DriverConnectionConfig::new("mysql");
    let result = config.to_url();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Host is required"));
}

#[test]
fn test_sqlite_url_missing_file_path() {
    let config = DriverConnectionConfig::new("sqlite");
    let result = config.to_url();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("File path is required"));
}

#[test]
fn test_unsupported_driver() {
    let config = DriverConnectionConfig::new("oracle");
    let result = config.to_url();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unsupported driver"));
}