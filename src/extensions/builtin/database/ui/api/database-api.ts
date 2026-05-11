/**
 * 数据库导航 API 层
 *
 * 统一处理与 Tauri 后端的通信
 * 遵循规范：所有数据交互必须通过 tauri.invoke 调用 Rust 核心接口
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 数据库元数据 — 树形导航根节点
 */
export interface IDatabaseMeta {
  /** 数据库名称 */
  name: string
  /** Schema 列表 */
  schemas?: ISchemaMeta[]
}

/**
 * Schema 元数据
 */
export interface ISchemaMeta {
  /** Schema 名称 */
  name: string
  /** 表列表 */
  tables?: ITableMeta[]
  /** 视图列表 */
  views?: IViewMeta[]
}

/**
 * 表元数据
 */
export interface ITableMeta {
  /** 表名 */
  name: string
  /** 表类型（TABLE / VIEW） */
  type: string
  /** 列列表 */
  columns?: IColumnMeta[]
  /** 索引列表 */
  indexes?: IIndexMeta[]
  /** 约束列表 */
  constraints?: IConstraintMeta[]
}

/**
 * 索引元数据
 */
export interface IIndexMeta {
  name: string
  columns: string[]
  unique: boolean
}

/**
 * 约束元数据
 */
export interface IConstraintMeta {
  name: string
  type: string
  columns: string[]
}

/**
 * 视图元数据
 */
export interface IViewMeta {
  /** 视图名 */
  name: string
  /** 列列表 */
  columns?: IColumnMeta[]
}

/**
 * 列元数据（对齐后端 ColumnMeta）
 *
 * @see ColumnMeta (Rust metadata_commands.rs)
 */
export interface IColumnMeta {
  /** 列名 */
  name: string
  /** 数据类型 */
  dataType: string
  /** 是否可空 */
  isNullable: boolean
  /** 默认值 */
  defaultValue?: string
  /** 是否主键 */
  isPrimaryKey?: boolean
}

/**
 * 加载 Catalog 列表（ANSI SQL 标准三层结构：Catalog → Schema → Table）
 *
 * @param connectionId - 连接标识符
 * @returns 数据库元数据树形数组
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
 *
 * @param connectionId - 连接标识符
 * @param dbName - 数据库（Catalog）名称
 * @returns Schema 元数据数组
 */
export async function loadSchemas(connectionId: string, dbName: string): Promise<ISchemaMeta[]> {
  return await invoke<ISchemaMeta[]>('load_schemas', { connectionId, dbName })
}

/**
 * 加载表列表
 *
 * @param connectionId - 连接标识符
 * @param dbName - 数据库（Catalog）名称
 * @param schemaName - Schema 名称
 * @returns 表元数据数组
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
 *
 * @param connectionId - 连接标识符
 * @param dbName - 数据库（Catalog）名称
 * @param schemaName - Schema 名称
 * @returns 视图元数据数组
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
 *
 * @param connectionId - 连接标识符
 * @param dbName - 数据库（Catalog）名称
 * @param schemaName - Schema 名称
 * @param tableName - 表名或视图名
 * @returns 列元数据数组
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
 *
 * @param connectionId - 连接标识符
 * @param dbName - 数据库（Catalog）名称
 * @param schemaName - Schema 名称
 * @returns 存储过程名数组
 */
export async function loadProcedures(
  connectionId: string,
  dbName: string,
  schemaName: string
): Promise<{ name: string }[]> {
  return await invoke<{ name: string }[]>('load_procedures', {
    connectionId,
    dbName,
    schemaName,
  })
}

/**
 * 加载函数列表
 *
 * @param connectionId - 连接标识符
 * @param dbName - 数据库（Catalog）名称
 * @param schemaName - Schema 名称
 * @returns 函数名数组
 */
export async function loadFunctions(
  connectionId: string,
  dbName: string,
  schemaName: string
): Promise<{ name: string }[]> {
  return await invoke<{ name: string }[]>('load_functions', {
    connectionId,
    dbName,
    schemaName,
  })
}

/**
 * 加载过程/函数的 DDL 源码 (DBeaver-style Source Tab)
 * R15 新增 — get_routine_source() trait 方法 + L1 缓存
 *
 * @param connectionId - 连接标识符
 * @param dbName - 数据库（Catalog）名称
 * @param schemaName - Schema 名称
 * @param routineName - 例程名（过程或函数名）
 * @param routineKind - 例程类型（PROCEDURE / FUNCTION）
 * @returns 例程源码元数据
 */
export interface RoutineSourceMeta {
  name: string
  routineKind: string
  sourceCode: string | null
}

export async function loadRoutineSource(
  connectionId: string,
  dbName: string,
  schemaName: string,
  routineName: string,
  routineKind: string
): Promise<RoutineSourceMeta> {
  return await invoke<RoutineSourceMeta>('load_routine_source', {
    connId: connectionId,
    dbName,
    schemaName,
    routineName,
    routineKind,
  })
}

// 索引/约束 API 待后端实现
// - load_indexes: Tauri command
// - load_constraints: Tauri command

/**
 * API 版本信息
 */
export interface ApiVersionResponse {
  version: string
  major: number
  minor: number
  patch: number
  codename: string
}

export async function getApiVersion(): Promise<ApiVersionResponse> {
  return await invoke<ApiVersionResponse>('get_api_version')
}

/**
 * 连接健康检查（ping）
 */
export async function pingConnection(connId?: string): Promise<boolean> {
  return await invoke<boolean>('ping_connection', { connId })
}

/**
 * SQL 审计日志记录
 */
export interface SqlAuditRecord {
  id: string
  sql: string
  connId: string | null
  dbType: string | null
  executedAt: string
  durationMs: number | null
  success: boolean | null
  errorMessage: string | null
  rowsAffected: number | null
  rowsReturned: number | null
}
