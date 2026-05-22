/**
 * useAddDataSource — 新增数据源提交逻辑封装
 *
 * 统一管理新增/编辑数据源的状态、校验、快照、协议链与提交载荷构建。
 * 对应文档: docs/frontend/connection/add-datasource-frontend-plan.md (Phase 4-5)
 */

import { v4 as uuidv4 } from 'uuid'
import { reactive, ref } from 'vue'

import { useProjectStore } from '@/core/project/stores/project'

import { useEnvironmentStore } from '../stores/environmentStore'

import type { ConnectDatabaseInput } from '../../domain/types'

// ==================== 类型定义 ====================

/** 连接作用域 */
export interface ConnectionScope {
  global: boolean
  project: boolean
}

/** 常规表单数据 */
export interface GeneralFormData {
  host: string
  port: number
  database: string
  username: string
  password: string
}

/** 协议类型 */
export type ProtocolType = 'ssh' | 'proxy' | 'ssl'

/** 协议链跳 */
export interface ChainHopItem {
  id: string
  protocol: ProtocolType
  enabled: boolean
  mode: 'select' | 'new'
  profileId: string | null
  profileSource: 'global' | 'project' | null
  customData: ChainHopCustom | null
}

/** 跳自定义配置 */
export interface ChainHopCustom {
  host?: string
  port?: number
  username?: string
  password?: string
  authType?: 'password' | 'key'
  keyPath?: string
  proxyType?: 'http' | 'socks5'
  sslMode?: 'disable' | 'require' | 'verify-ca' | 'verify-full'
  caCertPath?: string
  clientCertPath?: string
  clientKeyPath?: string
}

/** 安全策略 */
export interface SecurityPolicy {
  readonly: boolean
  writeConfirm: boolean
  ddlConfirm: boolean
  dropConfirm: 'disable' | 'confirm' | 'allow'
  autocommit: boolean
  rowLimit: number
  sizeLimit: number
}

/** Schema 策略 */
export interface SchemaPolicy {
  autoLoad: boolean
  loadDepth: number
  showSystem: boolean
  refreshInterval: number
}

/** 性能策略 */
export interface PerformancePolicy {
  poolSize: number
  queryTimeout: number
  connectTimeout: number
  heartbeat: number
  maxReconnect: number
}

/** 审计策略 */
export interface AuditPolicy {
  sqlLog: boolean
  operationRecord: boolean
  sensitiveTableAlert: boolean
}

/** UI 策略 */
export interface UiPolicy {
  topBarColor: string
  tabIndicator: boolean
  sqlWarningBanner: boolean
  writeBtnStyle: 'normal' | 'confirm' | 'danger'
}

/** 环境策略集合 */
export interface EnvironmentPolicies {
  security: SecurityPolicy
  schema: SchemaPolicy
  performance: PerformancePolicy
  audit: AuditPolicy
  ui: UiPolicy
}

/** DuckDB 加速配置 */
export interface DuckdbAccelConfig {
  enabled: boolean
  syncStrategy: 'schedule' | 'manual' | 'on_change'
  syncIntervalMin: number
  memoryLimitMB: number
  threads: number
  localPath: string
}

/** 提交载荷（含协议链/env策略/duckdb） */
export interface SaveConnectionInput {
  name: string
  description: string
  scope: 'global' | 'project'
  driver_id: string
  host: string
  port: number
  database: string
  username: string
  password: string
  environment_id: string | null
  auth_config_id: string | null
  network_config_id: string | null
  driver_properties: string
  advanced_options: string
}

/** 校验结果 */
export interface ValidationResult {
  valid: boolean
  errors: Record<string, string>
}

// ==================== 辅助函数 ====================

function getDefaultChain(): ChainHopItem[] {
  return [
    { id: uuidv4(), protocol: 'ssh', enabled: false, mode: 'select', profileId: null, profileSource: null, customData: null },
    { id: uuidv4(), protocol: 'proxy', enabled: false, mode: 'select', profileId: null, profileSource: null, customData: null },
    { id: uuidv4(), protocol: 'ssl', enabled: false, mode: 'select', profileId: null, profileSource: null, customData: null },
  ]
}

function defaultDuckdbAccel(): DuckdbAccelConfig {
  return {
    enabled: false,
    syncStrategy: 'schedule',
    syncIntervalMin: 60,
    memoryLimitMB: 512,
    threads: 2,
    localPath: '',
  }
}

function defaultSecurityPolicy(): SecurityPolicy {
  return { readonly: false, writeConfirm: true, ddlConfirm: true, dropConfirm: 'confirm', autocommit: true, rowLimit: 1000, sizeLimit: 100 }
}

function defaultSchemaPolicy(): SchemaPolicy {
  return { autoLoad: true, loadDepth: 2, showSystem: false, refreshInterval: 300 }
}

function defaultPerformancePolicy(): PerformancePolicy {
  return { poolSize: 5, queryTimeout: 30, connectTimeout: 10, heartbeat: 60, maxReconnect: 3 }
}

