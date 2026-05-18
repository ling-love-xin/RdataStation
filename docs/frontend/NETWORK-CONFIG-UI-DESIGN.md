# 网络配置 UI 设计文档

> 版本：v1.0
> 创建：2026-05-19
> 状态：📋 规划中 — v0.5.0 前端开发待启动
> 后端进度：✅ SSH隧道 + SSL证书 + service/cmd层已完成

---

## 一、概述

本文档定义 RdataStation 网络连接配置的前端 UI 设计方案，覆盖 SSH 隧道、SSL/TLS 加密、代理三种配置的 CRUD 管理界面，以及新建连接时引用网络配置的交互流程。

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

## 二、目录结构规划

```
src/
├── extensions/builtin/connection/ui/
│   ├── components/
│   │   ├── AddDataSourceDialog.vue          ← 已有：新增数据源对话框（需改造）
│   │   ├── tabs/
│   │   │   ├── NetworkTab.vue               ← 已有：网络配置Tab（需升级为naive-ui）
│   │   │   ├── GeneralTab.vue               ← 已有
│   │   │   ├── DriverPropsTab.vue           ← 已有
│   │   │   └── AdvancedTab.vue              ← 已有
│   │   └── network/                         ← 新增：网络配置管理组件
│   │       ├── NetworkConfigPanel.vue        ← 网络配置管理面板（dockview-vue面板）
│   │       ├── NetworkConfigList.vue         ← 配置列表视图
│   │       ├── NetworkConfigForm.vue         ← 新建/编辑配置表单
│   │       ├── SshConfigFields.vue           ← SSH 配置表单字段
│   │       ├── SslConfigFields.vue           ← SSL 配置表单字段
│   │       └── ProxyConfigFields.vue         ← Proxy 配置表单字段
│   ├── stores/
│   │   └── networkConfigStore.ts            ← 新增：网络配置 Pinia Store
│   ├── services/
│   │   └── connection.ts                    ← 已有：需扩展 connectDatabase 参数
│   └── types/
│       └── networkConfig.ts                 ← 新增：网络配置类型定义
```

### 设计原则

1. **naive-ui 优先**：所有表单控件使用 naive-ui 组件（NInput, NSelect, NSwitch 等）
2. **dockview-vue 集成**：网络配置管理面板作为 dockview-vue 的独立面板
3. **类型安全**：TS 类型与 Rust 结构体一一对应，serde rename_all = "snake_case"
4. **复用优先**：表单字段组件化（SshConfigFields / SslConfigFields / ProxyConfigFields），在多处复用

---

## 三、数据结构定义

### 3.1 TypeScript 类型

```typescript
// src/extensions/builtin/connection/ui/types/networkConfig.ts

/** 网络配置类型 */
export type NetworkType = 'ssh' | 'ssl' | 'http_proxy' | 'socks_proxy'

/** 网络配置实体 */
export interface NetworkConfig {
  id: string
  name: string
  network_type: NetworkType
  config: string           // JSON 序列化字符串
  created_at: string
  updated_at: string
}

/** TLS 版本 */
export type TlsVersion = 'tls1_0' | 'tls1_1' | 'tls1_2' | 'tls1_3'

/** SSH 认证方式 */
export type SshAuthType = 'password' | 'private_key' | 'agent'

/** SSH 配置 */
export interface SshConfig {
  host: string
  port: number            // 默认 22
  username: string
  auth_type: SshAuthType
  password?: string
  key_path?: string
  passphrase?: string
  remote_host: string
  remote_port: number
  local_port: number      // 0 = 自动分配
  timeout_secs: number    // 默认 30
}

/** SSL 配置 */
export interface SslConfig {
  verify_server_cert: boolean
  ca_cert_path?: string
  client_cert_path?: string
  client_key_path?: string
  min_tls_version: TlsVersion
}

/** 代理类型 */
export type ProxyType = 'http' | 'socks5'

/** 代理配置 */
export interface ProxyConfig {
  host: string
  port: number
  proxy_type: ProxyType
  auth_username?: string
  auth_password?: string
  no_proxy: string[]
  timeout_secs: number
}

/** ConnectDatabaseInput 扩展（新增 network_config_id）*/
export interface ConnectDatabaseInput {
  db_type: string
  url: string
  name?: string
  connection_type?: 'global' | 'project'
  project_id?: string
  description?: string
  driver_id?: string
  environment_id?: string
  auth_config_id?: string
  network_config_id?: string       // ← 新增：引用网络配置 ID
  driver_properties?: string
  advanced_options?: string
}
```

