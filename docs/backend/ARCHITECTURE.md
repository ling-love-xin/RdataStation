# RdataStation 后端架构设计

> 最后更新：2026-04-23

---

## 一、核心定位

RdataStation 后端是一个**跨平台数据库管理工具**的核心引擎，采用 Rust 编写，通过 Tauri 框架提供桌面应用支持。

**核心使命**：打造新一代跨平台数据库管理工具，轻量、高效、可扩展，支撑 10 年生命周期。

---

## 二、架构风格

### 2.1 四层微内核架构

```
Rust Core（微内核）→ Tauri Host → Wasm Plugin → UI
```

### 2.2 架构分层

```
┌─────────────────────────────────────────────────────────┐
│  Tauri Commands (适配层)                                 │
│  - 接收前端请求，调用 Core 层                             │
│  - 返回 Arrow/JSON 结果                                  │
├─────────────────────────────────────────────────────────┤
│  DBI (统一数据访问入口)                                   │
│  ├── DBI::query() / execute()                           │
│  ├── Session (会话管理)                                  │
│  ├── Context (查询上下文)                                │
│  └── QueryRouter (智能路由)                              │
├─────────────────────────────────────────────────────────┤
│  执行引擎                                                │
│  ├── DriverEngine (原生驱动执行)                         │
│  ├── DuckDBEngine (本地加速/联邦查询)                     │
│  └── StreamEngine (流式处理/合并)                        │
├─────────────────────────────────────────────────────────┤
│  数据库驱动层                                            │
│  ├── Database trait (统一接口)                           │
│  ├── MySQL / PostgreSQL / SQLite / DuckDB               │
│  └── SmartPool (智能连接池)                              │
├─────────────────────────────────────────────────────────┤
│  基础设施层                                              │
│  ├── Connection (连接管理)                               │
│  ├── Persistence (持久化)                                │
│  ├── Cache (缓存)                                        │
│  └── Project (项目管理)                                  │
└─────────────────────────────────────────────────────────┘
```

---

## 三、目录结构

```
src-tauri/src/core/
├── mod.rs                  # Core 入口，模块依赖规则定义
├── error.rs                # 统一错误处理（域错误设计）
├── models.rs               # 核心数据模型
├── arrow.rs                # Arrow 数据处理
├── stream.rs               # 流式传输
├── macros.rs               # 工具宏
│
├── dbi/                    # 🔥 统一数据访问入口（总调度）
│   ├── mod.rs
│   ├── dbi.rs              # 对外唯一接口：DBI::query() / execute()
│   ├── session.rs          # 会话、事务、上下文
│   ├── context.rs          # 查询上下文（连接信息、配置、权限）
│   └── engine/
│       ├── mod.rs
│       ├── driver_engine.rs  # 下发到数据库驱动执行
│       ├── duckdb_engine.rs  # 本地分析/加速/联邦查询引擎
│       └── stream_engine.rs  # 流拼接、合并、后处理
│
├── driver/                 # 🔥 驱动核心：native + JDBC + wasm 统一管理
│   ├── mod.rs
│   ├── traits.rs           # Database / Transaction / Stream trait
│   ├── registry.rs         # 驱动注册表
│   ├── factory.rs          # 驱动工厂
│   ├── metadata.rs         # 驱动元数据
│   ├── smart_pool.rs       # 智能连接池包装器
│   ├── utils.rs            # 驱动工具函数
│   ├── native/             # 原生驱动实现
│   │   ├── mysql.rs        # MySQL (sqlx)
│   │   ├── mysql_pool.rs
│   │   ├── postgres.rs     # PostgreSQL (sqlx)
│   │   ├── sqlite.rs       # SQLite (rusqlite)
│   │   ├── duckdb.rs       # DuckDB (duckdb-rs)
│   │   └── duckdb_pool.rs
│   ├── jdbc/               # JDBC 驱动（预留）
│   └── wasm/               # Wasm 驱动（预留）
│
├── connection/             # 连接管理：配置、池、生命周期
│   ├── mod.rs
│   ├── config.rs           # 连接配置
│   ├── connector.rs        # 连接器
│   ├── factory.rs          # 连接工厂
│   └── stream.rs           # 连接流
│
├── datasource/             # 数据源管理：注册/路由
│   ├── mod.rs
│   └── router.rs           # 数据源路由
│
├── services/               # 业务服务：SQL执行、连接管理
│   ├── mod.rs
│   ├── connection_manager.rs
│   ├── connection_service.rs
│   └── sql_service.rs
│
├── persistence/            # 持久化：SQLite 项目库、历史记录
│   ├── mod.rs
│   ├── connection_store.rs
│   ├── history_store.rs
│   ├── project_db.rs       # 项目级数据库（SQLite + DuckDB）
│   └── project_store.rs
│
├── project/                # 项目管理：项目、配置、存储
│   ├── mod.rs
│   ├── models.rs
│   └── store.rs
│
├── cache/                  # 缓存：元数据、查询缓存
│   ├── mod.rs
│   ├── lru_cache.rs
│   ├── query_cache.rs
│   ├── metadata_cache.rs
│   └── cache_manager.rs
│
└── utils/                  # core 内部工具
    ├── mod.rs
    ├── hash.rs
    ├── string.rs
    └── time.rs
```

