# 新增数据源功能 — 全链路 Checklist + 进度分析

> 最后更新：2026-06-09
> 状态：审计完成，0 项已实现

---

## 一、整体进度概览

| 分类 | 总数 | ✅ 已实现 | ⚠️ 有问题 | ❌ 缺失 | 空壳/TODO |
|------|:---:|:---:|:---:|:---:|:---:|
| 对话框生命周期 | 6 | 6 | 0 | 0 | 0 |
| 驱动选择流程 | 8 | 7 | 1 | 0 | 0 |
| GeneralTab 通用配置 | 12 | 11 | 0 | 0 | 1 |
| NetworkTab 协议链 | 14 | 11 | 1 | 0 | 2 |
| AdvancedTab 高级配置 | 18 | 14 | 2 | 2 | 0 |
| Capabilities / Properties | 6 | 4 | 0 | 2 | 0 |
| Staging 暂存区 | 6 | 5 | 1 | 0 | 0 |
| Apply 批量应用 | 12 | 11 | 0 | 0 | 1 |
| Test Connection 测试连接 | 8 | 6 | 1 | 0 | 1 |
| Scope 作用域 | 5 | 4 | 0 | 0 | 1 |
| 错误/边界 | 9 | 5 | 3 | 1 | 0 |
| **合计** | **104** | **84** | **9** | **5** | **6** |

---

## 二、全链路 Checklist

### 1. 对话框生命周期 (6/6 ✅)

- [x] 1.1 打开对话框 — 新建数据源 ← AddDataSourceSidebar + DataSourceHeader + 5 Tab
- [x] 1.2 打开对话框 — 编辑已有连接 ← isEditing=true 分支
- [x] 1.3 关闭对话框（点取消/点叉号） ← `handleClose()` → `emit('update:modelValue', false)`
- [x] 1.4 关闭对话框（Apply 成功后） ← `handleCreateApply` / `handleEditApply` 结尾关闭
- [x] 1.5 `onMounted` 初始化 ← 加载数据源类型+驱动+环境+认证配置+网络配置
- [x] 1.6 国际化 $t() 覆盖 ← 标题/标签/按钮/占位符

### 2. 驱动选择流程 (7/8 ⚠1)

- [x] 2.1 侧边栏按 category 分组展示 DataSourceType
- [x] 2.2 搜索过滤数据源类型 ← `searchQuery` + `filteredTypes`
- [x] 2.3 选中类型后驱动下拉更新 ← `selectedTypeId` → `driverOptions` computed
- [x] 2.4 选择驱动 → onDriverChange ← `headerData.selectedDriverId` → 查找 Driver 对象
- [x] 2.5 URL 模板实时预览 ← `useUrlBuilder.generateUrl(formData, driver.url_template)`
- [x] 2.6 驱动切换时表单重置 ← `onDriverChange()` 清空 formData 部分字段
- [x] 2.7 `driver.enabled` 过滤 ← `driverOptions` 只返回 `d.enabled`
- [ ] 2.8 旧 StagingItem 引用了 disabled 驱动时的降级展示 ← 当前直接隐藏 disabled 驱动，回选不到 ⚠

### 3. GeneralTab 通用配置 (11/12 ❌1)

- [x] 3.1 host / port / database 输入
- [x] 3.2 username / password 输入
- [x] 3.3 文件路径选择（file-based 驱动如 SQLite）
- [x] 3.4 认证方式选择 → `authMethod` 下拉
- [x] 3.5 已保存认证配置选择 → `filteredAuthConfigOpts` 下拉
- [x] 3.6 手动填写认证信息 → "— 手动填写 —" 选项
- [x] 3.7 AuthConfigManager Modal 打开 → `showAuthManager = true`
- [x] 3.8 AuthConfigManager 关闭后刷新列表 → `onAuthManagerClose()` → `loadAuthConfigs()`
- [x] 3.9 驱动支持的认证类型过滤 → `authMethodOpts` 根据 `supportedAuthTypes` 过滤
- [x] 3.10 密码显隐切换
- [x] 3.11 formData emit 回父组件 → `@update:form-data="onFormData"`
- [ ] 3.12 表单字段由 config_schema 动态驱动 ← 当前使用旧 DriverField[] 硬编码 ❌

### 4. NetworkTab 协议链 (11/14 ⚠1 ❌2)

