import { invoke } from '@tauri-apps/api/core'

import type {
  ConnectDatabaseInput,
  ConnectDatabaseResponse,
  ConnectionInfoResponse,
  RecentConnectionRecord,
  DriverDescriptor,
  ConnectionConfig,
} from '../types/connection-service'

/**
 * 连接服务
 *
 * 负责与后端连接相关的所有 API 调用
 */

/**
 * 创建数据库连接
 */
export async function connectDatabase(
  dbType: string,
  url: string,
  name?: string
): Promise<ConnectDatabaseResponse> {
  const input: ConnectDatabaseInput = { db_type: dbType, url, name }
  return invoke('connect_database', { input })
}

/**
 * 获取所有连接
 */
export async function getConnections(): Promise<ConnectionInfoResponse[]> {
  return invoke('get_connections')
}

/**
 * 获取当前活动连接
 */
export async function getActiveConnection(): Promise<ConnectionInfoResponse | null> {
  return invoke('get_active_connection')
}

/**
 * 切换活动连接
 */
export async function switchConnection(connId: string): Promise<void> {
  return invoke('switch_connection', { connId })
}

/**
 * 关闭指定连接
 */
export async function closeConnection(connId: string): Promise<void> {
  return invoke('close_connection', { connId })
}

/**
 * 关闭所有连接
 */
export async function closeAllConnections(): Promise<void> {
  return invoke('close_all_connections')
}

/**
 * 获取最近连接列表
 */
export async function getRecentConnections(): Promise<RecentConnectionRecord[]> {
  return invoke('get_recent_connections')
}

/**
 * 移除最近连接记录
 */
export async function removeRecentConnection(name: string): Promise<void> {
  return invoke('remove_recent_connection', { name })
}

/**
 * 测试连接（不保存）
 */
export async function testConnection(dbType: string, url: string): Promise<void> {
  return invoke('test_connection', { dbType, url })
}

/**
 * 获取所有支持的驱动
 */
export async function getDrivers(): Promise<DriverDescriptor[]> {
  return invoke('get_drivers')
}

/**
 * 获取指定驱动信息
 */
export async function getDriverInfo(driverId: string): Promise<DriverDescriptor | null> {
  return invoke('get_driver_info', { driverId })
}

/**
 * 使用配置创建连接
 */
export async function createConnection(config: ConnectionConfig): Promise<ConnectDatabaseResponse> {
  return invoke('create_connection', { config })
}

/**
 * 测试连接配置
 */
export async function testConnectionConfig(config: ConnectionConfig): Promise<void> {
  return invoke('test_connection_config', { config })
}

// ==================== Schema Explorer ====================

/**
 * 获取数据库列表
 */
export async function listDatabases(connId: string): Promise<string[]> {
  return invoke('list_databases', { connId })
}

/**
 * 获取 Schema 列表
 */
export async function listSchemas(connId: string, database: string): Promise<string[]> {
  return invoke('list_schemas', { connId, database })
}

/**
 * 获取表列表
 */
export async function listTables(
  connId: string,
  database: string,
  schema?: string
): Promise<Array<{ name: string; kind: string; children?: any[] }>> {
  return invoke('list_tables', { connId, database, schema })
}

/**
 * 获取列列表
 */
export async function listColumns(
  connId: string,
  database: string,
  table: string,
  schema?: string
): Promise<Array<{ name: string; kind: string }>> {
  return invoke('list_columns', { connId, database, table, schema })
}

// ==================== Database Navigator API ====================

/**
 * 获取数据库列表（带详细信息）
 */
export async function getDatabases(connId: string): Promise<Array<{ name: string }>> {
  return invoke('get_databases', { connId })
}

/**
 * 获取 Schema 列表
 */
export async function getSchemas(connId: string, database: string): Promise<Array<{ name: string }>> {
  return invoke('get_schemas', { connId, database })
}

/**
 * 获取表列表
 */
export async function getTables(
  connId: string,
  database: string,
  schema: string
): Promise<Array<{ name: string; type?: string }>> {
  return invoke('get_tables', { connId, database, schema })
}

/**
 * 获取视图列表
 */
export async function getViews(
  connId: string,
  database: string,
  schema: string
): Promise<Array<{ name: string; type?: string }>> {
  return invoke('get_views', { connId, database, schema })
}

/**
 * 获取列信息
 */
export async function getColumns(
  connId: string,
  database: string,
  schema: string,
  table: string
): Promise<Array<{
  name: string
  data_type: string
  nullable?: boolean
  default_value?: string
  is_primary_key?: boolean
}>> {
  return invoke('get_columns', { connId, database, schema, table })
}
