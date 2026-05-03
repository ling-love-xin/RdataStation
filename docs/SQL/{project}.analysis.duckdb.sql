-- 作用：结果缓存、联邦加速、全局元数据搜索

-- 查询结果缓存表
CREATE TABLE IF NOT EXISTS query_cache (
    cache_id        TEXT PRIMARY KEY,
    connection_id   TEXT,               -- 关联连接ID
    sql_hash        TEXT,               -- SQL哈希
    dataset_name    TEXT,               -- 数据集名称
    storage_mode    TEXT,               -- 存储模式：session/persistent
    rows            BIGINT,             -- 行数
    bytes           BIGINT,             -- 占用字节
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    COMMENT '查询结果缓存'
);

-- 联邦连接状态表
CREATE TABLE IF NOT EXISTS federation (
    connection_id   TEXT PRIMARY KEY,
    driver_type     TEXT NOT NULL,      -- 驱动类型
    attach_name     TEXT NOT NULL,      -- DuckDB内别名
    last_attached   TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    COMMENT 'DuckDB联邦连接挂载状态'
);

-- 文件数据集表
CREATE TABLE IF NOT EXISTS file_dataset (
    file_id         TEXT PRIMARY KEY,
    file_path       TEXT,               -- 文件路径
    file_type       TEXT,               -- 文件类型：csv/parquet/excel
    table_name      TEXT UNIQUE,        -- 映射表名
    rows            BIGINT,
    bytes           BIGINT,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    COMMENT '外部文件分析索引'
);

-- 用户自建分析表
CREATE TABLE IF NOT EXISTS user_dataset (
    table_name      TEXT PRIMARY KEY,
    source_type     TEXT,  -- 来源：query/file/federation
    description     TEXT,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    COMMENT '用户手动创建的分析表'
);