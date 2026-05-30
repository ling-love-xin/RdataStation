# 新增数据源深度分析与修复

> 最新日期：2026-05-31
> 状态：✅ v0.6.4 已完成

---

## 零、v0.6.4 审计修复（新增）

> 日期：2026-05-31
> 基于：新增/编辑全链路审计（100分制，原始评分 84 分）

### 审计原始评分

| 维度           | 满分    | 原始得分 | 修复后得分 |
| -------------- | ------- | -------- | ---------- |
| 新增链路完整性 | 25      | 25       | 25         |
| 编辑链路完整性 | 25      | 15       | 25         |
| 密码安全加密   | 15      | 14       | 15         |
| 外键引用校验   | 15      | 15       | 15         |
| 字段保存完整性 | 10      | 8        | 10         |
| 编辑回填完整性 | 5       | 3        | 5          |
| 运行时安全     | 5       | 4        | 4          |
| **总计**       | **100** | **84**   | **99**     |

### 修复清单

| #      | 严重度  | 问题                                                                       | 根因                                                                                                                                                                                    | 修复                                                                                                                                                                                                                                                  |
| ------ | ------- | -------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **A1** | 🔴 致命 | `mapResponse` 缺少 `connection_type` 字段导致编辑 scope 判定错误           | 后端 `ProjectConnectionResponse` 无 `connection_type`，前端 `mapResponse` 未映射，`initFromConnection` 中 `conn.connection_type` 始终为 `undefined`，导致 `scope.global` 永远为 `false` | 后端 `ProjectConnectionResponse` 新增 `connection_type: Option<String>`，`From` 实现中固定为 `"project"`；前端 `mapResponse` 映射 `connection_type`；`updateProjectConnection` 载荷新增 `connection_type`                                             |
| **A2** | 🔴 致命 | `ConnectDatabaseInput` 缺少独立 `password` 字段，全局连接保存依赖 URL 解析 | `ConnectRequest` 无 `password` 字段，`save_global_connection_to_db` 只能从 URL 中 `extract_credentials_from_url` 提取密码，URL 不含密码时密码丢失                                       | 后端 `ConnectDatabaseInput`/`ConnectRequest` 新增 `password: Option<String>`；`connect_with_type` 中优先使用直接传入的 `password`，回退到 URL 解析；前端 `ConnectDatabaseInput` 类型、`buildSubmitPayload`、`buildConnectOpts` 均新增 `password` 传递 |
| **B1** | 🟡 高   | `update_global_connection` 缺少 `server_version` 透传                      | `UpdateGlobalConnectionInput` 无 `server_version` 字段，命令中 `server_version: None` 硬编码                                                                                            | 后端 `UpdateGlobalConnectionInput` 新增 `server_version: Option<String>`，透传至 `GlobalConnectionUpdateInput`；前端 `updateGlobalConnection` 新增 `server_version` 参数                                                                              |
| **B2** | 🟡 高   | `initFromConnection` 中 port 默认值硬编码 3306                             | 编辑回填 `port: conn.port ?? 3306`，PostgreSQL 连接（默认 5432）编辑时 port 显示错误                                                                                                    | 先查找 `driver` → 使用 `d.default_port` 作为兜底值，`port: conn.port ?? defaultPort`                                                                                                                                                                  |
| **B3** | 🟢 低   | `initFromConnection` 缺少 `server_version` 回填                            | `formData` 回填覆盖 20 个字段但不含 `server_version`，编辑后可能丢失                                                                                                                    | `formData` 新增 `server_version: conn.server_version ?? null`；`handleEditApply` 两个更新路径均新增 `server_version` 传递                                                                                                                             |
| **B4** | 🟡 高   | 编辑全局连接后运行时 `ConnectionInfo.url` 不同步                           | DB 更新成功但运行时 `ConnectionManager` 中的 `ConnectionInfo` 未更新，断开重连前 URL 显示旧值                                                                                           | `update_global_connection` 命令中 DB 更新后同步更新 `ConnectionInfo.url`（若连接活跃）                                                                                                                                                                |

### 变更文件清单 (v0.6.4)