---

## 四、数据库分层架构

### 4.1 四层数据库设计

RdataStation 采用四层数据库架构，确保系统级、项目级、连接级数据严格隔离：

```
┌─────────────────────────────────────────────────────────┐
│  系统级数据库 (System Level)                              │
│  位置：{data_dir}/RdataStation/system/                   │
│  ├─ global.db (SQLite 连接池)                            │
│  │   ├─ 全局连接信息 (不跟随项目)                          │
│  │   ├─ 全局设置 (主题、快捷键等)                          │
│  │   └─ 最近连接记录                                      │
│  ├─ global_metadata/ (每个连接独立文件)                   │
│  │   ├─ conn_oracle_001.sqlite (可能 500MB)              │
│  │   └─ conn_mysql_002.sqlite (可能 50MB)                │
│  └─ analytics.duckdb (DuckDB 长连接)                      │
│      ├─ 查询缓存                                         │
│      └─ 全局分析数据                                      │
├─────────────────────────────────────────────────────────┤
│  项目级数据库 (Project Level)                             │
│  位置：{project_path}/                                   │
│  ├─ meta/                                                │
│  │   ├─ project.db (SQLite 连接池)                       │
│  │   │   ├─ 项目连接信息 (跟随项目)                        │
│  │   │   ├─ SQL 历史                                     │
│  │   │   └─ 项目设置                                     │
│  │   └─ connection_metadata/ (每个连接独立文件)           │
│  │       ├─ conn_pg_001.sqlite                           │
│  │       └─ conn_sqlite_002.sqlite                       │
│  └─ analytics/                                           │
│      └─ data.duckdb (DuckDB 长连接)                       │
│          ├─ 项目分析数据                                  │
│          └─ 持久化结果集                                  │
├─────────────────────────────────────────────────────────┤
│  连接级数据库 (Connection Level)                          │
│  位置：用户指定的数据库服务器                              │
│  ├─ MySQL/PostgreSQL/Oracle 等                           │
│  └─ 通过驱动连接，不存储本地                              │
└─────────────────────────────────────────────────────────┘
```

### 4.2 目录结构

```
系统级 (system/)
├── global.db              # SQLite 连接池
├── global_metadata/       # 全局连接元数据缓存
│   └── conn_{id}.sqlite   # 每个连接独立文件
└── analytics.duckdb       # DuckDB 长连接

项目级 (project/)
├── meta/
│   ├── project.db         # 项目 SQLite 连接池
│   └── connection_metadata/
│       └── conn_{id}.sqlite  # 项目连接元数据缓存
└── analytics/
    └── data.duckdb        # 项目 DuckDB 长连接
```

### 4.3 设计理由

| 设计决策 | 理由 |
|---------|------|
| **元数据跟随连接信息** | 项目迁移时只需复制项目目录，元数据自动跟随 |
| **每个连接独立文件** | 大型数据库（如 Oracle 10 万表）元数据不影响其他连接 |
| **系统级独立目录** | 避免与用户数据混淆，系统级数据库长期运行 |
| **连接池 + 长连接** | SQLite 用连接池支持并发，DuckDB 用长连接保持分析状态 |

### 4.4 连接管理

#### SQLite 连接池

```rust
pub struct GlobalSqlitePool {
    pool: Arc<Semaphore>,        // 并发控制
    db_path: PathBuf,
    connections: Mutex<Vec<Connection>>,
}

impl GlobalSqlitePool {
    pub async fn acquire(&self) -> Result<Connection, CoreError>;
    pub async fn release(&self, conn: Connection);
}
```

**特性**：
- WAL 模式：支持并发读写
- 共享缓存：提高读取性能
- 信号量控制：限制并发连接数