### 3.2 Rust ↔ TS 字段映射

| Rust SshConfig | TS SshConfig | 说明 |
|---------------|--------------|------|
| `host: String` | `host: string` | SSH 主机 |
| `port: u16` | `port: number` | 端口（默认22）|
| `username: String` | `username: string` | 用户名 |
| `auth: SshAuth::Password { password }` | `auth_type: 'password'` + `password: string` | 密码认证 |
| `auth: SshAuth::PrivateKey { key_path, passphrase }` | `auth_type: 'private_key'` + `key_path: string` + `passphrase?: string` | 私钥认证 |
| `auth: SshAuth::Agent` | `auth_type: 'agent'` | Agent 认证 |
| `remote_host: String` | `remote_host: string` | 目标DB主机 |
| `remote_port: u16` | `remote_port: number` | 目标DB端口 |
| `local_port: u16` | `local_port: number` | 本地端口（0=自动）|

---

## 四、页面设计

### 4.1 网络配置管理面板 (NetworkConfigPanel.vue)

作为 dockview-vue 的独立面板嵌入 IDE 布局，类似 VSCode 的侧边栏面板。

**布局结构**：

```
┌──────────────────────────────────┐
│ NetworkConfigPanel               │
│ ┌──────────────────────────────┐ │
│ │ 标题栏：网络配置    [+ 新建] │ │
│ ├──────────────────────────────┤ │
│ │ NTabs: [SSH] [SSL] [Proxy]  │ │
│ ├──────────────────────────────┤ │
│ │ NetworkConfigList            │ │
│ │ ┌──────────────────────────┐ │ │
│ │ │ ○ 生产跳板机        SSH  │ │ │
│ │ │ ○ 开发SSL证书       SSL  │ │ │
│ │ │ ○ 公司代理         Proxy │ │ │
│ │ │ ...                      │ │ │
│ │ └──────────────────────────┘ │ │
│ └──────────────────────────────┘ │
└──────────────────────────────────┘
```

**交互说明**：
- NTabs 按 `network_type` 分类展示（SSH / SSL / Proxy）
- 列表项支持：点击编辑、右键菜单（编辑/复制/删除/测试连接）
- "新建"按钮打开 `NetworkConfigForm` 模态框
- 列表项左侧有类型图标区分（Server / Shield / Globe）

### 4.2 配置列表视图 (NetworkConfigList.vue)

```vue
<template>
  <n-list hoverable clickable>
    <n-list-item v-for="item in filteredConfigs" :key="item.id" @click="onEdit(item)">
      <template #prefix>
        <component :is="typeIcon(item.network_type)" :size="16" />
      </template>
      <n-thing :title="item.name" :description="typeLabel(item.network_type)">
        <template #action>
          <n-button text size="tiny" @click.stop="onTest(item)">
            <Plug :size="14" />
          </n-button>
        </template>
      </n-thing>
    </n-list-item>
  </n-list>
</template>
```

### 4.3 配置表单 (NetworkConfigForm.vue) — 模态框

```
┌─────────────────────────────────────┐
│  新建网络配置                    ✕  │
├─────────────────────────────────────┤
│  ┌ 名称 ──────────────────────────┐ │
│  │ NInput: 生产环境跳板机         │ │
│  └────────────────────────────────┘ │
│  ┌ 类型 ──────────────────────────┐ │
│  │ NSelect: SSH隧道 / SSL / Proxy │ │
│  └────────────────────────────────┘ │
│                                      │
│  === 根据类型动态渲染表单字段 ===    │
│                                      │
│  <SshConfigFields />                 │
│  or <SslConfigFields />              │
│  or <ProxyConfigFields />            │
│                                      │
├─────────────────────────────────────┤
│           [测试连接] [取消] [保存]   │
└─────────────────────────────────────┘
```

### 4.4 SSH 配置字段 (SshConfigFields.vue)

