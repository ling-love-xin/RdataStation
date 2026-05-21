# 新增数据源 — 前端完整开发计划

> 版本：v1.2
> 日期：2026-05-19
> 更新：2026-05-21（NetworkTab + GeneralTab + AuthConfigManager 已完成实施）
> 对应原型：[add-datasource-v5.html](../../../prototype/add-datasource-v5.html)
> 后端文档：[后端 DATA-SOURCE-MODULE](../../backend/DATA-SOURCE-MODULE.md)
> 网络 UI 设计：[NETWORK-CONFIG-UI-DESIGN](../NETWORK-CONFIG-UI-DESIGN.md)

---

## 一、概述

本文档描述 **"新增数据源"** 对话框的前端完整实现方案，覆盖：

- 组件树拆分与职责
- TypeScript 类型体系
- Pinia Stores 设计
- 各 Tab 组件详细设计（NetworkTab 协议链、AdvancedTab 环境策略）
- 覆盖层（Overlay）管理器
- IPC 接口契约（前端视角）
- 开发阶段与文件清单

目标：基于 v5 原型，将当前仅支持 `direct | ssl | ssh` 三种连接方式的对话框，升级为支持**动态协议链 + 环境策略引擎**的完整数据源创建体验。

### 0.1 动态表单渲染（v1.1 已实现）

驱动发现数据管道已打通，无需再硬编码表单字段：

| 层 | 组件 | 职责 |
|----|------|------|
| 数据源 | `ui/composables/useDriverRegistry.ts` | 调用 `invoke('get_data_source_types')` + `invoke('get_available_drivers')` 获取 SQLite 数据 |
| 适配器 | `ui/adapters/driver-adapter.ts` | `parseConfigSchema()` 解析 `config_schema` JSON → `DriverFormSchema` |
| 渲染器 | `ui/components/DynamicFormRenderer.vue` | 接受 `DriverFormSchema` 动态渲染 NInput/NInputNumber/NSelect 等 |
| 类型 | `domain/types.ts` | `DataSourceType`, `Driver`, `DriverConfigSchema` 匹配 SQLite struct |

**新增数据库类型无需发版**：`INSERT INTO drivers (...) VALUES (...)` 写入 `config_schema` JSON 即可。

---

## 二、核心设计原则

### 2.1 作用域：应用 vs 项目

| 作用域 | 数据存储 | 环境列表来源 | 网络配置列表来源 |
|--------|---------|------------|----------------|
| **应用（Global）** | `global.db` 的 `global_connections` | 仅 `G_xxx` | 仅 `G_xxx` |
| **项目（Project）** | `{project}/.RSmeta/project.db` 的 `connections` | `P_xxx` + `GP_xxx`（快照） | `P_xxx` + `GP_xxx`（快照） |

规则：项目连接可以引用应用级的全局配置（通过快照 `GP_xxx`），应用连接不能引用项目私有配置。

### 2.2 ID 前缀约定（前端可见）

后端定义的 ID 前缀规则，前端通过解析 ID 来识别数据来源：

```
G_xxx  = 全局（global.db）
P_xxx  = 项目本地创建（project.db）
GP_xxx = 从全局快照到项目（project.db）
```

前端工具函数：

```typescript
type IdSource = 'global' | 'project' | 'global_snapshot' | 'unknown'

function getSourceFromId(id: string): IdSource {
  if (id.startsWith('GP_')) return 'global_snapshot'
  if (id.startsWith('P_'))  return 'project'
  if (id.startsWith('G_'))  return 'global'
  return 'unknown'
}
```

### 2.3 环境 = 策略引擎

环境不是简单的"分组标签"，而是携带 5 类策略的引擎：

| 策略类别 | policy_type | 包含字段 |
|---------|------------|---------|
| **安全** | `security` | readonly / writeConfirm / ddlConfirm / dropConfirm / autocommit / rowLimit / sizeLimit |
| **Schema** | `schema` | autoLoad / loadDepth / showSystem / refreshInterval |
| **性能** | `performance` | poolSize / queryTimeout / connectTimeout / heartbeat / maxReconnect |
| **审计** | `audit` | sqlLog / operationRecord / sensitiveTableAlert |
| **UI** | `ui` | topBarColor / tabIndicator / sqlWarningBanner / writeBtnStyle |

预设 5 个环境：开发 / 测试 / 预发布 / 生产 / 沙箱，每个环境 5 条策略。

---

## 三、组件树

### 3.1 顶层结构

