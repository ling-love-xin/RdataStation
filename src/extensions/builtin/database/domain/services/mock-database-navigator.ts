/**
 * 数据库导航器 Mock 数据服务
 *
 * 用于前端开发和演示，提供模拟的数据库结构数据
 */

import type { NavigatorNode, NodeProperties } from '@/shared/types/databaseMeta'

// Mock 数据库连接数据
const mockConnections = [
  {
    id: 'conn-1',
    name: '本地 MySQL',
    dbType: 'mysql',
    host: 'localhost',
    port: 3306,
    databases: ['test_db', 'production_db', 'analytics_db']
  },
  {
    id: 'conn-2',
    name: 'PostgreSQL 开发环境',
    dbType: 'postgresql',
    host: 'dev-pg.example.com',
    port: 5432,
    databases: ['dev_db', 'staging_db']
  },
  {
    id: 'conn-3',
    name: 'SQLite 本地文件',
    dbType: 'sqlite',
    filePath: '/data/local.db',
    databases: ['main']
  }
]

// Mock 数据库结构
const mockDatabaseStructure: Record<string, any> = {
  'test_db': {
    tables: [
      { name: 'users', comment: '用户表', rowCount: 15420 },
      { name: 'orders', comment: '订单表', rowCount: 89321 },
      { name: 'products', comment: '产品表', rowCount: 3421 },
      { name: 'categories', comment: '分类表', rowCount: 156 },
      { name: 'inventory', comment: '库存表', rowCount: 8921 },
    ],
    views: [
      { name: 'v_active_users', comment: '活跃用户视图' },
      { name: 'v_monthly_sales', comment: '月度销售视图' },
    ],
    procedures: [
      { name: 'sp_get_user_stats', comment: '获取用户统计' },
      { name: 'sp_cleanup_old_data', comment: '清理旧数据' },
    ],
    functions: [
      { name: 'fn_calculate_discount', comment: '计算折扣' },
    ]
  },
  'production_db': {
    tables: [
      { name: 'customers', comment: '客户表', rowCount: 892341 },
      { name: 'transactions', comment: '交易表', rowCount: 4521092 },
      { name: 'audit_log', comment: '审计日志', rowCount: 89234109 },
    ],
    views: [
      { name: 'v_daily_report', comment: '日报视图' },
    ],
    procedures: [],
    functions: []
  },
  'dev_db': {
    tables: [
      { name: 'employees', comment: '员工表', rowCount: 150 },
      { name: 'departments', comment: '部门表', rowCount: 12 },
      { name: 'projects', comment: '项目表', rowCount: 45 },
    ],
    views: [],
    procedures: [],
    functions: []
  }
}