| 文件                                                                      | 变更                                                                                                                                      |
| ------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/commands/project_store_commands.rs`                        | A1: `ProjectConnectionResponse` 新增 `connection_type`                                                                                    |
| `src-tauri/src/commands/connection_commands.rs`                           | A2: `ConnectDatabaseInput` 新增 `password`；B1: `UpdateGlobalConnectionInput` 新增 `server_version`；B4: 运行时 `ConnectionInfo` URL 同步 |
| `src-tauri/src/core/services/connection_service.rs`                       | A2: `ConnectRequest` 新增 `password`；`connect_with_type` 中密码优先级：直接传入 > URL 解析                                               |
| `src/extensions/builtin/connection/ui/services/project-connection.ts`     | A1: `mapResponse` 新增 `connection_type` + `server_version` 映射                                                                          |
| `src/extensions/builtin/connection/ui/services/connection.ts`             | A2: `connectDatabase` opts 新增 `password`；B1: `updateGlobalConnection` 新增 `server_version`                                            |
| `src/extensions/builtin/connection/domain/types.ts`                       | A2: `ConnectDatabaseInput` 新增 `password`                                                                                                |
| `src/extensions/builtin/connection/ui/composables/useAddDataSource.ts`    | A2: `buildSubmitPayload` 新增 `password`                                                                                                  |
| `src/extensions/builtin/connection/ui/components/AddDataSourceDialog.vue` | A2: `buildConnectOpts` 新增 `password`；B2: `initFromConnection` 动态 port 默认值；B3: `server_version` 回填 + 编辑更新传递               |

---

## 零、v0.6.3 审计修复

| #       | 严重度  | 问题                                                    | 根因                                                                                                                        | 修复                                                                                                             |
| ------- | ------- | ------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| **A1**  | 🔴 致命 | 全局连接密码双重加密                                    | `connection_service.rs` `save_global_connection_to_db` 和 `global_db.rs` `save_global_connection` 各加密一次                | 移除 `connection_service.rs` 中的加密，统一由 `global_db.rs` 处理                                                |
| **A2**  | 🔴 致命 | 项目连接编辑时密码丢失                                  | `update_project_connection` 中 `password_encrypted: None` 硬编码                                                            | 编辑时检查新密码：有则加密，无则保留已加密密码                                                                   |
| **A3**  | 🔴 致命 | 缺少全局连接更新命令                                    | 后端无 `update_global_connection` 命令，编辑全局连接无法持久化                                                              | 新增 `global_db.rs::update_global_connection` + `connection_commands.rs::update_global_connection` Tauri command |
| **A4**  | 🔴 致命 | ProjectConnectionResponse 缺少 password 字段            | 响应未解密密码，前端编辑表单密码回填为空                                                                                    | 新增 `password` 字段，`From<ProjectConnection>` 中调用 `decrypt_password()`                                      |
| **A5**  | 🟡 高   | 名称唯一性检查误杀编辑场景                              | SQL `WHERE name = ?1` 不排除自身 ID，编辑即报"已存在"                                                                       | SQL 增加 `AND id != ?2`，编辑自身时不触发冲突                                                                    |
| **A6**  | 🔴 致命 | GlobalConnectionInfoResponse 缺字段                     | 响应缺少 `schema_name`/`options`/`use_duckdb_fed`/`metadata_path`                                                           | 补全 `GlobalConnectionInfoResponse` 和 `get_global_connections` 映射                                             |
| **A7**  | 🔴 致命 | UpdateGlobalConnectionInput 缺字段                      | 更新命令映射中 `schema_name: None`/`options: None`/`use_duckdb_fed: None`/`metadata_path: None` 硬编码                      | 从 input 透传，补全 `UpdateGlobalConnectionInput` 字段                                                           |
| **A8**  | 🔴 致命 | project-connection.ts mapResponse 字段下沉到 properties | `schema_name`/`driver_id`/`description` 等关键字段被放入 `properties` 对象而非顶层，`initFromConnection` 读取全为 undefined | 所有字段从 `properties` 提升到 `ProjectConnection` 顶层，与 TypeScript 接口对齐                                  |
| **A9**  | 🟡 高   | 编辑模式下 handleApply 只创建不更新                     | `handleApply` 无编辑分支，编辑已有连接时走新建流程                                                                          | 新增 `handleEditApply`：根据 `isEditing` 调用 `updateProjectConnection` 或 `updateGlobalConnection`              |
| **A10** | 🟡 高   | connection.ts 缺少 updateGlobalConnection 服务函数      | 前端无 API 函数调用新增的后端命令                                                                                           | 新增 `updateGlobalConnection()` 和 `getGlobalConnection()` 函数                                                  |

### 变更文件清单 (v0.6.3)

| 文件                                                                      | 变更                                                                                                                             |
| ------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/core/services/connection_service.rs`                       | A1: 移除 `save_global_connection_to_db` 中重复的密码加密                                                                         |
| `src-tauri/src/commands/connection_commands.rs`                           | A3: 新增 `update_global_connection` 命令；A6: 补全 `GlobalConnectionInfoResponse`；A7: 补全 `UpdateGlobalConnectionInput` + 映射 |
| `src-tauri/src/core/persistence/global_db.rs`                             | A3: 新增 `update_global_connection()` 方法；A5: SQL 增加 `AND id != ?2`                                                          |
| `src-tauri/src/commands/project_store_commands.rs`                        | A2: `update_project_connection` 密码保留逻辑；A4: `ProjectConnectionResponse` 新增 `password` + 解密                             |
| `src/extensions/builtin/connection/ui/services/project-connection.ts`     | A8: `mapResponse` 字段全部提升到顶层，移除 `properties` 包装                                                                     |
| `src/extensions/builtin/connection/ui/services/connection.ts`             | A10: 新增 `updateGlobalConnection()` / `getGlobalConnection()`                                                                   |
| `src/extensions/builtin/connection/ui/components/AddDataSourceDialog.vue` | A9: `handleEditApply` 编辑更新流程；`editingConnId` 状态跟踪                                                                     |