```
AddDataSourceDialog.vue                              ← 入口 + 对话框壳 + 底部操作栏
├── DataSourceSidebar.vue                             ← 左侧：数据库类型树 + 暂存列表
├── DataSourceHeader.vue                              ← 顶部：名称/描述/驱动/URI/作用域
├── tabs/
│   ├── GeneralTab.vue                                ← 常规：主机/端口/用户/密码/数据库
│   ├── NetworkTab.vue          ← 🔴 重写            ← 动态协议链 + 拓扑预览
│   ├── CapabilitiesTab.vue                           ← 能力矩阵（只读）
│   ├── DriverPropsTab.vue                            ← 驱动属性键值对
│   └── AdvancedTab.vue         ← 🟡 大改            ← 环境选择 + 策略 + DuckDB 加速
├── overlays/（Teleported）
│   ├── NetworkProfileManager.vue ← 🔴 新建           ← 网络配置文件 CRUD 覆盖层
│   └── EnvironmentManager.vue    ← 🔴 新建           ← 环境类型 CRUD 覆盖层
└── footer                                            ← 测试连接 / 取消 / 保存
```

### 3.2 各组件职责与 Props/Emits

| 组件 | 职责 | 关键 Props | 关键 Emits |
|------|------|----------|-----------|
| `AddDataSourceDialog` | 全局状态持有、数据聚合、表单提交 | `visible`, `editData?` | `@close`, `@saved` |
| `DataSourceSidebar` | 数据库类型选择、暂存列表 | `drivers` | `@select` |
| `DataSourceHeader` | 名称/描述/驱动/URI/作用域 | `modelValue`, `scope` | `@update:modelValue`, `@update:scope` |
| `GeneralTab` | 主机/端口/用户/密码/数据库 | `modelValue`, `driverSchema` | `@update:modelValue` |
| **`NetworkTab`** | 协议链管理、拓扑预览 | `modelValue: ChainHopItem[]`, `scope` | `@update:modelValue` |
| `CapabilitiesTab` | 驱动能力展示 | `capabilities` | - |
| `DriverPropsTab` | 驱动属性编辑 | `props` | `@update:props` |
| **`AdvancedTab`** | 环境选择、策略编辑、DuckDB | `envId`, `policies`, `duckdbAccel`, `scope` | `@update:envId`, `@update:policies`, `@update:duckdbAccel` |

---

## 四、TypeScript 类型体系

### 4.1 新增类型定义

```typescript
// ==================== 连接作用域 ====================
interface ConnectionScope {
  global: boolean
  project: boolean
}

// ==================== 协议链 ====================
type ProtocolType = 'ssh' | 'proxy' | 'ssl'

interface ChainHopItem {
  id: string
  protocol: ProtocolType
  enabled: boolean
  mode: 'select' | 'new'
  profileId: string | null        // P_xxx / GP_xxx / G_xxx
  profileSource: 'global' | 'project' | null
  customData: ChainHopCustom | null
}

interface ChainHopCustom {
  // -- 共有 --
  host?: string
  port?: number
  username?: string
  password?: string
  // -- SSH --
  authType?: 'password' | 'key'
  keyPath?: string
  // -- Proxy --
  proxyType?: 'http' | 'socks5'
  // -- SSL --
  sslMode?: 'disable' | 'require' | 'verify-ca' | 'verify-full'
  caCertPath?: string
  clientCertPath?: string
  clientKeyPath?: string
}

// ==================== 环境 ====================
interface Environment {
  id: string                      // G_xxx / P_xxx / GP_xxx
  name: string
  description: string
  color: string
  sortOrder: number
  source: 'global' | 'project'
  sourceLabel: string            // 🌐 应用级 / 📁 项目级
  policies: EnvironmentPolicies
}

interface EnvironmentPolicies {
  security: SecurityPolicy
  schema: SchemaPolicy
  performance: PerformancePolicy
  audit: AuditPolicy
  ui: UiPolicy
}

interface SecurityPolicy {
  readonly: boolean
  writeConfirm: boolean
  ddlConfirm: boolean
  dropConfirm: 'disable' | 'confirm' | 'allow'
  autocommit: boolean
  rowLimit: number               // 0 = 不限制
  sizeLimit: number              // 0 = 不限制 (MB)
}

interface SchemaPolicy {
  autoLoad: boolean
  loadDepth: number              // 1=仅结构 2=+索引 3=+外键
  showSystem: boolean
  refreshInterval: number        // 秒，0 = 不刷新
}

interface PerformancePolicy {
  poolSize: number
  queryTimeout: number           // 秒
  connectTimeout: number         // 秒
  heartbeat: number              // 秒
  maxReconnect: number
}

interface AuditPolicy {
  sqlLog: boolean
  operationRecord: boolean
  sensitiveTableAlert: boolean
}

interface UiPolicy {
  topBarColor: string
  tabIndicator: boolean
  sqlWarningBanner: boolean
  writeBtnStyle: 'normal' | 'confirm' | 'danger'
}

// ==================== 网络配置 ====================
interface NetworkConfig {
  id: string                     // G_NET_xxx / P_NET_xxx / GP_NET_xxx
  name: string
  network_type: 'ssh' | 'ssl' | 'proxy'
  config: SshConfigData | SslConfigData | ProxyConfigData
  origin: 'project' | 'global_snapshot'
  source_id: string | null
  snapshot_at: string | null
}

// ==================== DuckDB 加速 ====================
interface DuckdbAccelConfig {
  enabled: boolean
  syncStrategy: 'schedule' | 'manual' | 'on_change'
  syncIntervalMin: number
  memoryLimitMB: number
  threads: number
  localPath: string
}
```

