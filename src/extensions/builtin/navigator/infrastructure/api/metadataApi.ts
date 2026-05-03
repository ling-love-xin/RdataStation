/**
 * 元数据 API
 * 调用后端 Tauri 命令获取真实数据库结构
 */

import { invoke } from '@tauri-apps/api/core'

import type { NavigatorNode } from '../../types'

// 后端响应类型
interface NavigatorNodeResponse {
  id: string
  node_type: string
  name: string
  parent_id: string | null
  path: string
  depth: number
  is_leaf: boolean
  metadata?: Record<string, unknown>
}

/**
 * 将后端响应转换为前端 NavigatorNode
 */
function convertToNavigatorNode(response: NavigatorNodeResponse): NavigatorNode {
  return {
    id: response.id,
    type: response.node_type,
    name: response.name,
    parentId: response.parent_id,
    path: response.path,
    depth: response.depth,
    isLeaf: response.is_leaf,
    metadata: response.metadata
  }
}

/**
 * 获取数据库列表
 */
export async function getDatabases(connId: string): Promise<NavigatorNode[]> {
  const response = await invoke<NavigatorNodeResponse[]>('get_databases', { conn_id: connId })
  return response.map(convertToNavigatorNode)
}

/**
 * 获取 Schema 列表
 */
export async function getSchemas(connId: string, database: string): Promise<NavigatorNode[]> {
  const response = await invoke<NavigatorNodeResponse[]>('get_schemas', { 
    conn_id: connId,
    database
  })
  return response.map(convertToNavigatorNode)
}

/**
 * 获取表列表
 */
export async function getTables(
  connId: string, 
  database: string, 
  schema: string
): Promise<NavigatorNode[]> {
  const response = await invoke<NavigatorNodeResponse[]>('get_tables', { 
    conn_id: connId,
    database,
    schema
  })
  return response.map(convertToNavigatorNode)
}

/**
 * 获取视图列表
 */
export async function getViews(
  connId: string, 
  database: string, 
  schema: string
): Promise<NavigatorNode[]> {
  const response = await invoke<NavigatorNodeResponse[]>('get_views', { 
    conn_id: connId,
    database,
    schema
  })
  return response.map(convertToNavigatorNode)
}

/**
 * 获取列列表
 */
export async function getColumns(
  connId: string, 
  database: string, 
  schema: string,
  table: string
): Promise<NavigatorNode[]> {
  const response = await invoke<NavigatorNodeResponse[]>('get_columns', { 
    conn_id: connId,
    database,
    schema,
    table
  })
  return response.map(convertToNavigatorNode)
}

/**
 * 获取表下的所有子节点（列、索引等）
 * 这是一个组合 API，用于展开表节点时获取其详细信息
 */
export async function getTableChildren(
  connId: string,
  database: string,
  schema: string,
  table: string
): Promise<NavigatorNode[]> {
  // 目前只获取列，后续可以扩展获取索引、约束等
  const columns = await getColumns(connId, database, schema, table)
  
  // 添加列文件夹节点
  const columnFolder: NavigatorNode = {
    id: `${connId}_db_${database}_schema_${schema}_table_${table}_columns`,
    type: 'column-folder',
    name: '列',
    parentId: `${connId}_db_${database}_schema_${schema}_table_${table}`,
    path: `${connId}/${database}/${schema}/${table}/columns`,
    depth: 4,
    isLeaf: false,
    metadata: { childCount: columns.length }
  }
  
  return [columnFolder, ...columns]
}
