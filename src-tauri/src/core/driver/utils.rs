use crate::core::driver::registry::ConnectionConfig as DriverConnectionConfig;
use crate::core::error::{ConnectionError, CoreError};

/// 构建数据库连接URL
pub fn build_connection_url(config: &DriverConnectionConfig) -> Result<String, CoreError> {
    match config.driver.as_str() {
        "mysql" => build_mysql_url(config),
        "postgres" => build_postgres_url(config),
        "sqlite" => build_sqlite_url(config),
        "duckdb" => build_duckdb_url(config),
        "clickhouse" => build_clickhouse_url(config),
        _ => Err(CoreError::connection(ConnectionError::DriverNotFound {
            driver: config.driver.clone(),
        })),
    }
}

/// 构建MySQL连接URL
fn build_mysql_url(config: &DriverConnectionConfig) -> Result<String, CoreError> {
    let host = config.host.as_deref().ok_or_else(|| {
        CoreError::connection(ConnectionError::InvalidConfig {
            conn_id: config.name.clone().unwrap_or_else(|| "mysql".to_string()),
            reason: "Host is required".to_string(),
        })
    })?;

    let port = config.port.unwrap_or(3306);
    let username = config.username.as_deref().unwrap_or("root");
    let password = config.password.as_deref().unwrap_or("");
    let database = config.database.as_deref().unwrap_or("");

    Ok(format!(
        "mysql://{}:{}@{}:{}/{}",
        username, password, host, port, database
    ))
}

/// 构建PostgreSQL连接URL
fn build_postgres_url(config: &DriverConnectionConfig) -> Result<String, CoreError> {
    let host = config.host.as_deref().ok_or_else(|| {
        CoreError::connection(ConnectionError::InvalidConfig {
            conn_id: config
                .name
                .clone()
                .unwrap_or_else(|| "postgres".to_string()),
            reason: "Host is required".to_string(),
        })
    })?;

    let port = config.port.unwrap_or(5432);
    let username = config.username.as_deref().unwrap_or("postgres");
    let password = config.password.as_deref().unwrap_or("");
    let database = config.database.as_deref().unwrap_or("postgres");

    Ok(format!(
        "postgres://{}:{}@{}:{}/{}",
        username, password, host, port, database
    ))
}

/// 构建SQLite连接URL
fn build_sqlite_url(config: &DriverConnectionConfig) -> Result<String, CoreError> {
    let database = config.database.as_deref().ok_or_else(|| {
        CoreError::connection(ConnectionError::InvalidConfig {
            conn_id: config.name.clone().unwrap_or_else(|| "sqlite".to_string()),
            reason: "Database path is required".to_string(),
        })
    })?;

    Ok(format!("sqlite://{}", database))
}

/// 构建DuckDB连接URL
fn build_duckdb_url(config: &DriverConnectionConfig) -> Result<String, CoreError> {
    let database = config.database.as_deref().unwrap_or(":memory:");

    Ok(format!("duckdb://{}", database))
}

/// 构建ClickHouse连接URL
fn build_clickhouse_url(config: &DriverConnectionConfig) -> Result<String, CoreError> {
    let host = config.host.as_deref().ok_or_else(|| {
        CoreError::connection(ConnectionError::InvalidConfig {
            conn_id: config
                .name
                .clone()
                .unwrap_or_else(|| "clickhouse".to_string()),
            reason: "Host is required".to_string(),
        })
    })?;

    let port = config.port.unwrap_or(9000);
    let username = config.username.as_deref().unwrap_or("default");
    let password = config.password.as_deref().unwrap_or("");
    let database = config.database.as_deref().unwrap_or("default");

    Ok(format!(
        "clickhouse://{}:{}@{}:{}/{}",
        username, password, host, port, database
    ))
}

/// 验证驱动配置
pub fn validate_driver_config(config: &DriverConnectionConfig) -> Result<(), CoreError> {
    match config.driver.as_str() {
        "mysql" | "postgres" | "clickhouse" => {
            // 验证网络数据库的必需字段
            if config.host.is_none() {
                return Err(CoreError::connection(ConnectionError::InvalidConfig {
                    conn_id: config.name.clone().unwrap_or_else(|| config.driver.clone()),
                    reason: "Host is required".to_string(),
                }));
            }
            if config.username.is_none() {
                return Err(CoreError::connection(ConnectionError::InvalidConfig {
                    conn_id: config.name.clone().unwrap_or_else(|| config.driver.clone()),
                    reason: "Username is required".to_string(),
                }));
            }
        }
        "sqlite" => {
            // 验证SQLite的必需字段
            if config.database.is_none() {
                return Err(CoreError::connection(ConnectionError::InvalidConfig {
                    conn_id: config.name.clone().unwrap_or_else(|| "sqlite".to_string()),
                    reason: "Database path is required".to_string(),
                }));
            }
        }
        "duckdb" => {
            // DuckDB不需要验证，默认使用内存数据库
        }
        _ => {
            return Err(CoreError::connection(ConnectionError::DriverNotFound {
                driver: config.driver.clone(),
            }));
        }
    }

    Ok(())
}

/// 解析驱动ID
pub fn parse_driver_id(url: &str) -> Option<&str> {
    if url.starts_with("mysql://") {
        Some("mysql")
    } else if url.starts_with("postgres://") {
        Some("postgres")
    } else if url.starts_with("sqlite://") {
        Some("sqlite")
    } else if url.starts_with("duckdb://") {
        Some("duckdb")
    } else if url.starts_with("clickhouse://") {
        Some("clickhouse")
    } else {
        None
    }
}
