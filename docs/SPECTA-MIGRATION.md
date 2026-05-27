# SPECTA 迁移方案文档

> **版本**: v1.0  
> **日期**: 2026-05-26  
> **状态**: 实施中（Rust 端完成，前端待 bindings 生成后执行）  
> **作者**: RdataStation 开发团队

---

## 一、迁移概述

### 1.1 目标

将项目从 `ts-rs` (TypeScript 类型生成) 迁移至 `specta` + `tauri-specta` (类型 + Tauri 命令绑定生成)。

### 1.2 核心收益

| 维度 | ts-rs (旧) | specta (新) |
|------|-----------|------------|
| 类型生成 | ✅ 仅类型定义 | ✅ 类型定义 |
| 命令绑定 | ❌ 手写 `invoke<>()` | ✅ 自动生成 `commands.xxx()` |
| 事件类型 | ❌ 不涉及 | ✅ `collect_events!` |
| 版本 | 10.0 / 12.0 (稳定) | 2.0.0-rc.25 (RC) |

### 1.3 影响范围

```
Cargo.toml          ← ts-rs 移除，specta + tauri-specta 新增
build.rs            ← 移除 ts-rs export_types() 调用
src/core/types.rs   ← 25 类型 TS → Type，移除 ts-rs 属性
src/core/models.rs  ← 1 类型 TS → Type，RecordBatch #[specta(skip)]
src/core/persistence/  ← 22 类型新增 #[derive(Type)]
src/core/services/  ← 22 类型新增 #[derive(Type)]
src/core/driver/    ← 30+ 类型新增 #[derive(Type)]
src/commands/*.rs   ← 23 文件、325 命令新增 #[specta::specta]
src/lib.rs          ← 预留 specta 导出块（待 API 确认后激活）
tests/specta_export.rs ← 类型导出测试占位
src/generated/specta/  ← 绑定输出目录（新建）
```

---

## 二、依赖变更

### Cargo.toml

```diff
- ts-rs = { version = "10.0", features = ["serde-compat"] }
+ specta = { version = "=2.0.0-rc.25", features = ["derive", "function", "collect", "serde_json"] }
+ tauri-specta = { version = "=2.0.0-rc.25", features = ["typescript", "derive"] }
```

**feature 说明**：

| feature | 作用 |
|---------|------|
| `derive` | `#[derive(Type)]` 宏 |
| `function` | `specta::collect_types!` 宏 |
| `collect` | 类型收集扩展 |
| `serde_json` | `serde_json::Value` 实现 `Type` |
| `typescript` (tauri-specta) | TypeScript 导出能力 |
| `derive` (tauri-specta) | 命令收集宏 |

---

## 三、Rust 端变更详情

### 3.1 类型标注（`#[derive(Type)]`）

**变更总数：75+ 类型**

