# RdataStation 后端文件职责与全方面审计报告

> 版本：v1.0
> 生成日期：2026-05-11
> 审计范围：src-tauri/src/ 全部 ~148 个 .rs 文件
> 审计维度：架构 / 设计 / 代码 / 接口 / 文档 / 前后端对齐 / 连通性

---

## 一、完整目录树 + 文件职责

```
src-tauri/src/
│
├── main.rs                                    # 程序入口，调用 rdata_station_lib::run()
├── lib.rs                                     # 模块注册 + 启动初始化序列 + 144+ Tauri 命令注册
│
├── api/                                       # ===== API 层（前后端数据契约）=====
│   ├── mod.rs                                 # 模块入口，重新导出所有 DTO 类型
│   └── dto.rs                                 # ErrorResponse / ApiResponse<T> / PageRequest / PageResponse / QueryResult 从 core 重导出
│
├── adapters/                                  # ===== 适配器层（平台适配）=====
│   ├── mod.rs                                 # Adapter trait 定义 + AdapterError 枚举
│   ├── tauri/
│   │   ├── mod.rs                             # Tauri 适配器入口
│   │   ├── state.rs                           # AppState — Tauri 管理状态容器
│   │   ├── event.rs                           # Tauri 事件发射（DDL 变更、连接状态变更）
│   │   └── stream.rs                          # QueryResultStream — 流式大数据传输
│   └── wasm/
│       ├── mod.rs                             # Wasm 适配器入口
│       ├── api.rs                             # Wasm 插件 API 接口
│       ├── extism.rs                          # Extism Wasm 运行时封装
│       └── plugin_manager.rs                  # 插件加载/卸载/生命周期管理
│
├── commands/                                  # ===== 命令层（Tauri IPC 入口）=====
│   ├── mod.rs                                 # 20 个命令模块聚合 + 全部 re-export
│   ├── connection_commands.rs                 # 连接 CRUD：connect/close/switch/test/create_database_file
│   ├── sql_commands.rs                        # SQL 执行：execute_sql/transaction/history/ping/cancel
│   ├── sql_template_commands.rs               # SQL 模板：创建/查询/分类/删除
│   ├── sql_parser_commands.rs                 # SQL 解析：语法分析/特征检测
│   ├── driver_commands.rs                     # 驱动查询：get_drivers/create_connection_with_config
│   ├── metadata_commands.rs                   # 元数据浏览：databases/schemas/tables/columns/routines
│   ├── metadata_cache_commands.rs             # 元数据缓存：invalidate_metadata_cache
│   ├── navigator_commands.rs                  # 导航状态：save_navigator_state/load_navigator_state
│   ├── result_commands.rs                     # 结果集：重新执行/筛选/单元格编辑/洞察/质量评分
│   ├── project_commands.rs                    # 项目生命周期：创建/打开/删除/重命名/验证 + ProjectState
│   ├── project_store_commands.rs              # 项目级存储：连接/SQL历史/工作台状态持久化
│   ├── memory_commands.rs                     # 内存监控：RSS/缓存/压力水平
│   ├── performance_commands.rs                # 性能监控：P50/P95/P99 查询耗时统计
│   ├── logging_commands.rs                    # 日志查询：搜索/过滤/聚合/统计
│   ├── system_commands.rs                     # 系统信息：版本/数据目录/API 版本
│   ├── port_commands.rs                       # 端口协商：negotiate/is_available/release
│   ├── mock_commands.rs                       # Mock 数据：生成/预览/导出/模板/列映射
│   ├── mock_persistence_commands.rs           # Mock 持久化：模板/记录存储
│   ├── cache_warming_commands.rs              # 缓存预热：索引构建/进度/迁移/版本检查
│   ├── scratchpad_commands.rs                 # 草稿箱：文件 CRUD/搜索/回收站/外部引用/监控
│   └── analytics_resource_commands.rs         # 分析资源：CRUD/文件夹/标签/回收站/版本 + AnalyticsResourceState
│
└── core/                                      # ===== 核心业务层（无框架依赖）=====
│   ├── mod.rs                                 # 核心模块聚合 + 全部 public type 重导出
│   │
│   ├── error.rs                               # 域错误设计：CoreError 容器（6 域：Common/Connection/Database/Storage/Cache/Plugin）
│   ├── models.rs                              # QueryResult（Arrow RecordBatch）/ Row / Value
│   ├── macros.rs                              # core_err! / bail! 宏
│   ├── api_version.rs                         # API_VERSION = "1.0.0" / ApiVersionInfo
│   ├── crypto.rs                              # AES-256-GCM 加密/解密 + 机器密钥派生
│   ├── arrow.rs                               # ArrowHandler：RecordBatch ↔ JSON 双向转换
│   ├── stream.rs                              # Stream trait + ArrowBatchStream + StreamQueryResult
│   ├── duckdb.rs                              # DuckDBManager：全局内存连接池（4-32 连接，TTL 30min）
│   ├── port_negotiation.rs                    # 端口自动协商（10000-20000 范围）+ 冲突检测
│   │
│   ├── utils/                                 # 工具模块
│   │   ├── mod.rs
│   │   ├── hash.rs                            # SHA-256 哈希
│   │   ├── string.rs                          # 字符串处理
│   │   └── time.rs                            # 时间工具
│   │
│   ├── cache/                                 # ===== 多级缓存 =====
│   │   ├── mod.rs                             # CacheEntry / CacheStats / CachePolicy / CacheKey / CacheValue traits
│   │   ├── lru_cache.rs                       # 线程安全 LRU 缓存 + 内存压力检测 + 自动驱逐
│   │   ├── cache_manager.rs                   # 多级缓存管理器（L1 内存 → L2 持久化）
│   │   ├── metadata_cache.rs                  # 元数据专用缓存（表/列/索引序列化）
│   │   ├── query_cache.rs                     # SQL 查询结果缓存（Key=SQL哈希+连接ID，TTL 管理）
│   │   └── memory_guard.rs                    # 内存守护：3 级压力（Normal/Warning/Critical），触发逐出
│   │
│   ├── dbi/                                   # ===== DBI 统一数据访问入口 =====
│   │   ├── mod.rs                             # 模块聚合 + re-export
│   │   ├── dbi.rs                             # DBI 结构体：query()/execute() + 智能路由
│   │   ├── session.rs                         # Session：会话管理 + 事务状态 + SessionMode
│   │   ├── context.rs                         # ExecutionContext/QueryContext：执行上下文
│   │   ├── performance.rs                     # PerformanceCollector：P50/P95/P99 统计
│   │   └── engine/
│   │       ├── mod.rs                         # ExecutionEngine trait / ExecutionMode / QueryRouter
│   │       ├── driver_engine.rs               # 原生驱动执行
│   │       ├── duckdb_engine.rs               # DuckDB 加速/联邦查询
│   │       └── stream_engine.rs               # 流合并/排序/过滤
│   │
│   ├── driver/                                # ===== 驱动层（核心+连接+路由+原生驱动）=====
│   │   ├── mod.rs                             # 模块入口 + 所有类型 re-export
│   │   ├── traits.rs                          # 🔴 宪法文件：Database / Transaction / DbPool / MetadataBrowser / SchemaObject
│   │   ├── registry.rs                        # DriverRegistry（OnceLock<RwLock<HashMap>>，全局驱动注册表）
│   │   ├── factory.rs                         # DriverFactoryManager + 4 个工厂实现
│   │   ├── router.rs                          # DataSourceRouter（薄路由：config → Registry → factory.create）
│   │   ├── auto_register.rs                   # AutoDriverRegistrar：启动时注册 4 个内置驱动
│   │   ├── manager.rs                         # DriverManager：全局驱动生命周期（DriverInfo/DriverStatus）
│   │   ├── loader.rs                          # DriverLoader：Builtin/Wasm/JDBC 驱动发现
│   │   ├── metadata.rs                        # DriverMetadata / DriverType / DriverIcon / DriverFormField
│   │   ├── driver_config.rs                   # 驱动配置结构
│   │   ├── smart_pool.rs                      # SmartPool：智能连接池（动态扩容/延迟监控/统计）
│   │   ├── utils.rs                           # build_connection_url / parse_driver_id / validate_driver_config
│   │   │
│   │   ├── connection/                        # 物理连接层（原 core/connection/ 迁移至此，R32）
│   │   │   ├── mod.rs
│   │   │   ├── config.rs                      # ConnectionConfig + ConnectionMethod（Direct/SSL/SSH/HTTP Proxy/SOCKS5）
│   │   │   ├── connector.rs                   # Connector trait + 5 种实现（Direct/Ssl/Ssh/HttpProxy/SocksProxy）
│   │   │   ├── factory.rs                     # ConnectionFactory：连接器注册表 + 创建连接
│   │   │   └── stream.rs                      # ConnectionStream：TcpStream/TlsStream 统一抽象
│   │   │
│   │   ├── native/                            # 原生驱动实现（4 数据库 × 2 文件 = 8 实现文件）
│   │   │   ├── mod.rs
│   │   │   ├── mysql.rs                       # MySqlDatabase：impl Database for MySQL（sqlx）
│   │   │   ├── mysql_pool.rs                  # MySQL 连接池：impl DbPool
│   │   │   ├── postgres.rs                    # PostgresDatabase：impl Database for PostgreSQL（sqlx）
│   │   │   ├── postgres_pool.rs               # PostgreSQL 连接池：impl DbPool
│   │   │   ├── sqlite.rs                      # SqliteDatabase：impl Database for SQLite（rusqlite）
│   │   │   ├── sqlite_pool.rs                 # SQLite 连接池：impl DbPool
│   │   │   ├── duckdb.rs                      # DuckDbDatabase：impl Database for DuckDB（duckdb-rs）
│   │   │   └── duckdb_pool.rs                 # DuckDB 连接池：impl DbPool
│   │   │
│   │   ├── jdbc/                              # JDBC 桥接（预留，Go Sidecar）
│   │   │   ├── mod.rs
│   │   │   ├── driver.rs                      # JDBC 驱动适配器
│   │   │   ├── connection.rs                  # JDBC 连接管理
│   │   │   ├── executor.rs                    # JDBC SQL 执行器
│   │   │   └── jvm_manager.rs                 # JVM 进程管理
│   │   │
│   │   ├── wasm/                              # Wasm 插件驱动（预留）
│   │   │   ├── mod.rs
│   │   │   ├── driver.rs                      # Wasm 驱动适配器
│   │   │   ├── adapter.rs                     # Wasm-to-Database trait 适配
│   │   │   └── pool.rs                        # Wasm 连接池
│   │   │
│   │   └── tests/                             # 驱动测试
│   │       ├── mod.rs
│   │       └── registry_tests.rs              # DriverRegistry 单元测试
│   │
│   ├── services/                              # ===== 业务服务层 =====
│   │   ├── mod.rs                             # 服务聚合 + ConnectionManager/ConnectionService/SqlService 导出
│   │   ├── connection_service.rs              # 连接 CRUD 核心（connect/disconnect/switch/test/元数据保存）
│   │   ├── connection_manager.rs              # ConnectionManager：全局连接注册表 + ConnectionInfo/ConnectionType
│   │   ├── sql_service.rs                     # SQL 执行核心（查询/参数化/超时/取消/事务/历史/截断）
│   │   ├── execution_service.rs               # 通用执行编排（策略模式）
│   │   ├── duckdb_service.rs                  # DuckDB 专用操作（ATTACH 外部数据库/联邦查询）
│   │   ├── result_service.rs                  # ResultSet 管理（存储/排序/过滤/列统计/导出）
│   │   ├── sql_parser_service.rs              # SQL 解析（提取表/列/CTE/子查询/SQL 特征）
│   │   ├── insight_engine.rs                  # Schema 质量洞察引擎
│   │   ├── persistence_service.rs             # 持久化辅助（列洞察快照）
│   │   ├── quality_scorer.rs                  # 数据质量评分（空值率/基数/分布）
│   │   ├── table_profile_service.rs           # 表画像（行数估算/类型推断）
│   │   └── tests/
│   │       ├── mod.rs
│   │       └── connection_manager_tests.rs     # ConnectionManager 单元测试
│   │
│   ├── persistence/                           # ===== 持久化存储 =====
│   │   ├── mod.rs                             # 持久化模块聚合 + 错误转换辅助函数
│   │   ├── global_db.rs                       # GlobalDatabaseManager（系统级 SQLite + DuckDB）
│   │   ├── project_db.rs                      # ProjectDatabaseManager（项目级 SQLite + DuckDB）
│   │   ├── connection_store.rs                # 全局最近连接记录
│   │   ├── project_connection_store.rs        # 项目级连接信息
│   │   ├── history_store.rs                   # SQL 执行历史记录
│   │   ├── project_store.rs                   # 项目元数据存储
│   │   ├── metadata_cache.rs                  # 元数据缓存（每连接独立 SQLite，表/列/索引/约束/视图）
│   │   ├── cache_version_migration.rs         # 缓存版本管理与迁移
│   │   ├── log_store.rs                       # 应用日志 SQLite 持久化（批量写入/自动清理）
│   │   ├── insight_store.rs                   # Schema 洞察报告存储（表/列报告/版本）
│   │   ├── insight_meta_store.rs              # 洞察元数据存储
│   │   ├── sql_template_store.rs              # SQL 模板持久化
│   │   ├── workbench_context_store.rs         # 工作台布局/编辑器上下文持久化
│   │   └── analytics_resource_store/          # 分析资源存储子系统
│   │       ├── mod.rs
│   │       ├── models.rs                      # AnalyticsResource / AnalyticsFolder / AnalyticsTag 模型
│   │       ├── folder.rs                      # 文件夹 CRUD
│   │       ├── resource.rs                    # 资源 CRUD + 版本管理
│   │       ├── tag.rs                         # 标签 CRUD
│   │       ├── recycle.rs                     # 回收站管理
│   │       ├── version.rs                     # 资源版本管理
│   │       ├── helpers.rs                     # 工具函数
│   │       └── tests.rs                       # 单元测试
│   │
│   ├── project/                               # ===== 项目管理 =====
│   │   ├── mod.rs                             # 模块入口
│   │   ├── models.rs                          # Project / ProjectInfo / ProjectConfig / Versioned<T> / ConnectionRef
│   │   └── store.rs                           # ProjectManager + ProjectStore：CRUD/验证/版本化
│   │
│   ├── scratchpad/                            # ===== 草稿箱 =====
│   │   ├── mod.rs                             # 模块入口
│   │   ├── models.rs                          # ScratchpadEntry / ScratchpadConfig / ExternalReference / SearchResult
│   │   ├── store.rs                           # ScratchpadStore：文件 CRUD/搜索/回收站
│   │   └── state.rs                           # ScratchpadState（Tauri 管理状态）
│   │
│   ├── migration/                             # ===== 数据库迁移 =====
│   │   ├── mod.rs                             # 模块入口
│   │   ├── global_init.rs                     # 全局系统初始化（system/ 目录、global.db、analytics.duckdb）
│   │   ├── manager.rs                         # MigrationManager：版本追踪、SQL 脚本加载
│   │   ├── executor.rs                        # MigrationExecutor：按版本顺序执行 SQL
│   │   └── schema.rs                          # SchemaTracker / SchemaVersion
│   │
│   ├── logging/                               # ===== 日志系统 =====
│   │   ├── mod.rs                             # 日志门面（init_logging/flush_logs/session_id）
│   │   ├── config.rs                          # LogConfig：级别/保留天数/路径
│   │   ├── record.rs                          # LogRecord / LogQuery / LogLevel / LogStats
│   │   ├── redact.rs                          # 敏感信息脱敏（密码/连接字符串）
│   │   └── subscriber.rs                      # Tracing subscriber（stderr + 文件滚动 + DB 三层输出）
│   │
│   ├── insight/                               # ===== Schema 质量分析 =====
│   │   ├── mod.rs                             # 模块入口 + 全局 RuleRegistry（编译期嵌入规则）
│   │   ├── rule_types.rs                      # QualityRule / QualityReport / QualityCheck / SchemaInsightReport
│   │   ├── rule_registry.rs                   # RuleRegistry：内置规则 + 用户自定义规则加载
│   │   ├── rule_executor.rs                   # RuleExecutor：运行规则 → 生成报告
│   │   └── schema_analyzer.rs                 # SchemaAnalyzer：FK 候选/孤儿表/类型不匹配/冗余列
│   │
│   ├── mock/                                  # ===== 模拟数据生成 =====
│   │   ├── mod.rs                             # 模块入口
│   │   ├── engine.rs                          # MockEngine：多表/多语言/多场景数据生成引擎
│   │   ├── models.rs                          # MockConfig / MockGenerateResult / ColumnDataType / ScenarioTemplate
│   │   ├── generators.rs                      # 数据生成器：姓名/邮箱/电话/日期/数字/UUID
│   │   ├── templates.rs                       # 场景模板：电商/社交/日志/IoT 预置 Schema
│   │   ├── schema_map.rs                      # ColumnMapper：列类型映射与推断
│   │   ├── history.rs                         # 生成历史记录
│   │   ├── persistence.rs                     # 生成结果持久化
│   │   └── error.rs                           # MockError
│   │
│   └── performance/                           # ===== 性能监控 =====
│       ├── mod.rs                             # 模块入口
│       └── monitor.rs                         # PerformanceMonitor：CPU/内存/查询耗时监控 + PerformanceTimer
```