### 4.2 domain/types.ts 扩展

```typescript
// 当前
export type ConnectionMethodType = 'direct' | 'ssl' | 'ssh' | 'http_proxy' | 'socks_proxy'

// 扩展后
export type ConnectionMethodType = 'direct' | 'chain' | 'ssl' | 'ssh' | 'http_proxy' | 'socks_proxy'
```

---

## 五、Pinia Stores

### 5.1 environmentStore.ts

```typescript
// stores/environmentStore.ts
export const useEnvironmentStore = defineStore('environment', () => {
  const globalEnvs = ref<Environment[]>([])
  const projectEnvs = ref<Environment[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchAll(projectId?: string) {
    loading.value = true
    error.value = null
    try {
      const [g, p] = await Promise.all([
        invoke<Environment[]>('list_environments', { scope: 'global' }),
        projectId
          ? invoke<Environment[]>('list_environments', { scope: 'project', projectId })
          : Promise.resolve([]),
      ])
      globalEnvs.value = g
      projectEnvs.value = p
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  // 合并列表：应用级在前，项目级在后，带 sourceLabel 区分
  const mergedForProject = computed<Environment[]>(() => [
    ...globalEnvs.value.map(e => ({ ...e, source: 'global' as const, sourceLabel: '🌐 应用级' })),
    ...projectEnvs.value.map(e => ({ ...e, source: 'project' as const, sourceLabel: '📁 项目级' })),
  ])

  // 仅应用级环境
  const globalOnly = computed(() => globalEnvs.value)

  // 按作用域返回
  function forScope(scope: ConnectionScope): Environment[] {
    if (scope.global && !scope.project) return globalOnly.value
    return mergedForProject.value
  }

  async function create(input: CreateEnvironmentInput): Promise<Environment> { /* IPC */ }
  async function update(env: Environment): Promise<void> { /* IPC */ }
  async function remove(id: string, scope: 'global' | 'project'): Promise<void> { /* IPC */ }

  return {
    globalEnvs, projectEnvs, loading, error,
    mergedForProject, globalOnly, forScope,
    fetchAll, create, update, remove,
  }
})
```

### 5.2 networkConfigStore.ts

```typescript
// stores/networkConfigStore.ts
export const useNetworkConfigStore = defineStore('networkConfig', () => {
  const allConfigs = ref<NetworkConfig[]>([])
  const loading = ref(false)

  const sshConfigs    = computed(() => allConfigs.value.filter(c => c.network_type === 'ssh'))
  const sslConfigs    = computed(() => allConfigs.value.filter(c => c.network_type === 'ssl'))
  const proxyConfigs  = computed(() => allConfigs.value.filter(c => c.network_type === 'proxy'))

  async function fetchAll(projectId?: string) {
    loading.value = true
    const [global, project] = await Promise.all([
      invoke<NetworkConfig[]>('list_network_configs', { scope: 'global' }),
      projectId
        ? invoke<NetworkConfig[]>('list_network_configs', { scope: 'project', projectId })
        : Promise.resolve([]),
    ])
    allConfigs.value = [...global, ...project]
    loading.value = false
  }

  function forScope(scope: ConnectionScope): NetworkConfig[] {
    if (scope.global && !scope.project) {
      return allConfigs.value.filter(c => c.id.startsWith('G_'))
    }
    return allConfigs.value.filter(c => c.id.startsWith('P_') || c.id.startsWith('GP_'))
  }

  function forProtocol(protocol: ProtocolType, scope: ConnectionScope): NetworkConfig[] {
    return forScope(scope).filter(c => c.network_type === protocol)
  }

  async function create(config: CreateNetworkConfigInput): Promise<NetworkConfig> { /* IPC */ }
  async function update(config: NetworkConfig): Promise<void> { /* IPC */ }
  async function remove(id: string, scope: 'global' | 'project'): Promise<void> { /* IPC */ }

  return {
    allConfigs, sshConfigs, sslConfigs, proxyConfigs, loading,
    forScope, forProtocol, fetchAll, create, update, remove,
  }
})
```

### 5.3 Composable: useAddDataSource

