/**
 * 基于元数据配置的导航栏服务
 * 动态生成不同数据库类型的导航结构
 */

import { DuckDBMetaConfig } from '@/shared/config/databaseMeta/duckdbMeta'
import { MySQLMetaConfig } from '@/shared/config/databaseMeta/mysqlMeta'
import { PostgresMetaConfig } from '@/shared/config/databaseMeta/postgresMeta'
import { SQLiteMetaConfig } from '@/shared/config/databaseMeta/sqliteMeta'
import type { DatabaseMetaConfig, NodeTypeConfig } from '@/shared/types/databaseMeta'

import * as metadataApi from '../../infrastructure/api/metadataApi'

import type { ConnectionInfo, NavigatorNode as NavigatorNodeType } from '../../types'

// 数据库类型到元数据配置的映射
const metaConfigMap: Record<string, DatabaseMetaConfig> = {
  mysql: MySQLMetaConfig,
  postgresql: PostgresMetaConfig,
  postgres: PostgresMetaConfig,
  sqlite: SQLiteMetaConfig,
  duckdb: DuckDBMetaConfig,
  mongodb: MySQLMetaConfig, // 暂时复用MySQL配置
  redis: MySQLMetaConfig,
  oracle: MySQLMetaConfig,
  sqlserver: MySQLMetaConfig
}

export class MetaNavigatorService {
  private useRealData = true // 默认使用真实后端数据

  /**
   * 设置是否使用真实数据
   */
  setUseRealData(useReal: boolean) {
    this.useRealData = useReal
  }

  /**
   * 获取数据库类型的元数据配置
   */
  getMetaConfig(dbType: string): DatabaseMetaConfig | null {
    return metaConfigMap[dbType] || null
  }

  /**
   * 根据连接信息生成分层导航节点
   */
  async generateConnectionTree(
    connection: ConnectionInfo,
    expandedKeys: Set<string>,
    filterTypes: string[]
  ): Promise<Array<NavigatorNodeType & { level: number; isLeaf: boolean }>> {
    console.log(`[Navigator] Generating tree for ${connection.name}, status: ${connection.status}, useRealData: ${this.useRealData}`)
    
    const metaConfig = this.getMetaConfig(connection.type)
    if (!metaConfig) {
      console.log(`[Navigator] No meta config for ${connection.type}, using default tree`)
      return this.generateDefaultTree(connection, expandedKeys, filterTypes)
    }

    // 如果使用真实数据且连接已建立，调用后端API
    if (this.useRealData && connection.status === 'connected') {
      console.log(`[Navigator] Using real data for ${connection.name}`)
      try {
        const result = await this.generateRealTree(connection, metaConfig, expandedKeys, filterTypes)
        console.log(`[Navigator] Real data result: ${result.length} nodes`)
        return result
      } catch (error) {
        console.error(`[Navigator] Failed to fetch real data for ${connection.name}:`, error)
        console.log(`[Navigator] Falling back to mock data`)
        return this.generateMetaBasedTree(connection, metaConfig, expandedKeys, filterTypes)
      }
    }

    console.log(`[Navigator] Using mock data for ${connection.name} (status: ${connection.status})`)
    // 使用基于元数据的模拟数据
    return this.generateMetaBasedTree(connection, metaConfig, expandedKeys, filterTypes)
  }

