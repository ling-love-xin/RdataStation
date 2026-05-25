# 新增数据源深度分析与修复 v0.6.2

> 日期：2026-05-23
> 状态：✅ 已完成
> 基于：前后端全链路深度分析

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

| # | 严重度 | 问题 | 修复 |
|---|--------|------|------|
| **F1** | 高 | "保存"按钮只存前端内存不持久化，用户误解 | 按钮文字改为"暂存"(`navigator.save` → `"暂存"`)，语义准确 |
| **F2** | 高 | handleApply 全局+项目双存储非事务：一个成功一个失败导致不一致 | 全局连接优先，失败则跳过不存项目；全局成功+项目失败 → `message.warning()` 警告用户 |
| **F3** | 高 | snapshot_global_auth/network 失败静默吞错 | 独立 `await` + `try-catch`，失败时 `message.warning()` 通知用户 |
| **F4** | 中 | connect_with_type 持久化失败只 warn 不阻断 | 保留现状（连接本身成功，持久化是附加操作），但日志级别提升 |
| **F5** | 中 | useAddDataSource composable 与对话框状态并行 | 记录为已知技术债，后续统一 |
| **F6** | 中 | 测试连接成功后暗门保存认证，用户无感知 | 改为 `useDialog.info()` 确认框："连接测试成功，是否保存认证信息？"，用户确认后才保存 |
| **F7** | 中 | 测试连接触发全局持久化，留下冗余 DB 记录 | `connect_with_type` 新增 `skip_persistence` 参数，`test_connection` 传入 `true` 跳过所有持久化 |
| **F8** | 低 | selectStaging 不恢复 scope | ✅ 已实现（第317行 `if (s.scope) { scope.global/scope.project }`） |

---

## 三、变更文件清单

### 前端

| 文件 | 变更 |
|------|------|
| `src/shared/locales/zh-CN.json` | `navigator.save` → `"暂存"`；清理重复 JSON key；新增 3 个 i18n key |
| `src/extensions/builtin/connection/ui/components/AddDataSourceDialog.vue` | F2 双重存储错误处理；F3 快照失败警告；F6 `useDialog` 确认框 |

### 后端

| 文件 | 变更 |
|------|------|
| `src-tauri/src/core/services/connection_service.rs` | `connect_with_type` 新增 `skip_persistence: Option<bool>` 参数 |
| `src-tauri/src/commands/connection_commands.rs` | `test_connection` → `skip_persistence: Some(true)`；`connect_database` → `None` |

### 新增文档

| 文件 | 说明 |
|------|------|
| `src-tauri/src/docs/add-datasource-deep-analysis-fixes.md` | 本文档 |

---

## 四、按钮行为（修复后）

| 按钮 | 方法 | 行为 | 持久化 | 关闭 |
|------|------|------|--------|------|
| **取消** | `resetAndClose()` | 清空状态 + emit close | 否 | 是 |
| **测试连接** | `handleTest()` | buildUrl → invoke test_connection(**skip persistence**) → 弹窗结果 | 否（用户手动确认后才存认证） | 否 |
| **暂存** | `saveToStaging()` | 写入前端暂存列表 | **否** | 否 |
| **应用** | `handleApply()` | 逐项 connectDatabase + project createConnection；**失败时警告不静默** | 是（全局+项目双层） | 成功时是 |

---

## 五、已知技术债

| 项 | 说明 | 优先级 |
|----|------|--------|
| F5 | `useAddDataSource` composable 与对话框并行维护两套状态 | 低 — 后续统一重构 |
| URI 手动编辑 | `manualUri` 不经校验直接用于连接 | 低 — 需增加 URL 格式校验 |
| 认证密钥处理 | 认证保存的加密方案待完善 | 中 — 需统一加密存储策略 |

---

## 六、验证

- `cargo check` → 0 errors, 0 warnings
- `pnpm run lint` → 0 errors, 263 warnings（全部已有）