```typescript
// composables/useAddDataSource.ts
export function useAddDataSource() {
  // ========== 状态 ==========
  const headerData = reactive({
    name: '', description: '', selectedDriverId: '', editUriMode: false,
  })
  const scope = reactive<ConnectionScope>({ global: true, project: false })

  const generalData = reactive<GeneralFormData>({ host: '', port: 3306, database: '', username: '', password: '' })
  const protocolChain = ref<ChainHopItem[]>(getDefaultChain())
  const selectedEnvId = ref<string | null>(null)
  const overriddenPolicies = ref<Partial<EnvironmentPolicies>>({})
  const duckdbAccel = reactive<DuckdbAccelConfig>(defaultDuckdbAccel())
  const driverProps = ref<Record<string, string>>({})

  // ========== 计算 ==========
  const isFileDb = computed(() => {
    const typeId = selectedDbType.value?.id
    return typeId === 'sqlite' || typeId === 'duckdb'
  })

  // ========== 初始化 ==========
  function initDefault() {
    protocolChain.value = [
      { id: nanoid(), protocol: 'ssh', enabled: false, mode: 'select', profileId: null, profileSource: null, customData: null },
      { id: nanoid(), protocol: 'proxy', enabled: false, mode: 'select', profileId: null, profileSource: null, customData: null },
      { id: nanoid(), protocol: 'ssl', enabled: false, mode: 'select', profileId: null, profileSource: null, customData: null },
    ]
  }

  function initFromEdit(data: ConnectionConfig) {
    // 回填所有字段
    headerData.name = data.name ?? ''
    headerData.description = data.description ?? ''
    scope.global = data.scope === 'global'
    scope.project = data.scope === 'project'
    generalData.host = data.host ?? ''
    generalData.port = data.port ?? 3306
    generalData.database = data.database ?? ''
    generalData.username = data.username ?? ''
    generalData.password = data.password ?? ''
    if (data.advanced_options) {
      const opts = JSON.parse(data.advanced_options)
      if (opts.protocol_chain) protocolChain.value = opts.protocol_chain
      if (opts.duckdb_accel) Object.assign(duckdbAccel, opts.duckdb_accel)
    }
    selectedEnvId.value = data.environment_id ?? null
  }

  // ========== 环境联动 ==========
  function selectEnv(envId: string) {
    const all = useEnvironmentStore().mergedForProject.value
    const env = all.find(e => e.id === envId)
    if (!env) return

    selectedEnvId.value = envId
    overriddenPolicies.value = structuredClone(env.policies)

    // 引用全局环境时触发快照
    if (envId.startsWith('G_')) {
      invoke<string>('snapshot_global_env', { globalEnvId: envId })
        .then(gpId => { selectedEnvId.value = gpId })
    }
  }

  function onPolicyOverride(path: string, value: unknown) {
    const keys = path.split('.')
    let obj: any = overriddenPolicies.value
    for (let i = 0; i < keys.length - 1; i++) {
      if (!obj[keys[i]]) obj[keys[i]] = {}
      obj = obj[keys[i]]
    }
    obj[keys[keys.length - 1]] = value
  }

  // ========== 提交 ==========
  function buildSubmitPayload(): SaveConnectionInput {
    return {
      name: headerData.name,
      description: headerData.description,
      scope: scope.global ? 'global' : 'project',
      driver_id: headerData.selectedDriverId,
      host: generalData.host,
      port: generalData.port,
      database: generalData.database,
      username: generalData.username,
      password: generalData.password,
      environment_id: selectedEnvId.value,
      auth_config_id: null,
      network_config_id: null,    // 协议链存入 advanced_options
      driver_properties: JSON.stringify(driverProps.value),
      advanced_options: JSON.stringify({
        protocol_chain: protocolChain.value.filter(h => h.enabled),
        env_policies: overriddenPolicies.value,
        duckdb_accel: duckdbAccel,
      }),
    }
  }

  function validate(): { valid: boolean; errors: Record<string, string> } {
    const errors: Record<string, string> = {}
    if (!headerData.name.trim()) errors.name = '请输入数据源名称'
    if (!isFileDb.value && !generalData.host.trim()) errors.host = '请输入主机地址'
    if (protocolChain.value.some(h => h.enabled && h.protocol === 'ssl')) {
      const sslIdx = protocolChain.value.findIndex(h => h.protocol === 'ssl')
      if (sslIdx !== protocolChain.value.length - 1) {
        errors.chain = 'SSL 必须在协议链末尾'
      }
    }
    return { valid: Object.keys(errors).length === 0, errors }
  }

  return {
    headerData, scope, generalData, protocolChain,
    selectedEnvId, overriddenPolicies, duckdbAccel, driverProps,
    isFileDb, initDefault, initFromEdit,
    selectEnv, onPolicyOverride,
    buildSubmitPayload, validate,
  }
}

// 辅助
function getDefaultChain(): ChainHopItem[] {
  return [
    { id: 'hop-ssh', protocol: 'ssh', enabled: false, mode: 'select', profileId: null, profileSource: null, customData: null },
    { id: 'hop-proxy', protocol: 'proxy', enabled: false, mode: 'select', profileId: null, profileSource: null, customData: null },
    { id: 'hop-ssl', protocol: 'ssl', enabled: false, mode: 'select', profileId: null, profileSource: null, customData: null },
  ]
}

function defaultDuckdbAccel(): DuckdbAccelConfig {
  return { enabled: false, syncStrategy: 'schedule', syncIntervalMin: 60, memoryLimitMB: 512, threads: 2, localPath: '' }
}
```

