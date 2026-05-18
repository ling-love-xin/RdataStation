# 新增数据源模块：后端架构 · 设计 · 开发 · 接口文档

> 版本：v1.0
> 初稿日期：2026-05-18
> 对应后端版本：R25+

---

## 概述

新增数据源模块为 RdataStation 提供可插拔的数据库驱动管理体系，支持：

- **驱动定义**：全局驱动目录（`global.db.drivers`）
- **驱动安装**：按需下载外部驱动文件（JDBC/WASM/Python）
- **项目隔离**：每个项目独立声明使用哪些驱动（`project_drivers` 表）
- **跨机器迁移**：项目迁移后自动检测缺失驱动，引导安装
- **环境与策略**：生产/预发布/开发环境 + 安全策略
- **可复用配置**：认证配置、网络配置独立管理

---

## 一、核心设计原则

### 1.1 驱动可见性：严格项目隔离

```
┌──────────────────────────────────────────────────────────────┐
│  铁律：项目 B 没有安装 Oracle 驱动 = 项目 B 看不到 Oracle     │
│                                                              │
│  实现方式：project_drivers 表（在项目 meta.db 中）             │
│  - 新项目创建时，只自动启用 4 个内置 Native 驱动               │
│  - Oracle JDBC 需要用户手动 "安装到项目" 才会出现条目          │
│  - 没有任何 project_drivers 条目 = 前端驱动列表不显示          │
└──────────────────────────────────────────────────────────────┘
```

### 1.2 三层驱动生命周期

```
 ┌──────────┐      ┌──────────┐      ┌──────────┐
 │ 1. 已定义 │ ──→  │ 2. 已安装 │ ──→  │ 3. 已启用 │ ──→ 可创建连接
 │ (Defined)│      │(Installed)│     │ (Enabled) │
 └──────────┘      └──────────┘      └──────────┘
 global.db         global.db         project
 .drivers          .driver_files     meta.db
                   + 文件系统        .project_drivers

 例：Oracle JDBC
 Step 1: drivers 表有 oracle-jdbc 定义（全局目录）            → 用户可以从 "驱动市场" 看到
 Step 2: 下载 ojdbc8.jar 到本机，注册到 driver_files         → 本机已安装
 Step 3: 项目 A 在 project_drivers 中启用 oracle-jdbc        → 项目 A 可以创建 Oracle 连接
         项目 B 的 project_drivers 中没有 oracle-jdbc        → 项目 B 看不到 Oracle 🔒
```

### 1.3 Native vs External 驱动

| DriverKind | 驱动文件来源 | 需要 driver_files? | 需要下载? | 迁移行为 |
|------------|-------------|-------------------|----------|---------|
| `Native` | Rust 二进制内置 (sqlx/rusqlite/duckdb-rs) | ❌ 不需要 | ❌ 不需要 | 透传，无额外操作 |
| `Jdbc` | .jar 文件 | ✅ 需要 | ✅ 需要 | 检查文件 → 缺失时引导下载 |
| `Odbc` | 系统 ODBC 驱动 | ✅ 需要 | ✅ 需要 | 检查系统驱动 |
| `Wasm` | .wasm 文件 | ✅ 需要 | ✅ 需要 | 检查文件 → 缺失时引导下载 |
| `Python` | .py 脚本 | ✅ 需要 | ✅ 需要 | 检查文件 → 缺失时引导下载 |

---

## 二、场景验证

### 场景 A：项目级驱动隔离（正常运行）

```
前提：全球驱动目录 global.db.drivers 中有 oracle-jdbc 定义

机器 M 上：

项目 A（已安装 Oracle JDBC）              项目 B（从未安装 Oracle）
  .RSmeta/meta.db                           .RSmeta/meta.db
    project_drivers:                          project_drivers:
      ✅ mysql-native                          ✅ mysql-native
      ✅ postgres-native                       ✅ postgres-native
      ✅ sqlite-native                         ✅ sqlite-native
      ✅ duckdb-native                         ✅ duckdb-native
      ✅ oracle-jdbc     ← 用户主动安装的      (无 oracle-jdbc)  ← 就这么干净

结果：
  项目 A 前端驱动列表：MySQL, PostgreSQL, SQLite, DuckDB, Oracle  ← 全部5个
  项目 B 前端驱动列表：MySQL, PostgreSQL, SQLite, DuckDB          ← 只有4个
  项目 B 无法创建 Oracle 连接 🔒
```

### 场景 B：项目迁移到无驱动的新机器

