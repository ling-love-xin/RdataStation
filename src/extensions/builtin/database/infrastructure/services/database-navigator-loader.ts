/**
 * 数据库导航器加载服务
 *
 * 根据数据库类型使用对应的适配器加载元数据
 * 支持：MySQL, SQLite, PostgreSQL 等
 */

import { getAdapter } from '@/adapters'
import type { NavigatorNode, QueryContext } from '@/shared/types/databaseMeta'

import * as mockService from './mock-database-navigator'


/**
 * 加载节点子节点
 * 根据节点类型和数据库类型选择正确的加载方式
 */
export async function loadNodeChildren(
  node: NavigatorNode,
  dbType: string = 'mysql'
): Promise<NavigatorNode[]> {
  console.log(`[NavigatorLoader] 加载节点: ${node.type}, dbType: ${dbType}`)

  // 获取适配器
  const adapter = getAdapter(dbType)
  if (!adapter) {
    console.warn(`[NavigatorLoader] 未找到适配器: ${dbType}, 使用 Mock 数据`)
    return mockService.loadMockChildren(node)
  }

  // 特殊处理：connection 节点
  if (node.type === 'connection') {
    return loadConnectionChildren(node, dbType, adapter)
  }

  // 构建查询上下文
  const context: QueryContext = {
    connectionId: node.connectionId || '',
    database: node.database,
    schema: node.schema,
    table: (node.metadata?.tableName as string) || node.name
  }

  // 获取查询 SQL
  const query = adapter.getChildrenQuery(node.type, context)
  console.log(`[NavigatorLoader] 查询 SQL:`, query)

  // 如果没有查询（如文件夹节点），直接解析
  if (!query) {
    return adapter.parseChildrenResult(node.type, [], context)
  }

  // 执行查询（这里应该调用后端 API，暂时使用 Mock 数据模拟）
  // TODO: 替换为真实的后端查询
  try {
    // 模拟查询结果
    const mockResult = await simulateQuery(query, node.type, dbType)
    return adapter.parseChildrenResult(node.type, mockResult, context)
  } catch (e) {
    console.error(`[NavigatorLoader] 查询失败:`, e)
    return []
  }
}

/**
 * 加载连接节点的子节点
 * 根据数据库类型返回不同的结构
 */
function loadConnectionChildren(
  node: NavigatorNode,
  dbType: string,
  adapter: any
): NavigatorNode[] {
  console.log(`[NavigatorLoader] 加载连接子节点, dbType: ${dbType}`)

  const context: QueryContext = {
    connectionId: node.connectionId || ''
  }

  // 使用适配器创建对象类型文件夹
  return adapter.parseChildrenResult('connection', [], context)
}

/**
 * 模拟查询（临时方案）
 * 后续替换为真实的后端 API 调用
 */
async function simulateQuery(
  query: string,
  nodeType: string,
  dbType: string
): Promise<any[]> {
  // 模拟延迟
  await new Promise(resolve => setTimeout(resolve, 300))

  // 根据节点类型返回模拟数据
  switch (nodeType) {
    case 'table-folder':
      return simulateTables(dbType)
    case 'view-folder':
      return simulateViews(dbType)
    case 'index-folder':
      return simulateIndexes(dbType)
    case 'trigger-folder':
      return simulateTriggers(dbType)
    case 'column-folder':
      return simulateColumns(dbType)
    default:
      return []
  }
}

/**
 * 模拟表数据
 */
function simulateTables(dbType: string): any[] {
  if (dbType === 'sqlite') {
    return [
      { name: 'users', ddl: 'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)' },
      { name: 'orders', ddl: 'CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER)' },
      { name: 'products', ddl: 'CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price REAL)' }
    ]
  }
  // MySQL 等网络数据库
  return [
    { name: 'users', engine: 'InnoDB', rowCount: 1000 },
    { name: 'orders', engine: 'InnoDB', rowCount: 5000 },
    { name: 'products', engine: 'InnoDB', rowCount: 100 }
  ]
}

/**
 * 模拟视图数据
 */
function simulateViews(dbType: string): any[] {
  if (dbType === 'sqlite') {
    return [
      { name: 'v_active_users', ddl: 'CREATE VIEW v_active_users AS SELECT * FROM users WHERE active = 1' }
    ]
  }
  return [
    { name: 'v_active_users', definer: 'root@localhost', isUpdatable: 'NO' }
  ]
}

/**
 * 模拟索引数据
 */
function simulateIndexes(dbType: string): any[] {
  if (dbType === 'sqlite') {
    return [
      { name: 'idx_users_name', tableName: 'users', ddl: 'CREATE INDEX idx_users_name ON users(name)', isUnique: 0 },
      { name: 'idx_orders_user_id', tableName: 'orders', ddl: 'CREATE INDEX idx_orders_user_id ON orders(user_id)', isUnique: 0 }
    ]
  }
  return [
    { name: 'PRIMARY', isUnique: 1, type: 'BTREE', columns: 'id' },
    { name: 'idx_users_name', isUnique: 0, type: 'BTREE', columns: 'name' }
  ]
}

/**
 * 模拟触发器数据
 */
function simulateTriggers(dbType: string): any[] {
  if (dbType === 'sqlite') {
    return [
      { name: 'trg_users_updated', tableName: 'users', ddl: 'CREATE TRIGGER trg_users_updated AFTER UPDATE ON users BEGIN UPDATE users SET updated_at = datetime() WHERE id = NEW.id; END' }
    ]
  }
  return []
}

/**
 * 模拟列数据
 */
function simulateColumns(dbType: string): any[] {
  if (dbType === 'sqlite') {
    return [
      { cid: 0, name: 'id', type: 'INTEGER', notnull: 0, dflt_value: null, pk: 1 },
      { cid: 1, name: 'name', type: 'TEXT', notnull: 0, dflt_value: null, pk: 0 },
      { cid: 2, name: 'email', type: 'TEXT', notnull: 0, dflt_value: null, pk: 0 },
      { cid: 3, name: 'created_at', type: 'TEXT', notnull: 0, dflt_value: null, pk: 0 }
    ]
  }
  return [
    { name: 'id', dataType: 'int', nullable: 'NO', isPrimaryKey: 1, isAutoIncrement: 1 },
    { name: 'name', dataType: 'varchar', nullable: 'YES', isPrimaryKey: 0 },
    { name: 'email', dataType: 'varchar', nullable: 'YES', isPrimaryKey: 0 }
  ]
}