---

## 六、核心组件：NetworkTab.vue（🔴 完整重写）

### 6.1 当前状态 vs 目标

**当前**：3 个独立 `NCollapse` 面板（SSH/SSL/Proxy开关 + 各一套配置），无协议链概念、无拖拽、无拓扑预览。

**目标**：v5 原型的动态协议链 —— 一个可增删拖拽的列表，每行一个协议跳，末尾带拓扑可视化。

### 6.2 交互模型

```
协议链列表（data = chainHops: ChainHopItem[]）

  [序号] [类型]          [配置文件/自定义配置]                    [开关]  [操作]
  ─────────────────────────────────────────────────────────────────────────────
  ①     🟢 SSH (未启用) [请选择SSH配置 ▼]                        [⬜ OFF] [管理] [🔒]
  ②     🟠 代理(已启用) [公司内网代理 (GP_NET_001)]                [✅ ON ] [管理] [🗑]
  ③     🔵 SSL (未启用) [请选择SSL配置 ▼]                        [⬜ OFF] [管理] [🔒]
  ─────────────────────────────────────────────────────────────────────────────
  [+ SSH] [+ 代理] [+ SSL]

  [拓扑预览]  本地 ─── [代理:✅] ─── DB
```

### 6.3 约束规则（前端硬编码）

| # | 规则 | 校验位置 | UI 表现 |
|---|------|---------|--------|
| R1 | SSL 必须末尾 | 拖拽 drop + addHop | SSL 行禁止拖动手柄；addHop 自动插入 SSL 之前 |
| R2 | SSH+Proxy ≤ 4 | addHop 前置检查 | 达到 4 跳时 "+SSH" "+代理" 按钮灰显 |
| R3 | 至少保留 1 个同类 hop | removeHop 前置检查 | 最后 1 个时 🗑 按钮灰显 |
| R4 | SSL 不可拖拽 | dragStart | 行上无拖拽手柄 |
| R5 | 文件型 DB 隐藏网络 | 组件级 v-if | 整个 Tab 替换为提示横幅 |
| R6 | 作用域影响下拉 | availableProfiles 计算 | 应用级仅 G_xxx；项目级 P_xxx + GP_xxx |

### 6.4 关键交互伪代码

```typescript
// 添加 hop
function addHop(protocol: ProtocolType) {
  if (protocol === 'ssl' && chain.value.some(h => h.protocol === 'ssl')) {
    message.warning('SSL 已存在，每个链最多一个 SSL 跳')
    return
  }
  if (protocol !== 'ssl' && countNetworkHops(chain.value) >= 4) {
    message.warning('网络跳数已达上限（4 跳）')
    return
  }

  const hop = createDefaultHop(protocol)
  if (protocol === 'ssl') {
    chain.value.push(hop)
  } else {
    // 插入到 SSL 之前（如果 SSL 存在的话）
    const sslIdx = chain.value.findIndex(h => h.protocol === 'ssl')
    sslIdx >= 0 ? chain.value.splice(sslIdx, 0, hop) : chain.value.push(hop)
  }
}

// 删除 hop
function removeHop(id: string) {
  const hop = chain.value.find(h => h.id === id)
  if (!hop) return
  const sameCount = chain.value.filter(h => h.protocol === hop.protocol).length
  if (sameCount <= 1) {
    message.info(`${protocolLabel(hop.protocol)} 至少保留一个实例`)
    return
  }
  chain.value = chain.value.filter(h => h.id !== id)
}

// 拖拽 drop
function onDrop(dragIdx: number, dropIdx: number) {
  const dragged = chain.value[dragIdx]
  const target = chain.value[dropIdx]

  // SSL 不能拖
  if (dragged.protocol === 'ssl') return
  // 不能放到 SSL 行上
  if (target.protocol === 'ssl') return

  const item = chain.value.splice(dragIdx, 1)[0]
  chain.value.splice(dropIdx, 0, item)
  // 确保 SSL 回到末尾
  ensureSslAtEnd()
}

function ensureSslAtEnd() {
  const sslIdx = chain.value.findIndex(h => h.protocol === 'ssl')
  if (sslIdx >= 0 && sslIdx !== chain.value.length - 1) {
    const ssl = chain.value.splice(sslIdx, 1)[0]
    chain.value.push(ssl)
  }
}

function countNetworkHops(chain: ChainHopItem[]): number {
  return chain.filter(h => h.protocol !== 'ssl').length
}
```

