# RdataStation 开发规范索引

> 版本：v1.0
> 最后更新：2026-05-03
> 状态：✅ 持续更新

---

## 概述

本文档是 RdataStation 项目开发规范的索引，AI 助手在生成代码时必须严格遵循以下规范。

---

## 规则文档

| 文档                                                         | 说明                                            | 优先级      |
| ------------------------------------------------------------ | ----------------------------------------------- | ----------- |
| [common-rules.md](./common-rules.md)                         | 架构红线、Trait约束、数据契约、连接池、错误处理 | 🔴 必读     |
| [rdata-station.md](./rdata-station.md)                       | 项目定位、技术栈、目录结构、编码规范            | 🔴 必读     |
| [technical-rules.md](./technical-rules.md)                   | 技术栈锁定、架构红线、代码规范                  | 🔴 必读     |
| [frontend-enterprise-spec.md](./frontend-enterprise-spec.md) | 前端规范（布局、UI、组件、TS）                  | 🟡 前端必读 |
| [git-commit-message.md](./git-commit-message.md)             | Git 提交规范（Gitmoji + Angular）               | 🟡 必读     |

---

## 快速导航

### 架构约束

- [架构红线](./common-rules.md#一架构红线architecture-gates) - 禁止循环依赖、层级越界
- [Trait 与接口约束](./common-rules.md#二trait-与-接口约束基于-traitsrs) - 禁止修改 driver/traits.rs

### 数据契约

- [IPC 生死线](./common-rules.md#三数据契约检查ipc-生死线) - 必须使用 Arrow 格式传输

### 连接池

- [Pool 下沉](./common-rules.md#四连接池与资源管理) - Pool 只负责连接，不负责 SQL 执行

### 错误处理

- [Rust 规范](./common-rules.md#五错误处理与-rust-规范) - 禁止 unwrap/expect，必须使用 CoreError

### 前端约束

- [SchemaObject 懒加载](./common-rules.md#六前端交互约束) - children: None 表示未加载

---

## 技术栈版本

### Rust Core

| 技术         | 版本   | 约束            |
| ------------ | ------ | --------------- |
| Rust Edition | 2021   | 禁止主版本升级  |
| Tokio        | 1.44.x | 允许 minor 升级 |
| Tauri        | 2.10.x | 允许 patch 升级 |
| sqlx         | 0.8.x  | 禁止 major 升级 |

### Vue 3 Frontend

| 技术          | 版本   | 约束            |
| ------------- | ------ | --------------- |
| Vue           | 3.5.x  | 允许 minor 升级 |
| TypeScript    | 5.8.x  | 允许 minor 升级 |
| AG Grid       | 33.x   | 允许 minor 升级 |
| Monaco Editor | 0.52.x | 允许 minor 升级 |

---

## 检查清单

在提交任何代码前，请自检以下问题：

- [ ] DuckDB 是否被写死为唯一执行引擎？（必须是可插拔的）
- [ ] QueryResult 内部是否包含 RecordBatch？
- [ ] services 层是否只调用 connection / driver，不直接碰 datasource？
- [ ] Rust 代码中是否存在 unwrap()？
- [ ] Pool 是否只负责连接，不负责 SQL 执行？

---

## 相关文档

| 文档          | 路径                           |
| ------------- | ------------------------------ |
| 项目文档中心  | `docs/README.md`               |
| 前端架构文档  | `docs/frontend/INDEX.md`       |
| 后端架构文档  | `docs/backend/README.md`       |
| Rust 详细文档 | `src-tauri/src/docs/README.md` |

---

## 版本历史

| 版本 | 日期       | 说明                   |
| ---- | ---------- | ---------------------- |
| v1.0 | 2026-05-03 | 初始版本，建立规则索引 |
