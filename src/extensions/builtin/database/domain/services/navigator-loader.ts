/**
 * 数据库导航器加载服务
 *
 * 根据数据库类型使用对应的适配器加载元数据
 * 支持：MySQL, SQLite, PostgreSQL 等
 */

import { invoke } from '@tauri-apps/api/core'

import { getAdapter } from '@/adapters'
import type { DatabaseMetaAdapter } from '@/adapters'
import type { NavigatorNode, QueryContext } from '@/shared/types/databaseMeta'

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
    console.warn(`[NavigatorLoader] 未找到适配器: ${dbType}`)
    return []
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
    table: (node.metadata?.tableName as string) || node.name,
  }

  // 获取查询 SQL
  const query = adapter.getChildrenQuery(node.type, context)
  console.log(`[NavigatorLoader] 查询 SQL:`, query)

  // 如果没有查询（如文件夹节点），直接解析
  if (!query) {
    return adapter.parseChildrenResult(node.type, [], context)
  }

  // 执行查询（调用后端 API）
  try {
    const result = await executeQuery(query, context)
    return adapter.parseChildrenResult(node.type, result, context)
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
  adapter: DatabaseMetaAdapter
): NavigatorNode[] {
  console.log(`[NavigatorLoader] 加载连接子节点, dbType: ${dbType}`)

  const context: QueryContext = {
    connectionId: node.connectionId || '',
  }

  // 使用适配器创建对象类型文件夹
  return adapter.parseChildrenResult('connection', [], context)
}

/**
 * 执行 SQL 查询
 * 通过 Tauri invoke 调用后端 execute_sql 命令
 */
async function executeQuery(query: string, context: QueryContext): Promise<unknown[]> {
  const response = await invoke<{ result: unknown[] }>('execute_sql', {
    input: {
      conn_id: context.connectionId,
      sql: query,
      timeout_ms: null,
    },
  })
  return response.result ?? []
}