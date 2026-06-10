# 新增数据源功能 — 全链路 Checklist + 进度分析

> 最后更新：2026-06-11
> 状态：8 轮审计 + 1 轮 GeneralTab 专项 + 1 轮全项目问题模式扫描 + 1 轮细节清扫 + 1 轮测试补充 + 1 轮 GeneralTab 字段过滤专项 + 1 轮测试连接超时深度审计 + 1 轮脏代码排查与UI紧凑化，共修复 83 项问题

---

## 一、修复总览

| 优先级 | 总数 | 已修复 | 修复内容 |
|:---:|:---:|:---:|------|
| 🔴 P0 | 7 | 7 | scope 双向选择 + saveToProject ID + NetworkTab 全局/项目 API + resolve_network_method_with_project 前缀路由 + handleTest projectPath 传递 |
| 🟡 P1 | 14 | 14 | 字段一致性 + addStaging + NetworkTab + DuckDB + os_auth + driver_kind + AuthConfigManager scope + useNetworkProfiles 合并 + advancedSchemaFields 死代码 + BASIC_SCHEMA_KEYS 分类 + test_connection project_path 传递 + test_network_config 项目级支持 + testChainHop projectPath |
| 🟢 P2 | 16 | 16 | url_template + findIndex + localStorage + savingRef + configSchema + 后端校验 + 关闭确认 + 死代码清理 ×2 + alert替换 ×2 + 认证类型补全 + EnvironmentSection ×4 + DataSourceSidebar + profScopeLabel 作用域 |
| 🧪 Test | 5 | 5 | Rust connection_commands + 前端 driver-adapter + useAddDataSource + useAuthConfig + GeneralTab |

### 已修复问题清单

| # | 优先级 | 文件 | 问题 | 修复 |
|:---:|:---:|------|------|------|
| 1 | 🔴 | useAddDataSource.ts | scope 序列化丢失 'both' | 三态三元表达式 |
| 2 | 🔴 | AddDataSourceDialog.vue | saveToProject 用原始 ID 而非快照后 ID | 改用 snapshotNetId/snapshotAuthId |
| 3 | 🟡 | AddDataSourceDialog.vue | saveToStaging 从 formData 读高级字段 | 改用独立 ref |
| 4 | 🟡 | useAddDataSource.ts | addStaging 重置不完全 | 补充 formData/protocolChain/envId/driverProps |
| 5 | 🟡 | NetworkTab.vue | scope.project 变化不重载 | watch(props.scope?.project) |
| 6 | 🟡 | DuckDBAccelSection.vue | 所有驱动显示 DuckDB | isDuckDBSupported 过滤 |
| 7 | 🟡 | AddDataSourceDialog.vue | os_auth/trust 弹窗 | 已排除（isAuthRequired 正确） |
| 8 | 🟡 | GeneralTab.vue | driver_kind 未差异化 | getDefaultFields 已有 |
| 9 | 🟢 | useAddDataSource.ts | localStorage 写满静默 | message.warning 通知 |
| 10 | 🟢 | AddDataSourceDialog.vue | findIndex 脆弱 fallback | 仅用 UUID 匹配 |
| 11 | 🟢 | AddDataSourceDialog.vue | saving ref 测试按钮 | 已有 :loading="testing" |
| 12 | 🟢 | useAddDataSource.ts | url_template 空值替换 | 填入实际 formData 值 |
| 13 | 🟢 | GeneralTab.vue | config_schema 降级 | driver_kind 差异化降级 |
| 14 | 🟢 | connection_commands.rs | 后端 name/port/url 无校验 | 添加正向验证 |
| 15 | 🟢 | AddDataSourceDialog.vue | handleClose 无确认 | NModal 确认弹窗 |
| 16 | 🟢 | i18n zh-CN/en | 关闭确认无国际化 | 3 个新 key |

### Round 3 修复 (12 项, 2026-06-09)

| # | 优先级 | 文件 | 问题 | 修复 |
|:---:|:---:|------|------|------|
| 17 | 🔴 | useAddDataSource.ts | buildSavePayload scope 不支持 'both' | 三态支持 |
| 18 | 🟡 | useAddDataSource.ts | formData 使用 Record<string,unknown> | ConnectionFormData interface |
| 19 | 🟡 | useAddDataSource.ts | isFileDatabase 硬编码 | KNOWN_FILE_DBS 常量 |
| 20 | 🟡 | useAddDataSource.ts | validateUrl 硬编码协议 | KNOWN_DB_PROTOCOLS 常量 |
| 21 | 🟡 | AddDataSourceDialog.vue | buildAuthData 硬编码 if-else | AUTH_TYPE_FIELDS 映射 |
| 22 | 🟡 | AddDataSourceDialog.vue | saveToStaging stagingIndex 越位 | 已有项时追加 |
| 23 | 🟡 | useAddDataSource.ts | addStaging 空项重复创建 | 原地重置 |
| 24 | 🟢 | useAddDataSource.ts | onPolicyOverride eslint-disable | PolicyOverrideNode 类型 |
| 25 | 🟢 | useAddDataSource.ts | countNetworkHops 计入禁用hop | 过滤 enabled !== false |
| 26 | 🟢 | useAddDataSource.ts | initDefault 写死 stagingIndex=0 | 选中最后命名项 |
| 27 | 🟢 | AddDataSourceDialog.vue | 驱动切换重复parse | same-driver 短路 |
| 28 | 🟢 | AddDataSourceDialog.vue | handleTest 无前端校验 | validate() 前置 |