  /**
   * 从后端获取真实数据生成树
   */
  private async generateRealTree(
    connection: ConnectionInfo,
    metaConfig: DatabaseMetaConfig,
    expandedKeys: Set<string>,
    filterTypes: string[]
  ): Promise<Array<NavigatorNodeType & { level: number; isLeaf: boolean }>> {
    const result: Array<NavigatorNodeType & { level: number; isLeaf: boolean }> = []

    // 1. 连接节点
    const connectionNode = this.createConnectionNode(connection)
    result.push(connectionNode)

    if (!expandedKeys.has(connection.id)) {
      return result
    }

    // 2. 获取数据库列表
    try {
      const databases = await metadataApi.getDatabases(connection.id)
      
      for (const db of databases) {
        const dbNode: NavigatorNodeType & { level: number; isLeaf: boolean } = {
          ...db,
          level: 1,
          isLeaf: false
        }
        result.push(dbNode)

        if (expandedKeys.has(db.id)) {
          if (metaConfig.features?.supportsSchema) {
            // 3. 获取 Schema 列表（支持 Schema 的数据库如 PostgreSQL）
            try {
              const schemas = await metadataApi.getSchemas(connection.id, db.name)
              
              for (const schema of schemas) {
                const schemaNode: NavigatorNodeType & { level: number; isLeaf: boolean } = {
                  ...schema,
                  level: 2,
                  isLeaf: false
                }
                result.push(schemaNode)

                if (expandedKeys.has(schema.id)) {
                  // 4. 获取表列表
                  if (filterTypes.includes('table')) {
                    try {
                      const tables = await metadataApi.getTables(connection.id, db.name, schema.name)
                      for (const table of tables) {
                        result.push({ ...table, level: 3, isLeaf: false })
                      }
                    } catch (e) {
                      console.warn('Failed to fetch tables:', e)
                    }
                  }

                  // 5. 获取视图列表
                  if (filterTypes.includes('view')) {
                    try {
                      const views = await metadataApi.getViews(connection.id, db.name, schema.name)
                      for (const view of views) {
                        result.push({ ...view, level: 3, isLeaf: true })
                      }
                    } catch (e) {
                      console.warn('Failed to fetch views:', e)
                    }
                  }
                }
              }
            } catch (e) {
              console.warn('Failed to fetch schemas:', e)
            }
          } else {
            // 不支持 Schema 的数据库（如 SQLite、MySQL）- 直接从数据库获取表和视图
            const defaultSchema = db.name
            
            // 添加表文件夹节点
            const tablesFolderId = `${db.id}_tables`
            const tablesFolder: NavigatorNodeType & { level: number; isLeaf: boolean } = {
              id: tablesFolderId,
              type: 'table-folder',
              name: '表',
              parentId: db.id,
              path: `${db.path}.tables`,
              depth: 2,
              isLeaf: false,
              level: 2
            }
            result.push(tablesFolder)

            if (expandedKeys.has(tablesFolderId) && filterTypes.includes('table')) {
              try {
                const tables = await metadataApi.getTables(connection.id, db.name, defaultSchema)
                for (const table of tables) {
                  result.push({ ...table, level: 3, isLeaf: true })
                }
              } catch (e) {
                console.warn('Failed to fetch tables:', e)
              }
            }

            // 添加视图文件夹节点
            const viewsFolderId = `${db.id}_views`
            const viewsFolder: NavigatorNodeType & { level: number; isLeaf: boolean } = {
              id: viewsFolderId,
              type: 'view-folder',
              name: '视图',
              parentId: db.id,
              path: `${db.path}.views`,
              depth: 2,
              isLeaf: false,
              level: 2
            }
            result.push(viewsFolder)

            if (expandedKeys.has(viewsFolderId) && filterTypes.includes('view')) {
              try {
                const views = await metadataApi.getViews(connection.id, db.name, defaultSchema)
                for (const view of views) {
                  result.push({ ...view, level: 3, isLeaf: true })
                }
              } catch (e) {
                console.warn('Failed to fetch views:', e)
              }
            }
          }
        }
      }
    } catch (e) {
      console.warn('Failed to fetch databases:', e)
    }

    return result
  }

  /**
   * 基于元数据配置生成树（模拟数据）
   */
  private generateMetaBasedTree(
    connection: ConnectionInfo,
    metaConfig: DatabaseMetaConfig,
    expandedKeys: Set<string>,
    filterTypes: string[]
  ): Array<NavigatorNodeType & { level: number; isLeaf: boolean }> {
    const result: Array<NavigatorNodeType & { level: number; isLeaf: boolean }> = []

    // 1. 连接节点
    const connectionNode = this.createConnectionNode(connection)
    result.push(connectionNode)

    if (!expandedKeys.has(connection.id)) {
      return result
    }

    // 2. 根据元数据配置的层级结构生成子节点
    const connectionTypeConfig = metaConfig.nodeTypes?.find(nt => nt.id === 'connection')
    if (!connectionTypeConfig?.children) {
      return result
    }

    // 3. 生成第一层子节点
    for (const childTypeId of connectionTypeConfig.children) {
      const childNodes = this.generateNodesByType(
        connection,
        metaConfig,
        childTypeId,
        connection.id,
        1,
        expandedKeys,
        filterTypes
      )
      result.push(...childNodes)
    }

    return result
  }

  /**
   * 根据节点类型生成节点列表
   */
  private generateNodesByType(
    connection: ConnectionInfo,
    metaConfig: DatabaseMetaConfig,
    nodeTypeId: string,
    parentId: string,
    level: number,
    expandedKeys: Set<string>,
    filterTypes: string[]
  ): Array<NavigatorNodeType & { level: number; isLeaf: boolean }> {
    const result: Array<NavigatorNodeType & { level: number; isLeaf: boolean }> = []

    const nodeTypeConfig = metaConfig.nodeTypes?.find(nt => nt.id === nodeTypeId)
    if (!nodeTypeConfig) return result

    const nodeId = `${parentId}_${nodeTypeId}`

    const node: NavigatorNodeType & { level: number; isLeaf: boolean } = {
      id: nodeId,
      type: nodeTypeId,
      name: nodeTypeConfig.label,
      parentId,
      path: `${parentId}.${nodeTypeId}`,
      depth: level,
      isLeaf: !nodeTypeConfig.isContainer,
      level,
      metadata: {
        icon: nodeTypeConfig.icon,
        showCount: nodeTypeConfig.showCount
      }
    }

    result.push(node)

    if (expandedKeys.has(nodeId) && nodeTypeConfig.children) {
      for (const childTypeId of nodeTypeConfig.children) {
        if (this.shouldIncludeType(childTypeId, filterTypes)) {
          const childNodes = this.generateNodesByType(
            connection,
            metaConfig,
            childTypeId,
            nodeId,
            level + 1,
            expandedKeys,
            filterTypes
          )
          result.push(...childNodes)
        }
      }
    }

    return result
  }

