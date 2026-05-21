/**
 * 项目连接服务
 *
 * 提供项目级连接配置的 API 调用
 * 映射到后端 project_commands.rs 中的 save_project_store_connection / get_project_store_connections 等命令
 */

import { invoke } from '@tauri-apps/api/core'

import type {
  ProjectConnection,
  StoredConnection,
  CreateProjectConnectionInput,
  ConnectionStatus,
} from '../../types/connection'

/**
 * 获取项目所有连接
 * 后端命令: get_project_store_connections
 */
export async function getProjectConnections(_projectPath: string): Promise<ProjectConnection[]> {
  // 后端 get_project_store_connections 需要 ProjectState，由 init_project_store 保证
  return invoke<ProjectConnection[]>('get_project_store_connections')
}

/**
 * 创建项目连接
 * 后端命令: save_project_store_connection
 *
 * 注意：这仅将连接配置保存到 project.db，不会创建运行时连接。
 * 需要单独调用 connect_database 来建立实际连接。
 */
export async function createProjectConnection(
  input: CreateProjectConnectionInput
): Promise<ProjectConnection> {
  const now = new Date().toISOString()
  const storedConn: Omit<StoredConnection, 'id' | 'created_at' | 'updated_at'> & { id?: string; created_at?: string; updated_at?: string } = {
    id: '', // 后端会生成 UUID
    name: input.name,
    driver: input.driver,
    host: input.host ?? null,
    port: input.port ?? null,
    database: input.database ?? null,
    username: input.username ?? null,
    password_encrypted: input.password ?? null,
    options: input.options ?? null,
    tags: input.tags ?? null,
    schema_name: null,
    use_duckdb_fed: input.use_duckdb_fed ?? false,
    metadata_path: null,
    is_active: false,
    created_at: now,
    updated_at: now,
  }
  await invoke('save_project_store_connection', { connection: storedConn })
  const conns = await invoke<ProjectConnection[]>('get_project_store_connections')
  return conns[conns.length - 1]
}

/**
 * 更新项目连接
 * 后端命令: save_project_store_connection (upsert)
 */
export async function updateProjectConnection(
  connection: ProjectConnection
): Promise<void> {
  const now = new Date().toISOString()
  const storedConn: StoredConnection = {
    id: connection.id,
    name: connection.name,
    driver: connection.driver,
    host: connection.host ?? null,
    port: connection.port ?? null,
    database: connection.database ?? null,
    username: connection.username ?? null,
    password_encrypted: connection.password ?? null,
    options: connection.options ?? null,
    tags: connection.tags ?? null,
    schema_name: null,
    use_duckdb_fed: false,
    metadata_path: null,
    is_active: connection.is_active ?? false,
    created_at: connection.created_at,
    updated_at: now,
  }
  await invoke('save_project_store_connection', { connection: storedConn })
}

/**
 * 更新项目连接状态
 *
 * 使用 get_project_store_connections 获取当前连接列表，
 * 查找目标连接后通过 save_project_store_connection upsert 更新 is_active 字段。
 */
export async function updateProjectConnectionStatus(
  _projectPath: string,
  connectionId: string,
  status: ConnectionStatus,
  _errorMessage?: string
): Promise<void> {
  const conns = await invoke<ProjectConnection[]>('get_project_store_connections')
  const conn = conns.find(c => c.id === connectionId)
  if (!conn) throw new Error(`Connection ${connectionId} not found`)

  const now = new Date().toISOString()
  const storedConn: StoredConnection = {
    id: conn.id,
    name: conn.name,
    driver: conn.driver,
    host: conn.host ?? null,
    port: conn.port ?? null,
    database: conn.database ?? null,
    username: conn.username ?? null,
    password_encrypted: conn.password ?? null,
    options: conn.options ?? null,
    tags: conn.tags ?? null,
    schema_name: null,
    use_duckdb_fed: false,
    metadata_path: null,
    is_active: status === 'connected',
    created_at: conn.created_at,
    updated_at: now,
  }
  await invoke('save_project_store_connection', { connection: storedConn })
}

/**
 * 删除项目连接
 * 后端命令: delete_project_store_connection
 */
export async function deleteProjectConnection(
  connectionId: string
): Promise<void> {
  return invoke('delete_project_store_connection', { id: connectionId })
}

/**
 * 搜索项目连接
 *
 * 设计选择：当前在前端本地过滤，避免后端增加搜索命令的复杂度。
 * 连接配置数据量通常较小（数十条），前端过滤性能完全可接受。
 */
export async function searchProjectConnections(
  projectPath: string,
  query: string,
  limit?: number,
  offset?: number
): Promise<ProjectConnection[]> {
  const all = await invoke<ProjectConnection[]>('get_project_store_connections')
  let results: ProjectConnection[]
  if (!query.trim()) {
    results = all
  } else {
    const q = query.toLowerCase()
    results = all.filter(c =>
      c.name.toLowerCase().includes(q) ||
      c.driver.toLowerCase().includes(q) ||
      (c.host && c.host.toLowerCase().includes(q)) ||
      (c.database && c.database.toLowerCase().includes(q))
    )
  }
  // Apply pagination
  const start = offset ?? 0
  const end = limit != null ? start + limit : undefined
  return results.slice(start, end)
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