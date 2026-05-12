# RdataStation DuckDB 分析引擎 — 完整实施方案

> 版本: v1.0
> 创建日期: 2026-05-12
> 状态: 待评审

---

## 一、需求分析

### 1.1 核心功能需求

| 功能模块 | 需求描述 | 优先级 |
|----------|----------|--------|
| 连接池管理 | 全局/项目双层连接池,1写+4读+1维护 | P0 |
| 临时表管理 | 命名规则、TTL清理、数量上限、可见性控制 | P0 |
| 统一SQL执行 | 业务模块禁止直连Connection,统一走executor | P0 |
| 联邦查询 | ATTACH外部数据源、物化远程表 | P1 |
| 数据导入导出 | COPY TO导出、read_csv_auto/read_parquet导入 | P1 |
| 全文搜索 | FTS索引维护与查询 | P1 |
| 查询计划分析 | EXPLAIN查询计划解析与展示 | P2 |
| 插件系统接口 | 沙箱式插件连接、权限控制 | P2 |
| DuckDB扩展管理 | 发现、安装、加载、卸载、状态查询 | P2 |

### 1.2 非功能需求

| 指标 | 要求 |
|------|------|
| 连接池性能 | 读取连接轮询分配 < 1ms |
| 临时表清理 | TTL惰性清理,不影响主流程 |
| 内存控制 | 全局+项目DuckDB总内存 ≤ 500MB |
| 错误处理 | 统一CoreError,禁止unwrap/expect |
| 并发安全 | 读写连接物理隔离,单写入者模型 |

### 1.3 系统边界

**职责范围:**
- ✅ DuckDB实例连接池管理
- ✅ 临时表生命周期管理
- ✅ 联邦查询ATTACH管理
- ✅ 数据导入导出
- ✅ 全文搜索索引
- ✅ 查询计划分析
- ✅ 插件安全访问接口
- ✅ DuckDB扩展生命周期管理

**非职责范围:**
- ❌ 外部DuckDB数据库连接 (归属: `core/driver/native/duckdb.rs`)
- ❌ SQL解析/生成/优化 (归属: `core/sql/`)
- ❌ 数据持久化存储管理 (归属: `core/persistence/`)

---

## 二、架构设计

### 2.1 整体架构

```
分析工作台、洞察面板、Mock生成器、结果集面板、联邦查询入口
    │
    ▼
core/duckdb/mod.rs (统一入口)
    │
    ├── manager.rs          ← 连接池管理(全局/项目复用结构)
    ├── executor.rs         ← 统一SQL执行接口
    ├── temp_table.rs       ← 临时表命名、TTL、数量管控
    ├── federation.rs       ← ATTACH外部数据源、物化
    ├── import_export.rs    ← 数据导入导出能力
    ├── fts.rs              ← 全文搜索索引与查询
    ├── explain.rs          ← 执行计划解析分析
    ├── plugin.rs           ← 插件安全访问接口
    └── extensions.rs       ← DuckDB扩展生命周期管理
```

### 2.2 双层连接池架构

| 层级 | 文件路径 | 写入连接数 | 读取连接池数 | 后台维护连接数 |
|------|----------|------------|--------------|----------------|
| 全局 | ~/.rdatastation/global_analytics.duckdb | 1 | 4 | 1 |
| 项目 | 由项目元数据决定 | 1 | 4 | 1 |

**关键规则:**
1. 写入连接固定1个: DuckDB单写入者模型,多写入任务排队串行执行
2. 读取连接池默认4个,高并发场景可调整为6个
3. 后台维护连接独立业务读写,避免资源争抢
4. 全局与项目复用同一结构体,仅存储路径不同

### 2.3 模块依赖关系

```
core/duckdb/ 可调用 core/sql/ (SQL解析检测)
反向不依赖: core/sql/ 不依赖 core/duckdb/
```

### 2.4 技术选型