// Mock 表结构
const mockTableStructure: Record<string, any> = {
  'users': {
    columns: [
      { name: 'id', type: 'BIGINT', nullable: false, default: 'AUTO_INCREMENT', comment: '主键ID' },
      { name: 'username', type: 'VARCHAR(50)', nullable: false, default: '', comment: '用户名' },
      { name: 'email', type: 'VARCHAR(100)', nullable: false, default: '', comment: '邮箱' },
      { name: 'password_hash', type: 'VARCHAR(255)', nullable: false, default: '', comment: '密码哈希' },
      { name: 'created_at', type: 'DATETIME', nullable: false, default: 'CURRENT_TIMESTAMP', comment: '创建时间' },
      { name: 'updated_at', type: 'DATETIME', nullable: true, default: null, comment: '更新时间' },
      { name: 'status', type: 'TINYINT', nullable: false, default: '1', comment: '状态:1正常,0禁用' },
    ],
    indexes: [
      { name: 'PRIMARY', type: 'PRIMARY', columns: ['id'] },
      { name: 'idx_username', type: 'UNIQUE', columns: ['username'] },
      { name: 'idx_email', type: 'UNIQUE', columns: ['email'] },
      { name: 'idx_created_at', type: 'INDEX', columns: ['created_at'] },
    ],
    foreignKeys: [],
    triggers: []
  },
  'orders': {
    columns: [
      { name: 'id', type: 'BIGINT', nullable: false, default: 'AUTO_INCREMENT', comment: '主键ID' },
      { name: 'user_id', type: 'BIGINT', nullable: false, default: '', comment: '用户ID' },
      { name: 'order_no', type: 'VARCHAR(32)', nullable: false, default: '', comment: '订单号' },
      { name: 'total_amount', type: 'DECIMAL(10,2)', nullable: false, default: '0.00', comment: '总金额' },
      { name: 'status', type: 'TINYINT', nullable: false, default: '0', comment: '订单状态' },
      { name: 'created_at', type: 'DATETIME', nullable: false, default: 'CURRENT_TIMESTAMP', comment: '创建时间' },
    ],
    indexes: [
      { name: 'PRIMARY', type: 'PRIMARY', columns: ['id'] },
      { name: 'idx_order_no', type: 'UNIQUE', columns: ['order_no'] },
      { name: 'idx_user_id', type: 'INDEX', columns: ['user_id'] },
    ],
    foreignKeys: [
      { name: 'fk_orders_user', column: 'user_id', refTable: 'users', refColumn: 'id' },
    ],
    triggers: []
  },
  'products': {
    columns: [
      { name: 'id', type: 'BIGINT', nullable: false, default: 'AUTO_INCREMENT', comment: '主键ID' },
      { name: 'name', type: 'VARCHAR(200)', nullable: false, default: '', comment: '产品名称' },
      { name: 'price', type: 'DECIMAL(10,2)', nullable: false, default: '0.00', comment: '价格' },
      { name: 'stock', type: 'INT', nullable: false, default: '0', comment: '库存' },
      { name: 'category_id', type: 'BIGINT', nullable: false, default: '', comment: '分类ID' },
    ],
    indexes: [
      { name: 'PRIMARY', type: 'PRIMARY', columns: ['id'] },
      { name: 'idx_category', type: 'INDEX', columns: ['category_id'] },
    ],
    foreignKeys: [],
    triggers: []
  }
}

/**
 * 获取 Mock 连接列表
 */
export function getMockConnections() {
  return mockConnections.map(conn => ({
    id: conn.id,
    name: conn.name,
    dbType: conn.dbType,
    host: conn.host,
    port: conn.port,
    filePath: conn.filePath
  }))
}

/**
 * 获取 Mock 数据库列表
 */
export function getMockDatabases(connectionId: string): NavigatorNode[] {
  const conn = mockConnections.find(c => c.id === connectionId)
  if (!conn) return []

  return conn.databases.map((dbName, index) => ({
    id: `db-${connectionId}-${dbName}`,
    type: 'database',
    name: dbName,
    state: 'idle',
    expanded: false,
    connectionId,
    database: dbName,
    metadata: {
      charset: 'utf8mb4',
      collation: 'utf8mb4_unicode_ci'
    }
  }))
}

/**
 * 获取 Mock 表列表
 */
export function getMockTables(connectionId: string, database: string): NavigatorNode[] {
  const dbStructure = mockDatabaseStructure[database]
  if (!dbStructure) return []

  return dbStructure.tables.map((table: any) => ({
    id: `table-${connectionId}-${database}-${table.name}`,
    type: 'table',
    name: table.name,
    state: 'idle',
    expanded: false,
    connectionId,
    database,
    metadata: {
      comment: table.comment,
      rowCount: table.rowCount,
      tableName: table.name
    }
  }))
}

/**
 * 获取 Mock 视图列表
 */
export function getMockViews(connectionId: string, database: string): NavigatorNode[] {
  const dbStructure = mockDatabaseStructure[database]
  if (!dbStructure) return []

  return dbStructure.views.map((view: any) => ({
    id: `view-${connectionId}-${database}-${view.name}`,
    type: 'view',
    name: view.name,
    state: 'idle',
    expanded: false,
    connectionId,
    database,
    metadata: {
      comment: view.comment,
      tableName: view.name
    }
  }))
}

/**
 * 获取 Mock 存储过程列表
 */
export function getMockProcedures(connectionId: string, database: string): NavigatorNode[] {
  const dbStructure = mockDatabaseStructure[database]
  if (!dbStructure) return []

  return dbStructure.procedures.map((proc: any) => ({
    id: `procedure-${connectionId}-${database}-${proc.name}`,
    type: 'procedure',
    name: proc.name,
    state: 'idle',
    expanded: false,
    connectionId,
    database,
    metadata: {
      comment: proc.comment
    }
  }))
}

