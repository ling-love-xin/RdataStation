# 数据流设计

> 版本：v2.0
> 最后更新：2026-05-09
> 状态：✅ 实际代码对齐

## 概述

本文档描述 RdataStation 后端的数据流，包括请求处理流程、元数据查询流程、错误传播机制等。

## 一、SQL 执行流程（实际）

```
Frontend (Vue3)
    │
    │ invoke('execute_sql', { connId: "conn_123", sql: "SELECT * FROM users" })
    ▼
Tauri Runtime
    │
    ▼
commands/sql_commands.rs
    │ 1. 输入校验（SQL 非空检查）
    │ 2. 获取 ConnectionManager
    │ 3. manager.get_connection(conn_id)
    │    → Option<DynDatabase>
    ▼
ConnectionManager
    │ 返回 Arc<dyn Database>
    ▼
commands/sql_commands.rs
    │ 4. db.query(sql)
    │ 5. 记录 SQL 历史（history_store）
    ▼
driver/native/{mysql,postgres,sqlite,duckdb}.rs
    │ Database::query(sql)
    │ → Pool::acquire() → 执行 SQL
    │ → 行数据 → Vec<Value> → QueryResult
    ▼
commands/sql_commands.rs
    │ 返回 QueryResult（自动序列化为 JSON）
    ▼
Frontend (Vue3)
```

### 实际代码路径

| 步骤     | 文件                                                                                                                                       |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| 命令入口 | [commands/sql_commands.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/commands/sql_commands.rs)                  |
| 连接获取 | [services/connection_manager.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/services/connection_manager.rs) |
| SQL 执行 | [sql_service.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/services/sql_service.rs)                        |
| 驱动实现 | [driver/native/](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/native/)                                 |

## 二、连接创建流程（实际）

```
Frontend (Vue3)
    │ invoke('create_connection', { config })
    ▼
commands/connection_commands.rs
    │
    ▼
ConnectionService::connect(config)
    │
    ├── 1. 解析连接 URL（parse_url）
    ├── 2. create_database(db_type, url)
    │       └── match db_type {  ← P0: 硬编码 4 种匹配
    │             "mysql" => MySqlDatabase::new(url)
    │             "postgres" => PostgresDatabase::new(url)
    │             ...
    │           }
    ├── 3. 注册到 ConnectionManager（add_connection）
    ├── 4. 初始化元数据缓存（initialize_connection_metadata）
    │       └── MetadataCacheManager::new(conn_id, type, project_path)
    │       └── 创建 conn_{id}.sqlite 缓存文件
    └── 5. 返回 conn_id + DataSourceMeta
```

### P0 问题：create_database 硬编码