| 组件 | 技术 | 版本 |
|------|------|------|
| DuckDB驱动 | duckdb-rs | 1.10502.0 |
| 错误处理 | CoreError + anyhow | 1.0 |
| 路径处理 | PathBuf | std |
| 日志 | tracing | 0.1 |
| 时间戳 | Instant + Duration | std |

---

## 三、详细设计

### 3.1 DuckDBManager 结构体设计

```rust
pub struct DuckDBManager {
    db: Database,                    // 全局唯一Database实例
    write_conn: Connection,          // 写入连接(1个,独占)
    read_pool: Vec<Connection>,      // 读取连接池(4个,轮询分配)
    maintenance_conn: Connection,    // 后台维护连接(1个,独立)
}
```

**公开方法:**

| 方法 | 功能 | 返回值 |
|------|------|--------|
| `open(path)` | 打开/创建DuckDB文件,初始化连接池 | `Result<Self>` |
| `write_conn()` | 获取唯一写入连接 | `&Connection` |
| `read_conn()` | 轮询获取读取连接 | `&Connection` |
| `maintenance_conn()` | 获取后台维护连接 | `&Connection` |

### 3.2 临时表命名规则

**格式:** `tmp_{来源缩写}_{描述}_{紧凑时间戳}`

| 来源 | 缩写 | 说明 | 示例 |
|------|------|------|------|
| 查询结果转入分析 | q | query | tmp_q_orders_20260512143025 |
| 洞察中间计算 | i | insight | tmp_i_col_amount_20260512143030 |
| Mock数据生成 | m | mock | tmp_m_orders_20260512143500 |
| 插件临时数据 | p | plugin | tmp_p_sql_format_20260512143040 |

**时间戳格式:** YYYYMMDDHHMMSS (14位纯数字,无分隔符)

### 3.3 临时表清理规则

| 前缀 | 类型 | TTL | 数量上限 | 清理触发时机 |
|------|------|-----|----------|--------------|
| tmp_i_ | 洞察中间表 | 30分钟 | 100 | 惰性清理(新建洞察表时触发) |
| tmp_q_ | 查询结果表 | 无 | 无 | 项目关闭时批量清理 |
| tmp_m_ | Mock临时表 | 无 | 无 | 项目关闭时批量清理 |
| tmp_p_ | 插件临时表 | 无 | 无 | 插件卸载时批量清理 |

### 3.4 插件权限级别

| 权限级别 | 适用插件类型 | 权限范围 |
|----------|--------------|----------|
| 只读 | SQL格式化、数据脱敏、Schema Diff | 仅允许SELECT,禁止写入 |
| 读写 | 数据导入导出、Mock增强 | 可创建tmp_p_临时表,允许增删改 |
| 管理 | 官方内置插件(JDBC Bridge) | 支持联邦ATTACH、远程表物化等高级操作 |

### 3.5 核心接口设计

#### 3.5.1 插件系统接口

| 方法 | 功能 | 权限要求 |
|------|------|----------|
| `create_plugin_connection(plugin_id, permission_level)` | 创建插件沙箱连接 | - |
| `execute_plugin_sql(plugin_id, sql)` | 带权限校验执行SQL | 只读/读写 |
| `attach_plugin_temp_table(plugin_id, table_name)` | 注册插件临时表纳入生命周期管理 | 读写 |
| `revoke_plugin_connection(plugin_id)` | 回收连接、清理插件临时表 | - |

#### 3.5.2 DuckDB扩展管理接口

| 方法 | 功能 | 说明 |
|------|------|------|
| `discover_extensions()` | 查询扩展状态 | 通过duckdb_extensions()查询 |
| `install_extension(name)` | 安装扩展 | 下载至统一目录 |
| `load_extension(name)` | 加载扩展 | 会话级操作 |
| `unload_extension(name)` | 卸载扩展 | 回收扩展资源 |
| `get_extension_status(name)` | 获取扩展详情 | 安装/加载/版本 |

### 3.6 统一SQL执行接口 (executor.rs)

**业务调用规范:**
- 只读分析SQL → 调用 `read_conn()`
- 写入临时表/物化数据 → 调用 `write_conn()`

