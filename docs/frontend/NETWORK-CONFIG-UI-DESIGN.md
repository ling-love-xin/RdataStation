# 网络配置 UI 设计文档

> 版本：v2.0
> 更新：2026-05-19
> 状态：📋 规划中 — v0.6.0 协议链 + 环境策略引擎
> 后端进度：✅ SSH隧道 + SSL证书 + service/cmd层 + ChainHop + process_chain + TunnelGuard 已完成
> 原型参考：[add-datasource-v5.html](file:///e:/myapps/tauirapps/RdataStation/rdata-station/prototype/add-datasource-v5.html)

---

## 一、概述

本文档定义 RdataStation 网络连接配置的前端 UI 设计方案。

**v2.0 重大变更**：从 v1.0 的"三个独立折叠面板"升级为 **动态协议链（Protocol Chain）** 模式，支持 SSH/Proxy 任意交替穿插（最多 4 跳），SSL/TLS 固定末尾。同时引入**环境策略引擎**，环境不再是颜色标签，而是安全/Schema/性能/审计/UI 五层策略集合。

### 前端技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Vue 3 | 3.5.x | 组件框架 (Composition API + `<script setup>`) |
| TypeScript | 6.0.x | 类型安全 |
| dockview-vue | 6.1.x | IDE 布局基座 |
| naive-ui | 2.44.x | 组件库 (NForm/NInput/NSelect/NTabs/NButton/NModal 等) |
| lucide-vue-next | 0.460.x | 图标 |
| Pinia | 3.0.x | 状态管理 |

---

## 二、设计原则

1. **naive-ui 优先**：所有表单控件使用 naive-ui 组件（NInput, NSelect, NSwitch 等）
2. **协议链为核心**：网络 Tab 采用动态协议链（拖拽排序 + 开关控制），不再用独立折叠面板
3. **环境 = 策略引擎**：环境下拉选择驱动安全/Schema/性能/审计/UI 五层策略预设填充，用户可逐字段覆盖
4. **类型安全**：TS 类型与 Rust 结构体一一对应
5. **配置独立于数据源**：网络配置管理器和环境管理器脱离具体数据源的 CRUD

### 目录结构

```
src/
├── extensions/builtin/connection/
│   ├── domain/
│   │   └── types.ts                          ← 扩展：ChainHop/ProtocolChain/Environment/EnvironmentPolicies
│   ├── ui/
│   │   ├── components/
│   │   │   ├── AddDataSourceDialog.vue       ← 改造：传递 chain/env 数据
│   │   │   ├── tabs/
│   │   │   │   ├── NetworkTab.vue            ← 🔴 重写：动态协议链
│   │   │   │   ├── AdvancedTab.vue           ← 🔴 改造：环境+策略+DuckDB
│   │   │   │   ├── GeneralTab.vue            ← 已有
│   │   │   │   └── DriverPropsTab.vue        ← 已有
│   │   │   └── network/                      ← 🆕 新增组件
│   │   │       ├── ProtocolChainEditor.vue    ← 协议链编辑器（拖拽列表）
│   │   │       ├── TopologyPreview.vue        ← 拓扑路径预览
│   │   │       ├── EnvironmentSelector.vue    ← 紧凑环境下拉
│   │   │       ├── SecurityPolicySection.vue  ← 安全策略可折叠面板
│   │   │       ├── EnvironmentManager.vue     ← 环境类型管理器（覆盖层）
│   │   │       └── NetworkProfileManager.vue  ← 网络配置文件管理器（覆盖层）
│   │   ├── stores/
│   │   │   ├── environmentStore.ts           ← 🆕 环境 + 策略 CRUD
│   │   │   ├── networkConfigStore.ts         ← 🆕 网络配置 CRUD（增强）
│   │   │   ├── project-connection-store.ts   ← 已有
│   │   │   └── runtime-connection-store.ts   ← 已有
│   │   ├── services/
│   │   │   └── connection.ts                 ← 改造：扩展参数
│   │   └── types/
│   │       └── connection.ts                 ← 扩展：ChainHop/Environment 等类型

---

## 三、核心数据结构（v2.0 协议链 + 环境策略）

详细的 TS 类型定义参见 [CONNECTION-METHOD-DESIGN.md 12.2.2 节](file:///e:/myapps/tauirapps/RdataStation/rdata-station/docs/backend/CONNECTION-METHOD-DESIGN.md#1222-typescript-类型定义扩展)。

### 3.1 协议链数据模型

```typescript
// 前端 UI 内部状态
interface ChainItem {
  id: string                    // 唯一 ID (如 'hop-1')，非持久化
  protocol: 'ssh' | 'ssl' | 'http_proxy' | 'socks_proxy'
  enabled: boolean              // 开关状态
  mode: 'select' | 'new'       // select=选已保存配置, new=内联新建
  profileId?: string            // 选中的配置 ID
}

// 持久化到 connection_method 字段
interface ChainHopConfig {
  protocol: 'ssh' | 'ssl' | 'http_proxy' | 'socks_proxy'
  enabled: boolean
  profileId?: string
}
```

### 3.2 后端 Rust 结构体对应

| 前端 ChainItem.protocol | 后端 ChainHop 变体 | 产生的网络节点 |
|-------------------------|-------------------|---------------|
| `'ssh'` | `ChainHop::Ssh(SshConfig)` | ✅ 是 |
| `'http_proxy'` | `ChainHop::HttpProxy(ProxyConfig)` | ✅ 是 |
| `'socks_proxy'` | `ChainHop::SocksProxy(ProxyConfig)` | ✅ 是 |
| `'ssl'` | `ChainHop::Ssl(SslConfig)` | ❌ 否（末尾流加密） |

### 3.3 协议链约束

| 约束 | 值 | 说明 |
|------|-----|------|
| SSH/Proxy 最大跳数 | 4 | `MAX_NETWORK_HOPS` 硬上限 |
| SSL 最多 | 1 | 链末尾，添加时替换已有 |
| 每种协议最少实例 | 1 | 唯一实例时删除按钮禁用 |
| 拖拽约束 | SSL 固定末尾 | 不可拖到中间，其他不可拖到 SSL 后 |
| 3 跳警告阈值 | ≥ 3 | 黄色横幅显示延迟风险 |

### 3.4 环境策略数据（从 SQLite 加载）

```
environments 表                  environment_policies 表
┌──────────────────────┐        ┌─────────────────────────────┐
│ id: env-prod         │        │ environment_id: env-prod     │
│ name: 生产环境        │──1:N──│ policy_type: security        │
│ color: #f38ba8       │        │ policy_config: JSON string   │
│ sort_order: 4        │        │   { readonly, writeConfirm,  │
└──────────────────────┘        │     ddlConfirm, rowLimit,    │
                                │     sizeLimit, autocommit }  │
                                ├─────────────────────────────┤
                                │ policy_type: schema          │
                                │ policy_type: performance     │
                                │ policy_type: audit           │
                                │ policy_type: ui              │
                                └─────────────────────────────┘
```

## 四、NetworkTab.vue — 动态协议链（重写）

### 4.1 整体布局

```
┌─ 网络 Tab ─────────────────────────────────────────────────┐
│                                                            │
│ ℹ 提示横幅：动态协议链 — SSH/Proxy 任意交替（最多 4 跳）   │
│   SSL/TLS 固定末尾，拖拽排序。新建跟随存储范围自动保存。    │
│                                                            │
│ ┌─ 列头 ─────────────────────────────────────────────────┐ │
│ │ ≡      #     协议             配置                  启用 操作 │
│ ├─────────────────────────────────────────────────────────┤ │
│ │                                                         │ │
│ │ ┌── ChainItem (SSH) ──────────────────────────────────┐ │ │
│ │ │ ≡ │ 1 │ [🔒 SSH 隧道] │ [▼选配置] [+新建] │ [on] │ 📋✕ │ │
│ │ └──────────────────────────────────────────────────────┘ │ │
│ │ ┌── ChainItem (Proxy) ────────────────────────────────┐ │ │
│ │ │ ≡ │ 2 │ [🌐 代理]     │ [▼选配置] [+新建] │ [on] │ 📋✕ │ │
│ │ └──────────────────────────────────────────────────────┘ │ │
│ │ ┌── ChainItem (SSL) — 蓝色左边框，标注"末尾层" ────────┐ │ │
│ │ │ 🔒│ 🔐│ [🛡 SSL/TLS]  │ [▼选配置] [+新建] │ [off]│ 📋  │ │
│ │ └──────────────────────────────────────────────────────┘ │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                            │
│ ⚠ 3 跳警告：延迟 ~75ms                                    │
│ [+ 添加协议节点]  [已达上限]                                │
│                                                            │
│ ┌─ 📡 数据路径预览 ──────────────────────────────────────┐ │
│ │ [🏠本机] →SSH→ [生产跳板机] →Proxy→ [公司代理] → [🗄DB] │ │
│ └─────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
```

### 4.2 核心交互

| 操作 | 实现 |
|------|------|
| **拖拽排序** | HTML5 Drag & Drop API，`dragstart/dragover/drop/dragend` 事件 |
| **开关切换** | 点击 toggle div，更新 `enabled`，刷新拓扑预览 |
| **配置下拉选择** | `v-for` 渲染对应协议类型的 profiles，选中后存 `profileId` |
| **"+ 新建"按钮** | 切换 `mode: 'new'`，展开内联表单（SSH含端口转发字段） |
| **新建保存** | 检查当前 scope 选择 → 生成 profile ID → push 到对应 profiles 数组 → 切换回 select 模式 |
| **"📋 管理"按钮** | 打开 `NetworkProfileManager` 覆盖层，修改返回后 `renderChain()` |
| **"✕ 删除"** | `countInstancesOfType(protocol) > 1` 时才可点击，否则灰色禁用 |
| **"+ 添加"按钮** | 弹出下拉菜单：SSH 隧道 / 代理 / SSL 加密（标末尾层）|
| **拓扑预览** | 过滤 `enabled` hops，过滤文件数据库，构建路径链 |

### 4.3 关键逻辑伪代码

```typescript
// 协议链核心状态
const protocolChain = ref<ChainItem[]>([
  { id: 'hop-1', protocol: 'ssh', enabled: true, mode: 'select', profileId: 'ssh-prod' },
  { id: 'hop-2', protocol: 'ssl', enabled: false, mode: 'select', profileId: undefined },
])

const MAX_HOPS = 4

function countNetworkHops(): number {
  return protocolChain.value.filter(h => h.protocol !== 'ssl').length
}

function addHop(protocol: string) {
  if (protocol === 'ssl') {
    // 替换已存在的 SSL（最多 1 个）
    const idx = protocolChain.value.findIndex(h => h.protocol === 'ssl')
    if (idx >= 0) protocolChain.value.splice(idx, 1)
    protocolChain.value.push({ id: 'hop-' + nextId(), protocol: 'ssl', enabled: true, mode: 'select' })
  } else {
    if (countNetworkHops() >= MAX_HOPS) return
    // 插入到 SSL 之前（SSL 固定末尾）
    const sslIdx = protocolChain.value.findIndex(h => h.protocol === 'ssl')
    const item = { id: 'hop-' + nextId(), protocol, enabled: true, mode: 'select' }
    sslIdx >= 0 ? protocolChain.value.splice(sslIdx, 0, item) : protocolChain.value.push(item)
  }
  renderChain()
  updateTopology()
}

function handleDrop(srcId, tgtId) {
  // HTML5 DnD → splice 重排 → ensureSslAtEnd() → renderChain()
}

function ensureSslAtEnd() {
  const sslIdx = protocolChain.value.findIndex(h => h.protocol === 'ssl')
  if (sslIdx >= 0 && sslIdx < protocolChain.value.length - 1) {
    const [ssl] = protocolChain.value.splice(sslIdx, 1)
    protocolChain.value.push(ssl)
  }
}

function canDelete(hopId: string): boolean {
  const hop = protocolChain.value.find(h => h.id === hopId)
  return hop ? protocolChain.value.filter(h => h.protocol === hop.protocol).length > 1 : false
}
```

## 五、AdvancedTab.vue — 环境策略 + 安全策略（改造）

### 5.1 新增内容

原有 AdvancedTab 包含：连接参数 + DuckDB 加速 + Schema 加载 + 编码。

**v2.0 新增**：

| 新增区域 | 组件 | 说明 |
|----------|------|------|
| **环境选择** | `EnvironmentSelector.vue` | 紧凑下拉，含策略行内标签 |
| **环境策略摘要** | 内联 Tag 行 | 只读/写确认/DDL确认/行限/大小限/审计 等 tag |
| **安全策略面板** | `SecurityPolicySection.vue` | 可折叠，7 个策略字段，环境覆盖指示器 |
| **DuckDB 加速（焕新）** | 内联 Card | 提高视觉权重，增加 benefits tag |

### 5.2 布局

```
┌─ 高级 Tab ───────────────────────────────────────────────┐
│                                                           │
│ 🏷 环境   [● 开发环境 ▾]   [管理]                          │
│  🔒读写 · ✍写确认 · DDL确认 · 行限10000 · 限100M           │
│                                                           │
│ ⚡ 本地加速引擎 (DuckDB)                                   │
│ ┌───────────────────────────────────────────────────────┐ │
│ │ 🦆 启用 DuckDB 查询加速                      [🔘]    │ │
│ │ ── 展开 ──                                            │ │
│ │ 🚀大表分析 🔗联邦查询 📊重复报表 💾列式压缩            │ │
│ │ 同步[▼] 间隔[15]min 内存[512]MB 线程[4]              │ │
│ │ 📁 .rdata/duckdb/accel.duckdb                         │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                           │
│ 🔐 安全策略 [▶] ← dev 预设 · 读写·无限制                  │
│ ┌─ 展开 ───────────────────────────────────────────────┐ │
│ │ [开关]默认只读  [开关]写确认  [开关]DDL确认            │ │
│ │ DROP [▼确认]  [开关]自动提交                          │ │
│ │ 结果行数上限[1000]  结果集上限[20]M                    │ │
│ └──────────────────────────────────────────────────────┘ │
│                                                           │
│ 连接参数                                                 │
│ 超时[30]s  查询超时[0]  保活[60]s  重连[3]               │
│                                                           │
│ Schema [▼自动加载]     编码 [▼UTF-8]                     │
└──────────────────────────────────────────────────────────┘
```

### 5.3 环境联动逻辑

```typescript
function selectEnv(envId: string) {
  currentEnvId.value = envId
  const env = environments.value.find(e => e.id === envId)
  if (!env) return

  // 1. 填充策略字段
  const p = env.policies
  polReadonly.value = p.security.readonly
  polWriteConfirm.value = p.security.writeConfirm
  polDdlConfirm.value = p.security.ddlConfirm
  polDropConfirm.value = p.security.dropConfirm
  polAutocommit.value = p.security.autocommit
  polRowLimit.value = p.security.rowLimit
  polSizeLimit.value = p.security.sizeLimit

  // 2. 填充连接参数
  connectTimeout.value = p.performance.connectTimeout
  queryTimeout.value = p.performance.queryTimeout
  heartbeat.value = p.performance.heartbeat
  maxReconnect.value = p.performance.maxReconnect

  // 3. Schema 加载
  schemaStrategy.value = p.schema.autoLoad ? 'auto' : 'manual'

  // 4. 更新指示器
  envIndicator.value = `← ${env.name} 预设`
  envIndicatorColor.value = env.color
  isOverridden.value = false
}

function onPolicyOverride() {
  isOverridden.value = true
  envIndicator.value = `⚠ 已覆盖预设`
  envIndicatorColor.value = 'var(--warning)'
}
```

## 六、覆盖层管理器

### 6.1 网络配置文件管理器 (NetworkProfileManager.vue)

```
┌─ 网络配置文件管理器 ──────────────────────────────── ✕ ─┐
│ [SSH 隧道] [SSL/TLS] [代理]                                │
├───────────────────────────────────────────────────────────┤
│ ┌─ 🌐全局 ──────────────────────────────────────────────┐ │
│ │ 生产跳板机    192.168.1.100:22 · root · 密钥 · 转发→  │ │
│ │ 测试跳板机    192.168.2.100:22 · admin · 密码         │ │
│ └───────────────────────────────────────────────────────┘ │
│ ┌─ 📝项目 ──────────────────────────────────────────────┐ │
│ │ 项目专用隧道  172.16.0.1:2222 · project-user · 密钥   │ │
│ └───────────────────────────────────────────────────────┘ │
│ [+ 新建 SSH 隧道配置]                                      │
└───────────────────────────────────────────────────────────┘
```

### 6.2 环境类型管理器 (EnvironmentManager.vue)

```
┌─ 环境类型管理 ────────────────────────────────────── ✕ ─┐
│                                                           │
│ ┌─ 内置环境 ────────────────────────────────────────────┐ │
│ │ 🟢 开发环境  内置                                      │ │
│ │   🔐读写·无限制  📊自动Schema  ⚡池10·超时0             │ │
│ ├───────────────────────────────────────────────────────┤ │
│ │ 🔴 生产环境  内置                                      │ │
│ │   🔐只读·写确认·DDL确认·DROP禁用  📋SQL审计            │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                           │
│ ┌─ 自定义环境 ──────────────────────────────────────────┐ │
│ │ ⚪ 演示环境                      [✕ 删除]              │ │
│ │   🔐读写·行限500  📊按需Schema                         │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                           │
│ ✦ 新建自定义环境                                         │
│ 名称[____] 图标[⚪] 颜色[■] 描述[____]                    │
│ 继承模板: [▼ 开发环境]                                   │
│ [+ 新建]                                                  │
└───────────────────────────────────────────────────────────┘
```

## 七、Stores 设计

### 7.1 environmentStore.ts

```typescript
// 环境 + 策略 Pinia Store
export const useEnvironmentStore = defineStore('environment', () => {
  const environments = ref<Environment[]>([])
  const currentEnvId = ref<string>('env-dev')
  const loading = ref(false)

  async function fetchAll() {
    environments.value = await invoke<Environment[]>('list_environments')
  }

  async function create(env: Omit<Environment, 'id'>) {
    const result = await invoke<Environment>('save_environment', { env })
    await fetchAll()
    return result
  }

  async function remove(envId: string) {
    await invoke('delete_environment', { id: envId })
    await fetchAll()
  }

  function getById(id: string): Environment | undefined {
    return environments.value.find(e => e.id === id)
  }

  return { environments, currentEnvId, loading, fetchAll, create, remove, getById }
})
```

### 7.2 networkConfigStore.ts（增强）

```typescript
export const useNetworkConfigStore = defineStore('networkConfig', () => {
  const sshProfiles = ref<NetworkProfile[]>([])
  const sslProfiles = ref<NetworkProfile[]>([])
  const proxyProfiles = ref<NetworkProfile[]>([])
  const loading = ref(false)

  async function fetchAll() {
    // 按类型分组拉取
    sshProfiles.value = await invoke<NetworkProfile[]>('list_network_configs_by_type', { networkType: 'ssh' })
    sslProfiles.value = await invoke<NetworkProfile[]>('list_network_configs_by_type', { networkType: 'ssl' })
    proxyProfiles.value = await invoke<NetworkProfile[]>('list_network_configs_by_type', { networkType: 'proxy' })
  }

  function getProfiles(protocol: string): NetworkProfile[] {
    if (protocol === 'ssh') return sshProfiles.value
    if (protocol === 'ssl') return sslProfiles.value
    return proxyProfiles.value
  }

  async function save(profile: Omit<NetworkProfile, 'id'>) {
    await invoke('save_network_config', { config: profile })
    await fetchAll()
  }

  async function remove(id: string) {
    await invoke('delete_network_config', { id })
    await fetchAll()
  }

  return { sshProfiles, sslProfiles, proxyProfiles, loading, fetchAll, getProfiles, save, remove }
})
```

## 八、Tauri Commands 前端需要调用的接口

### 8.1 已实现

| Command | 说明 |
|---------|------|
| `connect_database` | 创建连接（已支持 network_config_id / environment_id） |

### 8.2 待实现（v0.6.0 后端新增）

| Command | 说明 | Rust 调用 |
|---------|------|----------|
| `list_environments` | 列出所有环境（含策略） | `env_store.list_all()` |
| `save_environment` | 保存/更新环境 | `env_store.create()` / `update()` |
| `delete_environment` | 删除环境（内置不可删） | `env_store.delete()` |
| `list_network_configs_by_type` | 按 network_type + scope 过滤 | `network_store.list_by_type()` |
| `save_network_config` | 保存网络配置 | `network_store.create()` |
| `delete_network_config` | 删除网络配置 | `network_store.delete()` |
| `test_network_config` | 测试网络配置连通性 | 已有，需支持 chain |

## 九、实施步骤

| 阶段 | 内容 | 预估 |
|------|------|------|
| **后端增量** | Chain 校验 + 环境 Seed SQL + IPC Commands | 3 天 |
| **前端核心** | NetworkTab 重写 + AdvancedTab 改造 + TS 类型 | 5 天 |
| **管理面板** | EnvironmentManager + NetworkProfileManager + EnvironmentSelector + SecurityPolicySection | 3 天 |
| **集成联调** | AddDataSourceDialog 改造 + connection service + 端到端测试 | 2 天 |

**总计**：约 13 天（前后端并行可压缩至 10 天）

## 十、参考

- [后端实施文档 CONNECTION-METHOD-DESIGN.md 第十二章](file:///e:/myapps/tauirapps/RdataStation/rdata-station/docs/backend/CONNECTION-METHOD-DESIGN.md#十二v060-完整实施方案基于-add-datasource-v5-原型)
- [v5 原型 add-datasource-v5.html](file:///e:/myapps/tauirapps/RdataStation/rdata-station/prototype/add-datasource-v5.html)