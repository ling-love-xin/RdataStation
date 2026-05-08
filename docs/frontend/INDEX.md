# RdataStation 前端文档

> 版本：v2.3
> 更新日期：2026-05-08
> 状态：✅ SQL 编辑器架构优化全部完成（含方言高亮优化）

---

## 文档结构

```
docs/frontend/
├── INDEX.md                        # 本文档（总索引）
├── ARCHITECTURE.md                 # 完整架构文档（核心）
├── CONFIG-SYSTEM.md                # 配置系统设计文档
├── CONFIG-API.md                   # 配置系统 API 参考
├── CONFIG-PROGRESS.md              # 配置系统开发进度
├── DEVELOPMENT-GUIDE.md            # 快速开发指南
├── stores/README.md                # Store 文档索引
├── COMPONENTS.md                   # 组件开发规范
├── SQL-EDITOR.md                   # SQL 编辑器完整文档
├── SQL-EDITOR-OPTIMIZATION-PLAN.md # SQL 编辑器架构优化计划（⏳ 待确认）
│
├── 查询结果模块文档
│   ├── QUERY-RESULT.md                  # 结果面板  V3.1 (⏳ 重构中)
│   ├── QUERY-RESULT-DESIGN.md           # 结果面板需求文档
│   ├── QUERY-RESULT-OPTIMIZATION-PLAN.md    # 结果集模块架构优化计划 (⏳ 设计阶段)
│   ├── QUERY-RESULT-OPTIMIZATION-PROGRESS.md # 结果集优化进度追踪 (⏳ 0%)
│   ├── QUERY-RESULT-API-V2.md              # 结果集 V2 接口契约 (⏳ 设计阶段)
│   ├── QUERY-RESULT-ARCHITECTURE-V2.md     # 结果集 V2 架构设计 (⏳ 设计阶段)
│   └── PLUGIN-ARCHITECTURE-REFACTOR.md
│
├── 分析资源管理器文档
├── ANALYTICS_RESOURCE_ARCHITECTURE.md  # 分析资源架构设计
├── ANALYTICS_RESOURCE_PROGRESS.md      # 开发进度
├── ANALYTICS_RESOURCE_API_REFERENCE.md # API 接口参考
├── ANALYTICS_RESOURCE_INTEGRATION.md   # 前端集成指南
├── ANALYTICS_RESOURCE_SETTINGS.md      # 设置功能文档
└── ANALYTICS_RESOURCE_MANAGER_DESIGN.md # 前端设计方案（v1.0 设计阶段）
```

---

## 核心文档

### 1. [架构文档](./ARCHITECTURE.md) - 必读

**适用对象**：架构师、高级开发者、所有团队成员

**内容**：

- 架构概览与技术栈
- 完整目录结构
- 扩展系统（生命周期、ExtensionContext、预定义事件）
- DDD 分层架构（domain/infrastructure/ui）
- 插件间通信机制（EventBus）
- 类型系统（全局类型、插件类型）
- 统一 API 层
- 错误处理机制（AppError、Result 类型）
- 命名规范
- 最佳实践
- 优化记录（v2.0 变更历史）

**何时阅读**：

- 新成员入职
- 理解整体架构
- 架构决策参考

---

### 2. [配置系统](./CONFIG-SYSTEM.md) - 配置管理

**适用对象**：所有开发者

**内容**：

- 三层配置优先级（项目覆盖 > 全局默认 > 系统硬编码）
- 配置项可覆盖性矩阵
- 架构设计（useAppStore 单一数据入口）
- Store 实例生命周期
- 核心 API（saveConfig 抽象层、SaveResult）
- 初始化顺序
- 设置面板交互逻辑（应用按钮、恢复默认值）
- 项目覆盖 diff 模式
- Schema 版本管理
- 未来迁移路径（tauri-plugin-store → Rust SQLite）

**何时阅读**：

- 修改或新增配置项
- 理解配置优先级
- 迁移到 Rust 后端

---

### 2.1 [配置 API](./CONFIG-API.md) - 完整接口参考

**适用对象**：开发者

**内容**：所有配置类型定义、useAppStore 完整方法签名、SaveResult 类型、JSON 文件格式规范、组件使用示例。

**何时阅读**：使用配置 API 时查阅。

---

### 2.2 [配置进度](./CONFIG-PROGRESS.md) - 开发进度

**适用对象**：项目管理

**内容**：4 阶段进度条、任务清单、文件影响矩阵、技术债务、工时预估。

**何时阅读**：了解当前实现状态和待办任务。

---

### 3. [开发指南](./DEVELOPMENT-GUIDE.md) - 快速上手

**适用对象**：所有开发者

**内容**：

- 环境准备（安装依赖、启动开发服务器）
- 创建新插件（步骤、模板代码）
- 创建新组件
- 调用后端 API
- 错误处理
- 插件间通信
- 命名规范
- 常见问题

**何时阅读**：

- 开始开发新功能
- 遇到常见问题

---

### 4. [组件规范](./COMPONENTS.md) - 组件开发

**适用对象**：前端开发者

**内容**：

- 组件分类（基础组件、业务组件）
- 组件结构标准模板
- Props 规范
- Emits 规范
- 样式规范
- 状态管理
- 组合式函数（Composables）
- 图标使用
- 性能优化
- 测试
- 代码审查清单