```
机器 X（开发机）                              机器 Y（新电脑）
─────────────────                             ─────────────────
global.db:                                    global.db:
  driver_files: oracle-jdbc ✅                  driver_files: (空，新装应用)

项目 A meta.db → ──────── 复制项目目录 ────────→ 项目 A meta.db
  project_drivers:                                project_drivers:
    oracle-jdbc ✅                                  oracle-jdbc ✅ (已迁移)

迁移后打开项目 A：
  ┌──────────────────────────────────────────────────┐
  │  项目 A 驱动自检结果：                             │
  │                                                  │
  │  MySQL      ✅ 可用 (Native，Rust 二进制内置)      │
  │  PostgreSQL ✅ 可用 (Native，Rust 二进制内置)      │
  │  SQLite     ✅ 可用 (Native，Rust 二进制内置)      │
  │  DuckDB     ✅ 可用 (Native，Rust 二进制内置)      │
  │  Oracle     ⚠️ 驱动文件未安装                      │
  │                 [安装驱动]  [暂时禁用]              │
  │                                                  │
  │  点击 [暂时禁用] → Oracle 从当前会话隐藏            │
  │  点击 [安装驱动] → 下载 ojdbc8.jar → 自动可用      │
  └──────────────────────────────────────────────────┘
```

### 场景 C：新项目默认驱动集

```
用户创建 "项目 C"
    ↓
自动在项目 C 的 project_drivers 中种子4个内置驱动：
    mysql-native      ✅
    postgres-native   ✅
    sqlite-native     ✅
    duckdb-native     ✅
    (仅此而已，无其他驱动)

项目 C 前端驱动列表：MySQL, PostgreSQL, SQLite, DuckDB
```

---

## 三、数据库表设计

### 3.1 表分布

```
 global.db（系统级）                    project meta.db（项目级）
 ─────────────────                     ──────────────────────
 data_source_types  ← 类型目录         
 drivers            ← 驱动定义目录      project_drivers  ← 项目驱动启用表 ⭐
 driver_files       ← 本机文件注册      environments     ← 项目环境
 environments       ← 全局环境          env_policies     ← 项目策略
 env_policies       ← 全局策略          auth_configs     ← 项目认证
 auth_configs       ← 全局认证          network_configs  ← 项目网络
 network_configs    ← 全局网络          
 global_connections ← 全局连接 (扩展)   connections      ← 项目连接 (扩展)
```

### 3.2 全局表 DDL

> 迁移文件：`src-tauri/migrations/global/008_add_data_source_module.sql`