**统一接口保障:**
- 统一错误处理、日志记录
- 读写连接物理隔离
- 屏蔽底层连接管理细节

---

## 四、开发实现计划

### 4.1 文件拆分与重构策略

**现有代码分析:**
- 当前 `core/duckdb.rs` 包含连接池、临时表管理、持久化连接等功能
- 采用Mutex + Arc实现线程安全
- 使用OnceLock实现全局单例

**重构策略:**
1. 保留现有连接池逻辑,迁移至 `manager.rs`
2. 临时表管理迁移至 `temp_table.rs`
3. 新增模块按职责拆分,逐步迁移

### 4.2 开发阶段划分

#### 阶段一: 基础架构 (P0)

| 文件 | 职责 | 预估工作量 |
|------|------|------------|
| `mod.rs` | 模块入口,导出核心类型 | 0.5天 |
| `manager.rs` | 连接池管理 | 1天 |
| `executor.rs` | 统一SQL执行接口 | 0.5天 |
| `temp_table.rs` | 临时表管理 | 1天 |

#### 阶段二: 核心功能 (P1)

| 文件 | 职责 | 预估工作量 |
|------|------|------------|
| `federation.rs` | 联邦查询ATTACH管理 | 1天 |
| `import_export.rs` | 数据导入导出 | 1天 |
| `fts.rs` | 全文搜索 | 1天 |

#### 阶段三: 高级功能 (P2)

| 文件 | 职责 | 预估工作量 |
|------|------|------------|
| `explain.rs` | 查询计划分析 | 0.5天 |
| `plugin.rs` | 插件系统接口 | 1天 |
| `extensions.rs` | DuckDB扩展管理 | 1天 |

### 4.3 关键实现要点

#### 4.3.1 连接池管理

```rust
// manager.rs 核心结构
pub struct DuckDBManager {
    db: Database,
    write_conn: Connection,
    read_pool: Vec<Connection>,
    maintenance_conn: Connection,
    read_index: AtomicUsize,  // 轮询索引
}

impl DuckDBManager {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, CoreError> {
        // 1. 打开/创建DuckDB文件
        // 2. 初始化写入连接
        // 3. 初始化读取连接池(4个)
        // 4. 初始化后台维护连接
        // 5. 返回实例
    }

    pub fn write_conn(&self) -> &Connection {
        &self.write_conn
    }

    pub fn read_conn(&self) -> &Connection {
        let idx = self.read_index.fetch_add(1, Ordering::Relaxed) % self.read_pool.len();
        &self.read_pool[idx]
    }

    pub fn maintenance_conn(&self) -> &Connection {
        &self.maintenance_conn
    }
}
```

#### 4.3.2 临时表命名

```rust
// temp_table.rs 核心函数
pub fn generate_temp_table_name(source: TempTableSource, description: &str) -> String {
    let now = chrono::Local::now();
    let timestamp = now.format("%Y%m%d%H%M%S");
    format!("tmp_{}_{}_{:04}", source.abbreviation(), description, timestamp)
}

#[derive(Debug, Clone, Copy)]
pub enum TempTableSource {
    Query,    // q
    Insight,  // i
    Mock,     // m
    Plugin,   // p
}

impl TempTableSource {
    fn abbreviation(&self) -> &str {
        match self {
            TempTableSource::Query => "q",
            TempTableSource::Insight => "i",
            TempTableSource::Mock => "m",
            TempTableSource::Plugin => "p",
        }
    }
}
```

#### 4.3.3 统一SQL执行

```rust
// executor.rs 核心接口
pub struct DuckDBExecutor<'a> {
    manager: &'a DuckDBManager,
}

impl DuckDBExecutor {
    pub fn new(manager: &DuckDBManager) -> DuckDBExecutor {
        DuckDBExecutor { manager }
    }

    pub fn execute_read(&self, sql: &str) -> Result<QueryResult, CoreError> {
        let conn = self.manager.read_conn();
        // 执行只读SQL,返回QueryResult
    }

    pub fn execute_write(&self, sql: &str) -> Result<(), CoreError> {
        let conn = self.manager.write_conn();
        // 执行写入SQL
    }
}
```