#### DuckDB 长连接

```rust
pub struct GlobalDuckdbConnection {
    conn: Mutex<Option<Connection>>,
    db_path: PathBuf,
}
```

**特性**：
- 长期保持：软件运行期间一直开启
- 分析优化：适合复杂查询和联邦查询
- ATTACH 支持：可挂载外部数据库

---

## 五、元数据缓存架构

### 5.1 元数据缓存设计

每个数据库连接都有独立的 SQLite 文件用于缓存元数据：

```
元数据缓存文件 (conn_{id}.sqlite)
├── tables          # 表信息
├── columns         # 列信息
├── indexes         # 索引信息
├── constraints     # 约束信息
├── views           # 视图信息
└── metadata_fts    # FTS5 全文搜索索引
```

### 5.2 元数据表结构

```sql
-- 表信息
CREATE TABLE tables (
    id TEXT PRIMARY KEY,
    conn_id TEXT NOT NULL,
    schema_name TEXT,
    table_name TEXT NOT NULL,
    table_type TEXT,
    comment TEXT,
    row_count INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 列信息
CREATE TABLE columns (
    id TEXT PRIMARY KEY,
    conn_id TEXT NOT NULL,
    schema_name TEXT,
    table_name TEXT NOT NULL,
    column_name TEXT NOT NULL,
    data_type TEXT,
    is_nullable BOOLEAN,
    column_default TEXT,
    comment TEXT,
    ordinal_position INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- FTS5 全文搜索索引
CREATE VIRTUAL TABLE metadata_fts USING fts5(
    schema_name,
    table_name,
    column_name,
    comment,
    content='tables'
);
```

### 5.3 元数据同步策略

#### 基础同步策略

```rust
/// 元数据同步策略
pub enum MetadataSyncStrategy {
    /// 全量同步（首次连接或手动触发）
    Full,
    /// 增量同步（基于版本号/时间戳）
    Incremental,
    /// 按需同步（访问时同步）
    OnDemand,
    /// 后台异步同步
    Background,
}

/// 元数据同步配置
pub struct MetadataSyncConfig {
    pub strategy: MetadataSyncStrategy,
    pub auto_sync_interval: Option<Duration>,  // 自动同步间隔
    pub max_table_count: Option<usize>,         // 单次同步最大表数
    pub exclude_patterns: Vec<String>,          // 排除的表模式
}
```

#### V7 增量同步（完整支持）

**设计目标**：
- 首次同步：全量预热
- 后续同步：仅同步变化对象
- 预热时间减少 90%+

**核心机制**：

| 组件 | 说明 |
|------|------|
| **sync_snapshot** | 快照表，保存上次同步时的元数据状态 |
| **sync_operations** | 操作表，记录待同步的 create/update/delete |
| **object_hash** | SHA-256 Hash，用于快速检测变化 |
| **change views** | 变更检测视图（v_schema_changes 等） |

**API**：

```rust
// MetadataCacheOps V7 增量同步 API
fn calculate_object_hash(
    object_type: &str,
    name: &str,
    parent: Option<&str>,
    extra_data: Option<&str>,
) -> String

fn save_snapshot(
    &mut self,
    connection_id: &str,
    snapshot_type: &str,
    snapshots: Vec<SyncSnapshot>,
) -> Result<usize, CoreError>

fn detect_all_changes(
    &self,
    connection_id: &str,
) -> Result<ChangeDetectionResult, CoreError>

fn incremental_sync(
    &mut self,
    connection_id: &str,
) -> Result<ChangeDetectionResult, CoreError>
```

**性能对比**：

| 场景 | V6 优化后 | V7 增量（首次） | V7 增量（后续） |
|------|----------|---------------|----------------|
| 预热时间 | 150ms | 150ms | 15ms |
| 提升 | -70% | -70% | -97% |

详见 [COMPARISON.md](../COMPARISON.md) 和 [MIGRATION_SYSTEM.md](./MIGRATION_SYSTEM.md)

### 5.4 元数据懒加载

元数据采用懒加载策略，避免启动时全量加载：

```rust
pub struct MetadataCacheManager {
    db_path: PathBuf,
    conn_id: String,
    connection_type: ConnectionType,
}

impl MetadataCacheManager {
    /// 创建元数据缓存管理器
    pub fn new(
        conn_id: &str,
        connection_type: ConnectionType,
        project_path: Option<&str>,
    ) -> Result<Self, CoreError>;

    /// 打开元数据缓存数据库（自动执行迁移）
    pub fn open(&self) -> Result<Connection, CoreError>;

    /// 获取缓存路径
    pub fn db_path(&self) -> &PathBuf;
}
```