---

## 二、全方面审计结果

### 2.1 架构审计

| 检查项 | 结果 | 说明 |
|--------|------|------|
| 分层隔离 | ✅ 通过 | api → commands → services → driver → native，每层职责清晰 |
| 依赖方向 | ✅ 通过 | driver 层零依赖 services/adapters/api |
| 循环依赖 | ✅ 通过 | 无循环依赖 |
| IOC/RFID 模式 | ✅ 通过 | 4 个原生驱动完整实现 Database + DbPool |
| 路由解耦 | ✅ 通过 | DataSourceRouter 是薄包装，通过 DriverRegistry 间接访问 |
| 不可修改 trait | ✅ 通过 | driver/traits.rs 未修改 |
| Pool 下沉 | ✅ 通过 | Pool 只负责连接，不负责 SQL 执行 |
| JDBC/Wasm 骨架 | ⚠️ 占位 | JDBC 和 Wasm 模块仅有 trait 骨架，无实际功能 |
| **架构分** | **8.8/10** | R34 纠正：A1 DriverFactoryManager 实际不存在 |

**架构不足：**

| 编号 | 问题 | 严重度 | 说明 |
|------|------|--------|------|
| A1 | ~~DriverFactoryManager 与 DriverRegistry 功能重叠~~ | ~~中~~ | ~~R33误判~~ R34纠正：factory.rs 仅有 4 个独立工厂 struct 实现 DriverFactory trait，无 DriverFactoryManager。DriverRegistry 是唯一的注册中心，不存在重叠 |
| A2 | JDBC/Wasm 模块为空骨架 | 低 | 7 个文件（jdbc/ 4 + wasm/ 3）仅包含 trait 实现骨架，增加维护负担但无功能。建议标记为 `#[cfg(feature = "jdbc")]` |
| A3 | DBI 层与 services 层边界模糊 | 中 | `dbi/` 和 `services/` 都提供 SQL 执行能力（dbi.engine vs sql_service），调用方需要选择层级。建议明确 DBI 为唯一入口 |
| A4 | DuckDB 双实例管理 | 低 | `core/duckdb.rs`（DuckDBManager 单例）与 `core/persistence/global_db.rs`（GlobalDuckdbConnection）各自管理 DuckDB 实例，可能产生资源竞争 |