- [x] 4.1 协议链可视化展示 ← ProtocolChain 组件渲染
- [x] 4.2 添加 SSH hop ← `addHop('ssh')`
- [x] 4.3 添加 Proxy hop ← `addHop('proxy')`
- [x] 4.4 添加 SSL/TLS hop ← `addHop('ssl')` → append at end
- [x] 4.5 删除 hop ← `deleteHop(id)` 每种协议至少保留 1 跳
- [x] 4.6 启用/禁用 hop ← `toggleHop(id)`
- [x] 4.7 拖拽排序协议链 ← `onDragStart` / `onDrop` + `ensureSslAtEnd()`
- [x] 4.8 跳数上限提示 ← `isMaxNetworkHops` + `showHopWarning`
- [x] 4.9 拓扑预览 ← `topologyNodes` computed → 本机→SSH→Proxy→TLS
- [x] 4.10 预估延迟 ← `estimatedLatency` = 跳数 × 25ms
- [x] 4.11 三种配置模式切换 ← `select` / `new` / `custom`
- [ ] 4.12 onMounted 同时 loadAll() 全局 + loadAllProject() 项目 ← 需确认当前代码是否同时调用 ⚠
- [ ] 4.13 新建配置后实时同步到共享状态 ← useNetworkChain 独立 profiles 不同步 ❌
- [ ] 4.14 协议链导出为 networkConfig emit ← `saveChainToDb()` 空壳，emit 需确认 ❌

### 5. AdvancedTab 高级配置 (14/18 ⚠2 ❌2)

- [x] 5.1 环境选择器 ← `EnvironmentSelector` 组件
- [x] 5.2 环境策略标签展示 ← `envPolicyTags` 5 维标签
- [x] 5.3 策略覆盖检测 ← `isPolicyOverridden` 标记
- [x] 5.4 环境快照状态显示 ← `envSnapshotting` / `envSnapshotId`
- [x] 5.5 安全策略配置 ← `SecurityPolicySection`（只读/写确认/DDL确认/DROP策略/自动提交/行限制/大小限制）
- [x] 5.6 Schema 策略配置 ← autoLoad / loadDepth / showSystem / refreshInterval
- [x] 5.7 性能策略配置 ← poolSize / queryTimeout / connectTimeout / heartbeat / maxReconnect
- [x] 5.8 审计策略配置 ← sqlLog / retentionDays
- [x] 5.9 UI 策略配置 ← topBarColor / numberFormat / nullDisplay / maxColumnWidth
- [x] 5.10 DuckDB 加速开关 ← `duckdbEnabled` v-model
- [x] 5.11 DuckDB 同步策略选择 ← `duckdbSync` (manual/interval/trigger)
- [x] 5.12 DuckDB 同步间隔/内存/线程配置
- [ ] 5.13 环境选择后自动加载策略 ← 当前需手动 fetchPolicies ⚠
- [ ] 5.14 DuckDB 加速仅对支持的类型显示 ← 当前所有驱动都显示，未过滤 ⚠
- [x] 5.15 数据源元数据 ← schemaName / options / metadataPath / tags
- [ ] 5.16 高级配置序列化为 advancedOptions 传给父组件 ← 确认 `watch` 是否正确触发 ❌
- [ ] 5.17 `checkPolicyOverride` 过于冗余 ← 每个策略字段独立 watch ❌
- [ ] 5.18 文件大小 1162 行 → 需拆分 ← God Object 初期形态 ❌

### 6. Capabilities / Properties Tab (4/6 ❌2)

- [x] 6.1 解析 driver.capabilities JSON → 表格展示
- [x] 6.2 支持/不支持的能力图标区分
- [ ] 6.3 能力数据 emit 到父组件写入 advancedOptions ← 当前仅展示，emit 未实现 ❌
- [x] 6.4 解析 driver.driver_properties JSON → KV 表格可编辑
- [x] 6.5 驱动属性增删改
- [ ] 6.6 驱动属性 emit 到父组件 ← 需确认 emit 链路 ❌

### 7. Staging 暂存区 (5/6 ⚠1)

- [x] 7.1 保存到暂存 ← `saveToStaging()` → `localStorage`
- [x] 7.2 新增空白暂存 ← `handleAddStaging()` → push 空 StagingItem
- [x] 7.3 删除暂存项 ← `handleRemoveStaging(index)`
- [x] 7.4 选择暂存项恢复表单 ← `handleSelectStaging(index)`
- [x] 7.5 暂存计数显示 ← `stagingItems.length`
- [ ] 7.6 StagingItem 含明文 password ← buildStagingItem 未删除 password ⚠

### 8. Apply 批量应用 (11/12 ❌1)

- [x] 8.1 Apply 入口 ← `handleApply()` / `handleEditApply()`
- [x] 8.2 保存当前表单到暂存 ← 在 Apply 前调用 saveToStaging
- [x] 8.3 遍历 stagingItems 批量处理
- [x] 8.4 认证快照 ← `create_auth_config()` → 更新 authConfigId
- [x] 8.5 网络快照 ← `create_network_config()` → 更新 networkConfigId
- [x] 8.6 环境快照 ← `snapshot_global_env()` → 更新 environmentId
- [x] 8.7 创建项目连接 ← `create_project_connection(input)`
- [x] 8.8 创建全局连接 ← `create_global_connection(input)`
- [x] 8.9 自动建立连接 ← `connect_to_datasource(connId)`
- [x] 8.10 全部成功后清除暂存 ← `stagingItems.value = []`
- [x] 8.11 失败时中断并显示错误 ← try/catch + $message.error
- [ ] 8.12 handleCreateApply ~145 行过长 → 需拆分为 useBatchApply ❌

