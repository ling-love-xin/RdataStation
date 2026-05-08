import { invoke } from '@tauri-apps/api/core'

import type {
  ExecuteSqlResponse,
  ExecuteTransactionResponse,
  SqlHistoryResponse,
} from '../types/query-service'

/**
 * 查询服务
 *
 * 负责与后端 SQL 查询相关的所有 API 调用
 */

/**
 * 执行 SQL 查询
 */
export async function executeSql(
  sql: string,
  connId?: string,
  timeoutMs?: number
): Promise<ExecuteSqlResponse> {
  return invoke('execute_sql', {
    input: { sql, conn_id: connId, timeout_ms: timeoutMs },
  })
}

/**
 * 执行事务
 */
export async function executeTransaction(
  sqls: string[],
  connId?: string
): Promise<ExecuteTransactionResponse> {
  return invoke('execute_transaction', {
    input: { sqls, conn_id: connId },
  })
}

/**
 * 获取 SQL 历史记录
 */
export async function getSqlHistory(limit?: number): Promise<SqlHistoryResponse[]> {
  return invoke('get_sql_history', { limit })
}

/**
 * 搜索 SQL 历史记录
 */
export async function searchSqlHistory(
  keyword: string,
  limit?: number
): Promise<SqlHistoryResponse[]> {
  return invoke('search_sql_history', { keyword, limit })
}

/**
 * 清空 SQL 历史记录
 */
export async function clearSqlHistory(): Promise<void> {
  return invoke('clear_sql_history')
}

/**
 * 删除指定 SQL 历史记录
 */
export async function removeSqlHistory(id: string): Promise<void> {
  return invoke('remove_sql_history', { id })
}
