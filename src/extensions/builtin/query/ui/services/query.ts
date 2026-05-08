/**
 * 查询服务
 *
 * 提供 SQL 查询执行的 API 调用
 */

import { invoke } from '@tauri-apps/api/core'

import type {
  ExecuteSqlResponse,
  ExecuteTransactionResponse,
  SqlHistoryResponse,
} from '../../infrastructure/types/query-service'

/**
 * 执行 SQL（主入口）
 */
export async function executeSql(
  sql: string,
  connectionId?: string,
  _timeoutMs?: number
): Promise<ExecuteSqlResponse> {
  return invoke<ExecuteSqlResponse>('execute_sql', {
    input: {
      conn_id: connectionId,
      sql,
      timeout_ms: null,
    },
  })
}

/**
 * 执行事务
 */
export async function executeTransaction(
  sqls: string[],
  connectionId?: string
): Promise<ExecuteTransactionResponse> {
  return invoke<ExecuteTransactionResponse>('execute_transaction', {
    input: {
      conn_id: connectionId,
      sqls,
    },
  })
}

/**
 * 取消查询
 */
export async function cancelQuery(connId: string): Promise<boolean> {
  return invoke<boolean>('cancel_sql_query', { connId })
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
    error: undefined,
  }))
}

/**
 * 搜索 SQL 历史
 */
export async function searchSqlHistory(
  _keyword: string,
  _limit?: number
): Promise<SqlHistoryItem[]> {
  const result = await invoke<SqlHistoryResponse[]>('search_sql_history')
  return result.map(r => ({
    id: r.id,
    sql: r.sql,
    conn_id: r.connectionId,
    executed_at: r.executedAt,
    execution_time: r.executionTime,
    row_count: r.rowCount,
    success: r.success,
    error: undefined,
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

/**
 * 事务控制
 */

export interface TransactionStatus {
  connId: string
  isInTransaction: boolean
  transactionStartTimeMs: number | null
  transactionDurationMs: number | null
}

export async function beginTransaction(connId?: string): Promise<TransactionStatus> {
  return invoke<TransactionStatus>('begin_transaction', { connId })
}

export async function commitTransaction(connId?: string): Promise<TransactionStatus> {
  return invoke<TransactionStatus>('commit_transaction', { connId })
}

export async function rollbackTransaction(connId?: string): Promise<TransactionStatus> {
  return invoke<TransactionStatus>('rollback_transaction', { connId })
}

export async function getTransactionStatus(connId?: string): Promise<TransactionStatus> {
  return invoke<TransactionStatus>('get_transaction_status', { connId })
}

// ==================== DuckDB 加速查询 ====================

export interface DuckDBAcceleratedParams {
  sql: string
  connId: string
  dbType?: string
  dataDir?: string
}

export interface DuckDBAcceleratedResult {
  success: boolean
  columns: string[]
  rows: unknown[][]
  elapsedMs: number
  error: string | null
}

export async function executeDuckDBAccelerated(
  params: DuckDBAcceleratedParams
): Promise<DuckDBAcceleratedResult> {
  const raw = await invoke<{
    success: boolean
    columns: string[] | null
    rows: unknown[][] | null
    elapsed_ms: number
    error: string | null
  }>('execute_duckdb_accelerated', {
    sql: params.sql,
    connId: params.connId,
    dbType: params.dbType || null,
    dataDir: params.dataDir || null,
  })

  return {
    success: raw.success,
    columns: raw.columns ?? [],
    rows: raw.rows ?? [],
    elapsedMs: raw.elapsed_ms,
    error: raw.error,
  }
}

// ==================== DuckDB 外部数据库管理 ====================

export interface RegisterExternalDatabaseParams {
  connId: string
  name: string
  driver: string
  connectionString: string
}

export async function registerExternalDatabase(
  params: RegisterExternalDatabaseParams
): Promise<void> {
  return invoke('register_external_database', {
    connId: params.connId,
    name: params.name,
    driver: params.driver,
    connectionString: params.connectionString,
  })
}

export interface CreateExternalTableParams {
  connId: string
  schemaName: string
  tableName: string
  externalDbName: string
}

export async function createExternalTable(
  params: CreateExternalTableParams
): Promise<void> {
  return invoke('create_external_table', {
    connId: params.connId,
    schemaName: params.schemaName,
    tableName: params.tableName,
    externalDbName: params.externalDbName,
  })
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
