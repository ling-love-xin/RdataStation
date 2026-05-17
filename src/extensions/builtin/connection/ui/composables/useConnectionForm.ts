import { ref, computed, reactive } from 'vue'

import { useProjectStore } from '@/core/project/stores/project'

import type { DriverDescriptor } from '../types/connection'

export interface SshTunnelConfig {
  enabled: boolean
  host: string
  port: number
  username: string
  authType: 'password' | 'keyFile'
  password: string
  privateKey: string
  localPort: number
  keepAlive: number
}

export interface ProxyConnectionConfig {
  enabled: boolean
  type: 'http' | 'https' | 'socks4' | 'socks5'
  host: string
  port: number
  requireAuth: boolean
  username: string
  password: string
}

export interface ConnectionFormData {
  name: string
  driver: string
  host: string
  port?: number
  database: string
  username?: string
  password?: string
  connectionType: 'global' | 'project'
  useDuckdbFed: boolean
  options: Record<string, unknown>
  enableSsl: boolean
  sslCa: string
  sslCert: string
  sslKey: string
}

export interface TestResult {
  success: boolean
  message: string
  server_version?: string
  response_time_ms?: number
}

const DRIVER_DEFAULTS: Record<string, Partial<ConnectionFormData>> = {
  mysql: { host: 'localhost', port: 3306, username: 'root' },
  postgres: { host: 'localhost', port: 5432, username: 'postgres' },
  sqlite: { database: './database.db' },
  duckdb: { database: './database.duckdb' },
}

function getDefaultFormData(): ConnectionFormData {
  return {
    name: '',
    driver: '',
    host: '',
    port: undefined,
    database: '',
    username: '',
    password: '',
    connectionType: 'global',
    useDuckdbFed: false,
    options: {},
    enableSsl: false,
    sslCa: '',
    sslCert: '',
    sslKey: '',
  }
}

function getDefaultSshConfig(): SshTunnelConfig {
  return {
    enabled: false,
    host: '',
    port: 22,
    username: '',
    authType: 'password',
    password: '',
    privateKey: '',
    localPort: 0,
    keepAlive: 0,
  }
}

function getDefaultProxyConfig(): ProxyConnectionConfig {
  return {
    enabled: false,
    type: 'socks5',
    host: '',
    port: 1080,
    requireAuth: false,
    username: '',
    password: '',
  }
}

