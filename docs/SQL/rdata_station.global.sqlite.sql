-- 作用：整个应用只有一个，管理所有项目、所有插件、全局配置、驱动

-- 应用基础信息表
CREATE TABLE IF NOT EXISTS app_info (
    version         TEXT PRIMARY KEY,                     -- 应用版本号
    installed_at    INTEGER NOT NULL,                    -- 安装时间戳
    last_run_at     INTEGER NOT NULL,                    -- 最后运行时间戳
    COMMENT '应用基础信息，仅用于版本和运行记录'
);

-- 项目索引表（记录所有项目）
CREATE TABLE IF NOT EXISTS projects (
    id              TEXT PRIMARY KEY,                     -- 项目唯一ID
    name            TEXT NOT NULL,                        -- 项目名称
    path            TEXT NOT NULL,                        -- 项目物理存储路径
    description     TEXT,                                 -- 项目描述
    created_at      INTEGER NOT NULL,                    -- 创建时间
    updated_at      INTEGER NOT NULL,                    -- 更新时间
    last_open_at    INTEGER,                             -- 最后打开时间
    COMMENT '全局项目索引，记录所有项目位置'
);
CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name);

-- 全局数据库驱动模板表
CREATE TABLE IF NOT EXISTS global_drivers (
    driver_id       TEXT PRIMARY KEY,                     -- 驱动唯一标识
    display_name    TEXT NOT NULL,                        -- 驱动显示名称
    driver_type     TEXT NOT NULL,                        -- 驱动类型：mysql/postgres/mssql等
    default_config  JSON NOT NULL,                       -- 默认配置参数
    is_builtin      INTEGER DEFAULT 1,                   -- 是否内置驱动
    enabled         INTEGER DEFAULT 1,                   -- 是否启用
    COMMENT '全局数据库驱动管理，所有项目共享'
);

-- 全局系统设置表
CREATE TABLE IF NOT EXISTS global_settings (
    "key"           TEXT PRIMARY KEY,                     -- 设置项键名
    "value"         JSON,                                 -- 设置项值
    updated_at      INTEGER NOT NULL,                    -- 更新时间
    COMMENT '全局系统配置，主题、代理、编辑器等'
);

-- 全局收藏SQL表
CREATE TABLE IF NOT EXISTS global_saved_queries (
    id              TEXT PRIMARY KEY,                     -- 收藏ID
    name            TEXT NOT NULL,                        -- 收藏名称
    sql             TEXT NOT NULL,                        -- SQL语句
    group_name      TEXT,                                 -- 分组名称
    driver_id       TEXT,                                 -- 适用驱动ID
    created_at      INTEGER NOT NULL,                    -- 创建时间
    COMMENT '全局通用收藏SQL，所有项目可用'
);

-- 凭据存储表（不存密码，仅存系统密钥链ID）
CREATE TABLE IF NOT EXISTS credential_slots (
    id              TEXT PRIMARY KEY,                     -- 凭据槽ID
    label           TEXT,                                 -- 凭据名称
    system_key_id   TEXT NOT NULL,                        -- 系统密钥链中的ID
    created_at      INTEGER NOT NULL,                    -- 创建时间
    COMMENT '凭据索引，密码存储在系统密钥链，不入库'
);

-- ===========================================================================
-- ======================== 全局插件中心（唯一权威） =========================
-- ===========================================================================

-- 插件注册表
CREATE TABLE IF NOT EXISTS plugins (
    id              TEXT PRIMARY KEY,                     -- 插件唯一ID
    code            TEXT NOT NULL,                        -- 插件标识：如 ssh-tunnel、ai-suggest
    name            TEXT NOT NULL,                        -- 插件名称
    version         TEXT NOT NULL,                        -- 精确版本号：1.2.3
    author          TEXT,                                 -- 作者
    description     TEXT,                                 -- 插件描述
    repo_url        TEXT,                                 -- 插件下载地址
    plugin_type     TEXT NOT NULL,                        -- 类型：driver/ui/export/security
    manifest_json   JSON NOT NULL,                       -- 插件完整清单
    install_path    TEXT NOT NULL,                       -- 插件安装路径
    is_enabled      INTEGER DEFAULT 1,                   -- 全局是否启用
    is_builtin      INTEGER DEFAULT 0,                   -- 是否内置插件
    installed_at    INTEGER NOT NULL,                    -- 安装时间
    updated_at      INTEGER NOT NULL,                    -- 更新时间
    UNIQUE(code, version),                               -- 插件+版本唯一约束
    COMMENT '全局插件注册表，所有插件只在这里存储一份'
);

-- 插件依赖表
CREATE TABLE IF NOT EXISTS plugin_dependencies (
    plugin_id       TEXT NOT NULL,                        -- 主插件ID
    dep_code        TEXT NOT NULL,                        -- 依赖插件标识
    dep_version_range TEXT NOT NULL,                      -- 依赖版本范围：^1.0.0
    is_optional     INTEGER DEFAULT 0,                   -- 是否可选依赖
    PRIMARY KEY (plugin_id, dep_code),
    COMMENT '插件依赖关系，自动安装依赖'
);

-- 插件全局配置表
CREATE TABLE IF NOT EXISTS plugin_global_config (
    plugin_id       TEXT NOT NULL,                        -- 插件ID
    "key"           TEXT NOT NULL,                        -- 配置键
    "value"         JSON,                                 -- 配置值
    updated_at      INTEGER NOT NULL,                    -- 更新时间
    PRIMARY KEY (plugin_id, "key"),
    COMMENT '插件全局配置，所有项目共享'
);