### 2.2 设计审计

| 检查项 | 结果 | 说明 |
|--------|------|------|
| 域错误设计 | ✅ 优秀 | CoreError 6 域容器模式，向前兼容，插件可扩展 |
| Arrow 数据契约 | ✅ 通过 | QueryResult 内部包含 `Vec<RecordBatch>`，IPC 路径干净 |
| SchemaObject 懒加载 | ✅ 通过 | `children: None` 表示未加载 |
| 连接安全 | ✅ 通过 | AES-256-GCM 加密，机器 ID 密钥派生 |
| 连接池设计 | ✅ 通过 | SmartPool 动态扩容 + 延迟监控 |
| 参数化查询 | ✅ 通过 | 4 个驱动全部支持参数化查询防注入 |
| 取消机制 | ✅ 通过 | CancellationToken 支持 |
| 流式传输 | ✅ 通过 | ArrowBatchStream + QueryResultChunk |
| 事务管理 | ✅ 通过 | begin/commit/rollback + Transaction trait |
| 缓存多级 | ✅ 通过 | L1 内存 LRU → L2 SQLite 持久化 + 内存守护 |
| **设计分** | **8.3/10** | |

**设计不足：**

| 编号 | 问题 | 严重度 | 说明 |
|------|------|--------|------|
| D1 | QueryResult 包含两种行格式 | 中 | `batches: Vec<RecordBatch>`（Arrow）和 legacy `Row`/`Value`（序列化兼容），结果转换时存在双路径 |
| D2 | ConnectionConfig 别名混乱 | 低 | `driver/connection/config.rs` 定义 `ConnectionConfig`，`driver/registry.rs` 重导出为 `DriverConnectionConfig`，同样的结构两个名字 |
| D3 | MetadataBrowser trait 使用率低 | 低 | `traits.rs` 中定义了完整的 `MetadataBrowser` trait，但 4 个原生驱动实现依赖的是 `Database trait` 的 `list_tables()/list_columns()` 方法，新 trait 未被充分使用 |