### Round 4 修复 (6 项, 2026-06-09)

| # | 优先级 | 文件 | 问题 | 修复 |
|:---:|:---:|------|------|------|
| 29 | 🟡 | NetworkTab.vue | testChainHop 使用原生 alert() | useMessage() 替换 |
| 30 | 🟡 | useNetworkChain.ts | saveChainToDb .catch(() => null) | console.error + null |
| 31 | 🟡 | GeneralTab.vue | createNewDbFile 使用 prompt() | NModal + NInput |
| 32 | 🟢 | GeneralTab.vue | browseFile/browseCert 空 catch | console.warn |
| 33 | 🟢 | useAddDataSource.ts | initFromEdit JSON.parse 静默失败 | 增强日志 + 原始数据 |
| 34 | 🟢 | AddDataSourceDialog.vue | appliedIndices 收集后再标记 | 立即 markStagingApplied |

### Round 5 修复 (4 项, 2026-06-09)

| # | 优先级 | 文件 | 问题 | 修复 |
|:---:|:---:|------|------|------|
| 35 | 🟢 | useNetworkChain.ts | isMaxNetworkHops 计入禁用 hop | 改用 enabledNetworkHopCount |
| 36 | 🟢 | useNetworkChain.ts | remainingHops 计入禁用 hop | 改用 enabledNetworkHopCount |
| 37 | 🟢 | AddDataSourceDialog.vue | doSaveAuth 缺project path静默返回 | message.warning 提示 |
| 38 | 🟢 | useAddDataSource.ts | headerData.editUriMode 死代码 | 删除字段+重置代码 |

### GeneralTab 专项修复 (4 项, 2026-06-09)

| # | 优先级 | 文件 | 问题 | 修复 |
|:---:|:---:|------|------|------|
| 39 | 🔴 | GeneralTab.vue | username 重复渲染 | AUTH_MANAGED_KEYS 新增 'username' |
| 40 | 🔴 | GeneralTab.vue | 双解析器类型不一致 | 统一使用 parseConfigSchema，扩展 ConfigSchemaField |
| 41 | 🟢 | GeneralTab.vue | onMounted 重复调用 updateAdvancedSchemaFields | 移除重复调用 |
| 42 | 🟢 | GeneralTab.vue | parseSchemaToFormFields 未使用导入 | 移除导入 |

### Round 6 修复 — 全项目问题模式扫描 (8 项, 2026-06-09)

根因分析：GeneralTab 的问题源于 config_schema 解析逻辑被复制3次（driver-adapter.ts / schema-parser.ts / GeneralTab.vue），各自维护独立类型体系。全项目扫描发现同类模式。

| # | 优先级 | 文件 | 问题 | 修复 |
|:---:|:---:|------|------|------|
| 43 | 🔴 | NetworkTab.vue | saveNewProfile 永远调用全局API create_network_config | 根据 scope.project 选择 project_create_network_config 或 create_network_config |
| 44 | 🔴 | NetworkTab.vue | saveNewProfile 永远调用全局API create_auth_config | 根据 scope.project 选择 project_create_auth_config 或 create_auth_config |
| 45 | 🔴 | NetworkTab.vue | loadSavedAuthConfigs 只加载全局，不加载项目 | 合并全局+项目认证配置（与 useAuthConfig.loadAuthConfigs 对齐） |
| 46 | 🔴 | AuthConfigManager.vue | AUTH_TYPE_DEFS 缺少 ldap/os_auth/trust | 补全3种认证类型定义 |
| 47 | 🟡 | AuthConfigManager.vue | loadAuthConfigs 用 if/else 互斥而非合并 | 改为 if(isGlobal)+if(isProject) 合并模式 |
| 48 | 🟡 | useNetworkProfiles.ts | loadByTypeProject 覆盖（非合并）全局配置 | 改为 [...existing, ...new] 合并 |
| 49 | 🟢 | AuthConfigManager.vue | 三处使用原生 alert() | 替换为 useMessage().warning/.error |
| 50 | 🟢 | schema-parser.ts | 死代码（parseSchemaToFormFields 无人引用） | 删除文件 |

