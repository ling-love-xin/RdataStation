# IVM 数据库导航栏设计文档

> 版本：v2.0
> 最后更新：2026-05-10
> 状态：✅ 文档结构重整完成

---

## 文档索引

| 文档                                                   | 说明                    | 状态 |
| ------------------------------------------------------ | ----------------------- | ---- |
| [01-ARCHITECTURE.md](./01-ARCHITECTURE.md)             | IVM 架构设计            | ✅   |
| [02-DATAFLOW.md](./02-DATAFLOW.md)                     | 数据流设计              | ✅   |
| [03-INTERFACES.md](./03-INTERFACES.md)                 | 接口规范                | ✅   |
| [04-IMPLEMENTATION.md](./04-IMPLEMENTATION.md)         | 实施步骤                | ✅   |
| [05-OPTIMIZATION.md](./05-OPTIMIZATION.md)             | 优化策略                | ✅   |
| [06-CACHE-OPTIMIZATION.md](./06-CACHE-OPTIMIZATION.md) | 缓存优化（V7 增量同步） | ✅   |

### 模块文档

| 文档                                                          | 说明                       |
| ------------------------------------------------------------- | -------------------------- |
| [database-navigator.md](./database-navigator.md)              | 数据库导航模块完整实现     |
| [database-navigator-optimizations.md](./database-navigator-optimizations.md) | 导航栏优化更新 |
| [frontend-backend-alignment.md](./frontend-backend-alignment.md) | 前后端对齐与职责划分     |

### 外部

| 文档                          | 说明         |
| ----------------------------- | ------------ |
| [COMPARISON.md](../COMPARISON.md) | 竞品对比分析 |

---

## 已知问题

### 数据库表列表不显示

**根因分析**（已修复）：前端 `database-api.ts` 调用 `invoke('load_tables', ...)` 加载表列表，但该 Tauri 命令在 Rust 后端缺失（未实现）。

**第1轮修复（2026-04-23）**：在 `use-database-tree-loader.ts` 的 `loadChildren` 中，MySQL 无 Schema 时用 `dbName` 替代 `schemaName`。

**第2轮修复（2026-04-23）**：`execute_sql` 返回 `unknown[][]`（数组的数组），修复 `loadTablesFromDb`、`loadColumnsFromDb` 共 4 处 array-vs-object 映射错误。

**第3轮修复（2026-04-23）**：MySQL 无 Schema 时 `updateSchemaTables` 崩溃。修复 `DatabaseNode` 增加 `tables` 字段，增加 `db.tables` 回退。

### SQL 编辑器补全报错

**根因**：`sql-editor-service.ts` 调用不存在的 `invoke('get_tables')` 和 `invoke('get_columns')`。

**第1轮修复（2026-04-23）**：改用 `invoke('execute_sql', ...)` 查询 `information_schema.tables`。

**第2轮修复（2026-04-23）**：SQLite 无 `information_schema`，新增 `dbType` 参数使用 `sqlite_master` + `PRAGMA table_info`。

---

## 概述

基于 IVM（增量视图维护）的数据库导航栏设计方案，打造高性能、实时响应、资源友好的桌面级数据库导航体验。

## 核心特性

- **增量更新**：只更新变更部分，而非全量刷新
- **虚拟滚动**：只渲染视口内节点，支持百万级数据
- **实时同步**：WebSocket 推送元数据变更
- **离线优先**：本地物化视图支持断网浏览
- **智能缓存**：三级缓存架构（内存/IndexedDB/SQLite）

## 技术栈

- **前端**：Vue 3 + TypeScript + Pinia
- **桌面**：Tauri + Rust
- **缓存**：SQLite（本地）+ IndexedDB（浏览器）

## 架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│  Presentation Layer (Vue Components)                            │
│  ├── NavigatorTree.vue     导航树组件                           │
│  ├── NavigatorNode.vue     节点组件                             │
│  ├── ConnectionCard.vue    连接卡片                             │
│  └── SearchPanel.vue       搜索面板                             │
├─────────────────────────────────────────────────────────────────┤
│  View Layer (Incremental Views)                                 │
│  ├── ViewportView          视口视图                             │
│  ├── FilteredView          筛选视图                             │
│  ├── SortedView            排序视图                             │
│  └── AggregatedView        聚合视图                             │
├─────────────────────────────────────────────────────────────────┤
│  Engine Layer (IVM Core)                                        │
│  ├── ViewEngine            视图引擎                             │
│  ├── DeltaProcessor        增量处理器                           │
│  ├── ChangePropagator      变更传播器                           │
│  └── QueryOptimizer        查询优化器                           │
├─────────────────────────────────────────────────────────────────┤
│  Cache Layer (Multi-Level)                                      │
│  ├── L1 Cache (Memory)     内存缓存                             │
│  ├── L2 Cache (IndexedDB)  本地持久化                           │
│  └── L3 Cache (SQLite)     系统级缓存                           │
├─────────────────────────────────────────────────────────────────┤
│  Source Layer (Data Sources)                                    │
│  ├── WebSocket Source      实时数据源                           │
│  ├── HTTP Source           请求数据源                           │
│  └── Local Source          本地数据源                           │
└─────────────────────────────────────────────────────────────────┘
```

## 性能目标

| 指标     | 目标    | 说明           |
| -------- | ------- | -------------- |
| 初始加载 | < 100ms | 从本地缓存加载 |
| 滚动性能 | 60fps   | 虚拟滚动       |
| 搜索响应 | < 50ms  | 增量索引       |
| 内存占用 | < 50MB  | 视口内数据     |
| 实时延迟 | < 100ms | WebSocket 推送 |

## 相关资源

- [前端架构](../frontend/ARCHITECTURE.md)
- [架构总览](../architecture/overview.md)