/**
 * Tauri API 封装
 *
 * 统一封装 Tauri invoke 调用
 * 提供类型安全的 API 接口和错误处理
 */

import { invoke } from '@tauri-apps/api/core'

import type { SchemaObject } from '@/shared/types'
import type { DatabaseType } from '@/shared/types/sql'

/**
 * 统一调用 Tauri 命令
 */
export async function tauriInvoke<T = unknown>(
  command: string,
  args: Record<string, unknown> = {}
): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    console.error(`[TauriInvoke] Command "${command}" failed:`, error)
    throw error
  }
}

// ==================== 连接类型定义 ====================

export interface DataSourceMeta {
  supports_transaction: boolean
  supports_streaming: boolean
  supports_arrow: boolean
  supports_federated: boolean
  supports_concurrent_write: boolean
  is_in_memory: boolean
}

export interface ConnectDatabaseInput {
  db_type: DatabaseType
  url: string
  name?: string
  hosts?: Array<{
    host: string
    port?: number
    priority?: number
    role?: 'primary' | 'replica'
  }>
  connection_type?: string
  project_id?: string
}

export interface ConnectDatabaseResponse {
  conn_id: string
  name: string
  db_type: DatabaseType
  url: string
  connection_type: string
  project_id: string | null
  status: string
  meta: DataSourceMeta
}

export interface ConnectionInfoResponse {
  id: string
  name: string
  db_type: DatabaseType
  url: string
  connection_type: string
  project_id: string | null
  status: string
  is_active: boolean
  created_at_ms: number
}

// ==================== 连接相关 API ====================

export const connectionApi = {
  connectDatabase(input: ConnectDatabaseInput) {
    return tauriInvoke<ConnectDatabaseResponse>('connect_database', { input })
  },

  disconnectDatabase(connId: string) {
    return tauriInvoke<void>('disconnect_database', { conn_id: connId })
  },

  getConnections() {
    return tauriInvoke<ConnectionInfoResponse[]>('get_connections')
  },

  getConnectionInfo(connId: string) {
    return tauriInvoke<ConnectionInfoResponse>('get_connection_info', { conn_id: connId })
  },

  testConnection(dbType: string, url: string) {
    return tauriInvoke<boolean>('test_connection', { db_type: dbType, url })
  },
}

// ==================== SQL 执行类型定义 ====================

/** @deprecated QueryResult.columns 已改为 string[]，此类型保留向后兼容 */
export interface QueryColumn {
  name: string
  data_type: string
}

export interface QueryResult {
  columns: string[]
  rows: unknown[]
  total_rows: number
  affected_rows?: number
  is_read_only?: boolean
}

export interface ExecuteSqlInput {
  conn_id?: string
  sql: string
  timeout_ms?: number
}

export interface ExecuteSqlResponse {
  result: QueryResult
  elapsed_ms: number
  affected_rows?: number
  truncated: boolean
  error?: string
}

export interface SqlHistoryResponse {
  id: string
  sql: string
  conn_id: string | null
  executed_at: string
}

// ==================== SQL 执行相关 API ====================

export const sqlApi = {
  executeSql(input: ExecuteSqlInput) {
    return tauriInvoke<ExecuteSqlResponse>('execute_sql', { input })
  },

  executeTransaction(connId: string | null, sqls: string[]) {
    return tauriInvoke<{ results: ExecuteSqlResponse[] }>('execute_transaction', {
      input: { conn_id: connId, sqls },
    })
  },

  getSqlHistory(limit?: number) {
    return tauriInvoke<SqlHistoryResponse[]>('get_sql_history', { limit })
  },

  searchSqlHistory(keyword: string, limit?: number) {
    return tauriInvoke<SqlHistoryResponse[]>('search_sql_history', { keyword, limit })
  },

  clearSqlHistory() {
    return tauriInvoke<void>('clear_sql_history')
  },

  removeSqlHistory(id: string) {
    return tauriInvoke<void>('remove_sql_history', { id })
  },
}

// ==================== 导航器类型定义 ====================

export interface NavigatorNodeResponse {
  id: string
  node_type: string
  name: string
  parent_id: string | null
  path: string
  depth: number
  is_leaf: boolean
  metadata?: Record<string, unknown>
}

