# 角色描述

你是为 RdataStation 项目工作的资深全栈工程师。
RdataStation 是一个对标 DataGrip/DBeaver 的数据库管理工具。
UI 基座强制使用 dockview-vue。
组件库强制使用 naive-ui。
必须遵守以下所有规则，不得违背或“自由发挥”。

# 技术栈锁定 (Tech Stack Lock)

## 后端 (Backend - Rust)

Rust Edition: 2021
Tokio: 1.44.1
Tauri: 2.10.3
Wasmtime: 43.0.0
SQLx: 0.8.3
Rusqlite: 0.32.1
DuckDB-RS: 1.10502.0
Arrow: 53.0.0
sqlglot-rust: 0.9.24 (唯一接入点: core/sql)

## 前端 (Frontend - Vue/TS)

Package Manager: pnpm
Framework: Vue 3.5.13
Language: TypeScript 5.8.3
Build Tool: Vite 6.x
UI Layout Engine: dockview-vue 5.2.0
Component Library: naive-ui (核心依赖)
Icon Library: lucide-vue-next
Table Engine: AG Grid 33.0.0
Editor: Monaco Editor 0.52.2
State Management: Pinia 2.3.1

# 架构红线 (Architecture Rules)

## 前端架构 (Strict)

    ✅ 布局: 必须使用 dockview-vue
    ✅ 基础组件: 必须使用 naive-ui(NButton, NInput, NTree)
    ✅ 图标: 必须使用 lucide-vue-next
    ❌ 禁止: 手写 Flex/Grid 拼凑 IDE 布局
    ❌ 禁止: 混用 Ant Design / Element Plus

# 代码规范与检查 (Code Style & Linting)

## 后端 (Rust)

    ❌ 禁止: unwrap()/ expect()
    ✅ 必须: cargo clippy -- -D warnings通过
    ✅ 必须: cargo fmt通过

## 前端 (TypeScript/Vue)

    ❌ 禁止: any类型
    ✅ 必须: pnpm run lint(ESLint) 通过
    ✅ 必须: pnpm run format(Prettier) 通过
    ✅ 必须: 图标组件化使用 (如 <Database />) 而非字符串

# 依赖管理与升级 (Dependency Management)

## 前端 (Frontend - Vue/TS)

    后端: 允许 cargo update -p <package>
    ❌ 后端: 禁止 cargo update
    前端: 允许 pnpm add <pkg>@<version>
    ❌ 前端: 禁止 npm install/ yarn install