/**
 * 获取 Mock 函数列表
 */
export function getMockFunctions(connectionId: string, database: string): NavigatorNode[] {
  const dbStructure = mockDatabaseStructure[database]
  if (!dbStructure) return []

  return dbStructure.functions.map((fn: any) => ({
    id: `function-${connectionId}-${database}-${fn.name}`,
    type: 'function',
    name: fn.name,
    state: 'idle',
    expanded: false,
    connectionId,
    database,
    metadata: {
      comment: fn.comment
    }
  }))
}

/**
 * 获取 Mock 列列表
 */
export function getMockColumns(connectionId: string, database: string, tableName: string): NavigatorNode[] {
  const tableStructure = mockTableStructure[tableName]
  if (!tableStructure) return []

  return tableStructure.columns.map((col: any, index: number) => ({
    id: `column-${connectionId}-${database}-${tableName}-${col.name}`,
    type: 'column',
    name: col.name,
    state: 'idle',
    expanded: false,
    connectionId,
    database,
    metadata: {
      dataType: col.type,
      nullable: col.nullable,
      defaultValue: col.default,
      comment: col.comment,
      ordinalPosition: index + 1
    }
  }))
}

/**
 * 获取 Mock 索引列表
 */
export function getMockIndexes(connectionId: string, database: string, tableName: string): NavigatorNode[] {
  const tableStructure = mockTableStructure[tableName]
  if (!tableStructure) return []

  return tableStructure.indexes.map((idx: any) => ({
    id: `index-${connectionId}-${database}-${tableName}-${idx.name}`,
    type: 'index',
    name: idx.name,
    state: 'idle',
    expanded: false,
    connectionId,
    database,
    metadata: {
      indexType: idx.type,
      columns: idx.columns.join(', ')
    }
  }))
}

/**
 * 获取 Mock 外键列表
 */
export function getMockForeignKeys(connectionId: string, database: string, tableName: string): NavigatorNode[] {
  const tableStructure = mockTableStructure[tableName]
  if (!tableStructure || !tableStructure.foreignKeys) return []

  return tableStructure.foreignKeys.map((fk: any) => ({
    id: `fk-${connectionId}-${database}-${tableName}-${fk.name}`,
    type: 'foreignKey',
    name: fk.name,
    state: 'idle',
    expanded: false,
    connectionId,
    database,
    metadata: {
      column: fk.column,
      refTable: fk.refTable,
      refColumn: fk.refColumn
    }
  }))
}

/**
 * 获取 Mock 节点属性
 */
export function getMockNodeProperties(node: NavigatorNode): NodeProperties {
  const properties: NodeProperties = {
    general: {}
  }

  switch (node.type) {
    case 'connection': {
      const conn = mockConnections.find(c => c.id === node.connectionId)
      properties.general = {
        name: { label: '连接名称', value: node.name, type: 'text', editable: false },
        host: { label: '主机', value: conn?.host || '-', type: 'text', editable: false },
        port: { label: '端口', value: conn?.port?.toString() || '-', type: 'text', editable: false },
        type: { label: '数据库类型', value: conn?.dbType || '-', type: 'text', editable: false },
      }
      break
    }

    case 'database':
      properties.general = {
        name: { label: '数据库名', value: node.name, type: 'text', editable: false },
        charset: { label: '字符集', value: node.metadata?.charset || 'utf8mb4', type: 'text', editable: false },
        collation: { label: '排序规则', value: node.metadata?.collation || 'utf8mb4_unicode_ci', type: 'text', editable: false },
      }
      break

    case 'table':
      properties.general = {
        name: { label: '表名', value: node.name, type: 'text', editable: true },
        comment: { label: '注释', value: node.metadata?.comment || '-', type: 'text', editable: true },
        rowCount: { label: '行数', value: node.metadata?.rowCount?.toString() || '-', type: 'text', editable: false },
        engine: { label: '存储引擎', value: 'InnoDB', type: 'text', editable: false },
      }
      break

    case 'column':
      properties.general = {
        name: { label: '列名', value: node.name, type: 'text', editable: true },
        dataType: { label: '数据类型', value: node.metadata?.dataType || '-', type: 'text', editable: true },
        nullable: { label: '可空', value: node.metadata?.nullable ? '是' : '否', type: 'text', editable: true },
        defaultValue: { label: '默认值', value: node.metadata?.defaultValue || '-', type: 'text', editable: true },
        comment: { label: '注释', value: node.metadata?.comment || '-', type: 'text', editable: true },
      }
      break

    case 'index':
      properties.general = {
        name: { label: '索引名', value: node.name, type: 'text', editable: true },
        type: { label: '类型', value: node.metadata?.indexType || '-', type: 'text', editable: false },
        columns: { label: '列', value: node.metadata?.columns || '-', type: 'text', editable: false },
      }
      break
  }

  return properties
}