```
┌ SSH 服务器 ────────────────────────────────────┐
│ NInput: 主机    NInputNumber: 端口 (默认22)   │
│ NInput: 用户名                                  │
├────────────────────────────────────────────────┤
│ ┌ 认证方式 ───────────────────────────────────┐ │
│ │ NSegmented(?: NSelect): 密码 | 私钥 | Agent │ │
│ ├──────────────────────────────────────────────┤ │
│ │ [密码模式] NInput(type=password): 密码      │ │
│ │ [私钥模式] NInput: 密钥路径                 │ │
│ │           NInput(type=password): 密钥密码    │ │
│ │ [Agent模式] NTooltip: 待 v0.6.0 实现        │ │
│ └──────────────────────────────────────────────┘ │
├────────────────────────────────────────────────┤
│ ┌ 端口映射 ───────────────────────────────────┐ │
│ │ NInput: 目标主机   NInputNumber: 目标端口   │ │
│ │ NInputNumber: 本地端口 (0=自动分配)         │ │
│ └──────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

**认证方式联动逻辑**：
- 选择"密码"→ 显示密码输入框
- 选择"私钥"→ 显示密钥路径 + 可选的密钥密码
- 选择"Agent"→ 显示提示"SSH Agent 认证将在 v0.6.0 实现"

### 4.5 SSL 配置字段 (SslConfigFields.vue)

```
┌ 证书验证 ────────────────────────────────────┐
│ NSwitch: 验证服务器证书  (默认 ON)           │
├───────────────────────────────────────────────┤
│ NInput: CA 证书路径                           │
│ NInput: 客户端证书路径 (mTLS)                 │
│ NInput: 客户端私钥路径 (mTLS)                 │
├───────────────────────────────────────────────┤
│ NSelect: 最低 TLS 版本                        │
│   - TLS 1.0  /  TLS 1.1  /  TLS 1.2  /  TLS 1.3 │
│   默认: TLS 1.2                                │
└───────────────────────────────────────────────┘
```

### 4.6 代理配置字段 (ProxyConfigFields.vue)

```
┌ 代理类型 ────────────────────────────────────┐
│ NSelect: HTTP / SOCKS5                        │
├───────────────────────────────────────────────┤
│ NInput: 主机    NInputNumber: 端口           │
├───────────────────────────────────────────────┤
│ ┌ 认证（可选）──────────────────────────────┐ │
│ │ NSwitch: 启用认证                          │ │
│ │ NInput: 用户名    NInput(type=password): 密码 │
│ └─────────────────────────────────────────────┘ │
├───────────────────────────────────────────────┤
│ NDynamicTags(?: NInput): 不走代理的主机列表   │
│   [10.0.0.0/8] [*.internal.local]             │
└───────────────────────────────────────────────┘
```

---

## 五、新建连接对话框改造

### 5.1 改造范围

现有 [AddDataSourceDialog.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/connection/ui/components/AddDataSourceDialog.vue) 需要增加网络配置选择：

```
┌ AddDataSourceDialog ──────────────────────────┐
│  左侧: DataSourceSidebar (驱动选择)            │
│  右侧:                                        │
│    DataSourceHeader (名称/URL/保存位置)        │
│    Tabs:  [常规] [网络] [能力] [属性] [高级]  │
│                                              │
│    ~~ NetworkTab 改造 ~~                       │
│    ┌────────────────────────────────────────┐ │
│    │ NSelect: 选择网络配置 (下拉选择)       │ │
│    │   选项:                                 │ │
│    │   - 无（直连）                          │ │
│    │   - 生产跳板机 (SSH)                    │ │
│    │   - 开发SSL证书 (SSL)                   │ │
│    │   - 公司代理 (HTTP Proxy)              │ │
│    │   [+ 新建网络配置...]                   │ │
│    │                                         │ │
│    │ 选择后展开预览：                         │ │
│    │   SSH: host → 跳板机, remote → DB:3306  │ │
│    │   SSL: verify=true, CA: ca.pem          │ │
│    │   Proxy: HTTP → proxy:8080              │ │
│    ├────────────────────────────────────────┤ │
│    │ [手动配置模式] (保留原有折叠表单)       │ │
│    │  ← 兼容旧逻辑，手动填写SSH/SSL/Proxy   │ │
│    └────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘
```

### 5.2 交互流程

```
用户操作流程：
  1. 选择数据库驱动 → 填写连接信息
  2. 切换到 [网络] Tab
  3. 下拉选择已有的网络配置
     → 选择后自动填充 network_config_id
     → 展开预览面板显示配置摘要（只读）
  4. 或选择"手动配置"模式
     → 展开 SSH/SSL/Proxy 折叠面板
     → 手动填写配置参数
  5. 点击 [测试连接] / [保存]