```sql
-- ============================================================================
-- 1. 数据源类型目录（MySQL / PostgreSQL / Oracle / MongoDB / Kafka ...）
-- ============================================================================
CREATE TABLE IF NOT EXISTS data_source_types (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    category    TEXT NOT NULL,       -- relational | file-based | nosql | analytics | cloud | mq | http
    icon        TEXT,
    enabled     BOOLEAN DEFAULT 1,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 2. 驱动定义目录（全局注册表，所有项目共享）
--    描述驱动的元数据、连接参数 Schema、下载地址
-- ============================================================================
DROP TABLE IF EXISTS global_drivers;

CREATE TABLE IF NOT EXISTS drivers (
    id                    TEXT PRIMARY KEY,
    type_id               TEXT NOT NULL REFERENCES data_source_types(id) ON DELETE CASCADE,
    name                  TEXT NOT NULL,
    driver_kind           TEXT NOT NULL DEFAULT 'native', -- native|jdbc|odbc|wasm|adbc|http|python|js
    is_file               BOOLEAN DEFAULT 0,
    default_port          INTEGER,
    url_template          TEXT,
    download_url          TEXT,           -- ⭐ 外部驱动下载 URL
    download_checksum     TEXT,           -- ⭐ SHA256 校验和
    version               TEXT,           -- ⭐ 驱动版本号
    config_schema         TEXT NOT NULL,  -- JSON Schema：前端动态生成连接表单
    supported_auth_types  TEXT,           -- JSON 数组：支持的认证方式
    capabilities          TEXT,           -- JSON 数组：支持的能力
    enabled               BOOLEAN DEFAULT 1,
    created_at            TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_drivers_type ON drivers(type_id);
CREATE INDEX IF NOT EXISTS idx_drivers_kind ON drivers(driver_kind);
CREATE INDEX IF NOT EXISTS idx_drivers_enabled ON drivers(enabled);

-- ============================================================================
-- 3. 本机驱动文件注册表（记录本机已下载的外部驱动文件）
--    Native 驱动不需要此行
-- ============================================================================
CREATE TABLE IF NOT EXISTS driver_files (
    id           TEXT PRIMARY KEY,
    driver_id    TEXT NOT NULL REFERENCES drivers(id),
    file_path    TEXT NOT NULL,           -- 相对路径：{app_data}/RdataStation/drivers/{driver_id}/{version}/
    file_name    TEXT NOT NULL,
    file_size    INTEGER,
    checksum     TEXT,
    version      TEXT NOT NULL,
    installed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_driver_files_driver ON driver_files(driver_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_driver_files_unique ON driver_files(driver_id, version);

-- ============================================================================
-- 4. 环境表
-- ============================================================================
CREATE TABLE IF NOT EXISTS environments (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    description TEXT,
    color       TEXT,
    sort_order  INTEGER DEFAULT 0,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 5. 环境策略表
-- ============================================================================
CREATE TABLE IF NOT EXISTS environment_policies (
    id              TEXT PRIMARY KEY,
    environment_id  TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
    policy_type     TEXT NOT NULL,       -- read_only|navigation_filter|query_timeout|max_rows|ddl_blocked|dml_blocked|confirm_required
    policy_config   TEXT,
    enabled         BOOLEAN DEFAULT 1,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_env_policies_env ON environment_policies(environment_id);

-- ============================================================================
-- 6. 认证配置表（密码 AES-256-GCM 加密存储）
-- ============================================================================
CREATE TABLE IF NOT EXISTS auth_configs (
    id          TEXT PRIMARY KEY,
    name        TEXT,
    auth_type   TEXT NOT NULL,           -- password|keyfile|kerberos|oauth2|aws_iam|gcp_sa
    auth_data   TEXT NOT NULL,           -- JSON，密码字段已加密
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 7. 网络配置表（SSH 隧道 / 代理 / SSL）
-- ============================================================================
CREATE TABLE IF NOT EXISTS network_configs (
    id            TEXT PRIMARY KEY,
    name          TEXT,
    network_type  TEXT NOT NULL,         -- ssh|proxy|ssl|none
    config        TEXT NOT NULL,         -- JSON，具体结构依赖 network_type
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 8. 扩展 global_connections 表（ALTER TABLE，不丢失数据）
-- ============================================================================
ALTER TABLE global_connections ADD COLUMN description TEXT;
ALTER TABLE global_connections ADD COLUMN driver_id TEXT;
ALTER TABLE global_connections ADD COLUMN environment_id TEXT;
ALTER TABLE global_connections ADD COLUMN auth_config_id TEXT;
ALTER TABLE global_connections ADD COLUMN network_config_id TEXT;
ALTER TABLE global_connections ADD COLUMN driver_properties TEXT;
ALTER TABLE global_connections ADD COLUMN advanced_options TEXT;

UPDATE global_connections SET driver_id =
    CASE driver
        WHEN 'mysql'    THEN 'mysql-native'
        WHEN 'postgres' THEN 'postgres-native'
        WHEN 'sqlite'   THEN 'sqlite-native'
        WHEN 'duckdb'   THEN 'duckdb-native'
        ELSE driver || '-native'
    END
WHERE driver_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_gc_driver_id ON global_connections(driver_id);
CREATE INDEX IF NOT EXISTS idx_gc_env ON global_connections(environment_id);

-- ============================================================================
-- 9. 种子数据
-- ============================================================================
INSERT OR IGNORE INTO data_source_types (id, name, category, icon, enabled) VALUES
    ('mysql',      'MySQL',           'relational', '🐬', 1),
    ('postgresql', 'PostgreSQL',      'relational', '🐘', 1),
    ('sqlite',     'SQLite',          'file-based', '🪶', 1),
    ('duckdb',     'DuckDB',          'analytics',  '🦆', 1),
    ('mariadb',    'MariaDB',         'relational', '🦭', 1),
    ('oracle',     'Oracle',          'relational', '🔴', 1),
    ('mssql',      'SQL Server',      'relational', '🟢', 1),
    ('clickhouse', 'ClickHouse',      'analytics',  '🔵', 1),
    ('mongodb',    'MongoDB',         'nosql',      '🍃', 1),
    ('redis',      'Redis',           'nosql',      '🔶', 1);

-- 4 个内置 Native 驱动（driver_files 不需要，编译在 Rust 二进制中）
INSERT OR IGNORE INTO drivers (id, type_id, name, driver_kind, is_file, default_port,
    url_template, download_url, version, config_schema, supported_auth_types, capabilities, enabled) VALUES
('mysql-native', 'mysql', 'MySQL (Native)', 'native', 0, 3306,
 'mysql://{username}:{password}@{host}:{port}/{database}',
 NULL, 'builtin',
 '{"type":"object","properties":{"host":{"type":"string","title":"主机","default":"localhost"},"port":{"type":"integer","title":"端口","default":3306},"database":{"type":"string","title":"数据库名"},"username":{"type":"string","title":"用户名"},"password":{"type":"string","title":"密码","format":"password"}},"required":["host","port","username"]}',
 '["password","ssl"]',
 '["tree","health_check","transactions","prepared_stmts"]',
 1),

('postgres-native', 'postgresql', 'PostgreSQL (Native)', 'native', 0, 5432,
 'postgres://{username}:{password}@{host}:{port}/{database}',
 NULL, 'builtin',
 '{"type":"object","properties":{"host":{"type":"string","title":"主机","default":"localhost"},"port":{"type":"integer","title":"端口","default":5432},"database":{"type":"string","title":"数据库名"},"username":{"type":"string","title":"用户名"},"password":{"type":"string","title":"密码","format":"password"}},"required":["host","port","database","username"]}',
 '["password","ssl","kerberos"]',
 '["tree","health_check","schema_filter","transactions","prepared_stmts"]',
 1),

('sqlite-native', 'sqlite', 'SQLite (Native)', 'native', 1, NULL,
 'sqlite://{file_path}',
 NULL, 'builtin',
 '{"type":"object","properties":{"file_path":{"type":"string","title":"数据库文件路径","format":"file"}},"required":["file_path"]}',
 '["none"]',
 '["tree","transactions","in_memory"]',
 1),

('duckdb-native', 'duckdb', 'DuckDB (Native)', 'native', 1, NULL,
 'duckdb://{file_path}',
 NULL, 'builtin',
 '{"type":"object","properties":{"file_path":{"type":"string","title":"数据库文件路径","format":"file"},"memory_limit":{"type":"string","title":"内存限制"}},"required":["file_path"]}',
 '["none"]',
 '["tree","health_check","arrow","federated","transactions"]',
 1);
```