### 9. Test Connection 测试连接 (6/8 ⚠1 ❌1)

- [x] 9.1 点击测试连接 → `handleTest()`
- [x] 9.2 构建 ConnectDatabaseInput → `buildConnectOpts()`
- [x] 9.3 调用后端 test_connection 命令
- [x] 9.4 连接成功后自动断开 ← closeConnection()
- [x] 9.5 测试结果弹窗展示 ← TestResultModal（成功/失败/延迟/网络拓扑）
- [ ] 9.6 os_auth/trust 不弹保存认证对话框 ← `onTestModalClose` 未排除 ⚠
- [x] 9.7 测试结果持久化到底部状态 ← `testResult` ref
- [ ] 9.8 超时处理/重试 ← 当前无显式超时机制 ❌

### 10. Scope 作用域 (4/5 ❌1)

- [x] 10.1 全局勾选框 ← `scope.global` v-model
- [x] 10.2 项目勾选框 ← `scope.project` v-model
- [x] 10.3 全局+项目同时勾选 ← 互补共存，不互斥
- [x] 10.4 scope 变化时告警 ← `scopeChangedWarning` NAlert
- [ ] 10.5 scope 变化时重新加载认证/网络配置列表 ← 当前只 onMounted 加载一次 ❌

### 11. 错误/边界 (5/9 ⚠3 ❌1)

- [x] 11.1 空名称校验
- [x] 11.2 必选驱动校验
- [x] 11.3 端口范围校验（前端）
- [ ] 11.4 端口范围校验（后端） ← connection_service.rs 未做验证 ⚠
- [ ] 11.5 URL 非空校验（后端） ← connection_service.rs 未做验证 ⚠
- [x] 11.6 连接失败友好提示
- [x] 11.7 network error 捕获
- [ ] 11.8 重复连接名称检测 ← 未实现 ⚠
- [ ] 11.9 Driver 未安装时的安装引导 ← missingDrivers 有数据但前端未展示引导 ❌

---

## 三、已知问题详情（9 项）

| # | 位置 | 优先级 | 问题 | 影响 |
|:---:|------|:---:|------|------|
| 1 | `useNetworkChain.ts` | 🔴 P0 | 独立 profiles 与 useNetworkProfiles 不同步 | 协议链新建配置后下拉不显示 |
| 2 | `useAddDataSource.ts:buildStagingItem` | 🔴 P0 | formData 含明文 password 写入 localStorage | 安全风险 |
| 3 | `useAddDataSource.ts:isResetting` | 🔴 P0 | 全局 Boolean 标志屏蔽 watcher | 异步异常永久卡死 watcher |
| 4 | `GeneralTab.vue` | 🟡 P1 | 表单字段硬编码 DriverField[]，未用 config_schema | 切换驱动字段不变化 |
| 5 | `AdvancedTab.vue` | 🟡 P1 | 1162 行 God Object | 维护困难 |
| 6 | `AdvancedTab.vue:onEnvChange` | 🟡 P1 | 选环境不自动 fetchPolicies | 策略标签不更新 |
| 7 | `AdvancedTab:DuckDBAccelSection` | 🟡 P1 | 所有驱动显示 DuckDB 加速，未过滤 | 不支持的 DB 类型显示无效选项 |
| 8 | `connection_service.rs` | 🟡 P1 | 后端正输入验证缺失 | 无效端口/URL 不被拦截 |
| 9 | `CapabilitiesTab.vue` | 🟢 P2 | 能力数据仅展示不写入后端 | 查询引擎无法利用能力元数据 |

---

## 四、缺失功能（5 项）

| # | 功能 | 预期位置 | 说明 |
|:---:|------|------|------|
| 1 | 重复连接名称检测 | handleApply 入口 | Apply 前应检查 stagingItems 是否有同名连接 |
| 2 | Driver 未安装引导 | AddDataSourceSidebar | missingDrivers 有数据但前端未展示"安装"入口 |
| 3 | 测试连接超时机制 | handleTest | 前端无超时 promise race |
| 4 | URI 编辑模式模板校验 | DataSourceHeader | 手动编辑 URI 时不检查占位符对应关系 |
| 5 | scope 变化时重新加载认证/网络 | watch(scope) | 从全局切到全局+项目时认证/网络列表不变 |

---

## 五、TODO / 空壳（6 项）

| # | 位置 | 内容 |
|:---:|------|------|
| 1 | `GeneralTab.vue` | config_schema 动态表单解析器 |
| 2 | `NetworkTab:saveChainToDb` | 整链保存为 network_config（定义但无调用） |
| 3 | `NetworkTab:onMounted` | 是否同时 loadAll() + loadAllProject() |
| 4 | `AdvancedTab:advancedOptions watch` | 是否在所有策略变化时正确触发 emit |
| 5 | `AddDataSourceDialog:handleCreateApply` | 145 行 → 需要拆分为 useBatchApply |
| 6 | `TestResultModal:onClose` | os_auth/trust 过滤逻辑 |