### 6.5 配置文件下拉处理

```typescript
// 每个 hop 行上的下拉，内容取决于 hop.protocol + scope
function getProfileOptions(hop: ChainHopItem): SelectOption[] {
  const store = useNetworkConfigStore()
  const scope = inject<ConnectionScope>('scope')!
  const configs = store.forProtocol(hop.protocol, scope)
  
  return configs.map(c => ({
    label: `${c.name} ${getSourceFromId(c.id) === 'global_snapshot' ? '🌐' : '📁'}`,
    value: c.id,
    disabled: c.network_type !== hop.protocol,
  }))
}
```

---

## 七、核心组件：AdvancedTab.vue（🟡 大改）

### 7.1 当前状态 vs 目标

**当前**：连接参数网格 + Schema 加载 + 编码选择。无环境概念。

**目标**：增加环境选择、策略面板、DuckDB 加速焕新。

### 7.2 布局

```
AdvancedTab.vue
│
├── EnvironmentSelector.vue              ← 🔴 新增
│   └── 紧凑下拉（NPopover 触发器模式）
│       ├── 触发区：● 颜色圆点 + 环境名 + ▼
│       └── 面板：分组环境列表 + 管理入口
│
├── 策略摘要标签（选中环境后展示）        ← 🔴 新增
│   └── NTag × N，行内展示 5 类策略概要
│
├── SecurityPolicySection.vue            ← 🔴 新增
│   └── NCollapse 可折叠详情
│       ├── 安全: readonly / writeConfirm / ddlConfirm / dropConfirm / autocommit
│       ├── 数据: rowLimit / sizeLimit
│       ├── Schema: autoLoad / loadDepth / showSystem / refreshInterval
│       ├── 性能: poolSize / queryTimeout / connectTimeout / heartbeat
│       └── 审计: sqlLog / operationRecord / sensitiveTableAlert
│
├── DuckDB 加速卡                        ← 🟡 焕新
│   └── 启用时卡片高亮 + 收益标签行
│
├── 连接参数（保留）
├── Schema 加载（保留）
└── 编码选择（保留）
```

### 7.3 EnvironmentSelector 交互

```typescript
// 触发器显示
const triggerLabel = computed(() => {
  if (!selectedEnvId.value) return '不指定（使用默认）'
  const env = allEnvs.value.find(e => e.id === selectedEnvId.value)
  return env ? `${env.sourceLabel} · ${env.name}` : '未知环境'
})

// 策略摘要标签
const policyTags = computed(() => {
  if (!policies.value) return []
  return [
    { label: policies.value.security.readonly ? '🔒 只读' : '✏️ 可写', type: 'info' },
    { label: `⛔ DROP:${policies.value.security.dropConfirm}`, type: 'warning' },
    { label: `📊 ${policies.value.schema.autoLoad ? '自动Schema' : '手动Schema'}`, type: 'default' },
    { label: `📋 审计:${policies.value.audit.sqlLog ? '开' : '关'}`, type: 'default' },
  ]
})
```

### 7.4 DuckDB 加速卡焕新

```typescript
// 启用状态下：
// - 卡片边框变为主题色 (#2563EB)
// - 收益标签行显示：10-100x 加速 | 离线可用 | 零配置
// - 展开区域：NSelect 同步策略 + NInputNumber 间隔/内存/线程
```

---

## 八、覆盖层（Overlay）设计

### 8.1 NetworkProfileManager.vue

```
NModal (title="🔒 网络配置文件管理器")
├── NTabs (SSH隧道 | SSL/TLS | 代理)
├── 配置文件列表
│   ├── ProfileCard × N
│   │   ├── 作用域徽章（🌐 应用级 / 📁 项目级）
│   │   ├── 名称 + 详情摘要
│   │   └── 编辑/删除按钮
│   └── 空状态 + 新建提示
└── 新建/编辑内联表单（NSpace + NInput + NSelect + NButton）
    ├── 作用域选择（应用/项目）- 仅当连接为项目作用域时显示
    ├── 协议特定字段
    └── 保存/取消
```

关键特性：
- 管理者 CRUD 独立于数据源保存（实时写入对应数据库）
- 关闭覆盖层后，链列表下拉刷新
- 新建配置时，作用域受当前连接作用域约束

### 8.2 EnvironmentManager.vue

```
NModal (title="🔧 环境管理器")
├── 环境卡片列表
│   ├── 环境卡片
│   │   ├── 颜色圆点 + 图标 + 名称
│   │   ├── 默认标签（内置5环境）
│   │   ├── 描述
│   │   └── 编辑/删除按钮（内置环境隐藏删除）
│   └── 新建卡片（+ 图标）
├── 编辑面板（展开/折叠）
│   ├── 名称 + 描述 + 颜色选择器
│   └── 策略模板选择（从现有环境复制策略）
└── 保存/取消
```