### 3.3 项目级表 DDL

> 迁移文件：`src-tauri/migrations/project_meta/010_add_data_source_module.sql`

```sql
-- ============================================================================
-- 1. 项目驱动启用表（⭐ 核心隔离表）
--    决定当前项目能看到哪些驱动
--    新项目只自动种子4个 Native 驱动
--    没有记录 = 前端不显示
-- ============================================================================
CREATE TABLE IF NOT EXISTS project_drivers (
    id           TEXT PRIMARY KEY,
    driver_id    TEXT NOT NULL,              -- 逻辑引用 global.db.drivers.id (跨DB，Rust校验)
    enabled      BOOLEAN DEFAULT 1,
    installed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_project_drivers_unique ON project_drivers(driver_id);
CREATE INDEX IF NOT EXISTS idx_project_drivers_enabled ON project_drivers(enabled);

-- ============================================================================
-- 2. 环境表（项目独立）
-- ============================================================================
CREATE TABLE IF NOT EXISTS environments (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    description TEXT,
    color       TEXT,
    sort_order  INTEGER DEFAULT 0,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 3. 环境策略表（项目独立）
-- ============================================================================
CREATE TABLE IF NOT EXISTS environment_policies (
    id              TEXT PRIMARY KEY,
    environment_id  TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
    policy_type     TEXT NOT NULL,
    policy_config   TEXT,
    enabled         BOOLEAN DEFAULT 1,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_env_policies_env ON environment_policies(environment_id);

-- ============================================================================
-- 4. 认证配置表（项目独立）
-- ============================================================================
CREATE TABLE IF NOT EXISTS auth_configs (
    id          TEXT PRIMARY KEY,
    name        TEXT,
    auth_type   TEXT NOT NULL,
    auth_data   TEXT NOT NULL,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 5. 网络配置表（项目独立）
-- ============================================================================
CREATE TABLE IF NOT EXISTS network_configs (
    id            TEXT PRIMARY KEY,
    name          TEXT,
    network_type  TEXT NOT NULL,
    config        TEXT NOT NULL,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 6. 扩展 connections 表（ALTER TABLE，不丢失数据）
-- ============================================================================
ALTER TABLE connections ADD COLUMN description TEXT;
ALTER TABLE connections ADD COLUMN driver_id TEXT;
ALTER TABLE connections ADD COLUMN environment_id TEXT;
ALTER TABLE connections ADD COLUMN auth_config_id TEXT;
ALTER TABLE connections ADD COLUMN network_config_id TEXT;
ALTER TABLE connections ADD COLUMN driver_properties TEXT;
ALTER TABLE connections ADD COLUMN advanced_options TEXT;

UPDATE connections SET driver_id =
    CASE driver
        WHEN 'mysql'    THEN 'mysql-native'
        WHEN 'postgres' THEN 'postgres-native'
        WHEN 'sqlite'   THEN 'sqlite-native'
        WHEN 'duckdb'   THEN 'duckdb-native'
        ELSE driver || '-native'
    END
WHERE driver_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_conn_driver_id ON connections(driver_id);
CREATE INDEX IF NOT EXISTS idx_conn_env ON connections(environment_id);
```

### 3.4 表关系全景