| 模块 | 文件 | 类型数 | 关键类型 |
|------|------|:---:|------|
| core/types.rs | 1 | 25 | DatabaseMeta, ColumnMeta, CacheStats, ... |
| core/models.rs | 1 | 1 | QueryResult (RecordBatch 字段 #[specta(skip)]) |
| core/persistence/ | 8 | 22 | AuthConfig, NetworkConfig, Environment, DriverDescriptor, ... |
| core/services/ | 5 | 22 | ConnectionType, MissingDriver, ResultSet, ... |
| core/driver/ | 8 | 30+ | DriverDescriptor, ConnectionConfig, SshConfig, SslConfig, ... |

**示例变更**：

```rust
// 之前
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct AuthConfig { ... }

// 之后  
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AuthConfig { ... }
```

### 3.2 命令标注（`#[specta::specta]`）

**变更总数：325 个命令，23 个文件**

| 文件 | 命令数 |
|------|:---:|
| data_source_commands.rs | 45 |
| analytics_resource_commands.rs | 26 |
| project_commands.rs | 26 |
| scratchpad_commands.rs | 26 |
| plugin_commands.rs | 23 |
| result_commands.rs | 21 |
| metadata_commands.rs | 17 |
| connection_commands.rs | 16 |
| sql_commands.rs | 15 |
| mock_commands.rs | 13 |
| metadata_cache_commands.rs | 12 |
| cache_warming_commands.rs | 9 |
| port_commands.rs | 8 |
| logging_commands.rs | 7 |
| mock_persistence_commands.rs | 7 |
| project_store_commands.rs | 7 |
| sql_template_commands.rs | 6 |
| driver_commands.rs | 5 |
| memory_commands.rs | 5 |
| sql_parser_commands.rs | 5 |
| performance_commands.rs | 3 |
| navigator_commands.rs | 2 |
| system_commands.rs | 1 |
| **合计** | **325** |

### 3.3 serde_json::Value 处理

启用 `serde_json` feature 后，`serde_json::Value` 自动实现 `Type`。

**影响的字段**：

| 文件 | 结构体 | 字段 |
|------|--------|------|
| core/driver/metadata.rs | DriverFormField | default_value: Option\<Value\> |
| core/services/result_service.rs | ResultSet | rows: Vec\<Vec\<Value\>\> |
| core/services/result_service.rs | ColumnInsightFull | sample: Vec\<Value\> |

### 3.4 RecordBatch 处理

`arrow::array::RecordBatch` 不实现 `Type`，使用 `#[specta(skip)]` 跳过：

```rust
#[derive(Debug, Clone, Type)]
pub struct QueryResult {
    pub columns: Vec<String>,
    #[specta(skip)]  // ← Arrow 类型不导出到 TS
    pub batches: Vec<ArrowBatch>,
    pub affected_rows: Option<usize>,
    pub is_read_only: Option<bool>,
}
```

### 3.5 编译修复

| 问题 | 文件 | 修复 |
|------|------|------|
| 缺 `use specta::Type;` | standard_pool.rs | 添加导入 |
| `auth_config_id` move after borrow | connection_service.rs:231 | `.as_ref()` 模式匹配 |
| ts-rs `export-impl` feature 不存在 | Cargo.toml | 移除（已迁移） |
| Cargo.toml feature typo | Cargo.toml | `functions` → `function` |

---

## 四、已知阻塞问题

### 4.1 Tauri capability 配置错误（预存，非迁移引入）

**错误**：
```
error: proc macro panicked
  --> src/lib.rs:446:14
  = help: capability with identifier default not found
```

**状态**：预存问题，迁移前即存在。`capabilities/default.json` 文件和 `tauri.conf.json` 引用配置正确，但 `generate_context!()` 编译时无法解析。

**影响**：阻止 `cargo check/build/test` 运行，进而无法生成 `bindings.ts`。

**待解决**：
- 检查 Tauri CLI / `@tauri-apps/cli` 版本匹配
- 尝试 `cargo clean` 后重新构建
- 检查 Tauri 2.x 环境完整性

---

## 五、前端替换计划（待 bindings.ts 生成后执行）

### 5.1 变更模式

```typescript
// 之前
import { invoke } from '@tauri-apps/api/core'
const result = await invoke<NetworkProfile>('create_network_config', { nc: cfg })
const result = await invoke<AuthConfig>('create_auth_config', { ac: config })

// 之后
import { commands } from '@/generated/specta/bindings'
const result = await commands.createNetworkConfig({ nc: cfg })
const result = await commands.createAuthConfig({ ac: config })
// ↑ 函数名 snake→camelCase 自动转换，类型自动推导
```

### 5.2 需要修改的前端文件（12 个文件，~60 处调用）

| 文件 | 调用数 | 说明 |
|------|:---:|------|
| shared/api/index.ts | ~5 | 共享 API 层 |
| connection/ui/services/connection.ts | ~10 | 连接 CRUD |
| connection/ui/services/project-connection.ts | ~8 | 项目连接 |
| connection/ui/composables/useAddDataSource.ts | ~8 | 数据源创建 |
| connection/ui/composables/useAuthConfig.ts | ~6 | 认证配置 |
| connection/ui/composables/useNetworkChain.ts | ~5 | 网络链 |
| connection/ui/components/tabs/NetworkTab.vue | ~4 | 网络配置 UI |
| connection/ui/components/AddDataSourceDialog.vue | ~3 | 数据源对话框 |
| database/ui/services/metadata-cache-service.ts | ~5 | 元数据缓存 |
| database/ui/api/database-api.ts | ~3 | 数据库 API |
| workbench/ui/services/sql-editor-service.ts | ~2 | SQL 编辑器 |
| core/scoped-storage.ts | ~1 | 范围存储 |

### 5.3 替换步骤

1. **生成 bindings.ts**：解决 Tauri config 问题后运行 `cargo test --test specta_export`
2. **逐文件替换**：每个文件 `invoke()` → `commands.xxx()`
3. **类型检查**：`pnpm run typecheck`
4. **功能测试**：创建连接 → 配置 SSH → 保存 → 测试连接

---

## 六、TypeScript 绑定导出

### 6.1 导出测试

文件：`src-tauri/tests/specta_export.rs`

```bash
# 在 bindings API 确认后运行
cargo test --test specta_export -- --nocapture
```

### 6.2 导出机制（待激活）

specta v2 支持的导出方式：

**方式 A — tauri_specta::ts::export（简单）**：
```rust
tauri_specta::ts::export(
    specta::collect_types![...],
    "../src/generated/specta/bindings.ts",
).unwrap();
```

**方式 B — Builder 插件（推荐，支持事件）**：
```rust
let specta_builder = tauri_specta::ts::builder()
    .commands(tauri_specta::collect_commands![...])
    .events(tauri_specta::collect_events![...])
    .path("../src/generated/specta/bindings.ts")
    .into_plugin();

tauri::Builder::default()
    .plugin(specta_builder)
    ...
```

> **当前状态**：rc.25 API 路径待确认（`ts::export` / `collect_types` 模块路径与 rc.3-rc.21 文档有差异），故 `tests/specta_export.rs` 当前为占位。

---

## 七、文件变更清单

### 新增文件
| 文件 | 说明 |
|------|------|
| `src/generated/specta/` | 绑定输出目录 |
| `src-tauri/tests/specta_export.rs` | 类型导出测试 |

### 修改文件（Rust）
| 文件 | 变更类型 |
|------|------|
| `src-tauri/Cargo.toml` | 依赖替换（ts-rs → specta + tauri-specta） |
| `src-tauri/build.rs` | 移除 ts-rs export_types 调用 |
| `src-tauri/src/lib.rs` | 移除 specta export 块（等 API 确认） |
| `src-tauri/src/core/types.rs` | 25 类型 TS → Type，移除 ts-rs 属性，移除 export_types() |
| `src-tauri/src/core/models.rs` | 1 类型 TS → Type，RecordBatch skip |
| `src-tauri/src/core/persistence/auth_store.rs` | AuthConfig 加 Type |
| `src-tauri/src/core/persistence/network_store.rs` | NetworkConfig 加 Type |
| `src-tauri/src/core/persistence/env_store.rs` | Environment + EnvironmentPolicy 加 Type |
| `src-tauri/src/core/persistence/project_store.rs` | 4 类型加 Type |
| `src-tauri/src/core/persistence/driver_store.rs` | 3 类型加 Type |
| `src-tauri/src/core/persistence/plugin_store.rs` | 5 类型加 Type |
| `src-tauri/src/core/persistence/insight_meta_store.rs` | 1 类型加 Type |
| `src-tauri/src/core/services/connection_manager.rs` | ConnectionType enum 加 Type |
| `src-tauri/src/core/services/driver_service.rs` | MissingDriver 加 Type |
| `src-tauri/src/core/services/plugin_service.rs` | 2 类型加 Type |
| `src-tauri/src/core/services/sql_parser_service.rs` | SqlDialect enum 加 Type |
| `src-tauri/src/core/services/result_service.rs` | 17 类型加 Type |
| `src-tauri/src/core/services/connection_service.rs` | borrow fix (.as_ref()) |
| `src-tauri/src/core/driver/registry/descriptors.rs` | 6 类型加 Type |
| `src-tauri/src/core/driver/registry/config.rs` | 1 类型加 Type |
| `src-tauri/src/core/driver/driver_config.rs` | 4 类型加 Type |
| `src-tauri/src/core/driver/connection/config.rs` | 9 类型加 Type |
| `src-tauri/src/core/driver/introspection.rs` | 1 类型加 Type |
| `src-tauri/src/core/driver/metadata.rs` | 4 类型加 Type，serde_json::Value skip |
| `src-tauri/src/core/driver/traits.rs` | 7 类型加 Type |
| `src-tauri/src/core/driver/standard_pool.rs` | 添加 use specta::Type |
| `src-tauri/src/commands/*.rs` (23 files) | 325 #[specta::specta] 添加 |

### 修改文件（前端 — 待执行）
| 文件 | 变更类型 |
|------|------|
| `src/shared/api/index.ts` | invoke → commands |
| `src/extensions/builtin/connection/ui/services/connection.ts` | invoke → commands |
| `src/extensions/builtin/connection/ui/services/project-connection.ts` | invoke → commands |
| `src/extensions/builtin/connection/ui/composables/useAddDataSource.ts` | invoke → commands |
| `src/extensions/builtin/connection/ui/composables/useAuthConfig.ts` | invoke → commands |
| `src/extensions/builtin/connection/ui/composables/useNetworkChain.ts` | invoke → commands |
| `src/extensions/builtin/connection/ui/components/tabs/NetworkTab.vue` | invoke → commands |
| `src/extensions/builtin/connection/ui/components/AddDataSourceDialog.vue` | invoke → commands |
| `src/extensions/builtin/database/ui/services/metadata-cache-service.ts` | invoke → commands |
| `src/extensions/builtin/database/ui/api/database-api.ts` | invoke → commands |
| `src/extensions/builtin/workbench/ui/services/sql-editor-service.ts` | invoke → commands |
| `src/core/scoped-storage.ts` | invoke → commands |

### 删除/废弃
| 文件/函数 | 说明 |
|------|------|
| `core/types.rs::export_types()` | ts-rs 导出函数 |
| `build.rs` 中 `export_types()` 调用 | 移除 |
| `src/generated/*.ts` (ts-rs 生成) | 替换为 specta/bindings.ts |

---

## 八、待办事项

- [ ] **紧急**：修复 Tauri capability `default` 配置问题（`generate_context!()` panic）
- [ ] **高**：确认 tauri-specta rc.25 的 `ts::export` / `collect_types` / `collect_commands` 正确 API 路径
- [ ] **高**：激活 `tests/specta_export.rs` 中实际的导出代码
- [ ] **中**：生成 `bindings.ts` 后执行前端 12 文件 invoke() → commands.xxx() 替换
- [ ] **中**：运行 `pnpm run typecheck` + `pnpm run lint` 验证前端
- [ ] **低**：clean 旧的 `src/generated/*.ts` (ts-rs 生成文件)

---

## 九、版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.0 | 2026-05-26 | 初始迁移：75+ 类型标注，325 命令标注，依赖替换完成 |