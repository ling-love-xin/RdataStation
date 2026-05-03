# RdataStation 文档中心

> 版本：v2.0
> 最后更新：2026-05-03
> 状态：✅ 持续更新

---

## 快速导航

| 我想... | 查看文档 | 说明 |
|---------|----------|------|
| 了解项目架构 | [架构总览](./architecture.md) | 四层微内核、四层数据库架构 |
| 开始后端开发 | [后端文档](./backend/README.md) | Rust Core + Tauri |
| 开始前端开发 | [前端文档](./frontend/INDEX.md) | Vue 3 + TypeScript |
| 了解数据库导航 | [导航器文档](./navigator/README.md) | IVM 增量视图设计 |
| 查看任务进度 | [任务清单](./backend/TASKS.md) | 开发任务追踪 |

---

## 文档结构

```
docs/
├── README.md                    # 📍 文档中心（本文档）
├── architecture.md              # 项目级架构总览
├── backend-implementation.md    # 后端实现说明
├── connection-modal.md          # 连接模态框设计
├── database-navigator.md        # 数据库导航栏
├── navigator-frontend-backend-alignment.md  # 前后端对齐
│
├── backend/                    # ⚙️ 后端文档
│   ├── README.md               # 后端文档索引
│   ├── ARCHITECTURE.md         # 后端架构设计
│   ├── PROJECT_MODULE_ARCHITECTURE.md
│   ├── PROJECT_MODULE_OPTIMIZATION.md
│   ├── MIGRATION_SYSTEM.md     # 迁移系统
│   ├── SCHEMA_CHANGELOG.md     # 数据库变更
│   ├── TASKS.md               # 任务清单
│   └── TODO_CONNECTION_CLASSIFICATION.md
│
├── frontend/                   # 🌐 前端文档
│   ├── INDEX.md               # 前端文档索引（推荐先读）
│   ├── ARCHITECTURE.md        # 前端架构（插件化、DDD）
│   ├── COMPONENTS.md          # 组件规范
│   ├── DEVELOPMENT-GUIDE.md   # 开发指南
│   ├── SQL-EDITOR.md         # SQL 编辑器
│   ├── QUERY-RESULT.md       # 结果面板
│   ├── QUERY-RESULT-DESIGN.md
│   └── PLUGIN-ARCHITECTURE-REFACTOR.md
│
└── navigator/                   # 🧭 导航器文档
    ├── README.md               # 导航器索引 + 已知问题
    ├── 01-ARCHITECTURE.md     # IVM 架构设计
    ├── 02-DATAFLOW.md         # 数据流设计
    ├── 03-INTERFACES.md       # 接口规范
    ├── 04-IMPLEMENTATION.md   # 实施步骤
    ├── 05-OPTIMIZATION.md     # 优化策略
    └── 06-CACHE-OPTIMIZATION.md  # 缓存优化
```

---

## 技术栈

### 后端 (Rust Core)

| 技术 | 版本 | 用途 |
|------|------|------|
| Rust Edition | 2021 | 开发语言 |
| Tokio | 1.44.x | 异步运行时 |
| Tauri | 2.10.x | 桌面框架 |
| sqlx | 0.8.x | MySQL/PostgreSQL |
| rusqlite | 0.32.x | SQLite |
| duckdb-rs | 1.1.x | DuckDB |
| wasmtime | 43.x | WASM 运行时 |

### 前端 (Vue 3 + TS)

| 技术 | 版本 | 用途 |
|------|------|------|
| Vue | 3.5.x | 响应式框架 |
| TypeScript | 5.8.x | 类型安全 |
| Vite | 6.x | 构建工具 |
| dockview-vue | 5.2.x | IDE 布局 |
| naive-ui | latest | 组件库 |
| AG Grid | 33.x | 表格引擎 |
| Monaco Editor | 0.52.x | SQL 编辑器 |
| Pinia | 2.3.x | 状态管理 |

---

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                    RdataStation 四层微内核架构                │
├─────────────────────────────────────────────────────────────┤
│  Rust Core（微内核）  ──►  Tauri Host  ──►  Wasm Plugin  ──►  UI│
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐     │
│  │   DBI    │  │  Driver  │  │Services  │  │Persistence│    │
│  │ (统一入口)│  │ (抽象层) │  │ (业务层) │  │ (持久化) │     │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 相关资源

| 资源 | 路径 |
|------|------|
| AI 开发规范 | `.trae/rules/` |
| 前端技能规范 | `.trae/skills/frontend-enterprise-spec/` |
| Rust 后端详细文档 | `src-tauri/src/docs/` |
| SQL 迁移脚本 | `docs/SQL/` |

---

## 文档版本

| 版本 | 日期 | 说明 |
|------|------|------|
| v2.0 | 2026-05-03 | 创建文档中心，优化索引结构 |
| v1.0 | 2026-04-23 | 初始文档架构 |

---

## 维护

- **文档维护者**：RdataStation 开发团队
- **更新频率**：随架构变更同步更新
- **反馈渠道**：提交 Issue 或联系架构组
