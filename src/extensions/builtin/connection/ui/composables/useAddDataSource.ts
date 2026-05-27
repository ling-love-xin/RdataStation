/**
 * useAddDataSource — 新增数据源提交逻辑封装
 *
 * 统一管理新增/编辑数据源的状态、校验、快照、协议链与提交载荷构建。
 * 对应文档: docs/frontend/connection/add-datasource-frontend-plan.md (Phase 4-5)
 */

import { v4 as uuidv4 } from 'uuid'
import { reactive, ref, watch, onMounted } from 'vue'

import { useProjectStore } from '@/core/project/stores/project'

import { useEnvironmentStore } from '../stores/environmentStore'

import type { ConnectDatabaseInput } from '../../domain/types'

// ==================== 类型定义 ====================

/** 连接作用域 */
export interface ConnectionScope {
  global: boolean
  project: boolean
}

/** 暂存项 */
export interface StagingItem {
  id: string
  name: string
  /** 数据库类型名称 (e.g. "mysql", "postgres")，注意与 driverId（具体驱动实例ID）区分 */
  driver?: string
  /** 具体驱动实例 ID (e.g. "mysql_local_01") */
  driverId?: string
  url?: string
  formData?: Record<string, unknown>
  authConfigId?: string | null
  authMethod?: string
  networkConfigId?: string | null
  driverProperties?: string | null
  advancedOptions?: string | null
  environmentId?: string | null
  scope?: 'global' | 'project'
  description?: string
  schemaName?: string
  options?: string
  metadataPath?: string
  tags?: string
  useDuckdbFed?: boolean
  applied?: boolean
}

/** StagingItem 字段列表 */
const STAGING_FIELDS = [
  'id', 'name', 'driver', 'driverId', 'url', 'formData',
  'authConfigId', 'authMethod', 'networkConfigId',
  'driverProperties', 'advancedOptions', 'environmentId',
  'scope', 'description', 'schemaName', 'options',
  'metadataPath', 'tags', 'useDuckdbFed', 'applied'
] as const