```
 global.db                                       project meta.db
 ─────────                                       ───────────────

 data_source_types (1)
       │
       │ 1:N (global FK)
       ▼
 drivers (N)  ←──────── driver_id (逻辑引用) ──────────┐
       │                                                │
       │ 1:N (global FK)                               │
       ▼                                                │
 driver_files (0..N)                                   │
                                                        │
 environments ────── 独立副本 ────── environments       │
       │                                    │           │
       │ 1:N (同DB FK)                     │ 1:N       │
       ▼                                    ▼           │
 environment_policies              environment_policies │
                                                        │
 auth_configs ──────── 独立副本 ────── auth_configs     │
 network_configs ───── 独立副本 ────── network_configs  │
                                                        │
 global_connections                connections          │
  ├── FK → environments  (global)   ├── FK → environments  (project)  │
  ├── FK → auth_configs  (global)   ├── FK → auth_configs  (project)  │
  ├── FK → network_configs(global)  ├── FK → network_configs(project) │
  └── driver_id (逻辑→drivers)      └── driver_id (逻辑→drivers)      │
                                                        │
                                   project_drivers ⭐    │
                                    └── driver_id (逻辑→drivers)       │
```

---

## 四、Rust 模块设计

### 4.1 文件变更清单

```
新增文件：
  src-tauri/src/core/persistence/driver_store.rs      — drivers + driver_files CRUD (仅 global)
  src-tauri/src/core/persistence/env_store.rs         — environments + policies CRUD (纯函数)
  src-tauri/src/core/persistence/auth_store.rs        — auth_configs CRUD (纯函数)
  src-tauri/src/core/persistence/network_store.rs     — network_configs CRUD (纯函数)
  src-tauri/src/core/services/driver_service.rs       — 驱动发现/安装/验证
  src-tauri/src/commands/data_source_commands.rs      — 前端 API 命令

修改文件：
  src-tauri/src/core/driver/registry/descriptors.rs   — DriverDescriptor +7 字段
  src-tauri/src/core/persistence/global_db.rs         — 集成新表 CRUD；GlobalConnectionInfo +7 字段
  src-tauri/src/core/persistence/project_connection_store.rs — ProjectConnection +7 字段
  src-tauri/src/core/persistence/project_db.rs        — 集成 project_drivers CRUD
  src-tauri/src/core/persistence/mod.rs               — 注册新子模块
  src-tauri/src/core/services/connection_service.rs   — 连接创建时驱动校验
  src-tauri/src/core/project/store.rs                 — 项目打开时驱动自检
  src-tauri/src/commands/connection_commands.rs       — 新字段传递
  src-tauri/src/lib.rs                                — 注册新 commands

迁移文件：
  src-tauri/migrations/global/008_add_data_source_module.sql
  src-tauri/migrations/project_meta/010_add_data_source_module.sql
```

### 4.2 Rust 结构体

```rust
// ---------- 驱动相关 ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceType {
    pub id: String,
    pub name: String,
    pub category: String,
    pub icon: Option<String>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Driver {
    pub id: String,
    pub type_id: String,
    pub name: String,
    pub driver_kind: String,
    pub is_file: bool,
    pub default_port: Option<i32>,
    pub url_template: Option<String>,
    pub download_url: Option<String>,
    pub download_checksum: Option<String>,
    pub version: Option<String>,
    pub config_schema: String,
    pub supported_auth_types: Option<String>,
    pub capabilities: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverFile {
    pub id: String,
    pub driver_id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
    pub version: String,
    pub installed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverAvailability {
    Ready,                                    // 可用
    NotInstalled { download_url: String },    // 未安装但可下载
    NotEnabled,                               // 未在项目中启用
    NotDefined,                               // 驱动定义不存在
}

// ---------- 环境相关 ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentPolicy {
    pub id: String,
    pub environment_id: String,
    pub policy_type: String,    // read_only|navigation_filter|query_timeout|max_rows|ddl_blocked|dml_blocked|confirm_required
    pub policy_config: Option<String>,
    pub enabled: bool,
    pub created_at: String,
}

// ---------- 认证/网络配置 ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub id: String,
    pub name: Option<String>,
    pub auth_type: String,      // password|keyfile|kerberos|oauth2|aws_iam|gcp_sa
    pub auth_data: String,      // JSON，密码已 AES-256-GCM 加密
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub id: String,
    pub name: Option<String>,
    pub network_type: String,   // ssh|proxy|ssl|none
    pub config: String,         // JSON，结构依赖 network_type
    pub created_at: String,
    pub updated_at: String,
}

// ---------- 连接扩展字段（追加到已有结构体）----------
// GlobalConnectionInfo 和 ProjectConnection 各自新增：
// pub description: Option<String>,
// pub driver_id: Option<String>,
// pub environment_id: Option<String>,
// pub auth_config_id: Option<String>,
// pub network_config_id: Option<String>,
// pub driver_properties: Option<String>,   // JSON
// pub advanced_options: Option<String>,    // JSON
```

### 4.3 纯函数共享模式

四个 store 模块（`env_store` / `auth_store` / `network_store` / `driver_store`）设计为**纯函数**，接收 `rusqlite::Connection` 引用而非持有状态：

