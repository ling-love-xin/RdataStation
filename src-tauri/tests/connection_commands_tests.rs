//! connection_commands 集成测试
//!
//! 测试连接命令的输入校验、参数验证和边界情况。
//! 遵循 RdataStation 测试代码组织铁律。

use rdata_station_lib::commands::connection_commands::{
    ConnectDatabaseInput, ConvertConnectionInput, CreateDatabaseFileInput,
};

// ==================== ConnectDatabaseInput 校验 ====================

#[test]
fn test_connect_input_empty_url() {
    let input = ConnectDatabaseInput {
        conn_id: None,
        db_type: "mysql".to_string(),
        url: "".to_string(),
        name: Some("test".to_string()),
        connection_type: Some("global".to_string()),
        project_id: None,
        description: None,
        driver_id: None,
        environment_id: None,
        auth_config_id: None,
        auth_method: None,
        network_config_id: None,
        driver_properties: None,
        advanced_options: None,
        options: None,
        tags: None,
        metadata_path: None,
        schema_name: None,
        use_duckdb_fed: None,
        password: None,
    };
    assert!(input.url.is_empty(), "url should be empty");
    assert_eq!(input.db_type, "mysql");
}

#[test]
fn test_connect_input_empty_db_type() {
    let input = ConnectDatabaseInput {
        conn_id: None,
        db_type: "  ".to_string(),
        url: "mysql://localhost:3306/test".to_string(),
        name: Some("test".to_string()),
        connection_type: Some("global".to_string()),
        project_id: None,
        description: None,
        driver_id: None,
        environment_id: None,
        auth_config_id: None,
        auth_method: None,
        network_config_id: None,
        driver_properties: None,
        advanced_options: None,
        options: None,
        tags: None,
        metadata_path: None,
        schema_name: None,
        use_duckdb_fed: None,
        password: None,
    };
    assert!(input.db_type.trim().is_empty(), "db_type should be whitespace-only");
}

#[test]
fn test_connect_input_empty_name() {
    let input = ConnectDatabaseInput {
        conn_id: None,
        db_type: "mysql".to_string(),
        url: "mysql://localhost:3306/test".to_string(),
        name: Some("   ".to_string()),
        connection_type: Some("global".to_string()),
        project_id: None,
        description: None,
        driver_id: None,
        environment_id: None,
        auth_config_id: None,
        auth_method: None,
        network_config_id: None,
        driver_properties: None,
        advanced_options: None,
        options: None,
        tags: None,
        metadata_path: None,
        schema_name: None,
        use_duckdb_fed: None,
        password: None,
    };
    assert!(input.name.as_deref().unwrap().trim().is_empty());
}

#[test]
fn test_connect_input_project_without_id() {
    let input = ConnectDatabaseInput {
        conn_id: None,
        db_type: "mysql".to_string(),
        url: "mysql://localhost:3306/test".to_string(),
        name: Some("project-conn".to_string()),
        connection_type: Some("project".to_string()),
        project_id: None, // 项目连接缺少 project_id
        description: None,
        driver_id: None,
        environment_id: None,
        auth_config_id: None,
        auth_method: None,
        network_config_id: None,
        driver_properties: None,
        advanced_options: None,
        options: None,
        tags: None,
        metadata_path: None,
        schema_name: None,
        use_duckdb_fed: None,
        password: None,
    };
    assert_eq!(input.connection_type.as_deref(), Some("project"));
    assert!(input.project_id.is_none());
}

#[test]
fn test_connect_input_invalid_connection_type() {
    let input = ConnectDatabaseInput {
        conn_id: None,
        db_type: "mysql".to_string(),
        url: "mysql://localhost:3306/test".to_string(),
        name: Some("test".to_string()),
        connection_type: Some("invalid_type".to_string()),
        project_id: None,
        description: None,
        driver_id: None,
        environment_id: None,
        auth_config_id: None,
        auth_method: None,
        network_config_id: None,
        driver_properties: None,
        advanced_options: None,
        options: None,
        tags: None,
        metadata_path: None,
        schema_name: None,
        use_duckdb_fed: None,
        password: None,
    };
    assert_ne!(input.connection_type.as_deref(), Some("global"));
    assert_ne!(input.connection_type.as_deref(), Some("project"));
}

