/**
 * 连接类型定义
 */

import type { Connection, RecentConnection, ProjectConnection, CreateProjectConnectionInput, DriverConfig, DriverField, ConnectionFormData } from '../../types/connection'
import type { DriverOption, ConnectionMethodConfig } from '../../types/index'

export type {
  Connection,
  RecentConnection,
  ProjectConnection,
  CreateProjectConnectionInput,
  DriverConfig,
  DriverField,
  DriverOption,
  ConnectionFormData
}

// 连接状态
export type ConnectionStatus = 'connected' | 'disconnected' | 'error' | 'connecting'

// 连接配置
export interface ConnectionConfiguration {
  id?: string
  name: string
  driver: string
  host: string
  port?: number
  database?: string
  username?: string
  password?: string
  properties?: Record<string, unknown>
  options?: Record<string, unknown>
  connectionMethod?: ConnectionMethodConfig
  connectionType?: 'global' | 'project'
  useDuckdbFed?: boolean
  
  // 认证方式
  authMethod?: 'password' | 'trust' | 'ssh' | 'ssl'
  
  // SSH 配置
  sshHost?: string
  sshPort?: number
  sshUsername?: string
  sshPassword?: string
  sshKeyPath?: string
  
  // SSL 配置
  sslMode?: 'disable' | 'require' | 'verify-ca' | 'verify-full'
  sslCa?: string
  sslCert?: string
  sslKey?: string
}

// 连接配置（别名）
export type ConnectionConfig = ConnectionConfiguration

// 驱动描述符
export interface DriverDescriptor {
  id: string
  name: string
  icon: string
  version?: string
  features: string[]
  defaultPort?: number
  default_port?: number
  description?: string
  fields?: DriverField[]
  extraOptions?: DriverOption[]
  extra_options?: DriverOption[]
  require_file?: boolean
  requireFile?: boolean
  supportsSsl?: boolean
  supports_ssh_tunnel?: boolean
  supports_http_proxy?: boolean
  supports_socks_proxy?: boolean
  supportsSshTunnel?: boolean
  supportsHttpProxy?: boolean
  supportsSocksProxy?: boolean
}

// 连接方法类型
export type ConnectionMethodType = 'direct' | 'ssl' | 'ssh' | 'http_proxy' | 'socks_proxy'
