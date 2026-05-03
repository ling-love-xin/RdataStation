/**
 * 数据库树搜索功能
 * 
 * 提供搜索表/视图并定位到树节点的功能
 * 从 DatabaseNavigator.vue 中提取，实现业务逻辑与 UI 分离
 */

import type { ProjectConnection } from '@/extensions/builtin/connection/ui/types/connection'

import { useDatabaseNavigatorStore } from '../stores/database-navigator-store'
import { NodeKeyEncoder } from '../types/virtual-tree'

import type { VirtualTreeNode } from '../types/virtual-tree'

interface GlobalConnection {
  id: string
  name: string
  driver: string
  host: string | null
  port: number | null
  database: string | null
  tags: string[]
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface SearchResult {
  nodeKey: string
  tableName: string
  path: string
  connectionId: string
  dbName: string
  schemaName: string
}

interface FilterConfig {
  showTables: boolean
  showViews: boolean
  showSystemSchemas: boolean
  showColumns: boolean
}

export function useDatabaseTreeSearch() {
  const navigatorStore = useDatabaseNavigatorStore()

  /**
   * 搜索表/视图
   */
  function searchTables(
    query: string,
    filterConfig: FilterConfig,
    globalConnections: GlobalConnection[],
    projectConnections: ProjectConnection[]
  ): SearchResult[] {
    if (!query || query.trim().length === 0) return []

    const lowerQuery = query.toLowerCase()
    const results: SearchResult[] = []

    const allConnections = [...globalConnections, ...projectConnections]

    for (const conn of allConnections) {
      const databases = navigatorStore.getDatabases(conn.id)
      
      for (const db of databases) {
        if (!db.schemas) continue
        
        for (const schema of db.schemas) {
          if (!filterConfig.showSystemSchemas && 
              (schema.name === 'information_schema' || schema.name === 'pg_catalog')) {
            continue
          }

          if (filterConfig.showTables && schema.tables) {
            for (const table of schema.tables) {
              if (table.name.toLowerCase().includes(lowerQuery)) {
                const tableKey = NodeKeyEncoder.encode(['table', conn.id, db.name, schema.name, table.name])
                results.push({
                  nodeKey: tableKey,
                  tableName: table.name,
                  path: `${conn.name} / ${db.name} / ${schema.name} / ${table.name}`,
                  connectionId: conn.id,
                  dbName: db.name,
                  schemaName: schema.name
                })
              }
            }
          }

          if (filterConfig.showViews && schema.views) {
            for (const view of schema.views) {
              if (view.name.toLowerCase().includes(lowerQuery)) {
                const viewKey = NodeKeyEncoder.encode(['view', conn.id, db.name, schema.name, view.name])
                results.push({
                  nodeKey: viewKey,
                  tableName: view.name,
                  path: `${conn.name} / ${db.name} / ${schema.name} / ${view.name} (视图)`,
                  connectionId: conn.id,
                  dbName: db.name,
                  schemaName: schema.name
                })
              }
            }
          }
        }
      }
    }

    return results.slice(0, 50)
  }

  /**
   * 查找需要展开的节点路径
   */
  function findNodePath(
    connectionId: string,
    dbName: string,
    schemaName: string,
    nodes: VirtualTreeNode[]
  ): VirtualTreeNode[] {
    const path: VirtualTreeNode[] = []

    const connNode = nodes.find(n => {
      const parts = NodeKeyEncoder.decode(n.key)
      return parts[0] === 'connection' && parts[2] === connectionId
    })
    if (connNode) path.push(connNode)

    const dbNode = nodes.find(n => {
      const parts = NodeKeyEncoder.decode(n.key)
      return parts[0] === 'database' && parts[1] === connectionId && parts[2] === dbName
    })
    if (dbNode) path.push(dbNode)

    const schemaNode = nodes.find(n => {
      const parts = NodeKeyEncoder.decode(n.key)
      return parts[0] === 'schema' && parts[1] === connectionId && parts[2] === dbName && parts[3] === schemaName
    })
    if (schemaNode) path.push(schemaNode)

    return path
  }

  return {
    searchTables,
    findNodePath
  }
}