// ==================== 导航器相关 API ====================

export const navigatorApi = {
  getCatalogs(connId: string) {
    return tauriInvoke<NavigatorNodeResponse[]>('get_catalogs', { conn_id: connId })
  },

  getSchemas(connId: string, catalog: string) {
    return tauriInvoke<NavigatorNodeResponse[]>('get_schemas', {
      conn_id: connId,
      database: catalog,
    })
  },

  getTables(connId: string, catalog: string, schema: string) {
    return tauriInvoke<NavigatorNodeResponse[]>('get_tables', {
      conn_id: connId,
      database: catalog,
      schema,
    })
  },

  getViews(connId: string, catalog: string, schema: string) {
    return tauriInvoke<NavigatorNodeResponse[]>('get_views', {
      conn_id: connId,
      database: catalog,
      schema,
    })
  },

  getColumns(connId: string, catalog: string, schema: string, table: string) {
    return tauriInvoke<NavigatorNodeResponse[]>('get_columns', {
      conn_id: connId,
      database: catalog,
      schema,
      table,
    })
  },

  // 旧版兼容 API
  listCatalogs(connId: string) {
    return tauriInvoke<string[]>('list_catalogs', { conn_id: connId })
  },

  listSchemas(connId: string, catalog: string) {
    return tauriInvoke<string[]>('list_schemas', { conn_id: connId, database: catalog })
  },

  listTables(connId: string, catalog: string, schema?: string) {
    return tauriInvoke<SchemaObject[]>('list_tables', {
      conn_id: connId,
      database: catalog,
      schema,
    })
  },

  listColumns(connId: string, catalog: string, schema: string | null, table: string) {
    return tauriInvoke<SchemaObject[]>('list_columns', {
      conn_id: connId,
      database: catalog,
      schema,
      table,
    })
  },
}

// ==================== 联邦查询相关 API ====================

export const federatedApi = {
  registerExternalDatabase(
    connId: string | null,
    name: string,
    driver: string,
    connectionString: string
  ) {
    return tauriInvoke<void>('register_external_database', {
      input: {
        conn_id: connId,
        name,
        driver,
        connection_string: connectionString,
      },
    })
  },

  createExternalTable(
    connId: string | null,
    externalDbName: string,
    schemaName: string,
    tableName: string,
    externalTableName: string
  ) {
    return tauriInvoke<void>('create_external_table', {
      input: {
        conn_id: connId,
        external_db_name: externalDbName,
        schema_name: schemaName,
        table_name: tableName,
        external_table_name: externalTableName,
      },
    })
  },
}

// ==================== 项目相关 API ====================

/** 与 workbench/ui/services/project.ts 中 ProjectInfo 保持一致的共享类型 */
export interface ProjectInfo {
  id: string
  name: string
  description?: string
  path: {
    type: 'Local' | 'Remote'
    path?: string
    url?: string
    project_id?: string
  }
  status: string
  created_at: string
  updated_at: string
  last_opened_at?: string
  version: string
}

export interface CreateProjectInput {
  name: string
  path: string
  description?: string
}

export interface UpdateProjectInput {
  id: string
  name?: string
  description?: string
}

/** @deprecated 使用 ProjectInfo 替代 */
export type ProjectResponse = ProjectInfo

export const projectApi = {
  createProject(input: CreateProjectInput) {
    return tauriInvoke<ProjectInfo>('create_and_save_project', { input })
  },

  openProjectByPath(path: string) {
    return tauriInvoke<ProjectInfo>('open_project_by_path', { path })
  },

  openProjectById(id: string) {
    return tauriInvoke<ProjectInfo>('open_project_by_id', { id })
  },

  getRecentProjects(limit: number = 10) {
    return tauriInvoke<ProjectInfo[]>('get_recent_projects', { limit })
  },

  removeFromRecent(projectId: string) {
    return tauriInvoke<ProjectInfo>('remove_from_recent', { projectId })
  },

  deleteProject(projectId: string) {
    return tauriInvoke<void>('delete_project', { projectId })
  },

  deleteProjectDisk(projectId: string) {
    return tauriInvoke<void>('delete_project_disk', { projectId })
  },

  updateProject(input: UpdateProjectInput) {
    return tauriInvoke<void>('update_project', { input })
  },
}
