/**
 * 查询服务
 *
 * 提供 SQL 查询执行的 API 调用
 */

import { invoke } from '@tauri-apps/api/core'

import type { QueryResult } from '@/shared/types'

import type { ExecuteSqlResponse, ExecuteTransactionResponse, SqlHistoryResponse } from '../../infrastructure/types/query-service'

/**
 * 执行 SQL 查询
 */
export async function executeQuery(connectionId: string, sql: string): Promise<QueryResult> {
  return invoke<QueryResult>('execute_query', { connectionId, sql })
}

/**
 * 执行 SQL（兼容旧接口）
 */
export async function executeSql(sql: string, connectionId?: string, _timeoutMs?: number): Promise<ExecuteSqlResponse> {
  return invoke<ExecuteSqlResponse>('execute_sql', {
    input: {
      conn_id: connectionId,
      sql,
      timeout_ms: null
    }
  })
}

/**
 * 执行事务
 */
export async function executeTransaction(sqls: string[], connectionId?: string): Promise<ExecuteTransactionResponse> {
  return invoke<ExecuteTransactionResponse>('execute_transaction', {
    input: {
      conn_id: connectionId,
      sqls
    }
  })
}

/**
 * 取消查询
 */
export async function cancelQuery(queryId: string): Promise<void> {
  return invoke('cancel_query', { queryId })
}

/**
 * 获取查询历史
 */
export async function getQueryHistory(): Promise<SqlHistoryResponse[]> {
  return invoke<SqlHistoryResponse[]>('get_query_history')
}

/**
 * 获取 SQL 历史（别名）
 */
export async function getSqlHistory(_limit?: number): Promise<SqlHistoryItem[]> {
  const result = await invoke<SqlHistoryResponse[]>('get_sql_history')
  return result.map(r => ({
    id: r.id,
    sql: r.sql,
    conn_id: r.connectionId,
    executed_at: r.executedAt,
    execution_time: r.executionTime,
    row_count: r.rowCount,
    success: r.success,
    error: undefined
  }))
}

/**
 * 搜索 SQL 历史
 */
export async function searchSqlHistory(_keyword: string, _limit?: number): Promise<SqlHistoryItem[]> {
  const result = await invoke<SqlHistoryResponse[]>('search_sql_history')
  return result.map(r => ({
    id: r.id,
    sql: r.sql,
    conn_id: r.connectionId,
    executed_at: r.executedAt,
    execution_time: r.executionTime,
    row_count: r.rowCount,
    success: r.success,
    error: undefined
  }))
}

/**
 * 清除 SQL 历史
 */
export async function clearSqlHistory(): Promise<void> {
  return invoke('clear_sql_history')
}

/**
 * 删除 SQL 历史记录
 */
export async function removeSqlHistory(id: string): Promise<void> {
  return invoke('remove_sql_history', { id })
}

// SQL 历史项
export interface SqlHistoryItem {
  id: string
  sql: string
  conn_id: string
  executed_at: string
  execution_time: number
  row_count: number
  success: boolean
  error?: string
}

// 查询历史项
export interface QueryHistoryItem {
  id: string
  sql: string
  connectionId: string
  executedAt: string
  executionTime: number
  rowCount: number
  success: boolean
}