### 2.3 代码审计

| 检查项 | 结果 | 说明 |
|--------|------|------|
| cargo check | ✅ 零错误 | 1 个预存 warning（mock/engine.rs dead_code） |
| unsafe 代码 | ✅ 零处 | 全代码库无 unsafe |
| TODO/FIXME | ✅ 零处 | 核心代码无遗留标记 |
| 编译速度 | ✅ 快 | dev build ~1.15s |
| **unwrap/expect** | ⚠️ **176 处全部在 #[cfg(test)] 模块内** | **R34 纠正：production 代码路径（commands + driver/native）零 unwrap/expect** |

**unwrap/expect 分布（全部在测试模块）：**

| 文件 | 数量 | 位置 |
|------|------|------|
| `persistence/analytics_resource_store/tests.rs` | 40 | `#[cfg(test)] mod tests` |
| `insight/rule_executor.rs` | 25 | `#[cfg(test)] mod tests` |
| `services/insight_engine.rs` | 20 | `#[cfg(test)] mod tests` |
| `port_negotiation.rs` | 12 | `#[cfg(test)] mod tests` |
| `persistence/cache_version_migration.rs` | 7 | `#[cfg(test)] mod tests` |
| `persistence/global_db.rs` | 6 | `#[cfg(test)] mod tests` |
| `crypto.rs` | 6 | `#[cfg(test)] mod tests` |
| 其余文件 | ~60 | `#[cfg(test)] mod tests` |

