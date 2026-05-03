-- 作用：单个项目所有配置，只引用插件，不存储插件
-- 项目信息表
CREATE TABLE IF NOT EXISTS project (
    id              TEXT PRIMARY KEY,                     -- 项目ID
    name            TEXT NOT NULL,                        -- 项目名称
    description     TEXT,                                 -- 项目描述
    created_at      INTEGER NOT NULL,                    -- 创建时间
    updated_at      INTEGER NOT NULL,                    -- 更新时间
    COMMENT '项目基础信息'
);

-- 数据库连接表
CREATE TABLE IF NOT EXISTS connections (
    id              TEXT PRIMARY KEY,                     -- 连接ID
    driver_id       TEXT NOT NULL,                        -- 驱动ID
    name            TEXT NOT NULL,                        -- 连接名称
    host            TEXT,                                 -- 主机地址
    port            INTEGER,                              -- 端口
    db_name         TEXT,                                 -- 数据库名
    schema_name     TEXT,                                 -- Schema名
    username        TEXT,                                 -- 用户名
    extra_params    JSON,                                 -- 扩展参数
    use_duckdb_fed  INTEGER DEFAULT 0,                   -- 是否启用DuckDB联邦加速
    metadata_path   TEXT NOT NULL,                       -- 元数据文件路径
    is_enabled      INTEGER DEFAULT 1,                   -- 是否启用
    created_at      INTEGER NOT NULL,                    -- 创建时间
    updated_at      INTEGER NOT NULL,                    -- 更新时间
    COMMENT '项目内数据库连接配置'
);
CREATE INDEX IF NOT EXISTS idx_conn_driver ON connections(driver_id);

-- 查询历史表（企业级）
CREATE TABLE IF NOT EXISTS query_history (
    id              TEXT PRIMARY KEY,                     -- 历史ID
    connection_id   TEXT,                                 -- 关联连接ID
    database_name   TEXT,                                 -- 执行时数据库
    schema_name     TEXT,                                 -- 执行时Schema
    sql             TEXT NOT NULL,                        -- 执行SQL
    sql_hash        TEXT NOT NULL,                        -- SQL哈希值（去重）
    exec_mode       TEXT NOT NULL,                        -- 执行模式：native/duckdb_fed
    category        TEXT NOT NULL,                        -- 类型：query/ddl/dml
    success         INTEGER DEFAULT 1,                   -- 是否成功
    error_message   TEXT,                                 -- 错误信息
    duration_ms     INTEGER,                              -- 耗时（毫秒）
    rows_returned   BIGINT,                               -- 返回行数
    rows_affected   BIGINT,                               -- 影响行数
    created_at      INTEGER NOT NULL,                    -- 执行时间
    is_pinned       INTEGER DEFAULT 0,                   -- 是否固定置顶
    COMMENT 'SQL执行历史，支持去重、分析、慢查询'
);
CREATE INDEX IF NOT EXISTS idx_qh_conn ON query_history(connection_id);
CREATE INDEX IF NOT EXISTS idx_qh_time ON query_history(created_at);

-- 项目收藏SQL表
CREATE TABLE IF NOT EXISTS saved_queries (
    id              TEXT PRIMARY KEY,                     -- 收藏ID
    connection_id   TEXT,                                 -- 关联连接ID
    name            TEXT NOT NULL,                        -- 收藏名称
    sql             TEXT NOT NULL,                        -- SQL语句
    description     TEXT,                                 -- 描述
    group_name      TEXT,                                 -- 分组
    created_at      INTEGER NOT NULL,                    -- 创建时间
    updated_at      INTEGER NOT NULL,                    -- 更新时间
    COMMENT '项目内收藏SQL'
);

-- UI状态表
CREATE TABLE IF NOT EXISTS ui_state (
    "key"           TEXT PRIMARY KEY,                     -- 状态键
    "value"         JSON NOT NULL,                       -- 状态值
    updated_at      INTEGER NOT NULL,                    -- 更新时间
    COMMENT '界面状态：导航树、布局、展开状态'
);

-- DuckDB项目配置表
CREATE TABLE IF NOT EXISTS duckdb_config (
    id                      INTEGER PRIMARY KEY DEFAULT 1,
    cache_ttl_hours         INTEGER DEFAULT 24,           -- 缓存过期时间
    max_cache_size_gb       REAL DEFAULT 10.0,           -- 最大缓存空间
    auto_update_extensions  INTEGER DEFAULT 0,           -- 是否自动更新扩展
    federation_read_only    INTEGER DEFAULT 1,           -- 联邦查询是否只读
    COMMENT '项目DuckDB分析引擎配置'
);

-- ===========================================================================
-- ======================== 项目插件引用（锁定版本） ========================
-- ===========================================================================

-- 项目使用插件表
CREATE TABLE IF NOT EXISTS project_used_plugins (
    plugin_code         TEXT NOT NULL,                    -- 插件标识
    plugin_version      TEXT NOT NULL,                    -- 锁定精确版本
    enabled             INTEGER DEFAULT 1,               -- 项目内是否启用
    required            INTEGER DEFAULT 1,               -- 是否必需插件
    PRIMARY KEY (plugin_code, plugin_version),
    COMMENT '项目引用的插件，仅记录标识+版本，不存储插件'
);

-- 项目插件配置表
CREATE TABLE IF NOT EXISTS project_plugin_config (
    plugin_code         TEXT NOT NULL,
    plugin_version      TEXT NOT NULL,
    "key"               TEXT NOT NULL,
    "value"             JSON,
    updated_at          INTEGER NOT NULL,
    PRIMARY KEY (plugin_code, plugin_version, "key"),
    COMMENT '项目级插件配置，隔离不冲突'
);

-- 插件连接绑定表
CREATE TABLE IF NOT EXISTS plugin_connection_binding (
    id                  TEXT PRIMARY KEY,
    plugin_code         TEXT NOT NULL,
    plugin_version      TEXT NOT NULL,
    connection_id       TEXT NOT NULL,
    config_json         JSON,                             -- 绑定配置
    enabled             INTEGER DEFAULT 1,
    UNIQUE(plugin_code, plugin_version, connection_id),
    COMMENT '插件与数据库连接绑定：SSH、代理、加密等'
);