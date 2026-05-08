/**
 * 项目连接服务
 *
 * 提供项目级连接配置的 API 调用
 */

import { invoke } from '@tauri-apps/api/core'

import type {
  ProjectConnection,
  CreateProjectConnectionInput,
  ConnectionStatus,
} from '../../types/connection'

/**
 * 获取项目所有连接
 */
export async function getProjectConnections(projectPath: string): Promise<ProjectConnection[]> {
  return invoke<ProjectConnection[]>('get_project_connections', { projectPath })
}

/**
 * 创建项目连接
 */
export async function createProjectConnection(
  input: CreateProjectConnectionInput
): Promise<ProjectConnection> {
  return invoke<ProjectConnection>('create_project_connection', { input })
}

/**
 * 更新项目连接
 */
export async function updateProjectConnection(
  projectPath: string,
  connection: ProjectConnection
): Promise<void> {
  return invoke('update_project_connection', { projectPath, connection })
}

/**
 * 更新项目连接状态
 */
export async function updateProjectConnectionStatus(
  projectPath: string,
  connectionId: string,
  status: ConnectionStatus,
  errorMessage?: string
): Promise<void> {
  return invoke('update_project_connection_status', {
    projectPath,
    connectionId,
    status,
    errorMessage,
  })
}

/**
 * 删除项目连接
 */
export async function deleteProjectConnection(
  projectPath: string,
  connectionId: string
): Promise<void> {
  return invoke('delete_project_connection', { projectPath, connectionId })
}

/**
 * 搜索项目连接
 */
export async function searchProjectConnections(
  projectPath: string,
  query: string
): Promise<ProjectConnection[]> {
  return invoke<ProjectConnection[]>('search_project_connections', { projectPath, query })
}

/**
 * 构建连接 URL
 */
export function buildConnectionUrl(connection: ProjectConnection): string {
  const { driver, host, port, database, username, password } = connection

  switch (driver?.toLowerCase()) {
    case 'mysql':
      if (username && password) {
        return `mysql://${username}:${password}@${host || 'localhost'}:${port || 3306}/${database || ''}`
      }
      return `mysql://${host || 'localhost'}:${port || 3306}/${database || ''}`

    case 'postgresql':
    case 'postgres':
      if (username && password) {
        return `postgresql://${username}:${password}@${host || 'localhost'}:${port || 5432}/${database || ''}`
      }
      return `postgresql://${host || 'localhost'}:${port || 5432}/${database || ''}`

    case 'sqlite':
      return `sqlite://${host || ''}`

    case 'duckdb':
      return `duckdb://${host || ''}`

    default:
      throw new Error(`不支持的数据库类型: ${driver || 'unknown'}`)
  }
}

/**
 * 获取连接显示名称
 */
export function getConnectionDisplayName(connection: ProjectConnection): string {
  if (connection.name) {
    return connection.name
  }

  const { driver, host, port, database } = connection

  if (database) {
    return `${driver} - ${database}@${host || 'localhost'}`
  }

  if (port) {
    return `${driver} - ${host || 'localhost'}:${port}`
  }

  return `${driver} - ${host || 'localhost'}`
}
