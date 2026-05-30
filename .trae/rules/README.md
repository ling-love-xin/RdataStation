# RdataStation 开发规范索引

> 版本：v1.5
> 最后更新：2026-05-25
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

| 技术         | 版本      | 约束            |
| ------------ | --------- | --------------- |
| Rust Edition | 2021      | 禁止主版本升级  |
| Tokio        | 1.44.1    | 允许 minor 升级 |
| Tauri        | 2.10.3    | 允许 patch 升级 |
| sqlx         | 0.8.3     | 禁止 major 升级 |
| Rusqlite     | 0.32.1    | 禁止 major 升级 |
| DuckDB-RS    | 1.10502.0 | 禁止 major 升级 |
| Arrow        | 58.1.0    | 禁止 major 升级 |
| sqlglot-rust | 0.9.24    | 禁止 major 升级 |
| russh        | 0.49.2    | 禁止 major 升级 |
| russh-keys   | 0.49.2    | 禁止 major 升级 |
| native-tls   | 0.2.14    | 禁止 major 升级 |

### Vue 3 Frontend

| 技术         | 版本   | 约束            |
| ------------ | ------ | --------------- |
| Vue          | 3.5.x  | 允许 minor 升级 |
| TypeScript   | 6.0.x  | 允许 minor 升级 |
| Vite         | 8.0.x  | 允许 patch 升级 |
| AG Grid      | 35.3.x | 允许 minor 升级 |
| CodeMirror 6 | 6.x    | 允许 minor 升级 |
| Pinia        | 3.0.x  | 允许 minor 升级 |
| dockview-vue | 6.1.x  | 允许 minor 升级 |
| naive-ui     | 2.44.x | 允许 minor 升级 |

---

## 检查清单

在提交任何代码前，请自检以下问题：

- [ ] DuckDB 是否被写死为唯一执行引擎？（必须是可插拔的）
- [ ] QueryResult 内部是否包含 RecordBatch？
- [ ] services 层是否只调用 connection / driver，不直接碰 datasource？
- [ ] Rust 代码中是否存在 unwrap()？
- [ ] Pool 是否只负责连接，不负责 SQL 执行？
- [ ] 全局/项目 DB 字段映射是否一致？
- [ ] auth_data 只存认证凭据，不混入连接属性？
- [ ] 密码/Auth 凭据是否已 AES-256-GCM 加密？

---

## 相关文档

| 文档          | 路径                                        |
| ------------- | ------------------------------------------- |
| 项目文档中心  | `docs/README.md`                            |
| 前端架构文档  | `docs/frontend/INDEX.md`                    |
| 后端架构文档  | `docs/backend/README.md`                    |
| Rust 详细文档 | `src-tauri/src/docs/README.md`              |
| 网络连接设计  | `docs/backend/CONNECTION-METHOD-DESIGN.md`  |
| 网络配置 UI   | `docs/frontend/NETWORK-CONFIG-UI-DESIGN.md` |

---

## 版本历史

| 版本 | 日期       | 说明                                                                                                                                                                                                                                                                             |
| ---- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| v1.5 | 2026-05-25 | v0.5.2 新增数据源审计修复：StagingItem全字段补齐、selectStaging恢复authConfigId/authMethod、create_auth_config返回AuthConfig、NetworkTab ID后端生成、useAuthConfig支持project级加载、handleApply projectPath修复、os_auth/trust不写入空认证、NetworkTab.onMounted scope both分流 |
| v1.4 | 2026-05-25 | v0.5.1 数据源模块完善：auth_configs密码加密，network_configs新增auth_config_id引用，auth_type分类（DB/Network），global_connections/connections字段全链路对齐（25列），ConnectDatabaseInput补全                                                                                  |
| v1.3 | 2026-05-19 | v0.5.0 网络连接功能：SSH隧道(russh) + SSL/TLS(native-tls) + 代理配置，后端核心完成                                                                                                                                                                                               |
| v1.2 | 2026-05-18 | Vite 6→8 升级，Rolldown 引擎，vue-tsc 3.x，@vitejs/plugin-vue 6.x                                                                                                                                                                                                                |
| v1.1 | 2026-05-12 | 代码质量全面修复，TS/ESLint 零错误，依赖版本锁定                                                                                                                                                                                                                                 |
| v1.0 | 2026-05-03 | 初始版本，建立规则索引                                                                                                                                                                                                                                                           |
