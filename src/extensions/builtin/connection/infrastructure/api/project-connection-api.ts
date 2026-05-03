/**
 * 项目级连接服务
 *
 * 提供项目级数据库连接的 CRUD 操作
 * 所有连接信息都存储在项目目录下的 SQLite 数据库中
 */

import { invoke } from '@tauri-apps/api/core'

import type { ProjectConnection, CreateProjectConnectionInput } from '../types/connection'

/**
 * 创建项目连接
 */
export async function createProjectConnection(
  input: CreateProjectConnectionInput
): Promise<ProjectConnection> {
  return await invoke<ProjectConnection>('create_project_connection', { input })
}

/**
 * 获取项目所有连接
 */
export async function getProjectConnections(projectPath: string): Promise<ProjectConnection[]> {
  return await invoke<ProjectConnection[]>('get_project_connections', { projectPath })
}

/**
 * 获取单个项目连接
 */
export async function getProjectConnection(
  projectPath: string,
  connectionId: string
): Promise<ProjectConnection | null> {
  return await invoke<ProjectConnection | null>('get_project_connection', {
    projectPath,
    connectionId,
  })
}

/**
 * 更新项目连接
 */
export async function updateProjectConnection(
  projectPath: string,
  connection: ProjectConnection
): Promise<void> {
  return await invoke('update_project_connection', {
    projectPath,
    connection,
  })
}

/**
 * 删除项目连接
 */
export async function deleteProjectConnection(
  projectPath: string,
  connectionId: string
): Promise<void> {
  return await invoke('delete_project_connection', {
    projectPath,
    connectionId,
  })
}

/**
 * 搜索项目连接
 */
export async function searchProjectConnections(
  projectPath: string,
  query: string
): Promise<ProjectConnection[]> {
  return await invoke<ProjectConnection[]>('search_project_connections', {
    projectPath,
    query,
  })
}

/**
 * 关闭项目连接存储
 */
export async function closeProjectConnectionStore(projectPath: string): Promise<void> {
  // 现在由 ProjectStore 统一管理，无需单独关闭
  return await invoke('close_project_store', { projectPath })
}

/**
 * 构建连接 URL
 */
export function buildConnectionUrl(conn: ProjectConnection): string {
  const { driver, host, port, database, username, password } = conn

  // 文件型数据库
  if (driver === 'sqlite' || driver === 'duckdb') {
    return `${driver}:///${host || ''}`
  }

  // 网络型数据库
  const defaultPort = driver === 'mysql' ? 3306 : driver === 'postgres' ? 5432 : port
  const actualPort = port || defaultPort

  const auth = username
    ? password
      ? `${encodeURIComponent(username)}:${encodeURIComponent(password)}@`
      : `${encodeURIComponent(username)}@`
    : ''

  return `${driver}://${auth}${host || 'localhost'}:${actualPort}/${encodeURIComponent(database || '')}`
}

/**
 * 从连接配置生成显示名称
 */
export function getConnectionDisplayName(conn: ProjectConnection): string {
  if (conn.name) return conn.name

  if (conn.driver === 'sqlite' || conn.driver === 'duckdb') {
    const fileName = (conn.host || '').split(/[/\\]/).pop() || conn.host || ''
    return `${conn.driver.toUpperCase()}: ${fileName}`
  }

  return `${conn.driver.toUpperCase()}: ${conn.host || 'localhost'}:${conn.port}/${conn.database || ''}`
}

/**
 * 获取数据库类型显示标签
 */
export function getDbTypeLabel(dbType: string): string {
  const labels: Record<string, string> = {
    mysql: 'MySQL',
    postgres: 'PostgreSQL',
    sqlite: 'SQLite',
    duckdb: 'DuckDB',
  }
  return labels[dbType] || dbType.toUpperCase()
}

/**
 * 获取数据库类型图标颜色
 */
export function getDbTypeColor(dbType: string): string {
  const colors: Record<string, string> = {
    mysql: '#00758f',
    postgres: '#336791',
    sqlite: '#003b57',
    duckdb: '#ffbc00',
  }
  return colors[dbType] || '#666666'
}
