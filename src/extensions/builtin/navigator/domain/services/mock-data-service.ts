/**
 * Mock 数据服务
 * 提供模拟的导航栏数据用于开发和测试
 */

import type { NavigatorNode, ConnectionInfo } from '../../types'

export class MockDataService {
  private connections: ConnectionInfo[] = [
    {
      id: 'conn_1',
      name: 'MySQL Local',
      type: 'mysql',
      host: 'localhost',
      port: 3306,
      database: 'test_db',
      status: 'connected',
      latency: 5
    },
    {
      id: 'conn_2',
      name: 'PostgreSQL Dev',
      type: 'postgresql',
      host: 'dev.db.com',
      port: 5432,
      database: 'dev_db',
      status: 'connected'
    },
    {
      id: 'conn_3',
      name: 'SQLite Test',
      type: 'sqlite',
      database: 'test.db',
      status: 'disconnected'
    }
  ]

  private mockNodes: Map<string, NavigatorNode[]> = new Map()

  constructor() {
    this.initializeMockData()
  }

  /**
   * 获取连接列表
   */
  async getConnections(): Promise<ConnectionInfo[]> {
    await this.delay(100)
    return [...this.connections]
  }

  /**
   * 生成数据库列表
   */
  generateDatabases(connectionId: string): Array<{ id: string; name: string; type: string }> {
    const connection = this.connections.find(c => c.id === connectionId)
    if (!connection) return []

    if (connection.type === 'sqlite') {
      return [{ id: `${connectionId}_main`, name: 'main', type: 'database' }]
    }

    return [
      { id: `${connectionId}_db_production`, name: 'production', type: 'database' },
      { id: `${connectionId}_db_test`, name: 'test', type: 'database' },
      { id: `${connectionId}_db_development`, name: 'development', type: 'database' }
    ]
  }

  /**
   * 生成Schema列表
   */
  generateSchemas(databaseId: string): Array<{ id: string; name: string }> {
    return [
      { id: `${databaseId}_schema_public`, name: 'public' },
      { id: `${databaseId}_schema_app`, name: 'app' },
      { id: `${databaseId}_schema_audit`, name: 'audit' }
    ]
  }

  /**
   * 生成完整的树形结构
   */
  generateTree(
    connectionId: string,
    options?: { databaseId?: string; schemaId?: string }
  ): NavigatorNode[] {
    const connection = this.connections.find(c => c.id === connectionId)
    if (!connection) return []

    // 如果是SQLite，直接返回分类文件夹
    if (connection.type === 'sqlite') {
      return this.generateCategoryNodesForConnection(connectionId)
    }

    // 获取数据库列表
    const databases = this.generateDatabases(connectionId)
    const targetDatabaseId = options?.databaseId || databases[0]?.id

    if (!targetDatabaseId) return []

    // 获取Schema列表
    const schemas = this.generateSchemas(targetDatabaseId)
    const targetSchemaId = options?.schemaId || schemas[0]?.id

    if (!targetSchemaId) {
      // 返回Schema列表
      return schemas.map(schema => ({
        id: schema.id,
        type: 'schema',
        name: schema.name,
        parentId: targetDatabaseId,
        path: `${targetDatabaseId}.${schema.name}`,
        depth: 0,
        isLeaf: false,
        childrenCount: 4,
        metadata: { comment: `Schema ${schema.name}` }
      }))
    }

    // 返回分类文件夹（表、视图等）
    return this.generateCategoryNodes(targetSchemaId)
  }

  /**
   * 为连接生成分类节点（SQLite专用）
   */
  private generateCategoryNodesForConnection(connectionId: string): NavigatorNode[] {
    return [
      {
        id: `${connectionId}_tables`,
        type: 'folder',
        name: '表',
        parentId: connectionId,
        path: `${connectionId}.tables`,
        depth: 0,
        isLeaf: false,
        childrenCount: 12
      },
      {
        id: `${connectionId}_views`,
        type: 'folder',
        name: '视图',
        parentId: connectionId,
        path: `${connectionId}.views`,
        depth: 0,
        isLeaf: false,
        childrenCount: 3
      },
      {
        id: `${connectionId}_indexes`,
        type: 'folder',
        name: '索引',
        parentId: connectionId,
        path: `${connectionId}.indexes`,
        depth: 0,
        isLeaf: false,
        childrenCount: 8
      }
    ]
  }

  /**
   * 获取根节点
   */
  async getRootNodes(connectionId: string): Promise<NavigatorNode[]> {
    await this.delay(200)
    return this.generateTree(connectionId)
  }

  /**
   * 获取子节点
   */
  async getChildren(nodeId: string): Promise<NavigatorNode[]> {
    await this.delay(150)

    // 检查缓存
    if (this.mockNodes.has(nodeId)) {
      return this.mockNodes.get(nodeId)!
    }

    // 根据节点类型生成子节点
    const children = this.generateChildren(nodeId)
    this.mockNodes.set(nodeId, children)
    return children
  }

