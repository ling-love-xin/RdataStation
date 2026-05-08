/**
 * 数据库导航 API 层
 *
 * 统一处理与 Tauri 后端的通信
 * 遵循规范：所有数据交互必须通过 tauri.invoke 调用 Rust 核心接口
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 数据库元数据类型
 */
export interface IDatabaseMeta {
  name: string
  schemas?: ISchemaMeta[]
}

export interface ISchemaMeta {
  name: string
  tables?: ITableMeta[]
  views?: IViewMeta[]
}

export interface ITableMeta {
  name: string
  type: string
  columns?: IColumnMeta[]
  indexes?: IIndexMeta[]
  constraints?: IConstraintMeta[]
}

export interface IViewMeta {
  name: string
  columns?: IColumnMeta[]
}

export interface IColumnMeta {
  name: string
  dataType: string
  isNullable: boolean
  defaultValue?: string
}

export interface IIndexMeta {
  name: string
  columns: string[]
  isUnique: boolean
  isPrimary: boolean
}

export interface IConstraintMeta {
  name: string
  type: 'PRIMARY KEY' | 'FOREIGN KEY' | 'UNIQUE' | 'CHECK'
  columns: string[]
}

/**
 * 加载数据库列表
 */
export async function loadDatabases(connectionId: string): Promise<IDatabaseMeta[]> {
  return await invoke<IDatabaseMeta[]>('load_databases', { connectionId })
}

/**
 * 加载 Schema 列表
 */
export async function loadSchemas(connectionId: string, dbName: string): Promise<ISchemaMeta[]> {
  return await invoke<ISchemaMeta[]>('load_schemas', { connectionId, dbName })
}

/**
 * 加载表列表
 */
export async function loadTables(
  connectionId: string,
  dbName: string,
  schemaName: string
): Promise<ITableMeta[]> {
  return await invoke<ITableMeta[]>('load_tables', { connectionId, dbName, schemaName })
}

/**
 * 加载视图列表
 */
export async function loadViews(
  connectionId: string,
  dbName: string,
  schemaName: string
): Promise<IViewMeta[]> {
  return await invoke<IViewMeta[]>('load_views', { connectionId, dbName, schemaName })
}

/**
 * 加载列信息
 */
export async function loadColumns(
  connectionId: string,
  dbName: string,
  schemaName: string,
  tableName: string
): Promise<IColumnMeta[]> {
  return await invoke<IColumnMeta[]>('load_columns', {
    connectionId,
    dbName,
    schemaName,
    tableName,
  })
}

/**
 * 加载索引信息
 */
export async function loadIndexes(
  connectionId: string,
  dbName: string,
  schemaName: string,
  tableName: string
): Promise<IIndexMeta[]> {
  return await invoke<IIndexMeta[]>('load_indexes', { connectionId, dbName, schemaName, tableName })
}

/**
 * 加载约束信息
 */
export async function loadConstraints(
  connectionId: string,
  dbName: string,
  schemaName: string,
  tableName: string
): Promise<IConstraintMeta[]> {
  return await invoke<IConstraintMeta[]>('load_constraints', {
    connectionId,
    dbName,
    schemaName,
    tableName,
  })
}

/**
 * 断开数据库连接
 */
export async function disconnectDatabase(connectionId: string): Promise<void> {
  return await invoke<void>('disconnect_database', { connectionId })
}

/**
 * 刷新数据库元数据
 */
export async function refreshMetadata(connectionId: string): Promise<void> {
  return await invoke<void>('refresh_metadata', { connectionId })
}