**路径规则**：
- 全局连接：`{data_dir}/RdataStation/system/global_metadata/conn_{id}.sqlite`
- 项目连接：`{project_path}/meta/connection_metadata/conn_{id}.sqlite`

---

## 六、核心模块设计

### 6.1 DBI 统一数据访问层

DBI (Database Interface) 是后端的核心数据访问层，作为**统一数据访问入口**。

#### 6.1.1 核心接口

```rust
pub struct DBI {
    router: Arc<QueryRouter>,
    session: Arc<Session>,
}

impl DBI {
    pub async fn query(&self, sql: &str, mode: ExecutionMode) -> Result<QueryResult, CoreError>;
    pub async fn execute(&self, sql: &str) -> Result<QueryResult, CoreError>;
}
```

#### 6.1.2 执行模式

| 模式 | 说明 | 适用场景 |
|------|------|---------|
| `Native` | 原生数据库驱动执行 | 写操作、简单查询 |
| `DuckDB` | DuckDB 加速执行 | 复杂分析、跨库 JOIN |
| `Stream` | 流式执行 | 大数据量、流式处理 |
| `UserChoice` | 智能推荐 | 由系统自动判断 |

#### 6.1.3 智能推荐规则

| SQL 特征 | 推荐模式 | 原因 |
|----------|---------|------|
| `INSERT/UPDATE/DELETE` | Native | 写操作必须走原生 |
| `GROUP BY` | DuckDB | 列式存储聚合快 |
| `JOIN` | DuckDB | 向量化执行 |
| `ORDER BY + LIMIT` | DuckDB | 排序优化 |
| 简单 `SELECT` | UserChoice | 由用户决定 |

### 6.2 数据库驱动层

#### 6.2.1 Database trait

```rust
#[async_trait::async_trait]
pub trait Database: Send + Sync {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError>;
    async fn query_with_params(&self, sql: &str, params: Vec<Value>) -> Result<QueryResult, CoreError>;
    async fn query_with_cancel(&self, sql: &str, cancel_token: CancellationToken) -> Result<QueryResult, CoreError>;
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, CoreError>;
    fn meta(&self) -> DataSourceMeta;
    async fn list_tables(&self, db: &str, schema: Option<&str>) -> Result<Vec<SchemaObject>, CoreError>;
    async fn list_columns(&self, db: &str, schema: Option<&str>, table: &str) -> Result<Vec<SchemaObject>, CoreError>;
    async fn register_external_database(&self, name: &str, driver: &str, connection_string: &str) -> Result<(), CoreError>;
    async fn create_external_table(&self, external_db_name: &str, schema_name: &str, table_name: &str, external_table_name: &str) -> Result<(), CoreError>;
}
```

#### 6.2.2 驱动选型

| 数据库类型 | 驱动选择 | 原因 | 版本 |
|-----------|---------|------|------|
| MySQL | sqlx | 异步、编译期检查、连接池 | 0.8 |
| PostgreSQL | sqlx | 异步、编译期检查、连接池 | 0.8 |
| SQLite | rusqlite | 官方 Rust 驱动，bundled 特性 | 0.32 |
| DuckDB | duckdb-rs | 官方 Rust 驱动，分析型数据库 | 1.1 |

### 6.3 智能连接池

```rust
pub struct SmartPoolWrapper {
    inner_pool: Arc<dyn DbPool>,
    smart_pool: SmartPool,
    closed: Arc<AtomicBool>,
}
```

**特性**：
- 动态扩容：根据延迟自动调整池大小
- 延迟监控：记录获取延迟，触发扩容阈值
- 统计报告：提供池状态监控

### 6.4 项目级数据库

```
Project（项目）
├── SQLite (meta/project.db)      - 元数据索引、事务性信息
├── DuckDB (analytics/data.duckdb) - 分析数据、版本载体
└── Config (config/*.json)        - 连接配置、SQL文件
```

---

## 七、DuckDB 核心功能

### 7.1 本地加速分析

用户在创建 MySQL/PostgreSQL/SQLite 连接时，有一个开关：

```
☑️ 使用 DuckDB 进行本地加速分析
```

**开启后的行为**：
- DuckDB 通过 `ATTACH` 命令将该数据库注册为**只读外部数据源**
- 用户对该数据库的查询有**两种执行模式**：