```

### 5.3 connectDatabase 调用改造

```typescript
// 新增参数 network_config_id
async function connectDatabase(input: ConnectDatabaseInput) {
  return invoke<ConnectionResponse>('connect_database', {
    input: {
      db_type: input.db_type,
      url: input.url,
      name: input.name,
      connection_type: input.connection_type || 'global',
      project_id: input.project_id,
      driver_id: input.driver_id,
      network_config_id: input.network_config_id,    // ← 新增
      environment_id: input.environment_id,
      auth_config_id: input.auth_config_id,
      driver_properties: input.driver_properties,
      advanced_options: input.advanced_options,
      description: input.description,
    },
  })
}
```

---

## 六、Pinia Store 设计 (networkConfigStore.ts)

```typescript
// src/extensions/builtin/connection/ui/stores/networkConfigStore.ts

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { NetworkConfig, NetworkType, SshConfig, SslConfig, ProxyConfig } from '../types/networkConfig'

export const useNetworkConfigStore = defineStore('networkConfig', () => {
  const configs = ref<NetworkConfig[]>([])
  const loading = ref(false)
  const activeType = ref<NetworkType | 'all'>('all')

  // Getters
  const sshConfigs = computed(() => configs.value.filter(c => c.network_type === 'ssh'))
  const sslConfigs = computed(() => configs.value.filter(c => c.network_type === 'ssl'))
  const proxyConfigs = computed(() => configs.value.filter(c =>
    c.network_type === 'http_proxy' || c.network_type === 'socks_proxy'
  ))

  // Actions
  async function fetchAll() {
    loading.value = true
    try {
      configs.value = await invoke<NetworkConfig[]>('list_network_configs')
    } finally {
      loading.value = false
    }
  }

  async function save(config: Omit<NetworkConfig, 'id' | 'created_at' | 'updated_at'>) {
    const result = await invoke<NetworkConfig>('save_network_config', { config })
    await fetchAll()
    return result
  }

  async function remove(id: string) {
    await invoke('delete_network_config', { id })
    await fetchAll()
  }

  async function testConnection(id: string) {
    return invoke<{ success: boolean; message: string }>('test_network_config', { id })
  }

  return {
    configs, loading, activeType,
    sshConfigs, sslConfigs, proxyConfigs,
    fetchAll, save, remove, testConnection,
  }
})
```

---

## 七、NetworkTab.vue 改造方案

### 7.1 现状分析

当前 [NetworkTab.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/connection/ui/components/tabs/NetworkTab.vue)：
- 使用原生 HTML `<input>` 和 `<select>`
- 自建 CSS 样式（未使用 naive-ui）
- SSH/SSL/Proxy 折叠面板使用自定义 switch-toggle

### 7.2 改造计划

```vue
<!-- 改造后的 NetworkTab.vue -->
<template>
  <div class="network-tab">
    <!-- 快速模式：下拉选择已有配置 -->
    <n-form-item label="网络配置" :label-placement="'left'">
      <n-select
        v-model:value="selectedConfigId"
        :options="configOptions"
        placeholder="选择已有网络配置（可选）"
        clearable
        @update:value="onConfigSelect"
      />
      <template #feedback>
        <n-button text size="tiny" @click="showNewConfigDialog = true">
          <Plus :size="12" /> 新建网络配置
        </n-button>
      </template>
    </n-form-item>

    <!-- 选中配置摘要 -->
    <n-alert v-if="selectedConfigId && selectedConfigSummary" type="info" :bordered="false">
      {{ selectedConfigSummary }}
    </n-alert>

    <n-divider />

    <!-- 手动配置模式 -->
    <n-collapse>
      <n-collapse-item title="SSH 隧道" name="ssh">
        <template #header-extra>
          <n-switch v-model:value="sshEnabled" size="small" />
        </template>
        <SshConfigFields v-if="sshEnabled" v-model="sshConfig" />
      </n-collapse-item>

      <n-collapse-item title="SSL/TLS" name="ssl">
        <template #header-extra>
          <n-switch v-model:value="sslEnabled" size="small" />
        </template>
        <SslConfigFields v-if="sslEnabled" v-model="sslConfig" />
      </n-collapse-item>

      <n-collapse-item title="代理" name="proxy">
        <template #header-extra>
          <n-switch v-model:value="proxyEnabled" size="small" />
        </template>
        <ProxyConfigFields v-if="proxyEnabled" v-model="proxyConfig" />
      </n-collapse-item>
    </n-collapse>
  </div>