```rust
// env_store.rs
pub fn create_environment(conn: &Connection, env: &Environment) -> Result<(), CoreError> { ... }
pub fn list_environments(conn: &Connection) -> Result<Vec<Environment>, CoreError> { ... }
pub fn update_environment(conn: &Connection, env: &Environment) -> Result<(), CoreError> { ... }
pub fn delete_environment(conn: &Connection, id: &str) -> Result<(), CoreError> { ... }
```

调用方各自传入自己的 DB connection：

```rust
// global_db.rs 调用
let conn = self.sqlite_pool.acquire().await?;
env_store::create_environment(conn.inner()?, &env)?;

// project_db.rs 调用
let conn = self.get_connection()?;
env_store::create_environment(&conn, &env)?;
```

### 4.4 键逻辑：驱动可用性检查

```rust
// driver_service.rs

impl DriverService {
    /// 检查驱动对指定项目是否可用
    pub async fn check_driver_for_project(
        &self,
        driver_id: &str,
        project_db: &ProjectDatabaseManager,
    ) -> Result<DriverAvailability, CoreError> {
        // Step 1: 检查驱动定义是否存在
        let driver = match self.find_driver(driver_id)? {
            Some(d) => d,
            None => return Ok(DriverAvailability::NotDefined),
        };

        // Step 2: 检查项目是否启用了该驱动
        if !project_db.is_driver_enabled(driver_id)? {
            return Ok(DriverAvailability::NotEnabled);
        }

        // Step 3: Native 驱动无需文件检查
        if driver.driver_kind == "native" {
            return Ok(DriverAvailability::Ready);
        }

        // Step 4: 检查本机是否安装了驱动文件
        if self.is_driver_installed_locally(driver_id)? {
            Ok(DriverAvailability::Ready)
        } else {
            Ok(DriverAvailability::NotInstalled {
                download_url: driver.download_url.unwrap_or_default(),
            })
        }
    }
}
```

### 4.5 项目打开时驱动自检

```rust
// project/store.rs 中的 ProjectStore

impl ProjectStore {
    /// 打开项目时检测缺失驱动，返回列表给前端展示
    pub fn check_missing_drivers(&self, driver_service: &DriverService) -> Result<Vec<MissingDriver>, CoreError> {
        let enabled_drivers = self.project_db.list_enabled_project_drivers()?;
        let mut missing = Vec::new();

        for driver_id in enabled_drivers {
            if let DriverAvailability::NotInstalled { download_url } =
                driver_service.check_driver_for_project(&driver_id, &self.project_db)?
            {
                let driver = driver_service.find_driver(&driver_id)?
                    .ok_or_else(|| CoreError::common("驱动定义丢失"))?;
                missing.push(MissingDriver {
                    driver_id,
                    driver_name: driver.name,
                    download_url,
                });
            }
        }
        Ok(missing)
    }
}

pub struct MissingDriver {
    pub driver_id: String,
    pub driver_name: String,
    pub download_url: String,
}
```

---

## 五、API 接口文档

### 5.1 驱动管理

#### `get_data_source_types`
```
获取数据源类型目录（按 category 分组）
Tauri Command: get_data_source_types
参数: { category?: string }
返回: DataSourceType[]

前端用途: 左侧数据源树顶层
```

#### `get_available_drivers`
```
获取指定项目可用的驱动列表
Tauri Command: get_available_drivers
参数: { project_id?: string }
返回: { drivers: Driver[], missing: MissingDriver[] }

drivers 包含所有已启用且就绪的驱动
missing 包含已启用但本机未安装外部文件的驱动

前端用途: 
  - 新建连接时的驱动下拉
  - 项目打开时弹窗展示缺失驱动
```

#### `get_driver_detail`
```
获取单个驱动完整信息（含 config_schema）
Tauri Command: get_driver_detail
参数: { driver_id: string }
返回: Driver + { availability: string, ... }

前端用途: 按 JSON Schema 动态渲染连接表单
```

#### `install_driver`
```
下载并安装外部驱动文件
Tauri Command: install_driver
参数: { driver_id: string }
返回: { success: bool, message: string }

流程: 下载 → 校验 checksum → 写入驱动文件目录 → 注册到 driver_files 表
```

#### `enable_driver_for_project`
```
为指定项目启用一个驱动（写入 project_drivers 表）
Tauri Command: enable_driver_for_project
参数: { project_id: string, driver_id: string }
返回: { success: bool }

流程:
  1. 检查 driver_id 在 global.db.drivers 中是否存在
  2. INSERT INTO project_drivers (driver_id, enabled)
  3. 如果 driver_kind != native 且本机未安装 → 前端收到 missing 状态 → 引导安装
```

#### `disable_driver_for_project`
```
为指定项目禁用一个驱动（软操作：project_drivers.enabled = 0）
Tauri Command: disable_driver_for_project
参数: { project_id: string, driver_id: string }
返回: { success: bool }

注意: 已有连接不会自动删除，只是不能创建新连接
```

