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
  isPrimaryKey?: boolean
}

/**
 * 加载 Catalog 列表（ANSI SQL 标准三层结构：Catalog → Schema → Table）
 */
export async function loadCatalogs(connectionId: string): Promise<IDatabaseMeta[]> {
  return await invoke<IDatabaseMeta[]>('load_catalogs', { connectionId })
}

/**
 * @deprecated 请使用 loadCatalogs()
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
 * 加载存储过程列表
 */
export async function loadProcedures(
  connectionId: string,
  dbType: string,
  schemaName: string
): Promise<{ name: string }[]> {
  return await invoke<{ name: string }[]>('load_procedures', {
    connectionId,
    dbType,
    schemaName,
  })
}

/**
 * 加载函数列表
 */
export async function loadFunctions(
  connectionId: string,
  dbType: string,
  schemaName: string
): Promise<{ name: string }[]> {
  return await invoke<{ name: string }[]>('load_functions', {
    connectionId,
    dbType,
    schemaName,
  })
}

// 索引/约束 API 待后端实现
// - load_indexes: Tauri command
// - load_constraints: Tauri command