/** localStorage 存储键 */
const STAGING_STORAGE_KEY = 'rdata-station-staging-items'

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
  auth_method: string | null
  network_config_id: string | null
  driver_properties: string
  advanced_options: string
  schema_name: string | null
  options: string | null
  metadata_path: string | null
  tags: string | null
  use_duckdb_fed: boolean | null
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
  const formData = ref<Record<string, unknown>>({})

  // Auth / Network / Extra — owned by composable, synced by dialog
  const authConfigId = ref<string | null>(null)
  const authMethod = ref<string>('password')
  const networkConfigId = ref<string | null>(null)
  const schemaName = ref<string | null>(null)
  const options = ref<string | null>(null)
  const metadataPath = ref<string | null>(null)
  const tags = ref<string | null>(null)
  const useDuckdbFed = ref<boolean | null>(null)

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
    authConfigId.value = null
    authMethod.value = 'password'
    networkConfigId.value = null
    schemaName.value = null
    options.value = null
    metadataPath.value = null
    tags.value = null
    useDuckdbFed.value = null
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
      } catch (err) {
        console.warn('[useAddDataSource] 高级选项 JSON 解析失败:', err instanceof Error ? err.message : String(err))
      }
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
      auth_config_id: authConfigId.value,
      auth_method: authMethod.value,
      network_config_id: networkConfigId.value,
      driver_properties: JSON.stringify(driverProps.value),
      advanced_options: JSON.stringify({
        protocol_chain: protocolChain.value.filter(h => h.enabled),
        env_policies: overriddenPolicies.value,
        duckdb_accel: duckdbAccel,
      }),
      schema_name: schemaName.value,
      options: options.value,
      metadata_path: metadataPath.value,
      tags: tags.value,
      use_duckdb_fed: useDuckdbFed.value,
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
      conn_id: null,
      db_type: params.dbType,
      url: params.url,
      name: savePayload.name,
      connection_type: savePayload.scope,
      project_id: params.projectId ?? null,
      description: savePayload.description || null,
      driver_id: params.driverId ?? null,
      environment_id: savePayload.environment_id,
      auth_config_id: params.authConfigId ?? savePayload.auth_config_id,
      auth_method: params.authMethod ?? savePayload.auth_method ?? 'password',
      network_config_id: params.networkConfigId ?? savePayload.network_config_id,
      driver_properties: savePayload.driver_properties || null,
      advanced_options: savePayload.advanced_options || null,
      options: savePayload.options || null,
      tags: savePayload.tags || null,
      metadata_path: savePayload.metadata_path || null,
      schema_name: savePayload.schema_name || null,
      use_duckdb_fed: savePayload.use_duckdb_fed ?? null,
    }
  }

  // ========== 校验 ==========
  /** 验证结果类型 */
  interface ExtendedValidationResult extends ValidationResult {
    warnings?: string[]
  }

  /** 扩展验证（包含警告） */
  function validateExtended(): ExtendedValidationResult {
    const errs: Record<string, string> = {}
    const warnings: string[] = []

    if (!headerData.name.trim()) errs.name = '请输入数据源名称'
    if (!headerData.selectedDriverId) errs.driver = '请选择数据驱动'
    if (!scope.global && !scope.project) errs.scope = '请至少选择一个作用域'

    const hasEnabledChain = protocolChain.value.some(h => h.enabled)
    if (hasEnabledChain) {
      const sslIdx = protocolChain.value.findIndex(h => h.protocol === 'ssl' && h.enabled)
      if (sslIdx >= 0 && sslIdx !== protocolChain.value.length - 1) {
        errs.chain = 'SSL 必须在协议链末尾'
      }

      const nonSslCount = protocolChain.value.filter(h => h.enabled && h.protocol !== 'ssl').length
      if (nonSslCount > 3) {
        warnings.push('协议链超过 3 跳，可能影响连接性能')
      }
    }

    if (scope.project && !useProjectStore().currentProject) {
      errs.project = '请先打开一个项目'
    }

    return { valid: Object.keys(errs).length === 0, errors: errs, warnings }
  }

  /** 基础验证 */
  function validate(): ValidationResult {
    const result = validateExtended()
    return { valid: result.valid, errors: result.errors }
  }

  /** URL 验证 */
  function validateUrl(url: string): { valid: boolean; error?: string } {
    if (!url) return { valid: false, error: 'URL 不能为空' }

    try {
      const urlObj = new URL(url)
      if (!['mysql', 'postgres', 'postgresql', 'sqlite', 'duckdb', 'mongodb', 'redis'].some(p => urlObj.protocol.includes(p))) {
        return { valid: false, error: '不支持的数据库协议' }
      }
      return { valid: true }
    } catch {
      return { valid: false, error: '无效的 URL 格式' }
    }
  }

  /** 端口范围验证 */
  function validatePort(port: number): { valid: boolean; error?: string } {
    if (port < 1 || port > 65535) {
      return { valid: false, error: '端口号必须在 1-65535 范围内' }
    }
    return { valid: true }
  }

  /** IP 地址验证 */
  function validateHost(host: string): { valid: boolean; error?: string } {
    if (!host) return { valid: false, error: '主机地址不能为空' }
    const ipv4Pattern = /^(\d{1,3}\.){3}\d{1,3}$/
    const hostnamePattern = /^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/
    if (ipv4Pattern.test(host) || hostnamePattern.test(host) || host === 'localhost') {
      return { valid: true }
    }
    return { valid: false, error: '无效的主机地址格式' }
  }

  // ========== 类型守卫 ==========
  /** 判断是否为有效的 StagingItem */
  function isValidStagingItem(item: unknown): item is StagingItem {
    if (!item || typeof item !== 'object') return false
    const s = item as StagingItem
    return typeof s.name === 'string' && s.name.length > 0
  }

  /** 判断是否为文件型数据库 */
  function isFileDatabase(driverId: string): boolean {
    return ['sqlite', 'duckdb'].includes(driverId.toLowerCase())
  }

  /** 判断是否需要快照（全局配置引用） */
  function needsSnapshot(configId: string | null | undefined): boolean {
    return !!configId?.startsWith('G_') && !configId.startsWith('GP_')
  }

  // ========== 连接字符串构建 ==========
  /** 构建 JDBC 连接字符串 */
  function buildJdbcUrl(driverId: string, host: string, port: number, database: string): string {
    if (isFileDatabase(driverId)) {
      return `jdbc:${driverId}:${database}`
    }
    return `jdbc:${driverId}://${host}:${port}/${database}`
  }

  /** 构建标准连接 URL（优先使用 url_template，回退到硬编码模式） */
  function buildStandardUrl(driverId: string, host: string, port: number, database: string, urlTemplate?: string | null): string {
    if (urlTemplate) {
      return urlTemplate
        .replace('{host}', host || 'localhost')
        .replace('{port}', String(port || ''))
        .replace('{database}', database || '')
        .replace('{username}', '')
        .replace('{password}', '')
        .replace('{file_path}', database || '')
    }
    // Fallback for drivers without url_template
    if (isFileDatabase(driverId)) {
      return `${driverId}:${database}`
    }
    return `${driverId}://${host}:${port}/${database}`
  }

  /** 提取连接 URL 中的数据库名称 */
  function extractDatabaseFromUrl(url: string): string | null {
    try {
      const urlObj = new URL(url)
      const path = urlObj.pathname
      if (path && path.length > 1) {
        return decodeURIComponent(path.substring(1))
      }
      return null
    } catch {
      return null
    }
  }

  /** 从 URL 中提取主机和端口 */
  function extractHostAndPort(url: string): { host: string; port: number } | null {
    try {
      const urlObj = new URL(url)
      return {
        host: urlObj.hostname,
        port: parseInt(urlObj.port) || (urlObj.protocol.includes('mysql') ? 3306 : 5432),
      }
    } catch {
      return null
    }
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

  // ========== StagingItem 管理 ==========
  const stagingItems = ref<StagingItem[]>([{ id: '', name: '' }])
  const stagingIndex = ref(0)
  const isResetting = ref(false)

  /**
   * 构建 StagingItem（统一字段处理）
   */
  function buildStagingItem(
    name: string,
    driver: string | undefined,
    driverId: string | undefined,
    url: string,
    formData: Record<string, unknown>,
    authConfigId: string | null,
    authMethod: string,
    networkConfigId: string | null,
    driverProperties: string | null,
    advancedOptions: string | null,
    environmentId: string | null,
    description: string | undefined,
    schemaName: string | undefined,
    options: string | undefined,
    metadataPath: string | undefined,
    tags: string | undefined,
    useDuckdbFed: boolean
  ): StagingItem {
    return {
      id: uuidv4(),
      name,
      driver,
      driverId,
      url,
      formData: { ...formData },
      authConfigId,
      authMethod,
      networkConfigId,
      driverProperties,
      advancedOptions,
      environmentId,
      scope: scope.global ? 'global' : 'project',
      description,
      schemaName,
      options,
      metadataPath,
      tags,
      useDuckdbFed,
      applied: false,
    }
  }

  /**
   * 从 StagingItem 更新表单数据（含 auth/network/properties 恢复）
   */
  function applyStagingItem(item: StagingItem) {
    isResetting.value = true
    headerData.name = item.name || ''
    headerData.description = item.description || ''
    formData.value = item.formData ? { ...item.formData } : {}
    if (item.scope) {
      scope.global = item.scope === 'global'
      scope.project = item.scope === 'project'
    }
    authConfigId.value = item.authConfigId ?? null
    authMethod.value = item.authMethod ?? 'password'
    networkConfigId.value = item.networkConfigId ?? null
    schemaName.value = item.schemaName ?? null
    options.value = item.options ?? null
    metadataPath.value = item.metadataPath ?? null
    tags.value = item.tags ?? null
    useDuckdbFed.value = item.useDuckdbFed ?? null
    isResetting.value = false
  }

  /**
   * 加载持久化的暂存项
   */
  function loadStagingItems() {
    try {
      const stored = localStorage.getItem(STAGING_STORAGE_KEY)
      if (stored) {
        const parsed = JSON.parse(stored)
        if (Array.isArray(parsed) && parsed.length > 0) {
          stagingItems.value = parsed
        }
      }
    } catch (e) {
      console.error('[Staging] 加载暂存项失败:', e)
    }
  }

  /**
   * 保存暂存项到 localStorage
   */
  function saveStagingItems() {
    try {
      localStorage.setItem(STAGING_STORAGE_KEY, JSON.stringify(stagingItems.value))
    } catch (e) {
      console.error('[Staging] 保存暂存项失败:', e)
    }
  }

  /**
   * 清空持久化的暂存项
   */
  function clearStagingItems() {
    try {
      localStorage.removeItem(STAGING_STORAGE_KEY)
      stagingItems.value = [{ id: '', name: '' }]
      stagingIndex.value = 0
    } catch (e) {
      console.error('[Staging] 清空暂存项失败:', e)
    }
  }

  /**
   * 添加暂存项（同时重置 auth/network/extra 状态）
   */
  function addStaging() {
    stagingItems.value.push({ id: uuidv4(), name: '', applied: false })
    stagingIndex.value = stagingItems.value.length - 1
    authConfigId.value = null
    authMethod.value = 'password'
    networkConfigId.value = null
    schemaName.value = null
    options.value = null
    metadataPath.value = null
    tags.value = null
    useDuckdbFed.value = null
  }

  /**
   * 标记暂存项为已应用
   */
  function markStagingApplied(index: number) {
    if (stagingItems.value[index]) {
      stagingItems.value[index].applied = true
    }
  }

  /**
   * 移除暂存项
   */
  function removeStaging(index: number) {
    if (stagingItems.value.length <= 1) return
    stagingItems.value.splice(index, 1)
    if (stagingIndex.value >= stagingItems.value.length) {
      stagingIndex.value = stagingItems.value.length - 1
    }
  }

  /**
   * 选择暂存项
   */
  function selectStaging(index: number) {
    stagingIndex.value = index
  }

  // 初始化时加载暂存项
  onMounted(() => {
    loadStagingItems()
  })

  // 监听暂存项变化，自动持久化
  watch(stagingItems, saveStagingItems, { deep: true })

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
    formData,
    authConfigId,
    authMethod,
    networkConfigId,
    schemaName,
    options,
    metadataPath,
    tags,
    useDuckdbFed,
    saving,
    error,
    // 暂存项管理
    stagingItems,
    stagingIndex,
    isResetting,
    buildStagingItem,
    applyStagingItem,
    addStaging,
    removeStaging,
    selectStaging,
    clearStagingItems,
    markStagingApplied,
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
    validateExtended,
    validateUrl,
    validatePort,
    validateHost,
    // 协议链
    addHop,
    removeHop,
    onDrop,
    toggleHop,
    countNetworkHops,
    ensureSslAtEnd,
    // 类型守卫
    isValidStagingItem,
    isFileDatabase,
    needsSnapshot,
    // 连接字符串
    buildJdbcUrl,
    buildStandardUrl,
    extractDatabaseFromUrl,
    extractHostAndPort,
    // 默认值工具
    getDefaultChain,
    defaultDuckdbAccel,
    defaultPolicies,
  }
}