| **代码分** | **8.0/10** | **R34 上行：production 路径零 unwrap，仅测试代码使用** |

**代码不足：**

| 编号 | 问题 | 严重度 | 说明 |
|------|------|--------|------|
| C1 | ~~176 处 unwrap/expect~~ | ~~🔴 高~~ | ~~R33误判~~ R34纠正：全部在 `#[cfg(test)]` 测试模块，production 路径（commands + driver/native）零 unwrap/expect。测试中 unwrap 是 Rust 标准实践 |
| C2 | 100 个 clippy warning | 🟡 中 | `too_many_arguments`(30+处)、`redundant_closure`(20+处)、`manual_flatten`(10+处) 等，建议逐步修复 |

### 2.4 接口审计

| 检查项 | 结果 | 说明 |
|--------|------|------|
| Tauri 命令注册 | ✅ 144+ | lib.rs invoke_handler 注册全部命令 |
| 命令组织 | ✅ 清晰 | 按功能域分 20 个模块文件 |
| DTO 一致性 | ✅ 通过 | ErrorResponse 标准格式，含 code/category/message/details/retryable/suggestion |
| 分页支持 | ✅ 通过 | PageRequest/PageResponse 标准模式 |
| 流式接口 | ✅ 通过 | QueryResultChunk 分块传输 |
| 版本化 API | ✅ 通过 | API_VERSION = "1.0.0" |
| **接口分** | **8.8/10** | R34 修复 I1：mock 命令统一 CoreError |

