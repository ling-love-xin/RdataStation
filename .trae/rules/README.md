# RdataStation 开发规范索引

> 版本：v1.6
> 最后更新：2026-06-20
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

### 安全规范

- [密码脱敏](./common-rules.md#十数据源模块专项约束v051) - URL 中密码必须脱敏（mask_password_in_url）
- [AES-256-GCM 加密](./common-rules.md#101-认证配置auth_configs) - 密码写入前必须 encrypt_auth_data()
- [参数化查询](./common-rules.md#三数据契约检查ipc-生死线) - 使用 query_with_params() 防止 SQL 注入

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

| 技术            | 版本   | 约束            |
| --------------- | ------ | --------------- |
| Vue             | 3.5.x  | 允许 minor 升级 |
| TypeScript      | 6.0.x  | 允许 minor 升级 |
| Vite            | 8.0.x  | 允许 patch 升级 |
| AG Grid         | 35.3.x | 允许 minor 升级 |
| CodeMirror 6    | 6.x    | 允许 minor 升级 |
| Pinia           | 3.0.x  | 允许 minor 升级 |
| dockview-vue    | 6.1.x  | 允许 minor 升级 |
| naive-ui        | 2.44.x | 允许 minor 升级 |

### 测试依赖

| 技术                  | 版本   | 用途              |
| --------------------- | ------ | ----------------- |
| vitest                | 3.x    | 前端测试运行器    |
| @vue/test-utils       | 2.x    | Vue 组件挂载/查询 |
| @testing-library/vue  | 8.x    | 用户交互模拟      |
| jsdom                 | 25.x   | DOM 环境模拟      |

---

## 检查清单

在提交任何代码前，请自检以下问题：

### 架构检查
- [ ] DuckDB 是否被写死为唯一执行引擎？（必须是可插拔的）
- [ ] QueryResult 内部是否包含 RecordBatch？
- [ ] services 层是否只调用 connection / driver，不直接碰 datasource？
- [ ] Rust 代码中是否存在 unwrap()？
- [ ] Pool 是否只负责连接，不负责 SQL 执行？

### 数据安全
- [ ] 全局/项目 DB 字段映射是否一致？
- [ ] auth_data 只存认证凭据，不混入连接属性？
- [ ] 密码/Auth 凭据是否已 AES-256-GCM 加密（`AES:` 前缀）？
- [ ] 日志和错误消息中 URL 密码是否已脱敏（`mask_password_in_url`）？
- [ ] 网络认证配置是否同时支持全局和项目级查询？

### 测试检查
- [ ] 新增功能是否编写了单元测试？
- [ ] 敏感操作（密码加密/脱敏）是否编写了测试用例？
- [ ] 错误传播路径是否覆盖了测试？
- [ ] 前端组件是否按分层方案（L1渲染/L2交互/L3集成）编写了测试？
- [ ] E2E 全链路关键路径是否覆盖了测试？
- [ ] mod.rs 是否包含测试代码？（绝对禁止）

---

## 相关文档

| 文档                | 路径                                          |
| ------------------- | --------------------------------------------- |
| 项目文档中心        | `docs/README.md`                              |
| 前端架构文档        | `docs/frontend/INDEX.md`                      |
| 后端架构文档        | `docs/backend/README.md`                      |
| Rust 详细文档       | `src-tauri/src/docs/README.md`                |
| 网络连接设计        | `docs/backend/CONNECTION-METHOD-DESIGN.md`    |
| 网络配置 UI         | `docs/frontend/NETWORK-CONFIG-UI-DESIGN.md`   |
| **测试报告**        | `src-tauri/tests/TEST_REPORT.md`              |
| **连接最佳实践**    | `src-tauri/tests/BEST_PRACTICES.md`           |
| **E2E 测试 Spec**   | `.trae/specs/datasource-component-e2e-tests/` |

---

## 版本历史

| 版本 | 日期       | 说明                                                                                                                                                                                                                                                                                                                |
| ---- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| v1.6 | 2026-06-20 | v0.5.3 全面安全审计与测试覆盖：深度安全审计（SEC-001~SEC-008，修复 6/8 项）、Vue 组件分层测试（8 组件 24 用例 L1+L2+L3）、E2E Tauri 集成测试（15 用例 4 条全链路）、前端 Store/Composables 测试（72 用例）、后端测试修复（16 项编译错误+5 项安全缺陷）、TEST_REPORT.md v3.0 + BEST_PRACTICES.md v2.0、测试依赖（vitest/@vue/test-utils/@testing-library/vue/jsdom） |
| v1.5 | 2026-05-25 | v0.5.2 新增数据源审计修复：StagingItem全字段补齐、selectStaging恢复authConfigId/authMethod、create_auth_config返回AuthConfig、NetworkTab ID后端生成、useAuthConfig支持project级加载、handleApply projectPath修复、os_auth/trust不写入空认证、NetworkTab.onMounted scope both分流                                                    |
| v1.4 | 2026-05-25 | v0.5.1 数据源模块完善：auth_configs密码加密，network_configs新增auth_config_id引用，auth_type分类（DB/Network），global_connections/connections字段全链路对齐（25列），ConnectDatabaseInput补全                                                                                                                     |
| v1.3 | 2026-05-19 | v0.5.0 网络连接功能：SSH隧道(russh) + SSL/TLS(native-tls) + 代理配置，后端核心完成                                                                                                                                                                                                                                  |
| v1.2 | 2026-05-18 | Vite 6→8 升级，Rolldown 引擎，vue-tsc 3.x，@vitejs/plugin-vue 6.x                                                                                                                                                                                                                                                   |
| v1.1 | 2026-05-12 | 代码质量全面修复，TS/ESLint 零错误，依赖版本锁定                                                                                                                                                                                                                                                                    |
| v1.0 | 2026-05-03 | 初始版本，建立规则索引                                                                                                                                                                                                                                                                                              |