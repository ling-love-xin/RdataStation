/**
 * 数据库驱动类型定义
 *
 * 与后端 Rust 结构体保持同步
 * 用于动态渲染连接配置表单
 */

/** 驱动字段类型 */
export type DriverFieldType =
  | 'text'
  | 'password'
  | 'number'
  | 'file'
  | 'select'

/** 驱动选项类型 */
export type DriverOptionType =
  | { type: 'string' }
  | { type: 'number' }
  | { type: 'boolean' }
  | { type: 'select'; options: string[] }
  | { type: 'file' }

/** Select 选项 */
export interface SelectOption {
  label: string
  value: string
}

/** 驱动字段定义（用于前端表单渲染） */
export interface DriverField {
  /** 字段键 */
  key: string
  /** 显示标签 */
  label: string
  /** 字段类型 */
  field_type: DriverFieldType
  /** 是否必需 */
  required: boolean
  /** 默认值 */
  default_value?: string
  /** 占位符文本 */
  placeholder?: string
  /** 选项列表（用于 select 类型） */
  options?: SelectOption[]
}

/** 驱动选项定义 */
export interface DriverOption {
  /** 选项键 */
  key: string
  /** 显示名称 */
  label: string
  /** 默认值 */
  default_value: string
  /** 选项类型 */
  option_type: DriverOptionType
  /** 是否必需 */
  required: boolean
  /** 描述 */
  description?: string
}

/** 驱动描述符（类似 DBeaver 的驱动定义） */
export interface DriverDescriptor {
  /** 驱动 ID */
  id: string
  /** 显示名称 */
  name: string
  /** 驱动描述 */
  description: string
  /** 默认端口 */
  default_port?: number
  /** 是否需要数据库名 */
  require_database: boolean
  /** 是否需要文件路径 */
  require_file: boolean
  /** 是否支持 SSL/TLS */
  supports_ssl: boolean
  /** 是否支持 SSH 隧道 */
  supports_ssh_tunnel: boolean
  /** 是否支持 HTTP 代理 */
  supports_http_proxy: boolean
  /** 是否支持 SOCKS 代理 */
  supports_socks_proxy: boolean
  /** 表单字段定义 */
  fields: DriverField[]
  /** 额外选项 */
  extra_options: DriverOption[]
}

/** 连接方式类型 */
export type ConnectionMethodType =
  | 'direct'
  | 'ssl'
  | 'ssh'
  | 'http_proxy'
  | 'socks_proxy'

/** TLS 版本 */
export type TlsVersion = 'tls1_0' | 'tls1_1' | 'tls1_2' | 'tls1_3'

/** SSL 配置 */
export interface SslConfig {
  verify_server_cert: boolean
  ca_cert_path?: string
  client_cert_path?: string
  client_key_path?: string
  min_tls_version: TlsVersion
}

/** SSH 认证方式 */
export type SshAuthType = 'password' | 'private_key' | 'agent'

/** SSH 认证配置 */
export interface SshAuth {
  type: SshAuthType
  password?: string
  key_path?: string
  passphrase?: string
}

/** SSH 配置 */
export interface SshConfig {
  host: string
  port: number
  username: string
  auth: SshAuth
  remote_host: string
  remote_port: number
  local_port?: number
  timeout_secs: number
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
  no_proxy?: string[]
  timeout_secs: number
}

/** 连接方式配置 */
export interface ConnectionMethodConfig {
  type: ConnectionMethodType
  ssl_config?: SslConfig
  ssh_config?: SshConfig
  proxy_config?: ProxyConfig
}

/** 连接配置（统一模型） */
export interface ConnectionConfig {
  /** 驱动类型 */
  driver: string
  /** 连接名称 */
  name?: string
  /** 主机地址 */
  host?: string
  /** 端口 */
  port?: number
  /** 数据库名 */
  database?: string
  /** 用户名 */
  username?: string
  /** 密码 */
  password?: string
  /** 文件路径（SQLite/DuckDB） */
  file_path?: string
  /** 连接方式 */
  connection_method: ConnectionMethodConfig
  /** 额外选项 */
  options: Record<string, string>
}

/** 表单数据对象 */
export interface FormData {
  [key: string]: unknown
  options?: Record<string, string>
}
