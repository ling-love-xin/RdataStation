# RdataStation 变更日志

> 版本：v2.3
> 最后更新：2026-05-05
> 状态：📋 持续更新

---

## 目录

- [项目变更日志](#项目变更日志)
- [文档变更日志](#文档变更日志)
- [提交规范](#提交规范)

---

## 项目变更日志

### [v2.4] - 2026-05-06

#### 优化

- **增量同步支持（V7）**
  - 对象级 Hash 变化检测（SHA-256）
  - sync_snapshot 表保存元数据快照
  - sync_operations 表记录变更操作
  - 首次全量同步后，后续仅同步变化对象
  - 预期预热时间减少 90%+

- **build_cache_index V3 优化版**
  - 支持增量模式（可选）
  - 每次同步后自动保存快照
  - 新增 change detection views（v_schema_changes/v_table_changes/v_column_changes）
  - 增量同步相关 API

#### 新增

- **MetadataCacheOps V7 新方法**
  - `calculate_object_hash()` - 计算对象 Hash
  - `save_snapshot()` - 保存同步快照
  - `get_snapshot()` - 获取快照
  - `has_snapshot()` - 检查快照
  - `detect_schema_changes()` - 检测 Schema 变更
  - `detect_table_changes()` - 检测表变更
  - `detect_column_changes()` - 检测列变更
  - `detect_all_changes()` - 检测所有变更
  - `save_sync_operations()` - 保存同步操作
  - `get_pending_operations()` - 获取待处理操作
  - `mark_operation_processed()` - 标记操作为已处理
  - `clear_old_operations()` - 清理旧操作
  - `incremental_sync()` - 完整增量同步流程

#### 新增 Tauri 命令

- `build_cache_index` - 支持增量模式（incremental 参数可选）
  - 响应包含 create/update/delete 计数
  - 保存快照用于下次增量

#### 迁移文件

- `007_incremental_sync.sql` - 增量同步表（sync_snapshot/sync_operations/change views）

### [v2.3] - 2026-05-05

#### 优化

- **build_cache_index V2 并行优化**
  - JoinSet 多 Schema 并行获取（减少 40-50% 时间）
  - JoinSet 表级并行获取 Columns（减少 60-70% 时间）
  - 优化后整体预热流程时间减少 70%+

- **SQLite 性能优化**
  - WAL 模式（Write-Ahead Logging）
  - Memory-Mapped I/O 256MB
  - 增大缓存至 2MB
  - 外键约束启用
  - 同步模式 NORMAL

### [v2.2] - 2026-05-04

#### 新增

- **分页懒加载支持（V6）**
  - metadata_index 索引表支持快速定位
  - 分页加载避免全量查询百万级表
  - introspect_level 分级加载（1=索引, 2=概要, 3=详情）

- **DataGrip 风格内省级别（V6）**
  - 根据对象数量自动计算内省级别
  - 当前 Schema: N<=1000→Level3, N<=3000→Level2, 否则→Level1
  - 非当前 Schema: N<=3000→Level3, N<=10000→Level2, 否则→Level1
  - get_introspect_level_suggestion 自动建议

- **同步状态跟踪（V6）**
  - connection_sync_status 跟踪同步进度
  - 支持取消同步操作
  - 后台增量同步任务队列

- **后台任务队列（V6 完整）**
  - sync_tasks 表支持后台任务队列
  - 入队/认领/完成完整生命周期
  - 批量入队支持事务

- **分块读取（V6）**
  - get_tables_chunk 分块获取表名
  - ChunkResult 通用分块结果结构
  - 避免大数据量 OOM

- **MetadataCacheOps V6 新方法**
  - `save_index_entry()` - 保存索引条目
  - `save_index_entries_batch()` - 批量保存索引
  - `get_index_entries()` - 分页获取索引
  - `calculate_introspect_level()` - DataGrip 风格级别计算
  - `get_schema_object_counts()` - 获取对象统计
  - `build_metadata_index()` - 构建元数据索引（完整预热流程）
  - `save_index_entries_internal()` - 内部批量保存（事务）
  - `enqueue_sync_task()` - 入队同步任务
  - `enqueue_sync_tasks_batch()` - 批量入队
  - `enqueue_indexing_tasks()` - 入队索引任务
  - `get_next_sync_task()` - 获取下一个任务
  - `claim_sync_task()` - 认领任务
  - `complete_sync_task()` - 完成同步任务
  - `get_pending_task_count()` - 获取待处理任务数
  - `get_tables_chunk()` - 分块获取表
  - `update_sync_status()` - 更新同步状态
  - `get_sync_status()` - 获取同步状态
  - `is_syncing()` - 检查是否正在同步
  - `cancel_sync()` - 取消同步

#### 新增 Tauri 命令

- `build_cache_index` - 构建缓存索引（完整预热流程-V2 优化版）
  - JoinSet 多 Schema 并行获取
  - JoinSet 表级并行获取 Columns
  - 流式写入（每 500 条一批）
  - 进度回调（Tauri Event: `cache_warming_progress`）
  - 取消支持（CancellationToken）

#### 迁移文件

- `006_add_metadata_index.sql` - 索引表与分页懒加载

### [v2.1] - 2026-05-04

#### 新增

- **元数据缓存规范化（V4）**
  - 规范化表结构：schemata / tables / columns / indexes / views / routines 独立表
  - 外键约束确保数据完整性
  - 向后兼容视图保持旧接口可用

- **FTS5 全文搜索增强（V5）**
  - 规范化表数据同步到 FTS5 虚拟表
  - 支持增量同步（按类型）
  - 搜索结果高亮显示

- **级联删除支持（V5）**
  - 删除 Schema 时自动级联删除关联数据
  - FTS 索引同步清理

- **MetadataCacheOps 新方法**
  - `sync_fts_index()` - FTS 索引同步
  - `search_fts()` - 全文搜索
  - `delete_schema()` - 级联删除

#### 迁移文件

- `005_normalized_fts_and_cascade.sql` - FTS 同步与级联删除

### [v2.0] - 2026-04-23

#### 新增

- **插件化架构优化**
  - DDD 分层架构（domain/infrastructure/ui）
  - 事件总线（EventBus）插件间通信机制
  - 统一 API 层
  - ExtensionContext 生命周期管理

- **前端架构**
  - 完整的插件系统
  - shared/ 共享资源中心
  - 全局类型系统

#### 变更

- **SQL 编辑器**
  - 1:n 编辑器-结果集关系重构
  - 多结果标签页支持

- **数据库导航**
  - IVM 增量视图维护
  - 三级缓存架构

---

### [v1.0] - 2026-04-20

#### 新增

- 初始插件化架构
- 内置插件：
  - connection（连接管理）
  - database（数据库导航）
  - navigator（通用导航器）
  - query（查询执行）
  - workbench（SQL 工作台）

---

## 文档变更日志

### [v2.0] - 2026-05-03

#### 新增

- `docs/README.md` - 项目文档中心总索引
- `docs/backend/TECHNICAL_OVERVIEW.md` - 技术概览

#### 变更

- `docs/backend/README.md` - 统一文档格式（版本、日期、状态）
- `docs/navigator/README.md` - 统一文档格式
- `docs/frontend/INDEX.md` - 统一文档格式
- `src-tauri/src/docs/README.md` - 统一文档格式

---

## 提交规范

项目使用 **Gitmoji + Angular** 提交规范：

### 格式

```
<emoji> <type>(<scope>): <subject>
```

### 类型

| 类型 | Emoji | 说明 |
|------|-------|------|
| feat | ✨ | 新增功能 |
| fix | 🐛 | 修复 Bug |
| docs | 📝 | 文档变更 |
| refactor | ♻️ | 代码重构 |
| perf | ⚡️ | 性能优化 |
| style | 💄 | 格式调整 |
| test | 🧪 | 测试相关 |
| build | 📦 | 构建相关 |
| chore | 🔧 | 杂项配置 |

### 示例

```bash
✨ feat(workbench): 实现 SQL 编辑器多标签页
🐛 fix(sqlite): 修复百万级数据查询超时
📝 docs: 补充前端架构文档
♻️ refactor(database): 重构导航器缓存逻辑
⚡️ perf(query): 优化查询缓存命中率
🔧 chore: 更新依赖版本
```

---

## 版本管理策略

### 分支策略

- `main` - 主分支，稳定版本
- `develop` - 开发分支
- `feature/*` - 功能分支
- `fix/*` - 修复分支

### 发布流程

1. 功能开发完成 → 合并到 `develop`
2. 测试验证 → 合并到 `main`
3. 打标签发布 → `git tag v{x.y.z}`

### 兼容性约束

- 接口遵循语义化版本（SemVer）
- 禁止破坏性变更（major 版本内）
- 10 年向前兼容目标

---

## 维护

- **最后更新**：2026-05-03
- **更新频率**：每次重要变更后同步更新
- **格式要求**：使用 Keep a Changelog 规范