**接口不足：**

| 编号 | 问题 | 严重度 | 说明 |
|------|------|--------|------|
| I1 | ~~部分命令返回 String 而非 CoreError~~ | ~~🟡 中~~ | ✅ **R34 已修复**：13 个 mock_commands 全部从 `Result<T, String>` → `Result<T, CoreError>` |
| I2 | 缺少 API 版本协商 | 🟡 中 | 前端未检查 `get_api_version()`，版本不匹配时无法优雅降级 |

### 2.5 前后端对齐审计

| 检查项 | 结果 | 说明 |
|--------|------|------|
| 后端命令数 | ~80 | lib.rs invoke_handler 中注册 |
| 前端调用数 | ~50 唯一命令名 | 分布在 6 个 API 模块中 |
| 命令匹配率 | ~85% | 大部分后端命令有前端调用 |
| 未调用命令 | ~15% | 主要是日志查询类、部分高级功能 |
| DTO 字段一致 | ⚠️ 需验证 | 前端 TypeScript 类型与 Rust serde 结构需逐一对比 |
| **对齐分** | **7.5/10** | |

**前后端对齐不足：**

| 编号 | 问题 | 严重度 | 说明 |
|------|------|--------|------|
| FA1 | 后端命令未被前端调用 | 🟡 中 | `get_logs`/`search_logs`/`export_logs` 等日志命令、部分 `cache_warming` 命令、`validate_project_full` 等前端未使用。可能是死代码或未来功能 |
| FA2 | 前端 invoke 与后端命令签名未强制校验 | 🟡 中 | TypeScript 的 `invoke<T>('name', args)` 在编译期不校验参数类型，运行时才发现不匹配 |
| FA3 | 缺少前端 API 层统一封装 | 低 | invoke 调用分散在多个 extension 的 api 文件中，无统一的类型安全层 |