### 4.4 测试策略

#### 4.4.1 单元测试

| 模块 | 测试用例 | 目标 |
|------|----------|------|
| manager.rs | 连接池初始化、轮询分配、路径缓存 | 覆盖率>80% |
| temp_table.rs | 命名规则、TTL清理、数量上限 | 覆盖率>80% |
| executor.rs | 读写分离、错误处理 | 覆盖率>80% |
| federation.rs | ATTACH/DETACH、跨库查询 | 覆盖率>80% |

#### 4.4.2 集成测试

| 场景 | 测试内容 |
|------|----------|
| 全局+项目双层池 | 初始化、独立操作、互不干扰 |
| 临时表生命周期 | 创建、TTL过期清理、项目关闭清理 |
| 联邦查询 | ATTACH全局库、跨库联表查询 |

---

## 五、文档管理计划

### 5.1 文档清单

| 文档类型 | 文件路径 | 内容 |
|----------|----------|------|
| 架构设计文档 | `src-tauri/src/docs/duckdb-engine-architecture.md` | 系统架构图、模块关系图 |
| 详细设计文档 | `src-tauri/src/docs/duckdb-engine-design.md` | 类图、时序图、流程图 |
| API接口文档 | `src-tauri/src/docs/duckdb-engine-api.md` | 接口定义、参数说明、示例 |
| 开发指南 | `src-tauri/src/docs/duckdb-engine-guide.md` | 快速开始、最佳实践 |

### 5.2 版本控制机制

- 文档与代码同步提交,使用相同的commit message
- 文档头部包含版本号、日期、状态
- 重大变更需更新版本号并记录变更日志

---

## 六、质量保障要求

### 6.1 代码质量

| 指标 | 要求 | 检查方式 |
|------|------|----------|
| 代码覆盖率 | ≥ 80% | `cargo tarpaulin` |
| Clippy警告 | 0 | `cargo clippy -- -D warnings` |
| 代码格式 | 通过 | `cargo fmt --check` |
| unwrap/expect | 0 (生产代码) | 代码审查 |

### 6.2 性能指标

| 指标 | 要求 | 测试方式 |
|------|------|----------|
| 连接池分配延迟 | < 1ms | 基准测试 |
| 临时表创建延迟 | < 5ms | 基准测试 |
| TTL清理耗时 | < 10ms (100表) | 基准测试 |

### 6.3 安全要求

| 检查项 | 要求 |
|--------|------|
| SQL注入防护 | 参数化查询,禁止拼接 |
| 权限控制 | 插件权限级别校验 |
| 扩展签名 | 仅加载签名扩展 |
| 错误信息 | 不暴露内部实现细节 |

---

## 七、风险与应对

| 风险 | 影响 | 应对措施 |
|------|------|----------|
| DuckDB单写入者瓶颈 | 写入性能受限 | 任务队列串行化,合理分配写入时机 |
| 临时表数量激增 | 内存占用过高 | 数量上限控制,惰性清理 |
| 扩展兼容性问题 | 加载失败 | 版本校验,降级处理 |
| 项目迁移路径失效 | 连接失败 | SQLite路径缓存,自动修复 |

---

## 八、总结

本实施方案基于任务书要求,结合现有代码结构,制定了从需求分析到开发实现的完整计划。核心要点:

1. **双层连接池架构**: 全局/项目物理隔离,结构复用
2. **临时表生命周期管理**: 命名规范、TTL清理、数量控制
3. **统一SQL执行接口**: 屏蔽底层细节,统一错误处理
4. **模块化设计**: 9个子模块职责清晰,易于维护扩展
5. **质量保障**: 代码覆盖率≥80%,性能指标达标

实施过程中需严格遵循项目规范,确保代码质量与系统稳定性。
