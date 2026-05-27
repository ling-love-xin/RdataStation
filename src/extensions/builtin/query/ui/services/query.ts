/**
 * 查询服务
 *
 * 提供 SQL 查询执行的 API 调用
 * 使用 tauri-specta 生成的 typed commands
 */

import { commands } from '@/generated/specta/bindings'
import type { ExecuteSqlInput, ExecuteTransactionInput } from '@/generated/specta/bindings'
import { typed, tauriInvoke } from '@/shared/api'

import type {
  ExecuteSqlResponse,
  ExecuteTransactionResponse,
} from '../../infrastructure/types/query-service'

/**
 * 执行 SQL（主入口）
 */
export async function executeSql(
  sql: string,
  connectionId?: string,
  timeoutMs?: number
): Promise<ExecuteSqlResponse> {
  const input: ExecuteSqlInput = {
    conn_id: connectionId ?? null,
    sql,
    timeout_ms: timeoutMs ?? null,
  }
  return typed(commands.executeSql(input)) as unknown as ExecuteSqlResponse
}

/**
 * 执行事务
 */
export async function executeTransaction(
  sqls: string[],
  connectionId?: string
): Promise<ExecuteTransactionResponse> {
  const input: ExecuteTransactionInput = {
    conn_id: connectionId ?? null,
    sqls,
  }
  return typed(commands.executeTransaction(input)) as unknown as ExecuteTransactionResponse
}

/**
 * 取消查询
 */
export async function cancelQuery(connId: string): Promise<boolean> {
  return typed(commands.cancelSqlQuery(connId))
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
  return typed(commands.beginTransaction(connId ?? null)) as unknown as TransactionStatus
}

export async function commitTransaction(connId?: string): Promise<TransactionStatus> {
  return typed(commands.commitTransaction(connId ?? null)) as unknown as TransactionStatus
}

export async function rollbackTransaction(connId?: string): Promise<TransactionStatus> {
  return typed(commands.rollbackTransaction(connId ?? null)) as unknown as TransactionStatus
}

export async function getTransactionStatus(connId?: string): Promise<TransactionStatus> {
  return typed(commands.getTransactionStatus(connId ?? null)) as unknown as TransactionStatus
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

/**
 * DuckDB 加速查询 — 尚未在 specta collect_commands! 中注册
 */
export async function executeDuckDBAccelerated(
  params: DuckDBAcceleratedParams
): Promise<DuckDBAcceleratedResult> {
  const raw = await tauriInvoke<{
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
  await typed(commands.registerExternalDatabase({
    conn_id: params.connId ?? null,
    name: params.name,
    driver: params.driver,
    connection_string: params.connectionString,
  }))
}

export interface CreateExternalTableParams {
  connId: string
  schemaName: string
  tableName: string
  externalDbName: string
}

export async function createExternalTable(params: CreateExternalTableParams): Promise<void> {
  await typed(commands.createExternalTable({
    conn_id: params.connId ?? null,
    external_db_name: params.externalDbName,
    schema_name: params.schemaName,
    table_name: params.tableName,
    external_table_name: `${params.externalDbName}_${params.tableName}`,
  }))
}