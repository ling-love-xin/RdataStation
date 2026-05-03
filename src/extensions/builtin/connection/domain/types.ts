/**
 * 连接相关类型定义
 *
 * 集中管理所有与数据库连接相关的 TypeScript 类型
 * 与后端 API 模型保持同步
 */

/** 数据源元数据 */
export interface DataSourceMeta {
  supports_transaction: boolean
  supports_streaming: boolean
  supports_arrow: boolean
  supports_federated: boolean
  supports_concurrent_write: boolean
  is_in_memory: boolean
}

/** 创建数据库连接请求参数 */
export interface ConnectDatabaseInput {
  db_type: string
  url: string
  name?: string
}

/** 连接响应 */
export interface ConnectDatabaseResponse {
  conn_id: string
  name: string
  db_type: string
  url: string
  meta: DataSourceMeta
}

/** 连接信息响应 */
export interface ConnectionInfoResponse {
  conn_id: string
  name: string
  db_type: string
  url: string
  created_at: string
  is_active: boolean
  meta: DataSourceMeta
}

/** 最近连接记录 */
export interface RecentConnectionRecord {
  name: string
  db_type: string
  url: string
  last_used_at: string
}

/** 前端使用的连接对象 */
export interface Connection {
  connId: string
  name: string
  dbType: string
  url: string
  meta: {
    supportsTransaction: boolean
    supportsStreaming: boolean
    supportsArrow: boolean
    supportsFederated: boolean
    supportsConcurrentWrite: boolean
    isInMemory: boolean
  }
}

/** 前端使用的最近连接 */
export interface RecentConnection {
  name: string
  dbType: string
  url: string
  lastUsedAt: string
}

/** 驱动描述符 */
export interface DriverDescriptor {
  id: string
  name: string
  description: string
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
  fieldType: 'text' | 'number' | 'password' | 'file' | 'select'
  required: boolean
  defaultValue?: string
  placeholder?: string
  options?: { label: string; value: string }[]
}

/** 驱动选项 */
export interface DriverOption {
  name: string
  label: string
  optionType: 'string' | 'number' | 'boolean' | 'select'
  defaultValue?: string | number | boolean
  description?: string
  options?: { label: string; value: string }[]
}

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

/** 连接配置 */
export interface ConnectionConfig {
  driver: string
  name?: string
  host?: string
  port?: number
  database?: string
  username?: string
  password?: string
  filePath?: string
  connectionMethod: ConnectionMethodConfig
  options: Record<string, string>
}

/** Schema 对象类型 */
export type SchemaObjectKind = 'database' | 'schema' | 'table' | 'view' | 'column'

/** Schema 对象 */
export interface SchemaObject {
  name: string
  kind: SchemaObjectKind
  children?: SchemaObject[]
}

// ==================== 项目级连接类型 ====================

/** 项目连接 */
export interface ProjectConnection {
  id: string
  name: string
  db_type: string
  host: string
  port: number
  database: string
  username?: string
  password?: string
  options?: string
  connection_method: string
  method_config?: string
  created_at: string
  updated_at: string
}

/** 创建项目连接输入 */
export interface CreateProjectConnectionInput {
  project_path: string
  name: string
  db_type: string
  host: string
  port: number
  database: string
  username?: string
  password?: string
  options?: string
  connection_method: string
  method_config?: string
}