export function useConnectionForm(
  editingConnection: () => import('../types/connection').ConnectionConfig | null | undefined,
  selectedDriver: () => DriverDescriptor | null
) {
  const projectStore = useProjectStore()

  const formData = ref<ConnectionFormData>(getDefaultFormData())
  const sshConfig = reactive<SshTunnelConfig>(getDefaultSshConfig())
  const proxyConfig = reactive<ProxyConnectionConfig>(getDefaultProxyConfig())

  const errors = ref<Record<string, string>>({})
  const testResult = ref<TestResult | null>(null)

  const saveToGlobal = ref(true)
  const saveToProject = ref(false)

  const nameManuallyEdited = ref(false)
  let previousAutoName = ''

  const hasProject = computed(() => projectStore.hasProject)

  const requiresFile = computed(() => {
    const driver = selectedDriver()
    return driver?.requireFile === true || driver?.require_file === true
  })

  const connectionUrl = computed(() => {
    const driver = formData.value.driver
    if (!driver) return ''

    if (requiresFile.value) {
      return `${driver}://${formData.value.database}`
    }
    return `${driver}://${formData.value.host}:${formData.value.port}/${formData.value.database}`
  })

  const autoGenerateName = computed(() => {
    const driver = formData.value.driver
    if (!driver) return ''

    if (requiresFile.value) {
      const path = formData.value.database
      if (!path) return driver
      const parts = path.split(/[\\/]/)
      const fileName = parts[parts.length - 1]
      return fileName.split('.')[0] || driver
    }

    const host = formData.value.host || 'localhost'
    const database = formData.value.database
    return `${driver}@${host}${database ? '/' + database : ''}`
  })

  function applyDriverDefaults(driverId: string) {
    const defaults = DRIVER_DEFAULTS[driverId]
    if (!defaults) return

    if (defaults.host && !formData.value.host) {
      formData.value.host = defaults.host
    }
    if (defaults.port && !formData.value.port) {
      formData.value.port = defaults.port
    }
    if (defaults.username && !formData.value.username) {
      formData.value.username = defaults.username
    }
    if (defaults.database && !formData.value.database) {
      formData.value.database = defaults.database
    }
  }

  function resetNameEditState() {
    nameManuallyEdited.value = false
    formData.value.name = autoGenerateName.value
    previousAutoName = autoGenerateName.value
  }

  function updateFormData(data: Partial<ConnectionFormData>) {
    formData.value = {
      ...formData.value,
      ...data,
    }
  }

  function validateForm(): boolean {
    errors.value = {}

    if (!formData.value.name?.trim()) {
      errors.value.name = '请输入连接名称'
    }

    if (!requiresFile.value) {
      if (!formData.value.host?.trim()) {
        errors.value.host = '请输入主机地址'
      }
      if (!formData.value.database?.trim()) {
        errors.value.database = '请输入数据库名称'
      }
    } else {
      if (!formData.value.database?.trim()) {
        errors.value.database = '请选择或输入数据库文件路径'
      }
    }

    return Object.keys(errors.value).length === 0
  }

  function buildConnectionConfig(): Record<string, unknown> {
    const driver = formData.value.driver || selectedDriver()?.id || ''
    const url = connectionUrl.value

    return {
      id: '',
      name: formData.value.name || '',
      db_type: driver,
      host: requiresFile.value ? null : formData.value.host,
      port: requiresFile.value ? null : formData.value.port,
      database: formData.value.database,
      username: formData.value.username || null,
      password: formData.value.password || undefined,
      url,
      properties: {
        useDuckdbFed: formData.value.useDuckdbFed,
        enableSsh: sshConfig.enabled,
        sshConfig: sshConfig.enabled
          ? {
              host: sshConfig.host,
              port: sshConfig.port,
              username: sshConfig.username,
              authType: sshConfig.authType,
              password: sshConfig.authType === 'password' ? sshConfig.password : undefined,
              privateKey: sshConfig.authType === 'keyFile' ? sshConfig.privateKey : undefined,
              localPort: sshConfig.localPort || undefined,
              keepAlive: sshConfig.keepAlive || undefined,
            }
          : null,
        enableProxy: proxyConfig.enabled,
        proxyConfig: proxyConfig.enabled
          ? {
              type: proxyConfig.type,
              host: proxyConfig.host,
              port: proxyConfig.port,
              requireAuth: proxyConfig.requireAuth,
              username: proxyConfig.requireAuth ? proxyConfig.username : undefined,
              password: proxyConfig.requireAuth ? proxyConfig.password : undefined,
            }
          : null,
        enableSsl: formData.value.enableSsl,
        sslConfig: formData.value.enableSsl
          ? {
              caCert: formData.value.sslCa,
              clientCert: formData.value.sslCert,
              clientKey: formData.value.sslKey,
            }
          : null,
        ...formData.value.options,
      },
      saveToGlobal: saveToGlobal.value,
      saveToProject: saveToProject.value && projectStore.hasProject,
      useDuckdbFed: formData.value.useDuckdbFed,
    }
  }

  function resetForm() {
    formData.value = getDefaultFormData()
    Object.assign(sshConfig, getDefaultSshConfig())
    Object.assign(proxyConfig, getDefaultProxyConfig())
    errors.value = {}
    testResult.value = null
    nameManuallyEdited.value = false
    saveToGlobal.value = true
    saveToProject.value = false
  }

  function loadEditingData(conn: import('../types/connection').ConnectionConfig) {
    formData.value = {
      name: conn.name || '',
      driver: conn.driver || '',
      host: conn.host || '',
      port: conn.port,
      database: conn.database || '',
      username: conn.username || '',
      password: conn.password || '',
      connectionType: 'global',
      useDuckdbFed: false,
      options: {},
      enableSsl: false,
      sslCa: '',
      sslCert: '',
      sslKey: '',
    }
  }

  return {
    formData,
    sshConfig,
    proxyConfig,
    errors,
    testResult,
    saveToGlobal,
    saveToProject,
    hasProject,
    requiresFile,
    connectionUrl,
    autoGenerateName,
    applyDriverDefaults,
    resetNameEditState,
    updateFormData,
    validateForm,
    buildConnectionConfig,
    resetForm,
    loadEditingData,
    DRIVER_DEFAULTS,
  }
}