-- 作用：每个连接独立，存储表 / 视图 / 字段 / 函数 / 索引 / 触发器
-- 元数据总表
CREATE TABLE IF NOT EXISTS metadata (
    id              TEXT PRIMARY KEY,
    obj_type        TEXT NOT NULL,    -- 对象类型：table/view/column/index/function/trigger/fk/pk
    database_name   TEXT NOT NULL,    -- 数据库名
    schema_name     TEXT NOT NULL,    -- Schema名
    table_name      TEXT NOT NULL,    -- 表名
    name            TEXT,             -- 列名/索引名/函数名
    data_type       TEXT,             -- 数据类型
    is_nullable     INTEGER,          -- 是否可为空
    is_primary      INTEGER DEFAULT 0,-- 是否主键
    is_unique       INTEGER DEFAULT 0,-- 是否唯一
    comment         TEXT,             -- 注释
    definition      TEXT,             -- 定义：视图SQL、函数体、触发器体
    extra           JSON,             -- 扩展信息
    last_sync       INTEGER NOT NULL, -- 最后同步时间
    COMMENT '数据库全量元数据：表、视图、列、索引、函数、存储过程、触发器'
);

CREATE INDEX IF NOT EXISTS idx_meta_obj ON metadata(obj_type);
CREATE INDEX IF NOT EXISTS idx_meta_search ON metadata(database_name, schema_name, table_name, name);
CREATE INDEX IF NOT EXISTS idx_meta_schema ON metadata(database_name, schema_name);

-- 元数据同步日志表
CREATE TABLE IF NOT EXISTS sync_log (
    id              TEXT PRIMARY KEY,
    start_at        INTEGER NOT NULL,
    end_at          INTEGER NOT NULL,
    success         INTEGER DEFAULT 1,
    message         TEXT,
    objects_fetched INTEGER DEFAULT 0,
    COMMENT '元数据同步记录'
);