</template>
```

### 7.3 关键变更

| 变更项 | 原实现 | 新实现 |
|--------|--------|--------|
| 表单控件 | 原生 `<input>/<select>` | naive-ui NInput/NSelect/NSwitch |
| 折叠面板 | 自定义 collapse-panel | naive-ui NCollapse/NCollapseItem |
| 开关 | 自定义 switch-toggle | naive-ui NSwitch |
| 图标 | lucide-vue-next | lucide-vue-next（不变）|
| 网络配置选择 | ❌ 不支持 | NSelect 下拉 + 预览 |
| 样式 | 手写 scoped CSS | naive-ui 内置样式 + CSS变量 |

---

## 八、Tauri Commands 前端需要调用的接口

### 8.1 已实现的后端 Command

| Command | 说明 | 状态 |
|---------|------|------|
| `connect_database` | 创建连接（已支持 network_config_id）| ✅ |
| `get_connections` | 获取所有连接 | ✅ |
| `get_global_connections` | 获取全局连接（含 network_config_id）| ✅ |

### 8.2 需新增的后端 Command（前端依赖）

| Command | 说明 | 优先级 |
|---------|------|--------|
| `list_network_configs` | 列出所有网络配置 | 🔴 P0 |
| `save_network_config` | 保存/更新网络配置 | 🔴 P0 |
| `delete_network_config` | 删除网络配置 | 🔴 P0 |
| `test_network_config` | 测试网络配置连通性 | 🟡 P1 |
| `get_network_config` | 获取单个配置详情 | 🟡 P1 |

---

## 九、实施步骤（v0.5.0 前端）

| 步骤 | 内容 | 预估 |
|------|------|------|
| 1 | 新增 TS 类型定义 `types/networkConfig.ts` | 0.5天 |
| 2 | 新增 `networkConfigStore.ts`（Pinia Store）| 0.5天 |
| 3 | 实现配置表单字段组件（SshConfigFields/SslConfigFields/ProxyConfigFields）| 1天 |
| 4 | 实现 `NetworkConfigForm.vue`（新建/编辑模态框）| 0.5天 |
| 5 | 实现 `NetworkConfigList.vue` + `NetworkConfigPanel.vue` | 0.5天 |
| 6 | 改造 `NetworkTab.vue`（升级 naive-ui + 增加选择模式）| 1天 |
| 7 | 改造 `AddDataSourceDialog.vue` + `connection.ts`（传递 network_config_id）| 0.5天 |
| 8 | 后端新增 3 个 Command（list/save/delete network_config）| 1天 |
| 9 | 联调 + 端到端测试 | 1天 |

**总计**：约 6-7 天（前后端并行可压缩至 4-5 天）

---

## 十、附录

### A. 图标映射

```typescript
import { Server, Shield, Globe, Plug, Plus, Wifi, Lock, Key } from 'lucide-vue-next'

const typeIconMap: Record<NetworkType, Component> = {
  ssh: Server,
  ssl: Shield,
  http_proxy: Globe,
  socks_proxy: Wifi,
}
```

### B. 参考文档

- [后端网络连接架构文档](file:///e:/myapps/tauirapps/RdataStation/rdata-station/docs/backend/CONNECTION-METHOD-DESIGN.md)
- [前端企业级规范](file:///e:/myapps/tauirapps/RdataStation/rdata-station/.trae/rules/frontend-enterprise-spec.md)
- [naive-ui 文档](https://www.naiveui.com/)
- [dockview-vue 文档](https://dockview.dev/)