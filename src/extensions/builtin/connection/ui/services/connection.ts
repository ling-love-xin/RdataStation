/**
 * 连接服务
 *
 * 提供运行时连接的 API 调用
 */

import { invoke } from '@tauri-apps/api/core'

import type { ConnectionResponse, RecentConnectionResponse } from '../../types/connection'

/**
 * 获取所有活动连接
 */
export async function getConnections(): Promise<ConnectionResponse[]> {
  return invoke<ConnectionResponse[]>('get_connections')
}

/**
 * 连接数据库
 */
export async function connectDatabase(
  dbType: string,
  url: string,
  name?: string,
  connectionType?: 'global' | 'project',
  projectId?: string,
  opts?: {
    driverId?: string
    networkConfigId?: string | null
    environmentId?: string
    authConfigId?: string
    driverProperties?: string
    advancedOptions?: string
    description?: string
  }
): Promise<ConnectionResponse> {
  return invoke<ConnectionResponse>('connect_database', {
    input: {
      db_type: dbType,
      url,
      name,
      connection_type: connectionType || 'global',
      project_id: projectId,
      driver_id: opts?.driverId,
      network_config_id: opts?.networkConfigId || null,
      environment_id: opts?.environmentId,
      auth_config_id: opts?.authConfigId,
      driver_properties: opts?.driverProperties,
      advanced_options: opts?.advancedOptions,
      description: opts?.description,
    },
  })
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
export async function getRecentConnections(): Promise<RecentConnectionResponse[]> {
  return invoke<RecentConnectionResponse[]>('get_recent_connections')
}

/**
 * 删除最近连接记录
 */
export async function removeRecentConnection(name: string): Promise<void> {
  return invoke('remove_recent_connection', { name })
}

/**
 * 测试连接响应
 */
export interface TestConnectionResponse {
  success: boolean
  message: string
  server_version: string
  response_time_ms: number
}

/**
 * 测试连接
 */
export async function testConnection(
  dbType: string,
  url: string,
  networkConfigId?: string | null
): Promise<TestConnectionResponse> {
  return invoke<TestConnectionResponse>('test_connection', {
    dbType,
    url,
    ...(networkConfigId ? { networkConfigId } : {}),
  })
}

/**
 * 创建数据库文件响应
 */
export interface CreateDatabaseFileResponse {
  file_path: string
  success: boolean
  message: string
}

/**
 * 创建数据库文件（SQLite/DuckDB）
 */
export async function createDatabaseFile(
  dbType: string,
  filePath: string
): Promise<CreateDatabaseFileResponse> {
  return invoke<CreateDatabaseFileResponse>('create_database_file', {
    input: {
      db_type: dbType,
      file_path: filePath,
    },
  })
}

/**
 * 执行 SQL
 */
export async function executeSql(connId: string, sql: string): Promise<any> {
  return invoke('execute_sql', {
    input: {
      conn_id: connId,
      sql,
      timeout_ms: null,
    },
  })
}

/**
 * 获取项目级连接
 * 后端命令: get_project_connections
 */
export async function getProjectConnections(projectPath: string): Promise<any[]> {
  return invoke<any[]>('get_project_connections', { projectPath })
}

/**
 * 检测项目中的全局连接（发现全局连接是否与项目中的配置冲突或重叠）
 * 后端命令: detect_global_connections_in_project
 */
export async function detectGlobalConnectionsInProject(projectId: string): Promise<any[]> {
  return invoke<any[]>('detect_global_connections_in_project', { projectId })
}

/**
 * 转换连接类型（全局 ↔ 项目）
 * 后端命令: convert_connection_type
 */
export async function convertConnectionType(
  connectionId: string,
  fromType: 'global' | 'project',
  toType: 'global' | 'project',
  projectPath?: string,
  globalConnectionId?: string
): Promise<{ success: boolean; message: string }> {
  return invoke('convert_connection_type', {
    input: {
      fromType,
      toType,
      connectionId,
      projectPath: projectPath ?? null,
      globalConnectionId: globalConnectionId ?? null,
    },
  })
}

/**
 * 全局连接信息
 */
export interface GlobalConnectionInfo {
  id: string
  name: string
  driver: string
  host: string | null
  port: number | null
  database: string | null
  username: string | null
  password: string | null
  tags: string[]
  is_active: boolean
  created_at: string
  updated_at: string
}

/**
 * 获取所有全局连接
 */
export async function getGlobalConnections(): Promise<GlobalConnectionInfo[]> {
  return invoke<GlobalConnectionInfo[]>('get_global_connections')
}

/**
 * 保存导航器状态
 */
export async function saveNavigatorState(
  connectionId: string,
  expandedKeys: string[],
  selectedKeys: string[],
  filterConfig?: Record<string, unknown>
): Promise<void> {
  return invoke('save_navigator_state', {
    input: {
      connection_id: connectionId,
      expanded_keys: expandedKeys,
      selected_keys: selectedKeys,
      filter_config: filterConfig || null,
    },
  })
}

/**
 * 加载导航器状态
 */
export interface NavigatorState {
  expanded_keys: string[]
  selected_keys: string[]
  filter_config: Record<string, unknown> | null
}

export async function loadNavigatorState(connectionId: string): Promise<NavigatorState> {
  return invoke<NavigatorState>('load_navigator_state', { connectionId })
}