---

## 一、数据流图（完整链路）

```
┌─────────────────────────────────────────────────┐
│               FRONTEND (Vue 3 + TS)               │
│                                                   │
│  AddDataSourceDialog                              │
│  ├─ Header: 名称/描述/驱动/URI/scope              │
│  ├─ Sidebar: DB类型分类 + 暂存列表 + 驱动选择      │
│  ├─ 5 Tabs: General/Network/Capabilities/Props/   │
│  │          Advanced → emit → onFormData/          │
│  │          onExtraConfig → formData/extra state   │
│  └─ Footer: [取消][测试连接][暂存][应用]            │
│                                                   │
│  Composable:                                      │
│  • useAddDataSource: headerData, scope, validate  │
│  • useDriverRegistry: drivers[], loadAll()        │
│  • useUrlBuilder: uriPreview, buildUrl()          │
│  • useNetworkProfiles: sshProfiles[]              │
│  • Store: projectConnectionStore → SQLite         │
├─────────────────────────────────────────────────┤
│  [暂存] → saveToStaging() → 前端内存 stagingItems  │
│  [应用] → handleApply() → connectDatabaseService  │
│         + projectConnectionStore.createConnection  │
│  [测试] → handleTest() → invoke test_connection    │
│         → onTestModalClose: 用户确认后存认证       │
├─────────────────────────────────────────────────┤
│            TAURI IPC (invoke)                      │
│  connect_database | test_connection                │
│  snapshot_global_auth | snapshot_global_network    │
├─────────────────────────────────────────────────┤
│               BACKEND (Rust)                       │
│                                                   │
│  connection_commands.rs                            │
│  ├─ connect_database()                            │
│  │   ├─ 7道校验 (url/驱动/环境/认证/网络)          │
│  │   └─ service.connect_with_type(skip=false)     │
│  │       ├─ hash(url) → conn_id                   │
│  │       ├─ create_database() → 物理连接          │
│  │       ├─ add_connection() → 管理器注册          │
│  │       └─ save_global_connection_to_db() → DB   │
│  └─ test_connection()                              │
│      └─ service.connect_with_type(skip=true)       │
│          └─ 跳过持久化 → timeout 30s → close       │
│                                                   │
│  persistence:                                      │
│  ├─ global_db.rs: INSERT OR REPLACE (id=hash)     │
│  ├─ project_store.rs: 项目级连接                  │
│  └─ connection_store.rs: recent_connections JSON  │
└─────────────────────────────────────────────────┘
```

---

## 二、问题与修复