---

## 九、IPC 接口清单（前端视角）

### 9.1 已有接口（复用）

| Command | 用途 |
|---------|------|
| `get_drivers` | 获取驱动列表 |
| `load_driver_schema` | 加载驱动表单 Schema |
| `test_connection` | 测试连接 |
| `connect_database` | 建立连接 |

### 9.2 环境 CRUD

| Command | 入参 | 返回 |
|---------|------|------|
| `list_environments` | `{ scope: 'global'|'project', project_id?: string }` | `Environment[]` |
| `create_environment` | `{ scope, project_id?, name, description, color, sort_order }` | `Environment` |
| `update_environment` | `{ scope, project_id?, environment }` | `Environment` |
| `delete_environment` | `{ scope, project_id?, id }` | `void` |

### 9.3 环境策略 CRUD

| Command | 入参 | 返回 |
|---------|------|------|
| `list_environment_policies` | `{ scope, project_id?, environment_id }` | `Policy[]` |
| `create_environment_policy` | `{ scope, project_id?, environment_id, policy_type, policy_config }` | `Policy` |
| `update_environment_policy` | `{ scope, project_id?, policy }` | `Policy` |
| `delete_environment_policy` | `{ scope, project_id?, id }` | `void` |

### 9.4 网络配置 CRUD

| Command | 入参 | 返回 |
|---------|------|------|
| `list_network_configs` | `{ scope, project_id? }` | `NetworkConfig[]` |
| `create_network_config` | `{ scope, project_id?, name, network_type, config }` | `NetworkConfig` |
| `update_network_config` | `{ scope, project_id?, config }` | `NetworkConfig` |
| `delete_network_config` | `{ scope, project_id?, id }` | `void` |

### 9.5 快照相关（v2.0 新增）

| Command | 入参 | 返回 |
|---------|------|------|
| `snapshot_global_env` | `{ global_env_id, project_id }` | `string` (GP_xxx) |
| `snapshot_global_network` | `{ global_net_id, project_id }` | `string` (GP_xxx) |
| `snapshot_global_auth` | `{ global_auth_id, project_id }` | `string` (GP_xxx) |

### 9.6 校验相关（v2.0 新增）

| Command | 入参 | 返回 |
|---------|------|------|
| `validate_protocol_chain` | `{ chain: ChainHopItem[] }` | `{ valid, errors[] }` |

### 9.7 连接保存

| Command | 入参 | 返回 |
|---------|------|------|
| `save_connection` | `SaveConnectionInput`（含 environment_id / advanced_options） | `ConnectionInfo` |
| `update_connection` | `SaveConnectionInput` + `id` | `ConnectionInfo` |

---

## 十、开发阶段与文件清单

### 10.1 阶段划分

```
Phase 1: NetworkTab 协议链 ✅ 已完成
  文件：NetworkTab.vue, useNetworkProfiles.ts
  功能：动态协议链 + 内联表单 + 拖拽 + 拓扑预览 + 两栏SSH认证 + 测试按钮 + 配置管理器覆盖层

Phase 1b: GeneralTab 改造 ✅ 已完成
  文件：GeneralTab.vue, AuthConfigManager.vue
  功能：两栏数据库认证（方法+配置）+ 文件数据库新建按钮 + 认证配置管理器覆盖层

Phase 2-1: TS类型 + Stores（1天）
  文件：types/connection.ts, domain/types.ts
  依赖：Phase 0-1（后端 ID前缀 + 快照）已完成

Phase 2-2: AdvancedTab + EnvironmentSelector（1.5天）
  文件：AdvancedTab.vue, EnvironmentSelector.vue, SecurityPolicySection.vue
  依赖：Phase 2-1

Phase 2-3: 覆盖层管理器（1.5天）
  文件：EnvironmentManager.vue
  依赖：Phase 2-1

Phase 3-1: Stores + Composable（1.5天）
  文件：environmentStore.ts, networkConfigStore.ts, useAddDataSource.ts
  依赖：Phase 2

Phase 3-2: AddDataSourceDialog 集成（1.5天）
  文件：AddDataSourceDialog.vue, DataSourceHeader.vue
  依赖：Phase 3-1

Phase 4: 联调测试（2天）
  覆盖：13 个测试场景
```

### 10.2 完整文件清单