function defaultAuditPolicy(): AuditPolicy {
  return { sqlLog: true, operationRecord: false, sensitiveTableAlert: true }
}

function defaultUiPolicy(): UiPolicy {
  return { topBarColor: '', tabIndicator: true, sqlWarningBanner: true, writeBtnStyle: 'confirm' }
}

function defaultPolicies(): EnvironmentPolicies {
  return {
    security: defaultSecurityPolicy(),
    schema: defaultSchemaPolicy(),
    performance: defaultPerformancePolicy(),
    audit: defaultAuditPolicy(),
    ui: defaultUiPolicy(),
  }
}

// ==================== Composable ====================

export function useAddDataSource() {
  // ========== 状态 ==========
  const headerData = reactive({
    name: '',
    description: '',
    selectedDriverId: '',
    editUriMode: false,
  })

  const scope = reactive<ConnectionScope>({ global: true, project: false })

  const generalData = reactive<GeneralFormData>({
    host: '',
    port: 3306,
    database: '',
    username: '',
    password: '',
  })

  const protocolChain = ref<ChainHopItem[]>(getDefaultChain())
  const selectedEnvId = ref<string | null>(null)
  const overriddenPolicies = ref<Partial<EnvironmentPolicies>>({})
  const duckdbAccel = reactive<DuckdbAccelConfig>(defaultDuckdbAccel())
  const driverProps = ref<Record<string, string>>({})

  const saving = ref(false)
  const error = ref<string | null>(null)

  // ========== 计算属性 ==========
  /** 是否为文件型数据库（SQLite/DuckDB），由调用方在 onDriverChange 时设置 */
  const isFileDb = ref(false)

  function setFileDb(val: boolean) {
    isFileDb.value = val
  }

  // ========== 初始化 ==========
  function initDefault() {
    headerData.name = ''
    headerData.description = ''
    headerData.selectedDriverId = ''
    headerData.editUriMode = false
    scope.global = true
    scope.project = false
    generalData.host = ''
    generalData.port = 3306
    generalData.database = ''
    generalData.username = ''
    generalData.password = ''
    protocolChain.value = getDefaultChain()
    selectedEnvId.value = null
    overriddenPolicies.value = {}
    Object.assign(duckdbAccel, defaultDuckdbAccel())
    driverProps.value = {}
    saving.value = false
    error.value = null
  }

  function initFromEdit(data: {
    name?: string
    description?: string
    scope?: string
    host?: string
    port?: number
    database?: string
    username?: string
    password?: string
    environment_id?: string | null
    advanced_options?: string | null
  }) {
    headerData.name = data.name ?? ''
    headerData.description = data.description ?? ''
    scope.global = data.scope === 'global'
    scope.project = data.scope === 'project'
    generalData.host = data.host ?? ''
    generalData.port = data.port ?? 3306
    generalData.database = data.database ?? ''
    generalData.username = data.username ?? ''
    generalData.password = data.password ?? ''
    selectedEnvId.value = data.environment_id ?? null

    if (data.advanced_options) {
      try {
        const opts = JSON.parse(data.advanced_options)
        if (opts.protocol_chain) protocolChain.value = opts.protocol_chain
        if (opts.duckdb_accel) Object.assign(duckdbAccel, opts.duckdb_accel)
        if (opts.env_policies) overriddenPolicies.value = opts.env_policies
      } catch { /* invalid JSON, ignore */ }
    }
  }

  // ========== 环境联动 ==========
  async function selectEnv(envId: string) {
    const envStore = useEnvironmentStore()
    const env = envStore.getById(envId)
    if (!env) return

    selectedEnvId.value = envId
    overriddenPolicies.value = {}

    // 引用全局环境时触发快照
    if (envId.startsWith('G_')) {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const pp = useProjectStore().currentProject?.path
        const r = await invoke<{ snapshot_id: string }>('snapshot_global_env', { globalEnvId: envId, projectPath: pp })
        selectedEnvId.value = r.snapshot_id
      } catch (e) {
        error.value = `快照环境失败: ${e instanceof Error ? e.message : String(e)}`
      }
    }
  }

  function onPolicyOverride(path: string, value: unknown) {
    const keys = path.split('.')
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let obj: Record<string, unknown> = overriddenPolicies.value as Record<string, unknown>
    for (let i = 0; i < keys.length - 1; i++) {
      if (!obj[keys[i]] || typeof obj[keys[i]] !== 'object') {
        obj[keys[i]] = {}
      }
      obj = obj[keys[i]] as Record<string, unknown>
    }
    obj[keys[keys.length - 1]] = value
  }

  // ========== 提交载荷构建 ==========
  function buildSavePayload(): SaveConnectionInput {
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
      network_config_id: null,
      driver_properties: JSON.stringify(driverProps.value),
      advanced_options: JSON.stringify({
        protocol_chain: protocolChain.value.filter(h => h.enabled),
        env_policies: overriddenPolicies.value,
        duckdb_accel: duckdbAccel,
      }),
    }
  }

  /** 构建 connect_database IPC 调用所需的 ConnectDatabaseInput */
  function buildSubmitPayload(params: {
    dbType: string
    url: string
    driverId?: string | null
    projectId?: string | null
    authConfigId?: string | null
    authMethod?: string
    networkConfigId?: string | null
  }): ConnectDatabaseInput {
    const savePayload = buildSavePayload()
    return {
      db_type: params.dbType,
      url: params.url,
      name: savePayload.name,
      connection_type: savePayload.scope,
      project_id: params.projectId ?? undefined,
      description: savePayload.description || null,
      driver_id: params.driverId ?? null,
      environment_id: savePayload.environment_id,
      auth_config_id: params.authConfigId ?? null,
      auth_method: params.authMethod ?? 'password',
      network_config_id: params.networkConfigId ?? null,
      driver_properties: savePayload.driver_properties || null,
      advanced_options: savePayload.advanced_options || null,
    }
  }

  // ========== 校验 ==========
  function validate(): ValidationResult {
    const errs: Record<string, string> = {}
    if (!headerData.name.trim()) errs.name = '请输入数据源名称'
    if (!headerData.selectedDriverId) errs.driver = '请选择数据驱动'
    if (!scope.global && !scope.project) errs.scope = '请至少选择一个作用域'

    const hasEnabledChain = protocolChain.value.some(h => h.enabled)
    if (hasEnabledChain) {
      const sslIdx = protocolChain.value.findIndex(h => h.protocol === 'ssl' && h.enabled)
      if (sslIdx >= 0 && sslIdx !== protocolChain.value.length - 1) {
        errs.chain = 'SSL 必须在协议链末尾'
      }
    }

    return { valid: Object.keys(errs).length === 0, errors: errs }
  }

  // ========== 协议链操作 ==========
  function countNetworkHops(chain: ChainHopItem[]): number {
    return chain.filter(h => h.protocol !== 'ssl').length
  }

  function ensureSslAtEnd() {
    const sslIdx = protocolChain.value.findIndex(h => h.protocol === 'ssl')
    if (sslIdx >= 0 && sslIdx !== protocolChain.value.length - 1) {
      const ssl = protocolChain.value.splice(sslIdx, 1)[0]
      protocolChain.value.push(ssl)
    }
  }

  function addHop(protocol: ProtocolType) {
    if (protocol === 'ssl' && protocolChain.value.some(h => h.protocol === 'ssl')) {
      return // SSL 已存在
    }
    if (protocol !== 'ssl' && countNetworkHops(protocolChain.value) >= 4) {
      return // 已达 4 跳上限
    }
    const hop: ChainHopItem = {
      id: uuidv4(),
      protocol,
      enabled: false,
      mode: 'select',
      profileId: null,
      profileSource: null,
      customData: null,
    }
    if (protocol === 'ssl') {
      protocolChain.value.push(hop)
    } else {
      const sslIdx = protocolChain.value.findIndex(h => h.protocol === 'ssl')
      if (sslIdx >= 0) {
        protocolChain.value.splice(sslIdx, 0, hop)
      } else {
        protocolChain.value.push(hop)
      }
    }
  }

  function removeHop(id: string) {
    const hop = protocolChain.value.find(h => h.id === id)
    if (!hop) return
    const sameCount = protocolChain.value.filter(h => h.protocol === hop.protocol).length
    if (sameCount <= 1) return // 至少保留一个
    protocolChain.value = protocolChain.value.filter(h => h.id !== id)
  }

  function onDrop(dragIdx: number, dropIdx: number) {
    const dragged = protocolChain.value[dragIdx]
    const target = protocolChain.value[dropIdx]
    if (dragged.protocol === 'ssl' || target.protocol === 'ssl') return

    const item = protocolChain.value.splice(dragIdx, 1)[0]
    protocolChain.value.splice(dropIdx, 0, item)
    ensureSslAtEnd()
  }

  function toggleHop(id: string, enabled: boolean) {
    const hop = protocolChain.value.find(h => h.id === id)
    if (hop) hop.enabled = enabled
  }

  return {
    // 状态
    headerData,
    scope,
    generalData,
    protocolChain,
    selectedEnvId,
    overriddenPolicies,
    duckdbAccel,
    driverProps,
    saving,
    error,
    // 计算
    isFileDb,
    setFileDb,
    // 初始化
    initDefault,
    initFromEdit,
    // 环境
    selectEnv,
    onPolicyOverride,
    // 提交
    buildSavePayload,
    buildSubmitPayload,
    validate,
    // 协议链
    addHop,
    removeHop,
    onDrop,
    toggleHop,
    countNetworkHops,
    ensureSslAtEnd,
    // 默认值工具
    getDefaultChain,
    defaultDuckdbAccel,
    defaultPolicies,
  }
}