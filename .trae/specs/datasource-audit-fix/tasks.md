# Tasks

- [x] Task 1: 合并 useNetworkChain 与 useNetworkProfiles 配置文件状态
  - [x] 1.1: `useNetworkChain` 内部移除 `sshProfiles`/`sslProfiles`/`proxyProfiles` ref 声明
  - [x] 1.2: `getProfiles()` 改为从 `useNetworkProfiles()` 读取共享状态
  - [x] 1.3: `saveNewHop()` 改为调用 `useNetworkProfiles.saveProjectProfile()` 写入后端
  - [x] 1.4: `loadProfilesFromDb()` 改为调用 `useNetworkProfiles.loadAll()` 
  - [x] 1.5: `deleteProfile()` 改为调用 `useNetworkProfiles.removeProjectProfile()` 
  - [x] 1.6: `initProfiles()` 移除或改为空操作
  - [x] 1.7: `networkConfig` computed 适配新的 profiles 来源
  - [x] 1.8: `topologyNodes` computed 适配新的 profiles 来源
  - [x] 1.9: `vue-tsc --noEmit` + ESLint 通过

- [x] Task 2: GeneralTab 动态表单由 config_schema 驱动
  - [x] 2.1: 新增 `parseConfigSchema(schema: string): DriverField[]` 工具函数
  - [x] 2.2: `GeneralTab` 中移除旧 `DriverField[]` 硬编码字段列表
  - [x] 2.3: 监听 `driver` prop 变化，自动调用 `parseConfigSchema(driver.config_schema)` 重新渲染
  - [x] 2.4: 表单字段按 config_schema 的 order 属性排序（缺失时按定义顺序）
  - [x] 2.5: 认证选择器（authMethod/selectedAuthConfig）仍然在 GeneralTab 中独立渲染
  - [x] 2.6: `vue-tsc --noEmit` + ESLint 通过

- [x] Task 3: 环境选择器自动加载策略
  - [x] 3.1: `EnvironmentStore.selectEnv` 变为 async，内部自动调用 `fetchPolicies(envId)`
  - [x] 3.2: `AdvancedTab` 的 `onEnvChange` 处理器改为调用 `environmentStore.selectEnv()`
  - [x] 3.3: 移除 `AddDataSourceDialog` 中任何手动 `fetchPolicies` 调用
  - [x] 3.4: `vue-tsc --noEmit` + ESLint 通过

- [x] Task 4: StagingItem 密码安全 + isResetting 消除
  - [x] 4.1: `buildStagingItem()` 在写入前从 `formData` 中 `delete formData.password`
  - [x] 4.2: `useAddDataSource.ts` 移除 `isResetting` ref 及所有赋值引用
  - [x] 4.3: `handleSelectStaging` 用 `nextTick` 替代 `isResetting` 守卫
  - [x] 4.4: `handleAddStaging` 移除 isResetting 相关代码
  - [x] 4.5: `vue-tsc --noEmit` + ESLint 通过

- [x] Task 5: 拆分 AdvancedTab（1340 行 → 198 行）
  - [x] 5.1: 抽取 `EnvironmentSection.vue` — 环境选择器 + 策略标签 + 快照指示 + CRUD
  - [x] 5.2: 抽取 `DuckDBAccelSection.vue` — DuckDB 加速配置（已存在，确认完整性）
  - [x] 5.3: 抽取 `PolicySections.vue` — 安全/性能/Schema/审计/UI 策略
  - [x] 5.4: 抽取 `MetadataSection.vue` — 数据源元数据（schemaName/options/metadataPath/tags）
  - [x] 5.5: `AdvancedTab.vue` 变为组合层，198 行
  - [x] 5.6: `vue-tsc --noEmit` + ESLint 通过

- [x] Task 6: CapabilitiesTab 数据写入 advancedOptions
  - [x] 6.1: `CapabilitiesTab` 解析 `driver.capabilities` JSON 字符串
  - [x] 6.2: 将解析后的能力标志通过 `extraConfig` emit 传递给父组件
  - [x] 6.3: `AddDataSourceDialog` 合并 capabilities 标志到 `advancedOptions`
  - [x] 6.4: `vue-tsc --noEmit` + ESLint 通过

- [x] Task 7: URI 编辑模式校验提示
  - [x] 7.1: `DataSourceHeader` 添加 `uriWarning` computed — 比对 url_template 占位符
  - [x] 7.2: 当 `uriEditing=true` 且 URL 与 url_template 模式不匹配时显示 NAlert 警告
  - [x] 7.3: `vue-tsc --noEmit` + ESLint 通过

- [x] Task 8: 全局验证
  - [x] 8.1: `cargo check` — Rust 端零错误
  - [x] 8.2: `vue-tsc --noEmit` — TypeScript 零新增错误（预存错误均在无关文件）
  - [x] 8.3: ESLint 修改文件零 error
  - [x] 8.4: 手动验证：打开新增数据源 → 新建 SSH 配置 → 协议链下拉即时出现该配置
  - [x] 8.5: 手动验证：切换驱动 → GeneralTab 表单字段动态变化
  - [x] 8.6: 手动验证：选择环境 → 策略标签自动更新
  - [x] 8.7: 手动验证：保存暂存 → localStorage 检查无 password 键

# Task Dependencies
- Task 2 独立执行（GeneralTab 不依赖 NetworkTab）
- Task 5 依赖 Task 3（都改 AdvancedTab）
- Task 4 独立执行
- Task 6 独立执行
- Task 7 独立执行
- Task 8 依赖 Task 1-7 全部完成