| 执行模式 | 说明 | 适用场景 |
|---------|------|---------|
| **自身执行** | 直接在原数据库执行 | 简单查询、事务操作 |
| **DuckDB 执行** | 通过 DuckDB 联邦查询执行 | 复杂分析、跨库 JOIN、大数据量聚合 |

### 7.2 结果集二次分析

```
查询结果集 → 加入分析池 → 二次分析 → 新结果集 → 可继续分析...
```

**模式分类**：
| 模式 | 说明 | 存储位置 |
|------|------|---------|
| **会话级** | 当前会话有效，关闭后消失 | DuckDB 内存表 |
| **持久化** | 保存到项目，下次打开可用 | `analytics/data.duckdb` |

### 7.3 外部数据源集成

| 数据源 | 接入方式 | DuckDB 插件 |
|--------|---------|------------|
| **数据库结果集** | 从二次分析池引入 | 内存表/持久化表 |
| **Excel 文件** | 文件路径选择 | `httpfs` + `excel` 扩展 |
| **CSV 文件** | 文件路径选择 | 内置支持 |
| **Parquet 文件** | 文件路径选择 | 内置支持 |

### 7.4 数据流转

**所有数据流转使用 Arrow 格式**：

```
┌─────────────┐    Arrow     ┌──────────┐    Arrow     ┌──────────┐
│  原数据库    │ ──────────→ │ DuckDB   │ ──────────→ │  前端    │
│  (MySQL等)  │  零拷贝导入  │ (分析)   │  零拷贝导出  │  (AG Grid)│
└─────────────┘              └──────────┘              └──────────┘
```

---

## 八、错误处理

### 8.1 域错误设计

```
CoreError (核心错误容器)
├── Common (通用错误域)
├── Connection (连接错误域)
├── Database (数据库错误域)
├── Storage (存储错误域)
└── Plugin (插件错误域 - 预留扩展)
```

### 8.2 错误处理规范

- ❌ 禁止 `unwrap()` / `expect()`（生产代码）
- ✅ 必须使用 `CoreError` 统一错误处理
- ✅ 结合 `anyhow` 简化错误传递

---

## 九、架构约束

### 9.1 依赖规则

```
✅ 允许依赖：
- models → 无（基础层）
- error, macros → 无（基础层）
- driver → error, macros, models
- connection → error, models
- datasource → driver, connection, error, models
- persistence → error, models
- project → error, models, persistence
- services → driver, persistence, connection, error, models, project, cache
- cache → error, models
- dbi → driver, error, models, stream

❌ 禁止依赖：
- 任何 core 内部模块 → api
- driver → connection（应通过 trait 解耦）
- datasource → api
```

### 9.2 架构红线

- ❌ 禁止：循环依赖
- ❌ 禁止：层级越界
- ❌ 禁止：修改 trait 定义
- ✅ 必须：实现完整性
- ✅ 必须：Pool 下沉
- ✅ 必须：Acquire 返回 Database 实例

---

## 十、技术栈

| 技术 | 版本 | 说明 |
|------|------|------|
| Rust Edition | 2021 | 最新稳定版 |
| Tokio | 1.44.1 | 异步运行时 |
| Tauri | 2.10.3 | 桌面框架 |
| SQLx | 0.8.3 | MySQL/PostgreSQL 驱动 |
| Rusqlite | 0.32.1 | SQLite 驱动 |
| DuckDB-RS | 1.1.1 | DuckDB 驱动 |
| Arrow | 53.0.0 | 数据传输格式 |
| Wasmtime | 43.0.0 | Wasm 运行时 |
| Serde | 1.0 | 序列化 |
| thiserror | 1.0 | 错误处理 |
| anyhow | 1.0 | 错误处理 |

---

## 十一、关键设计决策

1. **DBI 统一入口**：所有数据操作通过 DBI，内部自动路由
2. **DuckDB 只读保证**：加速模式下外部数据库只读
3. **Arrow 零拷贝**：所有数据流转使用 Arrow 格式
4. **结果集命名**：支持用户自定义名称
5. **无自动持久化**：用户手动触发持久化
6. **文件绝对路径**：避免相对路径问题
7. **连接信息加密**：安全存储外部连接密码
8. **插件沙箱隔离**：插件崩溃不影响主程序
9. **元数据跟随连接信息**：项目迁移时只需复制项目目录
10. **每个连接独立元数据文件**：大型数据库元数据不影响其他连接
