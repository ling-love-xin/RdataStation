/**
 * 数据库树节点加载器
 *
 * 实现 DBeaver 级别的目录树结构
 * 支持动态渲染，根据数据库类型自适应结构
 */

import { useRuntimeConnectionStore } from '@/extensions/builtin/connection/ui/stores/runtime-connection-store'
import type { ProjectConnection } from '@/extensions/builtin/connection/ui/types/connection'
import type { NavigationConfig, NavigationFolderConfig } from '@/extensions/builtin/connection/ui/types/form-schema'
import { loadNavigationConfig, getDefaultNavigationConfig } from '@/extensions/builtin/connection/ui/utils/schema-loader'

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

const navConfigCache = new Map<string, NavigationConfig>()

async function getNavConfig(dbType: string): Promise<NavigationConfig> {
  const key = dbType.toLowerCase()
  if (navConfigCache.has(key)) {
    return navConfigCache.get(key)!
  }
  const config = await loadNavigationConfig(key)
  const resolved = config || getDefaultNavigationConfig()
  navConfigCache.set(key, resolved)
  return resolved
}

function isFolderEnabled(config: NavigationConfig, folderKey: string): boolean {
  const folder = config.folders[folderKey as keyof typeof config.folders] as NavigationFolderConfig | undefined
  return folder?.enabled ?? false
}