  /**
   * 判断是否包含该类型
   */
  private shouldIncludeType(nodeTypeId: string, filterTypes: string[]): boolean {
    if (nodeTypeId.endsWith('-folder')) return true
    const baseType = nodeTypeId.replace('-folder', '')
    return filterTypes.includes(baseType)
  }

  /**
   * 创建连接节点
   */
  private createConnectionNode(
    connection: ConnectionInfo
  ): NavigatorNodeType & { level: number; isLeaf: boolean } {
    return {
      id: connection.id,
      type: 'connection',
      name: connection.name,
      parentId: null,
      path: connection.id,
      depth: 0,
      isLeaf: false,
      level: 0,
      metadata: {
        engine: connection.type,
        comment: connection.host ? `${connection.host}:${connection.port || ''}` : '',
        status: connection.status
      }
    }
  }

  /**
   * 生成默认树结构
   */
  private generateDefaultTree(
    connection: ConnectionInfo,
    expandedKeys: Set<string>,
    filterTypes: string[]
  ): Array<NavigatorNodeType & { level: number; isLeaf: boolean }> {
    const result: Array<NavigatorNodeType & { level: number; isLeaf: boolean }> = []

    const connectionNode = this.createConnectionNode(connection)
    result.push(connectionNode)

    if (!expandedKeys.has(connection.id)) {
      return result
    }

    const categories = [
      { id: 'tables', name: '表', type: 'table' },
      { id: 'views', name: '视图', type: 'view' },
      { id: 'functions', name: '函数', type: 'function' },
      { id: 'procedures', name: '存储过程', type: 'procedure' }
    ]

    for (const cat of categories) {
      if (!filterTypes.includes(cat.type)) continue

      const catId = `${connection.id}_${cat.id}`
      const catNode: NavigatorNodeType & { level: number; isLeaf: boolean } = {
        id: catId,
        type: 'folder',
        name: cat.name,
        parentId: connection.id,
        path: `${connection.id}.${cat.id}`,
        depth: 1,
        isLeaf: false,
        level: 1
      }
      result.push(catNode)

      if (expandedKeys.has(catId)) {
        const objects = this.generateMockObjects(catId, cat.name, connection.id, 2)
        result.push(...objects)
      }
    }

    return result
  }

  /**
   * 生成模拟对象
   */
  private generateMockObjects(
    parentId: string,
    categoryName: string,
    connectionId: string,
    level: number
  ): Array<NavigatorNodeType & { level: number; isLeaf: boolean }> {
    const objects: Array<NavigatorNodeType & { level: number; isLeaf: boolean }> = []

    const nameMap: Record<string, string[]> = {
      '表': ['users', 'orders', 'products', 'categories', 'inventory', 'customers'],
      '视图': ['user_stats', 'order_summary', 'monthly_report'],
      '函数': ['fn_calculate_tax', 'fn_format_date', 'fn_get_user_role'],
      '存储过程': ['sp_create_user', 'sp_update_order', 'sp_delete_product']
    }

    const typeMap: Record<string, string> = {
      '表': 'table',
      '视图': 'view',
      '函数': 'function',
      '存储过程': 'procedure'
    }

    const names = nameMap[categoryName] || []
    const nodeType = typeMap[categoryName] || 'folder'

    for (const name of names) {
      objects.push({
        id: `${parentId}_${nodeType}_${name}`,
        type: nodeType,
        name,
        parentId,
        path: `${parentId}.${name}`,
        depth: level,
        isLeaf: nodeType !== 'table',
        level,
        metadata: nodeType === 'table' ? {
          rowCount: Math.floor(Math.random() * 100000),
          size: `${(Math.random() * 100).toFixed(2)} MB`
        } : undefined
      })
    }

    return objects
  }

  /**
   * 获取支持的节点类型
   */
  getSupportedNodeTypes(dbType: string): string[] {
    const metaConfig = this.getMetaConfig(dbType)
    return metaConfig?.supportedNodeTypes || ['connection', 'table', 'view']
  }

  /**
   * 获取节点类型配置
   */
  getNodeTypeConfig(dbType: string, nodeTypeId: string): NodeTypeConfig | null {
    const metaConfig = this.getMetaConfig(dbType)
    return metaConfig?.nodeTypes?.find(nt => nt.id === nodeTypeId) || null
  }
}

// 创建单例
export const metaNavigatorService = new MetaNavigatorService()
