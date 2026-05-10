/**
 * 连接服务相关类型定义
 *
 * 这些类型用于连接服务的 API 调用
 * 注意：后端返回的数据使用 snake_case 命名
 */

/**
 * 数据源元数据
 */
export interface DataSourceMeta {
  name: string
  type: string
  description?: string
}

/**
 * 连接数据库输入
 */
export interface ConnectDatabaseInput {
  db_type: string
  url: string
  name?: string
}

/**
 * 连接数据库响应（后端返回格式）
 */
export interface ConnectDatabaseResponse {
  conn_id: string
  name: string
  db_type: string
  url: string
  status: 'connected' | 'disconnected' | 'error'
  error?: string
  meta?: {
    supports_transaction?: boolean
    supports_streaming?: boolean
    supports_arrow?: boolean
    supports_federated?: boolean
    supports_concurrent_write?: boolean
    is_in_memory?: boolean
  }
}

/**
 * 连接信息响应（后端返回格式）
 */
export interface ConnectionInfoResponse {
  conn_id: string
  name: string
  db_type: string
  url: string
  status: 'connected' | 'disconnected' | 'error'
  is_active: boolean
  meta?: {
    supports_transaction?: boolean
    supports_streaming?: boolean
    supports_arrow?: boolean
    supports_federated?: boolean
    supports_concurrent_write?: boolean
    is_in_memory?: boolean
  }
}

/**
 * 最近连接记录（后端返回格式）
 */
export interface RecentConnectionRecord {
  id: string
  name: string
  db_type: string
  url: string
  connected_at: string
}

/**
 * 连接配置
 */
export interface ConnectionConfig {
  dbType: string
  host?: string
  port?: number
  database?: string
  username?: string
  password?: string
  filePath?: string
  ssl?: boolean
  ssh?: boolean
  options?: Record<string, unknown>
}

/**
 * 驱动字段定义
 */
export interface DriverField {
  key: string
  label: string
  field_type: 'text' | 'number' | 'password' | 'file' | 'select' | 'checkbox'
  required: boolean
  placeholder?: string
  options?: { label: string; value: string }[]
}

/**
 * 驱动选项
 */
export interface DriverOption {
  key: string
  label: string
  type: 'text' | 'number' | 'boolean' | 'select'
  defaultValue?: unknown
  description?: string
}

/**
 * 驱动描述符
 */
export interface DriverDescriptor {
  id: string
  name: string
  description?: string
  defaultPort?: number
  fields: DriverField[]
  extra_options?: DriverOption[]
  require_file?: boolean
}

/**
 * Schema 对象类型
 */
export type SchemaObjectKind = 'table' | 'view' | 'procedure' | 'function' | 'trigger' | 'index'

/**
 * Schema 对象
 */
export interface SchemaObject {
  name: string
  kind: SchemaObjectKind
  schema?: string
  catalog?: string
  comment?: string
}

/**
 * 连接元数据
 */
export interface ConnectionMeta {
  supportsTransaction: boolean
  supportsStreaming: boolean
  supportsArrow: boolean
  supportsFederated: boolean
  supportsConcurrentWrite: boolean
  isInMemory: boolean
}

/**
 * 连接对象（用于前端状态管理）
 */
export interface Connection {
  connId: string
  name: string
  dbType: string
  url: string
  status: 'connected' | 'disconnected' | 'error'
  isActive: boolean
  lastUsed?: string
  meta?: ConnectionMeta
}

/**
 * 最近连接
 */
export interface RecentConnection {
  id: string
  name: string
  dbType: string
  url: string
  connectedAt: string
}