**路径**: [connection_service.rs#L256](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/services/connection_service.rs#L256)

当前 `ConnectionService::create_database()` 绕过了 `DataSourceRouter` 和 `DriverRegistry`，直接硬编码匹配 4 种数据库类型。

**改进后流程**：

```
ConnectionService::connect(config)
    │
    ▼
DataSourceRouter::route(config)
    │ DriverRegistry::get(config.driver)
    ▼
DriverFactory::create(config)
    │ 工厂负责创建具体数据库实例
    ▼
DynDatabase
```

## 三、Schema 浏览流程

```
Frontend (Navigator Panel)
    │ 用户展开树节点
    ▼
Tauri Command
    │ get_connection(conn_id) → DynDatabase
    ▼
Database trait 方法
    │
    ├── list_databases()  → Vec<String>
    ├── list_schemas(db)  → Vec<String>
    ├── list_tables(db, schema) → Vec<SchemaObject { name, kind, children: None }>
    └── list_columns(db, schema, table) → Vec<SchemaObject { name, kind, children: None }>
```

> ⚠️ **P0-4**：`SchemaObject` 只有 `name`/`kind`/`children`，缺少列类型/注释等详情。

### 元数据缓存流程

```
首次请求:
    frontend → list_tables
        → MetadataCacheManager::get_tables(conn_id, db, schema)
        → 缓存未命中 → 执行实际 SQL → 写入缓存 → 返回

后续请求（缓存命中）:
    frontend → list_tables
        → MetadataCacheManager::get_tables(conn_id, db, schema)
        → 缓存命中 → 直接返回（无需查询数据库）

缓存预热:
    frontend → start_cache_warming(conn_id)
        → 后台任务：遍历所有 schema → 预取 tables + columns
        → 前端轮询 get_warming_progress
```

## 四、双层存储读写流程

### 连接信息存储（SQLite）

```
创建连接
    │
    ▼
ConnectionService::connect()
    │
    ├── ConnectionStore::save(conn_info)
    │   └── INSERT INTO connections (global.db)
    │
    └── ProjectConnectionStore::save(conn_info)
        └── INSERT INTO connections (project.db)
```

### 分析查询（DuckDB 加速）

```
execute_duckdb_accelerated(sql)
    │
    ▼
DuckDB 分析引擎
    │
    ├── ATTACH 'mysql://...' AS mysql_db
    ├── ATTACH 'postgres://...' AS pg_db
    │
    ├── SELECT * FROM mysql_db.orders
    │   JOIN pg_db.users ON ...
    │   (DuckDB 自动优化 + 谓词下推)
    │
    └── 返回 QueryResult
```

## 五、启动流程

```
main.rs → lib.rs::run()
    │
    ├── 1. register_drivers()
    │       AutoDriverRegistrar::auto_register()
    │       → DriverRegistry::register(MySqlDriverFactory)
    │       → DriverRegistry::register(PostgresDriverFactory)
    │       → DriverRegistry::register(SqliteDriverFactory)
    │       → DriverRegistry::register(DuckDbDriverFactory)
    │
    ├── 2. initialize_global_system()
    │       → 创建 system/global.db (SQLite)
    │       → 创建 system/analytics/global.duckdb (DuckDB)
    │       → 执行 MigrationType::GlobalSqlite
    │       → 执行 MigrationType::GlobalDuckDB
    │
    ├── 3. init_driver_manager()
    │       → 初始化全局 DriverManager
    │
    └── 4. Tauri Builder
            → manage(ProjectState)
            → manage(AnalyticsResourceState)
            → manage(ScratchpadState)
            → invoke_handler(70+ commands)
            → run()
```

## 六、错误传播机制

```
driver/native/{db}.rs
    │ sqlx::Error / rusqlite::Error
    ▼
    ? (自动转换)
    ▼
CoreError
    ├── CoreError::Connection(ConnectionError::Refused { ... })
    ├── CoreError::Database(DatabaseError::QueryError { ... })
    └── CoreError::Common(CommonError::General { ... })
    ▼
Tauri Command
    │ Result<T, CoreError>
    │ Tauri 自动序列化为 JSON ErrorResponse
    ▼
Frontend
    │ { code: "CONN_REFUSED", message: "..." }
```

### 错误类型层次

```
CoreError
├── ConnectionError      # 连接相关错误
│   ├── Refused         # 连接被拒绝
│   ├── Timeout         # 连接超时
│   ├── InvalidConfig   # 配置无效
│   ├── DriverNotFound  # 驱动未找到
│   └── Network         # 网络错误
├── DatabaseError        # 数据库相关错误
│   ├── QueryError      # 查询错误
│   ├── Driver          # 驱动错误
│   ├── ConstraintViolation  # 约束冲突
│   └── SyntaxError     # 语法错误
└── CommonError          # 通用错误
    ├── General         # 一般错误
    ├── NotSupported    # 不支持的操作
    └── ValidationError # 校验错误
```

## 七、关键约束

| 约束                | 说明                                                       |
| ------------------- | ---------------------------------------------------------- |
| ❌ 禁止跨层调用     | Command 不能直接调用 driver/native                         |
| ❌ 禁止 unwrap      | 所有生产代码使用 `?` 操作符                                |
| ✅ QueryResult 格式 | 必须包含 `columns: Vec<String>` 和 `rows: Vec<Vec<Value>>` |
| ✅ services 层      | 只调用 connection_manager / driver，不碰 driver/native     |
| ✅ Arrow IPC        | 插件通信使用 Arrow RecordBatch                             |