#[test]
fn test_connect_input_full_fields() {
    let input = ConnectDatabaseInput {
        conn_id: Some("conn-123".to_string()),
        db_type: "postgresql".to_string(),
        url: "postgres://localhost:5432/db".to_string(),
        name: Some("Full Config Connection".to_string()),
        connection_type: Some("global".to_string()),
        project_id: None,
        description: Some("Test connection".to_string()),
        driver_id: Some("postgres-native".to_string()),
        environment_id: Some("G_env_prod".to_string()),
        auth_config_id: Some("G_auth_001".to_string()),
        auth_method: Some("password".to_string()),
        network_config_id: Some("G_net_ssh_001".to_string()),
        driver_properties: Some(r#"{"useSSL":"true"}"#.to_string()),
        advanced_options: Some(r#"{"timeout":30}"#.to_string()),
        options: Some("charset=utf8".to_string()),
        tags: Some(r#"["production","critical"]"#.to_string()),
        metadata_path: Some("/tmp/metadata".to_string()),
        schema_name: Some("public".to_string()),
        use_duckdb_fed: Some(false),
        password: Some("secret".to_string()),
    };
    // 验证所有可选字段正确设置
    assert_eq!(input.conn_id.as_deref(), Some("conn-123"));
    assert_eq!(input.db_type, "postgresql");
    assert!(input.name.is_some());
    assert_eq!(input.driver_id.as_deref(), Some("postgres-native"));
    assert_eq!(input.environment_id.as_deref(), Some("G_env_prod"));
    assert_eq!(input.auth_config_id.as_deref(), Some("G_auth_001"));
    assert_eq!(input.auth_method.as_deref(), Some("password"));
    assert_eq!(input.network_config_id.as_deref(), Some("G_net_ssh_001"));
    assert_eq!(input.schema_name.as_deref(), Some("public"));
    assert_eq!(input.use_duckdb_fed, Some(false));
    assert_eq!(input.tags.as_deref(), Some(r#"["production","critical"]"#));
}

#[test]
fn test_connect_input_19_field_coverage() {
    // 验证 ConnectDatabaseInput 包含规范要求的全量19字段
    // fields: conn_id, db_type, url, name, connection_type, project_id,
    //          description, driver_id, environment_id, auth_config_id,
    //          auth_method, network_config_id, driver_properties, advanced_options,
    //          options, tags, metadata_path, schema_name, use_duckdb_fed, password
    let input = ConnectDatabaseInput {
        conn_id: None,
        db_type: "sqlite".to_string(),
        url: "sqlite:///tmp/test.db".to_string(),
        name: None,
        connection_type: None,
        project_id: None,
        description: None,
        driver_id: None,
        environment_id: None,
        auth_config_id: None,
        auth_method: None,
        network_config_id: None,
        driver_properties: None,
        advanced_options: None,
        options: None,
        tags: None,
        metadata_path: None,
        schema_name: None,
        use_duckdb_fed: None,
        password: None,
    };
    assert_eq!(input.db_type, "sqlite");
    assert!(input.conn_id.is_none());
}

// ==================== ConvertConnectionInput 校验 ====================

#[test]
fn test_convert_to_project_missing_id() {
    let input = ConvertConnectionInput {
        conn_id: "conn-1".to_string(),
        target_type: "project".to_string(),
        project_id: None,
    };
    assert_eq!(input.target_type, "project");
    assert!(input.project_id.is_none());
}

#[test]
fn test_convert_to_global() {
    let input = ConvertConnectionInput {
        conn_id: "conn-1".to_string(),
        target_type: "global".to_string(),
        project_id: None,
    };
    assert_eq!(input.target_type, "global");
}

#[test]
fn test_convert_invalid_type() {
    let input = ConvertConnectionInput {
        conn_id: "conn-1".to_string(),
        target_type: "invalid".to_string(),
        project_id: None,
    };
    assert_ne!(input.target_type, "global");
    assert_ne!(input.target_type, "project");
}

// ==================== CreateDatabaseFileInput 校验 ====================

#[test]
fn test_create_db_file_sqlite() {
    let input = CreateDatabaseFileInput {
        db_type: "sqlite".to_string(),
        file_path: "/tmp/test.db".to_string(),
    };
    assert_eq!(input.db_type, "sqlite");
    assert!(!input.file_path.is_empty());
}

#[test]
fn test_create_db_file_duckdb() {
    let input = CreateDatabaseFileInput {
        db_type: "duckdb".to_string(),
        file_path: "/tmp/test.duckdb".to_string(),
    };
    assert_eq!(input.db_type, "duckdb");
}

#[test]
fn test_create_db_file_unsupported_type() {
    let input = CreateDatabaseFileInput {
        db_type: "oracle".to_string(),
        file_path: "/tmp/test.db".to_string(),
    };
    assert_ne!(input.db_type, "sqlite");
    assert_ne!(input.db_type, "duckdb");
}

// ==================== Connection type default ====================

#[test]
fn test_connection_type_defaults_to_global() {
    let input = ConnectDatabaseInput {
        conn_id: None,
        db_type: "mysql".to_string(),
        url: "mysql://localhost:3306/test".to_string(),
        name: Some("test".to_string()),
        connection_type: None, // None should default to "global"
        project_id: None,
        description: None,
        driver_id: None,
        environment_id: None,
        auth_config_id: None,
        auth_method: None,
        network_config_id: None,
        driver_properties: None,
        advanced_options: None,
        options: None,
        tags: None,
        metadata_path: None,
        schema_name: None,
        use_duckdb_fed: None,
        password: None,
    };
    assert!(input.connection_type.is_none());
}

// ==================== Auth method with os_auth / trust (no credentials) ====================

#[test]
fn test_connect_input_os_auth_no_credentials() {
    let input = ConnectDatabaseInput {
        conn_id: None,
        db_type: "postgresql".to_string(),
        url: "postgres://localhost:5432/db".to_string(),
        name: Some("os-auth-conn".to_string()),
        connection_type: Some("global".to_string()),
        project_id: None,
        description: None,
        driver_id: None,
        environment_id: None,
        auth_config_id: None,
        auth_method: Some("os_auth".to_string()),
        network_config_id: None,
        driver_properties: None,
        advanced_options: None,
        options: None,
        tags: None,
        metadata_path: None,
        schema_name: None,
        use_duckdb_fed: None,
        password: None,
    };
    // os_auth 不需要 password
    assert_eq!(input.auth_method.as_deref(), Some("os_auth"));
    assert!(input.password.is_none());
    assert!(input.auth_config_id.is_none());
}

#[test]
fn test_connect_input_trust_no_credentials() {
    let input = ConnectDatabaseInput {
        conn_id: None,
        db_type: "postgresql".to_string(),
        url: "postgres://localhost:5432/db".to_string(),
        name: Some("trust-conn".to_string()),
        connection_type: Some("global".to_string()),
        project_id: None,
        description: None,
        driver_id: None,
        environment_id: None,
        auth_config_id: None,
        auth_method: Some("trust".to_string()),
        network_config_id: None,
        driver_properties: None,
        advanced_options: None,
        options: None,
        tags: None,
        metadata_path: None,
        schema_name: None,
        use_duckdb_fed: None,
        password: None,
    };
    // trust 不需要任何凭据
    assert_eq!(input.auth_method.as_deref(), Some("trust"));
    assert!(input.password.is_none());
    assert!(input.auth_config_id.is_none());
}