#### `get_all_drivers_catalog`
```
获取全局驱动目录（驱动市场，所有已定义的驱动）
Tauri Command: get_all_drivers_catalog
参数: { category?: string, driver_kind?: string }
返回: Driver[]

前端用途: "驱动市场" 页面，展示所有可安装的驱动
```

### 5.2 环境管理

#### `list_environments`
```
获取环境列表
Tauri Command: list_environments
参数: { scope: "global" | "project", project_id?: string }
返回: Environment[]
```

#### `create_environment` / `update_environment` / `delete_environment`
```
环境 CRUD
参数: { scope, project_id?, name, description?, color?, sort_order? }
返回: Environment / { success: bool }
```

### 5.3 环境策略

#### `list_environment_policies`
```
获取某环境的所有策略
Tauri Command: list_environment_policies
参数: { scope, project_id?, environment_id }
返回: EnvironmentPolicy[]
```

#### `create_environment_policy` / `update_environment_policy` / `delete_environment_policy`
```
策略 CRUD
参数: { scope, project_id?, environment_id, policy_type, policy_config?, enabled }
返回: EnvironmentPolicy / { success: bool }
```

### 5.4 认证配置

#### `list_auth_configs`
```
获取认证配置列表（auth_data 脱敏返回）
Tauri Command: list_auth_configs
参数: { scope, project_id?, auth_type? }
返回: { id, name, auth_type, created_at, updated_at }[]  // 不含 auth_data
```

#### `create_auth_config` / `delete_auth_config`
```
认证配置创建/删除（密码自动 AES-256-GCM 加密）
参数: { scope, project_id?, name, auth_type, auth_data: object }
返回: AuthConfig / { success: bool }
```

### 5.5 网络配置

#### `list_network_configs`
```
获取网络配置列表
Tauri Command: list_network_configs
参数: { scope, project_id?, network_type? }
返回: NetworkConfig[]
```

#### `create_network_config` / `update_network_config` / `delete_network_config`
```
网络配置 CRUD
参数: { scope, project_id?, name, network_type, config: object }
返回: NetworkConfig / { success: bool }
```

### 5.6 连接（扩展现有命令）

#### `create_connection` (扩展)
```
创建连接，新增参数
Tauri Command: create_connection
参数: {
  scope: "global" | "project",
  project_id?: string,
  name: string,
  driver_id: string,
  connection_params: {          // 驱动特定参数，遵循 drivers.config_schema
    host?: string, port?: number,
    database?: string, username?: string,
    password?: string, file_path?: string,
    ...其他
  },
  environment_id?: string,
  auth_config_id?: string,
  network_config_id?: string,
  driver_properties?: object,   // 用户自定义驱动属性 { "pool.max_connections": "10", ... }
  advanced_options?: object,    // 高级选项 { "connectionTimeout": 30, "queryTimeout": 0, ... }
  description?: string,
  tags?: string[]
}
返回: Connection

校验链:
  1. driver_id 是否在 project_drivers 中启用?
  2. driver_id 是否在 drivers 中已定义?
  3. 如果 driver_kind != native，driver_files 中是否已安装?
  4. environment_id / auth_config_id / network_config_id 是否有效?
  5. connection_params 是否符合 drivers.config_schema?
```

#### `test_connection` (扩展)
```
测试连接（不持久化）
参数: { driver_id, connection_params, network_config_id? }
返回: { success: bool, message: string, latency_ms?: number }
```

---

## 六、前端交互流程

### 6.1 新建连接

```
用户点击 [新建连接]
    ↓
① get_data_source_types() → 左侧数据源类型树（按 category 分组）
    ↓
② 用户选择 "Oracle"
    ↓
③ get_available_drivers(project_id) → 过滤 type_id=oracle → 返回可用驱动
    ├── 如果是项目 B（没安装 Oracle）→ 返回空列表 → 前端显示 "暂无可用驱动" 🔒
    └── 如果是项目 A（已安装 Oracle）→ 返回 oracle-jdbc → 继续
    ↓
④ 用户选择 "Oracle JDBC" → get_driver_detail("oracle-jdbc") → 获取 config_schema
    ↓
⑤ 按 JSON Schema 动态渲染表单：主机/端口/数据库/用户名/密码/SSL...
    ↓
⑥ 可选：选择环境 / 认证配置 / 网络配置
    ↓
⑦ 可选：点击 [测试连接]
    ↓
⑧ 点击 [保存] → create_connection()
```

### 6.2 驱动市场（安装新驱动）

```
用户进入 "驱动市场" 页面
    ↓
① get_all_drivers_catalog() → 所有已定义的驱动
    ↓
② 用户找到 "Oracle JDBC" → 点击 [安装]
    ↓
③ enable_driver_for_project(project_id, "oracle-jdbc")
    ├── 写入 project_drivers
    └── 如果未安装驱动文件 → 引导下载
    ↓
④ install_driver("oracle-jdbc")
    ├── 下载 ojdbc8.jar
    ├── 校验 checksum
    └── 注册到 driver_files
    ↓
⑤ 驱动就绪 → 出现在项目的可用驱动列表中
```