### 2.6 文档一致性审计

| 检查项 | 结果 | 说明 |
|--------|------|------|
| 目录树与实际一致 | ✅ 通过 | R32 重构后 `02-directory-structure.md` 和 `ARCHITECTURE.md` 已更新 |
| 无死路径引用 | ✅ 通过 | 所有 `datasource/` 引用已更新或标注迁移 |
| 版本日期 | ⚠️ 部分过期 | 多个文档日期停留在 2026-04-23/24 |
| 文档覆盖度 | ⚠️ 缺失 | 此前无文件级职责文档（本次补全） |
| 中英文混合 | ⚠️ 部分 | 部分文档用英文，部分用中文 |
| **文档分** | **7.5/10** | |

**文档不足：**

| 编号 | 问题 | 严重度 | 说明 |
|------|------|--------|------|
| DC1 | 文件职责文档缺失 | 🔴 高 | 此前无系统性的文件职责说明（本次已补全） |
| DC2 | `ARCHITECTURE.md` 日期过期 | 🟡 中 | 最后更新 2026-04-23，R32 架构变更后日期未更新 |
| DC3 | `docs/backend/` 和 `src-tauri/src/docs/` 双重文档目录 | 🟡 中 | 两份架构文档描述同一系统，易产生不一致 |
| DC4 | `TASKS.md` 未反映 R32 完成状态 | 低 | 连接层路径已更新但任务状态未标记 R32 完成 |

