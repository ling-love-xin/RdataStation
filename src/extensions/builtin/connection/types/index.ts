/**
 * 连接插件统一类型定义
 *
 * 所有连接相关类型的唯一来源
 * 其他模块只引用此文件，不重复定义
 */

// ============================================================================
// 基础连接类型
// ============================================================================

/** 数据源元数据 */
export interface DataSourceMeta {
  supportsTransaction: boolean
  supportsStreaming: boolean
  supportsArrow: boolean
  supportsFederated: boolean
  supportsConcurrentWrite: boolean
  isInMemory: boolean
}

/** 后端返回的元数据格式 */
export interface DataSourceMetaRaw {
  supports_transaction: boolean
  supports_streaming: boolean
  supports_arrow: boolean
  supports_federated: boolean
  supports_concurrent_write: boolean
  is_in_memory: boolean
}

/** 连接对象（前端使用） */
export interface Connection {
  connId: string
  name: string
  dbType: string
  url: string
  status: 'connected' | 'disconnected' | 'error'
  isActive: boolean
  meta: DataSourceMeta
}

/** 最近连接记录 */
export interface RecentConnection {
  id: string
  name: string
  dbType: string
  url: string
  connectedAt: string
}

/** 后端返回的最近连接格式 */
export interface RecentConnectionRaw {
  name: string
  db_type: string
  url: string
  last_used_at: string
}

// ============================================================================
// 项目连接类型
// ============================================================================

/** 项目连接 */
export interface ProjectConnection {
  id: string
  name: string
  db_type: string
  host: string
  port?: number
  database?: string
  username?: string
  password?: string
  properties?: Record<string, unknown>
  created_at: string
  updated_at: string
}

/** 创建项目连接输入 */
export interface CreateProjectConnectionInput {
  project_path: string
  name: string
  db_type: string
  host: string
  port?: number
  database?: string
  username?: string
  password?: string
  properties?: Record<string, unknown>
  connection_type?: 'global' | 'project'
  use_duckdb_fed?: boolean
}

// ============================================================================
// 驱动配置类型
// ============================================================================

/** 驱动描述符 */
export interface DriverDescriptor {
  id: string
  name: string
  description: string
  icon?: string
  defaultPort?: number
  requiresDatabase: boolean
  requiresFile: boolean
  supportsSsl: boolean
  supportsSshTunnel: boolean
  supportsHttpProxy: boolean
  supportsSocksProxy: boolean
  fields: DriverField[]
  extraOptions: DriverOption[]
}

/** 驱动字段定义 */
export interface DriverField {
  name: string
  label: string
  type: 'text' | 'number' | 'password' | 'file' | 'select' | 'checkbox'
  fieldType?: 'text' | 'number' | 'password' | 'file' | 'select' | 'checkbox'
  required?: boolean
  default?: unknown
  placeholder?: string
  description?: string
  options?: { label: string; value: unknown }[]
}

/** 驱动选项 */
export interface DriverOption {
  name: string
  label: string
  type: 'string' | 'number' | 'boolean' | 'select'
  optionType?: 'string' | 'number' | 'boolean' | 'select'
  option_type?: 'string' | 'number' | 'boolean' | 'select'
  default?: string | number | boolean
  description?: string
  options?: { label: string; value: string }[]
}

/** 驱动特性 */
export type DriverFeature =
  | 'schemas'
  | 'tables'
  | 'views'
  | 'procedures'
  | 'functions'
  | 'triggers'
  | 'indexes'
  | 'foreignKeys'
  | 'ssl'
  | 'sshTunnel'
  | 'httpProxy'

// ============================================================================
// SSL/SSH/代理配置类型
// ============================================================================

/** TLS 版本 */
export type TlsVersion = 'tls1_0' | 'tls1_1' | 'tls1_2' | 'tls1_3'

/** SSL 配置 */
export interface SslConfig {
  verifyServerCert: boolean
  caCertPath?: string
  clientCertPath?: string
  clientKeyPath?: string
  minTlsVersion: TlsVersion
}

/** SSH 认证方式 */
export type SshAuthType = 'password' | 'private_key' | 'agent'

/** SSH 认证配置 */
export interface SshAuth {
  type: SshAuthType
  password?: string
  keyPath?: string
  passphrase?: string
}

/** SSH 配置 */
export interface SshConfig {
  host: string
  port: number
  username: string
  auth: SshAuth
  remoteHost: string
  remotePort: number
  localPort?: number
  timeoutSecs: number
}

/** 代理认证 */
export interface ProxyAuth {
  username: string
  password: string
}

/** 代理配置 */
export interface ProxyConfig {
  host: string
  port: number
  auth?: ProxyAuth
  noProxy?: string[]
  timeoutSecs: number
}

/** 连接方式类型 */
export type ConnectionMethodType = 'direct' | 'ssl' | 'ssh' | 'http_proxy' | 'socks_proxy'

/** 连接方式配置 */
export interface ConnectionMethodConfig {
  type: ConnectionMethodType
  sslConfig?: SslConfig
  sshConfig?: SshConfig
  proxyConfig?: ProxyConfig
}

// ============================================================================
// 连接表单类型
// ============================================================================

/** 连接配置 */
export interface ConnectionConfig {
  name: string
  driver: string
  host?: string
  port?: number
  database?: string
  username?: string
  password?: string
  ssl?: boolean
  sshTunnel?: boolean
  properties?: Record<string, unknown>
}

// ============================================================================
// API 响应类型
// ============================================================================

/** 连接响应 */
export interface ConnectionResponse {
  conn_id: string
  name: string
  db_type: string
  url: string
  status: 'connected' | 'disconnected' | 'error'
  is_active: boolean
  meta?: DataSourceMetaRaw
}

/** 连接信息响应 */
export interface ConnectionInfoResponse {
  conn_id: string
  name: string
  db_type: string
  url: string
  created_at: string
  is_active: boolean
  meta: DataSourceMetaRaw
}

/** 项目连接响应 */
export interface ProjectConnectionResponse {
  id: string
  name: string
  db_type: string
  host: string
  port?: number
  database?: string
  username?: string
  password?: string
  properties?: Record<string, unknown>
  created_at: string
  updated_at: string
}

/** 创建连接输入 */
export interface CreateConnectionInput {
  project_path: string
  name: string
  db_type: string
  host: string
  port?: number
  database?: string
  username?: string
  password?: string
  properties?: Record<string, unknown>
}

// ============================================================================
// Schema 对象类型
// ============================================================================

/** Schema 对象类型 */
export type SchemaObjectKind = 'database' | 'schema' | 'table' | 'view' | 'column'

/** Schema 对象 */
export interface SchemaObject {
  name: string
  kind: SchemaObjectKind
  children?: SchemaObject[]
}