  /**
   * 搜索节点
   */
  async searchNodes(connectionId: string, query: string): Promise<NavigatorNode[]> {
    await this.delay(100)

    const allNodes: NavigatorNode[] = []

    // 收集所有节点
    for (const [, children] of this.mockNodes) {
      for (const child of children) {
        if (child.name.toLowerCase().includes(query.toLowerCase())) {
          allNodes.push(child)
        }
      }
    }

    return allNodes
  }

  /**
   * 生成子节点
   */
  private generateChildren(parentId: string): NavigatorNode[] {
    const parent = this.findNodeById(parentId)
    if (!parent) {
      // 尝试根据ID模式推断类型
      if (parentId.includes('_tables') || parentId.endsWith('_tables')) {
        return this.generateTableNodes(parentId)
      }
      if (parentId.includes('_views') || parentId.endsWith('_views')) {
        return this.generateViewNodes(parentId)
      }
      if (parentId.includes('_functions') || parentId.endsWith('_functions')) {
        return this.generateFunctionNodes(parentId)
      }
      if (parentId.includes('_procedures') || parentId.endsWith('_procedures')) {
        return this.generateProcedureNodes(parentId)
      }
      return []
    }

    switch (parent.type) {
      case 'database':
        return this.generateSchemaNodes(parentId)
      case 'schema':
        return this.generateCategoryNodes(parentId)
      case 'table':
        return this.generateColumnNodes(parentId)
      case 'folder':
        return this.generateObjectNodesByFolder(parentId, parent.name)
      default:
        return []
    }
  }

  /**
   * 生成 Schema 节点
   */
  private generateSchemaNodes(parentId: string): NavigatorNode[] {
    return [
      {
        id: `${parentId}_schema_public`,
        type: 'schema',
        name: 'public',
        parentId,
        path: `${parentId}.public`,
        depth: 1,
        isLeaf: false,
        childrenCount: 5,
        metadata: { comment: 'Default schema' }
      },
      {
        id: `${parentId}_schema_app`,
        type: 'schema',
        name: 'app',
        parentId,
        path: `${parentId}.app`,
        depth: 1,
        isLeaf: false,
        childrenCount: 3,
        metadata: { comment: 'Application schema' }
      }
    ]
  }

  /**
   * 生成分类节点（表、视图等）
   */
  private generateCategoryNodes(parentId: string): NavigatorNode[] {
    return [
      {
        id: `${parentId}_tables`,
        type: 'folder',
        name: '表',
        parentId,
        path: `${parentId}.tables`,
        depth: 1,
        isLeaf: false,
        childrenCount: 12
      },
      {
        id: `${parentId}_views`,
        type: 'folder',
        name: '视图',
        parentId,
        path: `${parentId}.views`,
        depth: 1,
        isLeaf: false,
        childrenCount: 3
      },
      {
        id: `${parentId}_procedures`,
        type: 'folder',
        name: '存储过程',
        parentId,
        path: `${parentId}.procedures`,
        depth: 1,
        isLeaf: false,
        childrenCount: 5
      },
      {
        id: `${parentId}_functions`,
        type: 'folder',
        name: '函数',
        parentId,
        path: `${parentId}.functions`,
        depth: 1,
        isLeaf: false,
        childrenCount: 8
      }
    ]
  }

  /**
   * 根据文件夹类型生成对象节点
   */
  private generateObjectNodesByFolder(parentId: string, folderName: string): NavigatorNode[] {
    const nameMap: Record<string, () => NavigatorNode[]> = {
      '表': () => this.generateTableNodes(parentId),
      'Tables': () => this.generateTableNodes(parentId),
      '视图': () => this.generateViewNodes(parentId),
      'Views': () => this.generateViewNodes(parentId),
      '函数': () => this.generateFunctionNodes(parentId),
      'Functions': () => this.generateFunctionNodes(parentId),
      '存储过程': () => this.generateProcedureNodes(parentId),
      'Stored Procedures': () => this.generateProcedureNodes(parentId),
      '索引': () => this.generateIndexNodes(parentId),
      'Indexes': () => this.generateIndexNodes(parentId)
    }

    return nameMap[folderName]?.() || []
  }