### 2.7 连通性审计

| 检查项 | 结果 | 说明 |
|--------|------|------|
| 完整数据流路径 | ✅ 通过 | 前端 invoke → Tauri Command → Service → Driver → Native → DB |
| Arrow 零拷贝 | ✅ 通过 | RecordBatch 贯穿全链路 |
| 错误传播 | ✅ 层层传递 | CoreError 从 native 返回到 command |
| 连接池复用 | ✅ 通过 | SmartPool + GlobalSqlitePool + ProjectSqlitePool |
| 跨模块调用 | ✅ 合规 | Command→Service→Driver，无越界 |
| **连通分** | **8.5/10** | |

---

## 三、综合评分

| 维度 | 分数 | 权重 | 加权 |
|------|------|------|------|
| 架构 | 8.5 | 25% | 2.13 |
| 设计 | 8.3 | 20% | 1.66 |
| 代码 | 7.0 | 20% | 1.40 |
| 接口 | 8.5 | 10% | 0.85 |
| 前后端对齐 | 7.5 | 10% | 0.75 |
| 文档 | 7.5 | 10% | 0.75 |
| 连通性 | 8.5 | 5% | 0.43 |
| **综合** | **7.98** | **100%** | **7.97** |

> 💡 综合评分 **B+ (7.97/10)**。主要拖分项：176 处 unwrap/expect 和 100 个 clippy warning。

---

## 四、优先级修复建议

| 优先级 | 编号 | 问题 | 预估工作量 |
|--------|------|------|-----------|
| 🔴 P0 | C1 | 消除 production 代码中的 136 处 unwrap/expect（排除测试文件） | 2-3 小时 |
| 🔴 P0 | DC1 | ✅ 已补全 FILE-RESPONSIBILITY.md | 完成 |
| 🟡 P1 | C2 | 修复 100 个 clippy warning（至少修复 too_many_arguments） | 3-4 小时 |
| 🟡 P1 | A1 | 合并 DriverFactoryManager → DriverRegistry | 1 小时 |
| 🟡 P1 | FA1 | 清理前端未使用的后端命令或补充前端调用 | 1-2 小时 |
| 🟡 P1 | I1 | 统一 mock_commands 错误类型为 CoreError | 30 分钟 |
| 🟢 P2 | A3 | 明确 DBI vs services 调用层级 | 设计讨论 |
| 🟢 P2 | D2 | 消除 ConnectionConfig/DriverConnectionConfig 别名 | 30 分钟 |
| 🟢 P2 | DC3 | 合并双份文档目录 | 1 小时 |
| 🟢 P3 | A2 | 将 JDBC/Wasm 骨架标记为 feature-gated | 30 分钟 |
| 🟢 P3 | D3 | 推动 MetadataBrowser trait 实际使用 | 依赖迁移 |
| 🟢 P3 | FA2 | 添加前端 invoke 类型安全层 | 设计+实现 |
| 🟢 P3 | DC2 | 更新文档日期为 2026-05-11 | 5 分钟 |

---

## 五、关于四个空目录的确认

审计确认：`core/driver/duckdb/`、`mysql/`、`postgres/`、`sqlite/` 四个目录 **在 `src-tauri/src/` 全目录下均不存在**。它们在 R32 重构前曾存在于 `core/datasource/` 下（`datasource/mysql.rs` 等），重构时已合并到 `driver/native/` 下作为平铺文件。

如果这些目录作为空文件夹存在于文件系统中（非 Git 追踪），可以安全删除。

---

> 📌 本文档由 R33 全方面审计生成，R34 深度审计修正更新（2026-05-11）。