**何时阅读**：

- 开发新组件
- 代码审查

---

## 快速查找

| 我想...                 | 查看文档                                             | 章节                   |
| ----------------------- | ---------------------------------------------------- | ---------------------- |
| 了解整体架构            | [架构文档](./ARCHITECTURE.md)                        | 架构概览、目录结构     |
| 了解 SQL 编辑器设计     | [SQL 编辑器](./SQL-EDITOR.md)                        | 全部                   |
| 了解 SQL 编辑器优化计划 | [优化计划](./SQL-EDITOR-OPTIMIZATION-PLAN.md)        | Phase 1-4              |
| 管理配置/主题/语言      | [配置设计](./CONFIG-SYSTEM.md)                       | 三层优先级、数据流     |
| 调用配置 API            | [配置 API](./CONFIG-API.md)                          | 方法签名、类型定义     |
| 了解配置进展            | [配置进度](./CONFIG-PROGRESS.md)                     | 阶段进度、技术债务     |
| Store 架构/数据流       | [Store 文档](../../src/stores/README.md)             | 迁移路径/自维护特性    |
| 创建新插件              | [开发指南](./DEVELOPMENT-GUIDE.md)                   | 创建新插件             |
| 创建新组件              | [组件规范](./COMPONENTS.md)                          | 创建新组件             |
| 调用后端 API            | [开发指南](./DEVELOPMENT-GUIDE.md)                   | 调用后端 API           |
| 处理错误                | [开发指南](./DEVELOPMENT-GUIDE.md)                   | 错误处理               |
| 插件间通信              | [架构文档](./ARCHITECTURE.md)                        | 插件间通信机制         |
| 命名规范                | [架构文档](./ARCHITECTURE.md)                        | 命名规范               |
| 组件 Props/Emits        | [组件规范](./COMPONENTS.md)                          | Props 规范、Emits 规范 |
| 性能优化                | [组件规范](./COMPONENTS.md)                          | 性能优化               |
| 代码审查                | [组件规范](./COMPONENTS.md)                          | 代码审查清单           |
| 了解分析资源管理器      | [分析资源架构](./ANALYTICS_RESOURCE_ARCHITECTURE.md) | 全部                   |
| 查看分析资源 API        | [API 参考](./ANALYTICS_RESOURCE_API_REFERENCE.md)    | 完整命令索引           |
| 集成分析资源功能        | [集成指南](./ANALYTICS_RESOURCE_INTEGRATION.md)      | Store API / 场景示例   |
| 查看开发进度            | [开发进度](./ANALYTICS_RESOURCE_PROGRESS.md)         | Phase 1-4              |
| 了解结果集优化计划      | [优化计划](./QUERY-RESULT-OPTIMIZATION-PLAN.md)      | A~H 组 39 项           |
| 查看结果集优化进度      | [优化进度](./QUERY-RESULT-OPTIMIZATION-PROGRESS.md)  | 8 阶段追踪             |
| 调用结果集 V2 接口      | [V2 接口契约](./QUERY-RESULT-API-V2.md)              | Store/Composable/组件  |
| 了解结果集 V2 架构      | [V2 架构设计](./QUERY-RESULT-ARCHITECTURE-V2.md)     | 组件树/数据流/DuckDB   |
| 配置资源管理设置        | [设置文档](./ANALYTICS_RESOURCE_SETTINGS.md)         | 设置类别与联动         |

---

## 架构版本

| 版本 | 日期       | 说明                                           |
| ---- | ---------- | ---------------------------------------------- |
| v2.3 | 2026-05-08 | SQL 编辑器架构优化全部完成（方言高亮增量更新） |
| v2.2 | 2026-05-08 | SQL 编辑器架构优化全部完成（4 Phase）          |
| v2.1 | 2026-05-08 | SQL 编辑器架构优化计划制定                     |
| v2.0 | 2026-04-23 | 插件化架构优化（DDD 分层、事件总线、统一 API） |
| v1.0 | 2026-04-20 | 初始插件化架构                                 |

---

## 当前架构概览

```
src/
├── app/                    # 应用入口
├── extensions/
│   ├── core/               # 扩展系统核心（event-bus、types）
│   └── builtin/            # 内置插件
│       ├── connection/     # 连接管理
│       ├── database/       # 数据库导航
│       ├── navigator/      # 通用导航器
│       ├── query/          # 查询执行
│       └── workbench/      # SQL 工作台
├── shared/                 # 共享资源（api、components、composables、types、utils）
└── core/                   # 核心业务（project）
```

---

## 核心设计原则

1. **插件隔离** - 通过事件总线通信，禁止直接引用其他插件 store
2. **DDD 分层** - domain/infrastructure/ui 职责清晰
3. **共享资源中心化** - 统一到 shared/ 目录
4. **命名规范** - 文件 kebab-case，组件 PascalCase

---

## 相关文档

| 文档         | 路径                             |
| ------------ | -------------------------------- |
| 后端架构文档 | `../backend/ARCHITECTURE.md`     |
| 项目规则     | `.trae/rules/`                   |
| 技术栈规范   | `.trae/rules/technical-rules.md` |

---

## 维护

- **文档维护者**：RdataStation 前端团队
- **更新频率**：每次架构变更时同步更新
- **反馈渠道**：提交 Issue 或联系架构组