/**
 * 模拟延迟
 */
function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

/**
 * 加载 Mock 子节点
 */
export async function loadMockChildren(node: NavigatorNode): Promise<NavigatorNode[]> {
  // 模拟网络延迟
  await delay(300 + Math.random() * 500)

  switch (node.type) {
    case 'connection':
      return getMockDatabases(node.connectionId || '')

    case 'database':
      // 返回分类文件夹
      return [
        {
          id: `tables-folder-${node.id}`,
        type: 'tables-folder',
        name: '表',
        state: 'idle',
        expanded: false,
        connectionId: node.connectionId,
        database: node.database,
        parentId: node.id,
        metadata: { isContainer: true }
      },
      {
        id: `views-folder-${node.id}`,
        type: 'views-folder',
        name: '视图',
        state: 'idle',
        expanded: false,
        connectionId: node.connectionId,
        database: node.database,
        parentId: node.id,
        metadata: { isContainer: true }
      },
      {
        id: `procedures-folder-${node.id}`,
        type: 'procedures-folder',
        name: '存储过程',
        state: 'idle',
        expanded: false,
        connectionId: node.connectionId,
        database: node.database,
        parentId: node.id,
        metadata: { isContainer: true }
      },
      {
        id: `functions-folder-${node.id}`,
        type: 'functions-folder',
        name: '函数',
        state: 'idle',
        expanded: false,
        connectionId: node.connectionId,
        database: node.database,
        parentId: node.id,
        metadata: { isContainer: true }
      }
      ]

    case 'tables-folder':
      return getMockTables(node.connectionId || '', node.database || '')

    case 'views-folder':
      return getMockViews(node.connectionId || '', node.database || '')

    case 'procedures-folder':
      return getMockProcedures(node.connectionId || '', node.database || '')

    case 'functions-folder':
      return getMockFunctions(node.connectionId || '', node.database || '')

    case 'table':
    case 'view':
      // 返回表子对象文件夹
      return [
        {
          id: `columns-folder-${node.id}`,
        type: 'columns-folder',
        name: '列',
        state: 'idle',
        expanded: false,
        connectionId: node.connectionId,
        database: node.database,
        parentId: node.id,
        metadata: { isContainer: true }
      },
      {
        id: `indexes-folder-${node.id}`,
        type: 'indexes-folder',
        name: '索引',
        state: 'idle',
        expanded: false,
        connectionId: node.connectionId,
        database: node.database,
        parentId: node.id,
        metadata: { isContainer: true }
      },
      {
        id: `foreignkeys-folder-${node.id}`,
        type: 'foreignkeys-folder',
        name: '外键',
        state: 'idle',
        expanded: false,
        connectionId: node.connectionId,
        database: node.database,
        parentId: node.id,
        metadata: { isContainer: true }
      }
      ]

    case 'columns-folder': {
      const parentTable = node.parentId?.replace(/^(table|view)-/, '').split('-').pop() || ''
      return getMockColumns(node.connectionId || '', node.database || '', parentTable)
    }

    case 'indexes-folder': {
      const idxTable = node.parentId?.replace(/^(table|view)-/, '').split('-').pop() || ''
      return getMockIndexes(node.connectionId || '', node.database || '', idxTable)
    }

    case 'foreignkeys-folder': {
      const fkTable = node.parentId?.replace(/^(table|view)-/, '').split('-').pop() || ''
      return getMockForeignKeys(node.connectionId || '', node.database || '', fkTable)
    }

    default:
      return []
  }
}