| 文件 | 操作 | Phase | 状态 |
|------|------|-------|------|
| `tabs/NetworkTab.vue` | 🔴 重写 | 1 | ✅ 已完成 |
| `composables/useNetworkProfiles.ts` | 🔴 新建 | 1 | ✅ 已完成 |
| `tabs/GeneralTab.vue` | 🟡 改造 | 1b | ✅ 已完成 |
| `components/AuthConfigManager.vue` | 🔴 新建 | 1b | ✅ 已完成 |
| `types/connection.ts` | 🟡 扩展 | 2-1 | 📋 待实施 |
| `domain/types.ts` | 🟡 扩展 `ConnectionMethodType` | 2-1 | 📋 待实施 |
| `tabs/AdvancedTab.vue` | 🟡 大改 | 2-2 | 📋 待实施 |
| `tabs/EnvironmentSelector.vue` | 🔴 新建 | 2-2 | 📋 待实施 |
| `tabs/SecurityPolicySection.vue` | 🔴 新建 | 2-2 | 📋 待实施 |
| `overlays/EnvironmentManager.vue` | 🔴 新建 | 2-3 | 📋 待实施 |
| `AddDataSourceDialog.vue` | 🟡 改造 | 3-2 | 📋 待实施 |
| `DataSourceHeader.vue` | 🟡 微调 | 3-2 | 📋 待实施 |
| `stores/environmentStore.ts` | 🔴 新建 | 3-1 | 📋 待实施 |
| `stores/networkConfigStore.ts` | 🟡 改造 | 3-1 | 📋 待实施 |
| `composables/useAddDataSource.ts` | 🔴 新建 | 3-1 | 📋 待实施 |

### 10.3 测试场景清单

| # | 场景 | 关键验证 |
|---|------|---------|
| T1 | 应用级连接 | 环境下拉仅 `G_xxx`，网络下拉仅 `G_xxx` |
| T2 | 项目级连接 + 全局环境 | 选择 `G_xxx` 环境 → 触发快照 → `GP_xxx` |
| T3 | 项目级连接 + 项目环境 | 直接引用 `P_xxx`，不触发快照 |
| T4 | 项目级连接 + 全局网络配置 | 选择 `G_xxx` 网络 → 触发快照 → `GP_xxx` |
| T5 | 协议链: 直连 | chain=[]，拓扑显示 `本地 → DB` |
| T6 | 协议链: 单跳 | `SSH(ON) → DB` |
| T7 | 协议链: 双跳 | `SSH(ON) → Proxy(ON) → DB` |
| T8 | 协议链: SSL末尾 | `SSH(ON) → SSL(ON) → DB` |
| T9 | 协议链: 4跳上限 | `Proxy → SSH → Proxy → SSH → SSL → DB` |
| T10 | 拖拽排序 | SSL 不可拖；拖拽后自动 `ensureSslAtEnd()` |
| T11 | 环境策略联动 | 选"生产环境" → 5类策略自动填充 → 可覆盖 |
| T12 | 覆盖层 CRUD | 网络/环境管理器中增删改，实时生效 |
| T13 | 文件型 DB | SQLite/DuckDB 不显示网络配置 Tab |

---

## 十一、关键风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| naive-ui `NPopover` 在 `NTabs` 内定位偏移 | 环境下拉面板位置异常 | 降级为 `NSelect` + 自定义 render-label/option；或使用 Teleported Popover |
| HTML5 Drag & Drop 在 Tauri WebView 兼容 | 拖拽功能不可用 | 降级为手动 `↑` `↓` 排序按钮 |
| `structuredClone` 深拷贝 Node 不支持 | SSR/老浏览器崩溃 | 使用 `JSON.parse(JSON.stringify(...))` |
| 快照 IPC 异步延迟 | 环境选择后 `GP_xxx` 未立即返回 | 前端乐观更新 + toast 提示"快照中..." |
| 环境策略 JSON 字段名不一致 | Rust serde 和前端 camelCase 对齐 | 在 `invoke` 封装层统一做 key 转换 |

---

## 十二、架构红线合规

- ✅ 组件/hooks/utils 严格分离 — composable 处理逻辑，组件仅渲染
- ✅ 所有数据交互通过 `tauri.invoke` — 不直接操作数据库
- ✅ Naive-UI 组件库统一 — 无混用其他 UI 库
- ✅ Lucide-vue-next 图标组件化使用
- ✅ 禁止 `any` 类型 — 所有类型通过 `types/connection.ts` 明确定义
- ✅ `pnpm run lint` / `pnpm run format` 通过

---

## 十三、相关文档

| 文档 | 路径 |
|------|------|
| 后端 DATA-SOURCE-MODULE v2.0 | [../../backend/DATA-SOURCE-MODULE.md](../../backend/DATA-SOURCE-MODULE.md) |
| 网络配置 UI 设计 v2.0 | [../NETWORK-CONFIG-UI-DESIGN.md](../NETWORK-CONFIG-UI-DESIGN.md) |
| 连接方法设计 | [../../backend/CONNECTION-METHOD-DESIGN.md](../../backend/CONNECTION-METHOD-DESIGN.md) |
| v5 原型 | [../../../prototype/add-datasource-v5.html](../../../prototype/add-datasource-v5.html) |
| 前端架构 | [../ARCHITECTURE.md](../ARCHITECTURE.md) |
| 前端组件索引 | [../INDEX.md](../INDEX.md) |