  /**
   * 生成表节点
   */
  private generateTableNodes(parentId: string): NavigatorNode[] {
    const tables = [
      { name: 'users', rowCount: 15420, size: '2.5 MB' },
      { name: 'orders', rowCount: 89300, size: '12.8 MB' },
      { name: 'products', rowCount: 3500, size: '1.2 MB' },
      { name: 'categories', rowCount: 45, size: '64 KB' },
      { name: 'inventory', rowCount: 12000, size: '3.1 MB' },
      { name: 'customers', rowCount: 25100, size: '4.2 MB' },
      { name: 'employees', rowCount: 150, size: '128 KB' },
      { name: 'departments', rowCount: 12, size: '16 KB' },
      { name: 'salaries', rowCount: 45000, size: '2.8 MB' },
      { name: 'logs', rowCount: 1000000, size: '156 MB' },
      { name: 'sessions', rowCount: 3200, size: '512 KB' },
      { name: 'audit_trail', rowCount: 500000, size: '89 MB' }
    ]

    return tables.map((table, index) => ({
      id: `${parentId}_table_${table.name}`,
      type: 'table',
      name: table.name,
      parentId,
      path: `${parentId}.${table.name}`,
      depth: 2,
      isLeaf: false,
      childrenCount: 5,
      metadata: {
        rowCount: table.rowCount,
        size: table.size,
        engine: 'InnoDB'
      }
    }))
  }

  /**
   * 生成视图节点
   */
  private generateViewNodes(parentId: string): NavigatorNode[] {
    const views = ['user_stats', 'order_summary', 'monthly_report', 'daily_active_users']

    return views.map(name => ({
      id: `${parentId}_view_${name}`,
      type: 'view',
      name,
      parentId,
      path: `${parentId}.${name}`,
      depth: 2,
      isLeaf: true,
      metadata: {
        comment: `View: ${name}`
      }
    }))
  }

  /**
   * 生成函数节点
   */
  private generateFunctionNodes(parentId: string): NavigatorNode[] {
    const functions = [
      'fn_calculate_tax',
      'fn_format_date',
      'fn_get_user_role',
      'fn_validate_email',
      'fn_generate_uuid',
      'fn_hash_password',
      'fn_parse_json',
      'fn_truncate_string'
    ]

    return functions.map(name => ({
      id: `${parentId}_function_${name}`,
      type: 'function',
      name,
      parentId,
      path: `${parentId}.${name}`,
      depth: 2,
      isLeaf: true,
      metadata: {
        comment: `Function: ${name}`
      }
    }))
  }

  /**
   * 生成存储过程节点
   */
  private generateProcedureNodes(parentId: string): NavigatorNode[] {
    const procedures = [
      'sp_create_user',
      'sp_update_order',
      'sp_delete_product',
      'sp_backup_db',
      'sp_cleanup_logs'
    ]

    return procedures.map(name => ({
      id: `${parentId}_procedure_${name}`,
      type: 'procedure',
      name,
      parentId,
      path: `${parentId}.${name}`,
      depth: 2,
      isLeaf: true,
      metadata: {
        comment: `Procedure: ${name}`
      }
    }))
  }

  /**
   * 生成索引节点
   */
  private generateIndexNodes(parentId: string): NavigatorNode[] {
    const indexes = [
      { name: 'idx_users_email', unique: true },
      { name: 'idx_users_created_at', unique: false },
      { name: 'idx_orders_user_id', unique: false },
      { name: 'idx_orders_status', unique: false },
      { name: 'idx_products_category', unique: false },
      { name: 'idx_products_name', unique: false }
    ]

    return indexes.map(idx => ({
      id: `${parentId}_index_${idx.name}`,
      type: 'index',
      name: idx.name,
      parentId,
      path: `${parentId}.${idx.name}`,
      depth: 2,
      isLeaf: true,
      metadata: {
        isUnique: idx.unique
      }
    }))
  }

  /**
   * 生成列节点
   */
  private generateColumnNodes(parentId: string): NavigatorNode[] {
    const columns = [
      { name: 'id', type: 'int', nullable: false, isPrimaryKey: true, isAutoIncrement: true },
      { name: 'name', type: 'varchar(255)', nullable: false },
      { name: 'email', type: 'varchar(255)', nullable: false, isUnique: true },
      { name: 'status', type: 'enum', nullable: false, defaultValue: 'active' },
      { name: 'created_at', type: 'datetime', nullable: false, defaultValue: 'CURRENT_TIMESTAMP' },
      { name: 'updated_at', type: 'datetime', nullable: true }
    ]

    return columns.map(col => ({
      id: `${parentId}_col_${col.name}`,
      type: 'column',
      name: col.name,
      parentId,
      path: `${parentId}.${col.name}`,
      depth: 3,
      isLeaf: true,
      metadata: {
        dataType: col.type,
        nullable: col.nullable,
        defaultValue: col.defaultValue,
        isPrimaryKey: col.isPrimaryKey,
        isUnique: col.isUnique
      }
    }))
  }

  /**
   * 根据 ID 查找节点
   */
  private findNodeById(nodeId: string): NavigatorNode | null {
    for (const [, children] of this.mockNodes) {
      for (const child of children) {
        if (child.id === nodeId) return child
      }
    }
    return null
  }

  /**
   * 延迟函数
   */
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms))
  }

  /**
   * 初始化 Mock 数据
   */
  private initializeMockData(): void {
    console.log('[MockDataService] Initialized')
  }
}

// 创建单例
export const mockDataService = new MockDataService()