### 6.3 项目迁移后驱动自检

```
用户打开项目 A（从其他机器迁移过来的）
    ↓
① ProjectStore::open() → check_missing_drivers()
    ├── 遍历 project_drivers 中所有启用的驱动
    ├── Native 驱动 (mysql/postgres/sqlite/duckdb) → Ready ✅
    └── Oracle JDBC → 检查 driver_files + 文件系统 → 文件不存在 → NotInstalled ⚠️
    ↓
② 返回 missing 列表给前端
    ↓
③ 前端弹窗：
    ┌───────────────────────────────────────────────┐
    │  ⚠️ 检测到缺失驱动                              │
    │                                                │
    │  以下驱动在 project_drivers 中启用但本机未安装：  │
    │  🔴 Oracle JDBC (21.0.0)                        │
    │     描述：Oracle 数据库 JDBC 驱动                │
    │     [安装驱动（自动下载）]  [暂时禁用]            │
    │                                                │
    │  点击 "暂时禁用" → project_drivers.enabled = 0   │
    │  点击 "安装驱动" → 下载并安装                      │
    └───────────────────────────────────────────────┘
```

---

## 七、实施计划

| 阶段 | 任务 | 文件 | 优先级 |
|------|------|------|--------|
| **P1** 迁移 SQL | `global/008_add_data_source_module.sql` | `migrations/global/` | 🔴 |
| **P1** 迁移 SQL | `project_meta/010_add_data_source_module.sql` | `migrations/project_meta/` | 🔴 |
| **P2** Model | 扩展 `DriverDescriptor`；新增 `DataSourceType`/`Driver`/`DriverFile`/`Environment` 等 struct | `descriptors.rs` + 各 store | 🔴 |
| **P2** Model | 扩展 `GlobalConnectionInfo` + `ProjectConnection` +7 字段 | `global_db.rs` + `project_connection_store.rs` | 🔴 |
| **P3** Store | `driver_store.rs` — drivers/driver_files CRUD (global only) | 新建 | 🟡 |
| **P3** Store | `env_store.rs` — environments/policies CRUD (纯函数) | 新建 | 🟡 |
| **P3** Store | `auth_store.rs` — auth_configs CRUD (纯函数) | 新建 | 🟡 |
| **P3** Store | `network_store.rs` — network_configs CRUD (纯函数) | 新建 | 🟡 |
| **P4** Service | `driver_service.rs` — 驱动发现/安装/验证/自检 | 新建 | 🟡 |
| **P4** Service | 扩展 `connection_service.rs` — 连接创建时驱动校验 | 修改 | 🟡 |
| **P4** Service | 扩展 `project/store.rs` — 项目打开时驱动自检 | 修改 | 🟡 |
| **P5** Command | `data_source_commands.rs` — 全部前端 API | 新建 | 🟢 |
| **P5** Command | 扩展 `connection_commands.rs` — 新字段传递 | 修改 | 🟢 |
| **P5** Command | `lib.rs` 注册新 commands | 修改 | 🟢 |

**总计**：新建 7 文件，修改 8 文件，2 个迁移 SQL。

---

## 八、关键设计决策

| 决策 | 结论 | 理由 |
|------|------|------|
| `project_drivers` 表 | 在 project meta.db 中 | 实现项目级驱动可见性隔离。新项目只种子4个 Native 驱动 |
| `drivers` 表 | 仅 global.db | 驱动定义是 "通用数据库知识"，不随项目变化 |
| `driver_files` 表 | 仅 global.db | 记录 "本机安装了什么"，是机器级信息 |
| 跨 DB 引用 | 不用 FK，Rust 层校验 | SQLite 不支持跨文件外键 |
| Native 驱动 | 不注册 driver_files | 编译在 Rust 二进制中，无需外部文件 |
| 密码加密 | 复用 crypto.rs AES-256-GCM | 已有成熟方案 |
| Store 复用 | 纯函数模式 | Global 和 Project 各传自己的 Connection，零代码重复 |
| 迁移策略 | ALTER TABLE ADD COLUMN | 不丢失现有数据 |

---

## 九、相关文档

| 文档 | 路径 |
|------|------|
| 数据库字典（含新表） | [DATABASE-DICTIONARY.md](./DATABASE-DICTIONARY.md) |
| 后端架构 | [ARCHITECTURE.md](./ARCHITECTURE.md) |
| 迁移系统 | [MIGRATION_SYSTEM.md](./MIGRATION_SYSTEM.md) |
| 项目模块架构 | [PROJECT_MODULE_ARCHITECTURE.md](./PROJECT_MODULE_ARCHITECTURE.md) |