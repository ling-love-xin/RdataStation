# RdataStation 前端文档

> 版本：v2.1
> 更新日期：2026-05-08
> 状态：✅ 持续更新

---

## 文档结构

```
docs/frontend/
├── INDEX.md                        # 本文档（总索引）
├── ARCHITECTURE.md                 # 完整架构文档（核心）
├── CONFIG-SYSTEM.md                # 配置系统文档（新增）
├── DEVELOPMENT-GUIDE.md            # 快速开发指南
├── COMPONENTS.md                   # 组件开发规范
├── SQL-EDITOR.md                   # SQL 编辑器完整文档
└── SQL-EDITOR-OPTIMIZATION-PLAN.md # SQL 编辑器架构优化计划（⏳ 待确认）
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

| 我想... | 查看文档 | 章节 |
|---------|----------|------|
| 了解整体架构 | [架构文档](./ARCHITECTURE.md) | 架构概览、目录结构 |
| 了解 SQL 编辑器设计 | [SQL 编辑器](./SQL-EDITOR.md) | 全部 |
| 了解 SQL 编辑器优化计划 | [优化计划](./SQL-EDITOR-OPTIMIZATION-PLAN.md) | Phase 1-4 |
| 管理配置/主题/语言 | [配置系统](./CONFIG-SYSTEM.md) | 三层配置优先级、核心 API |
| 创建新插件 | [开发指南](./DEVELOPMENT-GUIDE.md) | 创建新插件 |
| 创建新组件 | [组件规范](./COMPONENTS.md) | 创建新组件 |
| 调用后端 API | [开发指南](./DEVELOPMENT-GUIDE.md) | 调用后端 API |
| 处理错误 | [开发指南](./DEVELOPMENT-GUIDE.md) | 错误处理 |
| 插件间通信 | [架构文档](./ARCHITECTURE.md) | 插件间通信机制 |
| 命名规范 | [架构文档](./ARCHITECTURE.md) | 命名规范 |
| 组件 Props/Emits | [组件规范](./COMPONENTS.md) | Props 规范、Emits 规范 |
| 性能优化 | [组件规范](./COMPONENTS.md) | 性能优化 |
| 代码审查 | [组件规范](./COMPONENTS.md) | 代码审查清单 |

---

## 架构版本

| 版本 | 日期 | 说明 |
|------|------|------|
| v2.1 | 2026-05-08 | SQL 编辑器架构优化计划制定 |
| v2.0 | 2026-04-23 | 插件化架构优化（DDD 分层、事件总线、统一 API） |
| v1.0 | 2026-04-20 | 初始插件化架构 |

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

| 文档 | 路径 |
|------|------|
| 后端架构文档 | `../backend/ARCHITECTURE.md` |
| 项目规则 | `.trae/rules/` |
| 技术栈规范 | `.trae/rules/technical-rules.md` |

---

## 维护

- **文档维护者**：RdataStation 前端团队
- **更新频率**：每次架构变更时同步更新
- **反馈渠道**：提交 Issue 或联系架构组