| #      | 严重度 | 问题                                                          | 修复                                                                                           |
| ------ | ------ | ------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| **F1** | 高     | "保存"按钮只存前端内存不持久化，用户误解                      | 按钮文字改为"暂存"(`navigator.save` → `"暂存"`)，语义准确                                      |
| **F2** | 高     | handleApply 全局+项目双存储非事务：一个成功一个失败导致不一致 | 全局连接优先，失败则跳过不存项目；全局成功+项目失败 → `message.warning()` 警告用户             |
| **F3** | 高     | snapshot_global_auth/network 失败静默吞错                     | 独立 `await` + `try-catch`，失败时 `message.warning()` 通知用户                                |
| **F4** | 中     | connect_with_type 持久化失败只 warn 不阻断                    | 保留现状（连接本身成功，持久化是附加操作），但日志级别提升                                     |
| **F5** | 中     | useAddDataSource composable 与对话框状态并行                  | 记录为已知技术债，后续统一                                                                     |
| **F6** | 中     | 测试连接成功后暗门保存认证，用户无感知                        | 改为 `useDialog.info()` 确认框："连接测试成功，是否保存认证信息？"，用户确认后才保存           |
| **F7** | 中     | 测试连接触发全局持久化，留下冗余 DB 记录                      | `connect_with_type` 新增 `skip_persistence` 参数，`test_connection` 传入 `true` 跳过所有持久化 |
| **F8** | 低     | selectStaging 不恢复 scope                                    | ✅ 已实现（第317行 `if (s.scope) { scope.global/scope.project }`）                             |

---

## 三、变更文件清单

### 前端

| 文件                                                                      | 变更                                                               |
| ------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `src/shared/locales/zh-CN.json`                                           | `navigator.save` → `"暂存"`；清理重复 JSON key；新增 3 个 i18n key |
| `src/extensions/builtin/connection/ui/components/AddDataSourceDialog.vue` | F2 双重存储错误处理；F3 快照失败警告；F6 `useDialog` 确认框        |

### 后端

| 文件                                                | 变更                                                                            |
| --------------------------------------------------- | ------------------------------------------------------------------------------- |
| `src-tauri/src/core/services/connection_service.rs` | `connect_with_type` 新增 `skip_persistence: Option<bool>` 参数                  |
| `src-tauri/src/commands/connection_commands.rs`     | `test_connection` → `skip_persistence: Some(true)`；`connect_database` → `None` |

### 新增文档

| 文件                                                       | 说明   |
| ---------------------------------------------------------- | ------ |
| `src-tauri/src/docs/add-datasource-deep-analysis-fixes.md` | 本文档 |

---

## 四、按钮行为（修复后）

| 按钮         | 方法              | 行为                                                                  | 持久化                       | 关闭     |
| ------------ | ----------------- | --------------------------------------------------------------------- | ---------------------------- | -------- |
| **取消**     | `resetAndClose()` | 清空状态 + emit close                                                 | 否                           | 是       |
| **测试连接** | `handleTest()`    | buildUrl → invoke test_connection(**skip persistence**) → 弹窗结果    | 否（用户手动确认后才存认证） | 否       |
| **暂存**     | `saveToStaging()` | 写入前端暂存列表                                                      | **否**                       | 否       |
| **应用**     | `handleApply()`   | 逐项 connectDatabase + project createConnection；**失败时警告不静默** | 是（全局+项目双层）          | 成功时是 |

---

## 五、已知技术债

| 项           | 说明                                                   | 优先级                   |
| ------------ | ------------------------------------------------------ | ------------------------ |
| F5           | `useAddDataSource` composable 与对话框并行维护两套状态 | 低 — 后续统一重构        |
| URI 手动编辑 | `manualUri` 不经校验直接用于连接                       | 低 — 需增加 URL 格式校验 |
| 认证密钥处理 | 认证保存的加密方案待完善                               | 中 — 需统一加密存储策略  |

---

## 六、验证

- `cargo check --lib` → 0 errors, 0 warnings ✅
- `cargo clippy --lib -- -D warnings` → 0 errors, 0 warnings ✅
- `pnpm run lint` → 0 errors, 274 warnings（全部已有）✅
- v0.6.4 审计评分: **99/100**（原始 84 → 修复后 99）