### Round 7 修复 — 测试补充 (2026-06-09)

前端 connection 模块零测试 → 新增 **39 tests**（3 个测试文件），Rust 新增 **16 tests**。

| # | 优先级 | 文件 | 问题 | 修复 |
|:---:|:---:|------|------|------|
| 51 | 🧪 | (缺失) | Rust: connection_commands 零测试 | 新增 `connection_commands_tests.rs` (16 tests) |
| 52 | 🧪 | (缺失) | 前端: driver-adapter 解析器零测试 | 新增 `driver-adapter.test.ts` (11 tests) |
| 53 | 🧪 | (缺失) | 前端: useAddDataSource staging 零测试 | 新增 `useAddDataSource.test.ts` (15 tests) |
| 54 | 🧪 | (缺失) | 前端: useAuthConfig 认证管理零测试 | 新增 `useAuthConfig.test.ts` (13 tests) |

### Round 8 修复 — GeneralTab 字段过滤专项 (2026-06-11)

根因分析：`configSchemaFields` 计算属性包含全部非认证字段，导致 `advancedSchemaFields` 始终为空 — "高级连接参数"区域成为死代码。

| # | 优先级 | 文件 | 问题 | 修复 |
|:---:|:---:|------|------|------|
| 55 | 🔴 | GeneralTab.vue | configSchemaFields 包含全部非认证字段，advancedSchemaFields 始终为空 | 引入 `BASIC_SCHEMA_KEYS` 集合分类基础/高级字段 |
| 56 | 🟡 | GeneralTab.vue | "高级连接参数"区域死代码 | 修复后正确渲染非基础字段（connect_timeout, ssl_ca 等） |
| 57 | 🧪 | (缺失) | 前端: GeneralTab 字段过滤零测试 | 新增 `GeneralTab.test.ts` (30 tests) 覆盖 AUTH_MANAGED_KEYS 过滤、getDefaultFields 降级、parseConfigSchema 解析、isFieldDisabled 禁用、advancedSchemaFields 分类 |

#### BASIC_SCHEMA_KEYS 分类设计

```typescript
// 基础连接字段（主 v-for 渲染）
const BASIC_SCHEMA_KEYS = new Set([
  'host', 'port', 'database', 'url', 'wasmPath', 'headers', 'file_path',
])

// configSchemaFields 仅渲染基础字段
configSchemaFields = allFields.filter(f => BASIC_SCHEMA_KEYS.has(f.key) && !AUTH_MANAGED_KEYS.has(f.key))

// advancedSchemaFields 渲染其余非认证字段
advancedSchemaFields = allFields.filter(f => !basicKeys.has(f.key) && !AUTH_MANAGED_KEYS.has(f.key))
```

## 二、验证结果

| 检查项 | 结果 |
|------|:---:|
| `cargo clippy -- -D warnings` | ✅ 零错误 |
| `pnpm run lint` (ESLint) | ✅ 零错误（4 预存 warning，仅测试文件） |

## 三、修改文件清单（24 个文件）

```
src-tauri/src/core/services/connection_service.rs ✏️ resolve_network_method_with_project 前缀路由重写
src-tauri/src/commands/connection_commands.rs       ✏️ 测试连接 project_path 传递 + 资源泄漏修复
src-tauri/src/commands/data_source_commands.rs      ✏️ test_network_config 项目级支持
src-tauri/src/commands/project_store_commands.rs     — 未修改

src/extensions/builtin/connection/ui/composables/
  useAddDataSource.ts                                ✏️ 死代码清理 ×2 (Round 9+Round 10)
  useNetworkProfiles.ts                              ✏️ _pickCmd 删除 + JSON.stringify(e) 反模式修复
  useNetworkChain.ts                                 — 未修改
  useUrlBuilder.ts                                   ✏️ getProto 协议前缀修复

src/extensions/builtin/connection/ui/components/
  AddDataSourceDialog.vue                            ✏️ handleTest 字段校验 + 动态 import + JSON.stringify(err) + 模板绑定
  tabs/GeneralTab.vue                                ✏️ useI18n 修复 + UI 紧凑化
  tabs/NetworkTab.vue                                ✏️ testChainHop projectPath + 死解构 ×2 + chainAuthCfgOpts + Record<any> + UI
  tabs/AdvancedTab.vue                               ✏️ 死 prop/emit 删除 + UI 紧凑化
  tabs/DuckDBAccelSection.vue                        — 未修改
  tabs/advanced/EnvironmentSection.vue               — 未修改
  tabs/advanced/PolicySections.vue                   — 未修改
  tabs/advanced/MetadataSection.vue                  — 未修改
  DataSourceHeader.vue                               ✏️ UI 紧凑化

src/shared/locales/
  zh-CN.json                                         — Round 9 未修改
  en.json                                            — Round 9 未修改
```