export function useDatabaseTreeLoader() {
  const navigatorStore = useDatabaseNavigatorStore()
  const runtimeConnectionStore = useRuntimeConnectionStore()

  /**
   * 创建连接节点
   */
  function createConnectionNode(
    conn: ProjectConnection | GlobalConnection,
    scope: 'global' | 'project'
  ): VirtualTreeNode {
    const databases = navigatorStore.getDatabases(conn.id)
    const hasRuntimeConn = runtimeConnectionStore.runtimeConnectionIds.has(conn.id)
    const key = NodeKeyEncoder.encode(['connection', scope, conn.id])

    return {
      key,
      level: 0,
      isExpanded: false,
      isLeaf: false,
      label: conn.name,
      type: 'connection',
      data: { connectionId: conn.id, driver: conn.driver, scope },
      parentId: null,
      childCount: databases.length,
      connectionTags: [scope === 'global' ? '全局' : '项目'],
      connectionStatus: hasRuntimeConn ? 'connected' : 'disconnected',
    }
  }

  /**
   * 创建 Catalog 节点（ANSI SQL 标准：Catalog → Schema → Table）
   */
  function createCatalogNodes(
    connectionId: string,
    scope: 'global' | 'project'
  ): VirtualTreeNode[] {
    const databases = navigatorStore.getDatabases(connectionId)
    const parentKey = NodeKeyEncoder.encode(['connection', scope, connectionId])

    return databases.map(db => ({
      key: NodeKeyEncoder.encode(['catalog', connectionId, db.name]),
      level: 1,
      isExpanded: false,
      isLeaf: false,
      label: db.name,
      type: 'catalog',
      data: { connectionId, dbName: db.name },
      parentId: parentKey,
      childCount: db.schemas?.length || 0,
    }))
  }

  /**
   * 创建 Schema 节点（PostgreSQL/DuckDB）
   */
  function createSchemaNodes(
    connectionId: string,
    dbName: string,
    config: NavigationConfig
  ): VirtualTreeNode[] {
    const schemas = navigatorStore.getDatabaseSchemas(connectionId, dbName)
    const parentKey = NodeKeyEncoder.encode(['catalog', connectionId, dbName])

    return schemas
      .filter(schema => !config.systemSchemas.includes(schema.name))
      .map(schema => ({
        key: NodeKeyEncoder.encode(['schema', connectionId, dbName, schema.name]),
        level: 2,
        isExpanded: false,
        isLeaf: false,
        label: schema.name,
        type: 'schema',
        data: { connectionId, dbName, schemaName: schema.name },
        parentId: parentKey,
        childCount: 8, // Tables/Views/Functions/Procedures/Sequences/Triggers + 系统对象
      }))
  }

  /**
   * 创建对象文件夹（Catalog/Schema 下通用，也支持直接挂 Connection 下）
   */
  function createCatalogObjectNodes(
    connectionId: string,
    dbName: string,
    config: NavigationConfig,
    parentKey?: string,
    baseLevel?: number
  ): VirtualTreeNode[] {
    const key = parentKey || NodeKeyEncoder.encode(['catalog', connectionId, dbName])
    const level = baseLevel ?? 2
    const nodes: VirtualTreeNode[] = []

    if (isFolderEnabled(config, 'tables')) {
      nodes.push({
        key: NodeKeyEncoder.encode(['tables-folder', connectionId, dbName]),
        level,
        isExpanded: false,
        isLeaf: false,
        label: config.folders.tables.label,
        type: 'tables-folder',
        data: { connectionId, dbName },
        parentId: key,
        childCount: 0,
      })
    }

    if (isFolderEnabled(config, 'views')) {
      nodes.push({
        key: NodeKeyEncoder.encode(['views-folder', connectionId, dbName]),
        level,
        isExpanded: false,
        isLeaf: false,
        label: config.folders.views.label,
        type: 'views-folder',
        data: { connectionId, dbName },
        parentId: key,
        childCount: 0,
      })
    }

    if (isFolderEnabled(config, 'functions')) {
      nodes.push({
        key: NodeKeyEncoder.encode(['functions-folder', connectionId, dbName]),
        level,
        isExpanded: false,
        isLeaf: false,
        label: config.folders.functions.label,
        type: 'functions-folder',
        data: { connectionId, dbName },
        parentId: key,
        childCount: 0,
      })
    }

    if (isFolderEnabled(config, 'procedures')) {
      nodes.push({
        key: NodeKeyEncoder.encode(['procedures-folder', connectionId, dbName]),
        level,
        isExpanded: false,
        isLeaf: false,
        label: config.folders.procedures.label,
        type: 'procedures-folder',
        data: { connectionId, dbName },
        parentId: key,
        childCount: 0,
      })
    }

    if (isFolderEnabled(config, 'triggers')) {
      nodes.push({
        key: NodeKeyEncoder.encode(['triggers-folder', connectionId, dbName]),
        level,
        isExpanded: false,
        isLeaf: false,
        label: config.folders.triggers.label,
        type: 'triggers-folder',
        data: { connectionId, dbName },
        parentId: key,
        childCount: 0,
      })
    }

    return nodes
  }

  /**
   * 创建 Schema 下的对象文件夹（DBeaver 风格）
   */
  function createSchemaObjectNodes(
    connectionId: string,
    dbName: string,
    schemaName: string,
    config: NavigationConfig
  ): VirtualTreeNode[] {
    const parentKey = NodeKeyEncoder.encode(['schema', connectionId, dbName, schemaName])
    const nodes: VirtualTreeNode[] = []

    if (isFolderEnabled(config, 'tables')) {
      nodes.push({
        key: NodeKeyEncoder.encode(['tables-folder', connectionId, dbName, schemaName]),
        level: 3,
        isExpanded: false,
        isLeaf: false,
        label: config.folders.tables.label,
        type: 'tables-folder',
        data: { connectionId, dbName, schemaName },
        parentId: parentKey,
        childCount: 0,
      })
    }

    if (isFolderEnabled(config, 'views')) {
      nodes.push({
        key: NodeKeyEncoder.encode(['views-folder', connectionId, dbName, schemaName]),
        level: 3,
        isExpanded: false,
        isLeaf: false,
        label: config.folders.views.label,
        type: 'views-folder',
        data: { connectionId, dbName, schemaName },
        parentId: parentKey,
        childCount: 0,
      })
    }

    if (isFolderEnabled(config, 'functions')) {
      nodes.push({
        key: NodeKeyEncoder.encode(['functions-folder', connectionId, dbName, schemaName]),
        level: 3,
        isExpanded: false,
        isLeaf: false,
        label: config.folders.functions.label,
        type: 'functions-folder',
        data: { connectionId, dbName, schemaName },
        parentId: parentKey,
        childCount: 0,
      })
    }

    if (isFolderEnabled(config, 'procedures')) {
      nodes.push({
        key: NodeKeyEncoder.encode(['procedures-folder', connectionId, dbName, schemaName]),
        level: 3,
        isExpanded: false,
        isLeaf: false,
        label: config.folders.procedures.label,
        type: 'procedures-folder',
        data: { connectionId, dbName, schemaName },
        parentId: parentKey,
        childCount: 0,
      })
    }

    if (isFolderEnabled(config, 'sequences')) {
      nodes.push({
        key: NodeKeyEncoder.encode(['sequences-folder', connectionId, dbName, schemaName]),
        level: 3,
        isExpanded: false,
        isLeaf: false,
        label: config.folders.sequences.label,
        type: 'sequences-folder',
        data: { connectionId, dbName, schemaName },
        parentId: parentKey,
        childCount: 0,
      })
    }

    if (isFolderEnabled(config, 'triggers')) {
      nodes.push({
        key: NodeKeyEncoder.encode(['triggers-folder', connectionId, dbName, schemaName]),
        level: 3,
        isExpanded: false,
        isLeaf: false,
        label: config.folders.triggers.label,
        type: 'triggers-folder',
        data: { connectionId, dbName, schemaName },
        parentId: parentKey,
        childCount: 0,
      })
    }

    return nodes
  }

  /**
   * 创建表节点
   */
  function createTableNodes(
    connectionId: string,
    dbName: string,
    schemaName: string | undefined,
    config: NavigationConfig,
    parentLevel?: number
  ): VirtualTreeNode[] {
    const tables = navigatorStore.getSchemaTables(connectionId, dbName, schemaName || '')
    const parentKey = schemaName
      ? NodeKeyEncoder.encode(['tables-folder', connectionId, dbName, schemaName])
      : NodeKeyEncoder.encode(['tables-folder', connectionId, dbName])
    const level = parentLevel !== undefined ? parentLevel + 1 : (schemaName ? 4 : 3)

    return tables.map(table => ({
      key: NodeKeyEncoder.encode(['table', connectionId, dbName, schemaName || '', table.name]),
      level,
      isExpanded: false,
      isLeaf: false,
      label: table.name,
      type: 'table',
      data: { connectionId, dbName, schemaName, tableName: table.name },
      parentId: parentKey,
      childCount: config.tableChildren.columns ? (table.columns?.length || 0) + 3 : 0,
    }))
  }

  /**
   * 创建视图节点
   */
  function createViewNodes(
    connectionId: string,
    dbName: string,
    schemaName: string | undefined,
    parentLevel?: number
  ): VirtualTreeNode[] {
    const views = navigatorStore.getSchemaViews(connectionId, dbName, schemaName || '')
    const parentKey = schemaName
      ? NodeKeyEncoder.encode(['views-folder', connectionId, dbName, schemaName])
      : NodeKeyEncoder.encode(['views-folder', connectionId, dbName])
    const level = parentLevel !== undefined ? parentLevel + 1 : (schemaName ? 4 : 3)

    return views.map(view => ({
      key: NodeKeyEncoder.encode(['view', connectionId, dbName, schemaName || '', view.name]),
      level,
      isExpanded: false,
      isLeaf: false,
      label: view.name,
      type: 'view',
      data: { connectionId, dbName, schemaName, viewName: view.name },
      parentId: parentKey,
      childCount: view.columns?.length || 0,
    }))
  }

  /**
   * 创建存储过程节点
   */
  function createProcedureNodes(
    connectionId: string,
    dbName: string,
    schemaName: string | undefined
  ): VirtualTreeNode[] {
    const schema = navigatorStore
      .getDatabaseSchemas(connectionId, dbName)
      .find(s => s.name === (schemaName || ''))

    if (!schema || !schema.procedures) return []

    const parentKey = schemaName
      ? NodeKeyEncoder.encode(['procedures-folder', connectionId, dbName, schemaName])
      : NodeKeyEncoder.encode(['procedures-folder', connectionId, dbName])

    return schema.procedures.map(proc => ({
      key: NodeKeyEncoder.encode(['procedure', connectionId, dbName, schemaName || '', proc.name]),
      level: schemaName ? 4 : 3,
      isExpanded: false,
      isLeaf: true,
      label: proc.name,
      type: 'procedure',
      data: { connectionId, dbName, schemaName, procedureName: proc.name },
      parentId: parentKey,
      childCount: 0,
    }))
  }

  /**
   * 创建函数节点
   */
  function createFunctionNodes(
    connectionId: string,
    dbName: string,
    schemaName: string | undefined
  ): VirtualTreeNode[] {
    const schema = navigatorStore
      .getDatabaseSchemas(connectionId, dbName)
      .find(s => s.name === (schemaName || ''))

    if (!schema || !schema.functions) return []

    const parentKey = schemaName
      ? NodeKeyEncoder.encode(['functions-folder', connectionId, dbName, schemaName])
      : NodeKeyEncoder.encode(['functions-folder', connectionId, dbName])

    return schema.functions.map(func => ({
      key: NodeKeyEncoder.encode(['function', connectionId, dbName, schemaName || '', func.name]),
      level: schemaName ? 4 : 3,
      isExpanded: false,
      isLeaf: true,
      label: func.name,
      type: 'function',
      data: { connectionId, dbName, schemaName, functionName: func.name },
      parentId: parentKey,
      childCount: 0,
    }))
  }

  /**
   * 创建表的子文件夹节点（DBeaver 风格）
   */
  function createTableSubFolderNodes(
    connectionId: string,
    dbName: string,
    schemaName: string | undefined,
    tableName: string,
    config: NavigationConfig
  ): VirtualTreeNode[] {
    const table = navigatorStore
      .getSchemaTables(connectionId, dbName, schemaName || '')
      .find(t => t.name === tableName)

    if (!table) return []

    const parentKey = NodeKeyEncoder.encode([
      'table',
      connectionId,
      dbName,
      schemaName || '',
      tableName,
    ])
    const level = schemaName ? 5 : 4
    const nodes: VirtualTreeNode[] = []

    if (config.tableChildren.columns && table.columns) {
      nodes.push({
        key: NodeKeyEncoder.encode([
          'columns-folder',
          connectionId,
          dbName,
          schemaName || '',
          tableName,
        ]),
        level,
        isExpanded: false,
        isLeaf: false,
        label: 'Columns',
        type: 'columns-folder',
        data: { connectionId, dbName, schemaName, tableName },
        parentId: parentKey,
        childCount: table.columns.length,
      })
    }

    if (config.tableChildren.indexes && table.indexes && table.indexes.length > 0) {
      nodes.push({
        key: NodeKeyEncoder.encode([
          'indexes-folder',
          connectionId,
          dbName,
          schemaName || '',
          tableName,
        ]),
        level,
        isExpanded: false,
        isLeaf: false,
        label: 'Indexes',
        type: 'indexes-folder',
        data: { connectionId, dbName, schemaName, tableName },
        parentId: parentKey,
        childCount: table.indexes.length,
      })
    }

    if (config.tableChildren.constraints && table.constraints && table.constraints.length > 0) {
      nodes.push({
        key: NodeKeyEncoder.encode([
          'constraints-folder',
          connectionId,
          dbName,
          schemaName || '',
          tableName,
        ]),
        level,
        isExpanded: false,
        isLeaf: false,
        label: 'Constraints',
        type: 'constraints-folder',
        data: { connectionId, dbName, schemaName, tableName },
        parentId: parentKey,
        childCount: table.constraints.length,
      })
    }

    return nodes
  }

  /**
   * 创建列节点
   */
  function createColumnNodes(
    connectionId: string,
    dbName: string,
    schemaName: string | undefined,
    tableName: string
  ): VirtualTreeNode[] {
    const table = navigatorStore
      .getSchemaTables(connectionId, dbName, schemaName || '')
      .find(t => t.name === tableName)

    if (!table || !table.columns) return []

    const parentKey = NodeKeyEncoder.encode([
      'columns-folder',
      connectionId,
      dbName,
      schemaName || '',
      tableName,
    ])

    return table.columns.map(col => ({
      key: NodeKeyEncoder.encode([
        'column',
        connectionId,
        dbName,
        schemaName || '',
        tableName,
        col.name,
      ]),
      level: schemaName ? 6 : 5,
      isExpanded: false,
      isLeaf: true,
      label: col.name,
      type: 'column',
      data: {
        connectionId,
        dbName,
        schemaName,
        tableName,
        columnName: col.name,
        dataType: col.dataType,
      },
      parentId: parentKey,
      childCount: 0,
    }))
  }

  /**
   * 创建索引节点
   */
  function createIndexNodes(
    connectionId: string,
    dbName: string,
    schemaName: string | undefined,
    tableName: string
  ): VirtualTreeNode[] {
    const table = navigatorStore
      .getSchemaTables(connectionId, dbName, schemaName || '')
      .find(t => t.name === tableName)

    if (!table || !table.indexes) return []

    const parentKey = NodeKeyEncoder.encode([
      'indexes-folder',
      connectionId,
      dbName,
      schemaName || '',
      tableName,
    ])

    return table.indexes.map(idx => ({
      key: NodeKeyEncoder.encode([
        'index',
        connectionId,
        dbName,
        schemaName || '',
        tableName,
        idx.name,
      ]),
      level: schemaName ? 6 : 5,
      isExpanded: false,
      isLeaf: true,
      label: idx.name,
      type: 'index',
      data: {
        connectionId,
        dbName,
        schemaName,
        tableName,
        indexName: idx.name,
        isUnique: idx.isUnique,
        isPrimary: idx.isPrimary,
      },
      parentId: parentKey,
      childCount: 0,
    }))
  }

  /**
   * 创建约束节点
   */
  function createConstraintNodes(
    connectionId: string,
    dbName: string,
    schemaName: string | undefined,
    tableName: string
  ): VirtualTreeNode[] {
    const table = navigatorStore
      .getSchemaTables(connectionId, dbName, schemaName || '')
      .find(t => t.name === tableName)

    if (!table || !table.constraints) return []

    const parentKey = NodeKeyEncoder.encode([
      'constraints-folder',
      connectionId,
      dbName,
      schemaName || '',
      tableName,
    ])

    return table.constraints.map(con => ({
      key: NodeKeyEncoder.encode([
        'constraint',
        connectionId,
        dbName,
        schemaName || '',
        tableName,
        con.name,
      ]),
      level: schemaName ? 6 : 5,
      isExpanded: false,
      isLeaf: true,
      label: con.name,
      type: 'constraint',
      data: {
        connectionId,
        dbName,
        schemaName,
        tableName,
        constraintName: con.name,
        constraintType: con.type,
      },
      parentId: parentKey,
      childCount: 0,
    }))
  }

  /**
   * 加载子节点 - 核心加载逻辑（DBeaver 风格）
   */
  async function loadChildren(node: VirtualTreeNode): Promise<VirtualTreeNode[]> {
    const keyParts = NodeKeyEncoder.decode(node.key)
    if (keyParts.length === 0) return []

    const nodeType = keyParts[0]
    const connectionId = keyParts[1]
    const dbType = node.data.driver || navigatorStore.getDbType(connectionId) || ''
    const config = await getNavConfig(dbType)

    try {
      // Level 0: 连接节点 -> Catalog 列表 或 直接对象文件夹
      if (nodeType === 'connection') {
        const scope = keyParts[1] as 'global' | 'project'
        const connId = keyParts[2]

        const hasRuntimeConn = runtimeConnectionStore.runtimeConnectionIds.has(connId)

        if (hasRuntimeConn) {
          await navigatorStore.loadDatabases(connId)
        }

        if (config.hasCatalogs) {
          return createCatalogNodes(connId, scope)
        }

        const databases = navigatorStore.getDatabases(connId)
        const defaultDbName = databases[0]?.name || 'main'
        const connKey = NodeKeyEncoder.encode(['connection', scope, connId])
        return createCatalogObjectNodes(connId, defaultDbName, config, connKey, 1)
      }

      // Level 1: Catalog 节点 → Schema 或对象文件夹
      if (nodeType === 'catalog') {
        const dbName = keyParts[2]

        if (config.hasSchemas) {
          await navigatorStore.loadSchemas(connectionId, dbName)
          return createSchemaNodes(connectionId, dbName, config)
        } else {
          return createCatalogObjectNodes(connectionId, dbName, config)
        }
      }

      // Level 2: Schema 节点 -> 对象文件夹
      if (nodeType === 'schema') {
        const dbName = keyParts[2]
        const schemaName = keyParts[3]

        await Promise.all([
          navigatorStore.loadTables(connectionId, dbName, schemaName),
          navigatorStore.loadViews(connectionId, dbName, schemaName),
        ])

        return createSchemaObjectNodes(connectionId, dbName, schemaName, config)
      }

      // Level 2/3: Tables 文件夹 -> 表列表
      if (nodeType === 'tables-folder') {
        const dbName = keyParts[2]
        const schemaName = keyParts[3] || undefined

        if (schemaName) {
          await navigatorStore.loadTables(connectionId, dbName, schemaName)
        } else {
          // MySQL 等无 Schema 的数据库：用 dbName 代替 schemaName
          await navigatorStore.loadTables(connectionId, dbName, dbName)
        }

        return createTableNodes(connectionId, dbName, schemaName, config, node.level)

      }

      // Level 2/3: Views 文件夹 -> 视图列表
      if (nodeType === 'views-folder') {
        const dbName = keyParts[2]
        const schemaName = keyParts[3] || undefined

        if (schemaName) {
          await navigatorStore.loadViews(connectionId, dbName, schemaName)
        }

        return createViewNodes(connectionId, dbName, schemaName, node.level)
      }

      // Level 2/3: Procedures 文件夹 -> 存储过程列表
      if (nodeType === 'procedures-folder') {
        const dbName = keyParts[2]
        const schemaName = keyParts[3] || undefined

        if (schemaName) {
          await navigatorStore.loadProcedures(connectionId, dbName, schemaName)
        }

        return createProcedureNodes(connectionId, dbName, schemaName)
      }

      // Level 2/3: Functions 文件夹 -> 函数列表
      if (nodeType === 'functions-folder') {
        const dbName = keyParts[2]
        const schemaName = keyParts[3] || undefined

        if (schemaName) {
          await navigatorStore.loadFunctions(connectionId, dbName, schemaName)
        }

        return createFunctionNodes(connectionId, dbName, schemaName)
      }

      // Level 3/4: 表节点 -> Columns/Indexes/Constraints 文件夹
      if (nodeType === 'table') {
        const dbName = keyParts[2]
        const schemaName = keyParts[3] || undefined
        const tableName = keyParts[4]

        await navigatorStore.loadColumns(connectionId, dbName, schemaName || '', tableName)

        return createTableSubFolderNodes(connectionId, dbName, schemaName, tableName, config)
      }

      // Level 3/4: 视图节点 -> 列列表
      if (nodeType === 'view') {
        const dbName = keyParts[2]
        const schemaName = keyParts[3] || undefined
        const viewName = keyParts[4]
        return createColumnNodes(connectionId, dbName, schemaName, viewName)
      }

      // Level 4/5: Columns 文件夹 -> 列列表
      if (nodeType === 'columns-folder') {
        const dbName = keyParts[2]
        const schemaName = keyParts[3] || undefined
        const tableName = keyParts[4]
        return createColumnNodes(connectionId, dbName, schemaName, tableName)
      }

      // Level 4/5: Indexes 文件夹 -> 索引列表
      if (nodeType === 'indexes-folder') {
        const dbName = keyParts[2]
        const schemaName = keyParts[3] || undefined
        const tableName = keyParts[4]
        return createIndexNodes(connectionId, dbName, schemaName, tableName)
      }

      // Level 4/5: Constraints 文件夹 -> 约束列表
      if (nodeType === 'constraints-folder') {
        const dbName = keyParts[2]
        const schemaName = keyParts[3] || undefined
        const tableName = keyParts[4]
        return createConstraintNodes(connectionId, dbName, schemaName, tableName)
      }
    } catch (error) {
      console.error('加载树节点失败:', error)
    }

    return []
  }

  /**
   * 创建根节点列表
   */
  function createRootNodes(
    globalConnections: GlobalConnection[],
    projectConnections: ProjectConnection[]
  ): VirtualTreeNode[] {
    const rootNodes: VirtualTreeNode[] = []

    globalConnections.forEach(conn => {
      rootNodes.push(createConnectionNode(conn, 'global'))
    })

    projectConnections.forEach(conn => {
      rootNodes.push(createConnectionNode(conn, 'project'))
    })

    return rootNodes
  }

  return {
    createConnectionNode,
    createCatalogNodes,
    createSchemaNodes,
    createCatalogObjectNodes,
    createSchemaObjectNodes,
    createTableNodes,
    createViewNodes,
    createTableSubFolderNodes,
    createColumnNodes,
    createIndexNodes,
    createConstraintNodes,
    loadChildren,
    createRootNodes,
  }
}
