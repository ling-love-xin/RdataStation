/**
 * SQL 编辑器增强服务
 * 提供智能代码补全、语法验证等功能
 * 基于 sqlglot-rust 后端实现 SQL 解析、验证、格式化和转译
 */

import { invoke } from '@tauri-apps/api/core'
import * as monaco from 'monaco-editor'

// SQL 方言类型
export type SqlDialect = 
  | 'generic'
  | 'mysql'
  | 'postgres'
  | 'sqlite'
  | 'duckdb'
  | 'mssql'
  | 'oracle'
  | 'snowflake'
  | 'bigquery'
  | 'redshift'

// 缓存的表结构信息
interface TableSchema {
  name: string
  columns: string[]
}

interface DatabaseSchema {
  tables: TableSchema[]
  views: string[]
  functions: string[]
}

// 缓存项（带 TTL）
interface SchemaCacheItem {
  data: DatabaseSchema
  timestamp: number
  ttl: number // 毫秒
}

// 缓存
const schemaCache: Map<string, SchemaCacheItem> = new Map()
const CACHE_TTL = 5 * 60 * 1000 // 5 分钟

// 补全提供者管理
const completionDisposables: Map<string, monaco.IDisposable> = new Map()

/**
 * 获取数据库结构信息
 */
export async function getDatabaseSchema(
  connectionId: string,
  database: string,
  schema?: string,
  dbType?: string
): Promise<DatabaseSchema | null> {
  const cacheKey = `${connectionId}_${database}_${schema || 'default'}`
  
  // 检查缓存（带 TTL）
  const cacheItem = schemaCache.get(cacheKey)
  if (cacheItem) {
    const now = Date.now()
    if (now - cacheItem.timestamp < cacheItem.ttl) {
      return cacheItem.data
    }
    // 缓存过期，删除
    schemaCache.delete(cacheKey)
  }

  try {
    const isSqlite = dbType?.toLowerCase() === 'sqlite'

    // 获取表列表（SQLite 不支持 information_schema）
    let tablesSql: string
    if (isSqlite) {
      tablesSql = `SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name LIMIT 20`
    } else {
      tablesSql = schema
        ? `SELECT table_name FROM information_schema.tables WHERE table_catalog = '${database.replace(/'/g, "''")}' AND table_schema = '${schema.replace(/'/g, "''")}' ORDER BY table_name LIMIT 20`
        : `SELECT table_name FROM information_schema.tables WHERE table_catalog = '${database.replace(/'/g, "''")}' ORDER BY table_name LIMIT 20`
    }

    const tablesResult: any = await invoke('execute_sql', {
      input: { conn_id: connectionId, sql: tablesSql, timeout_ms: 5000 }
    })

    const tableRows: string[][] = tablesResult?.result?.rows || []
    const tableNames = tableRows.map((r: any) => String(Array.isArray(r) ? r[0] : r.name ?? r.table_name ?? '')).filter(Boolean)

    // 获取每个表的列信息
    const tableSchemas: TableSchema[] = []
    for (const tableName of tableNames.slice(0, 20)) {
      try {
        let colsSql: string
        if (isSqlite) {
          colsSql = `PRAGMA table_info('${tableName.replace(/'/g, "''")}')`
        } else {
          colsSql = schema
            ? `SELECT column_name FROM information_schema.columns WHERE table_catalog = '${database.replace(/'/g, "''")}' AND table_schema = '${schema.replace(/'/g, "''")}' AND table_name = '${tableName.replace(/'/g, "''")}' ORDER BY ordinal_position`
            : `SELECT column_name FROM information_schema.columns WHERE table_catalog = '${database.replace(/'/g, "''")}' AND table_name = '${tableName.replace(/'/g, "''")}' ORDER BY ordinal_position`
        }

        const colsResult: any = await invoke('execute_sql', {
          input: { conn_id: connectionId, sql: colsSql, timeout_ms: 5000 }
        })

        const colRows: string[][] = colsResult?.result?.rows || []
        // SQLite PRAGMA table_info: [cid, name, type, notnull, dflt_value, pk]
        // information_schema.columns: [column_name]
        const colNames = colRows.map((r: any) => String(Array.isArray(r) ? (isSqlite ? r[1] : r[0]) : r.name ?? r.column_name ?? '')).filter(Boolean)

        tableSchemas.push({
          name: tableName,
          columns: colNames
        })
      } catch (e) {
        console.warn(`Failed to get columns for ${tableName}:`, e)
      }
    }

    const dbSchema: DatabaseSchema = {
      tables: tableSchemas,
      views: [],
      functions: []
    }

    // 缓存结果（带 TTL）
    schemaCache.set(cacheKey, {
      data: dbSchema,
      timestamp: Date.now(),
      ttl: CACHE_TTL
    })
    return dbSchema
  } catch (error) {
    console.error('Failed to get database schema:', error)
    return null
  }
}

/**
 * 注册数据库相关的代码补全提供器
 * 返回 disposable 以便清理
 */
export function registerDatabaseCompletionProvider(
  connectionId: string,
  database: string,
  schema?: string,
  dbType?: string
): monaco.IDisposable {
  const disposableKey = `${connectionId}_${database}`
  
  // 清除之前的提供器
  if (completionDisposables.has(disposableKey)) {
    completionDisposables.get(disposableKey)!.dispose()
  }
  
  const disposable = monaco.languages.registerCompletionItemProvider('sql', {
    triggerCharacters: ['.', ' '],
    provideCompletionItems: async (model, position) => {
      const word = model.getWordUntilPosition(position)
      const range = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn
      }

      const suggestions: monaco.languages.CompletionItem[] = []

      // 获取数据库结构
      const dbSchema = await getDatabaseSchema(connectionId, database, schema, dbType)
      if (dbSchema) {
        // 添加表名补全
        dbSchema.tables.forEach((table, index) => {
          suggestions.push({
            label: table.name,
            kind: monaco.languages.CompletionItemKind.Class,
            insertText: table.name,
            detail: 'Table',
            range,
            sortText: `a${String(index).padStart(3, '0')}`
          })

          // 添加列名补全（带表名前缀）
          table.columns.forEach((col, colIndex) => {
            suggestions.push({
              label: `${table.name}.${col}`,
              kind: monaco.languages.CompletionItemKind.Field,
              insertText: `${table.name}.${col}`,
              detail: `Column of ${table.name}`,
              range,
              sortText: `b${String(index).padStart(3, '0')}${String(colIndex).padStart(3, '0')}`
            })
          })
        })
      }

      return { suggestions }
    }
  })
  
  // 存储 disposable
  completionDisposables.set(disposableKey, disposable)
  
  return disposable
}

/**
 * 清除指定连接的补全提供者
 */
export function unregisterCompletionProvider(connectionId: string) {
  const keysToDelete: string[] = []
  
  for (const key of completionDisposables.keys()) {
    if (key.startsWith(connectionId)) {
      completionDisposables.get(key)?.dispose()
      keysToDelete.push(key)
    }
  }
  
  keysToDelete.forEach(key => completionDisposables.delete(key))
}

/**
 * 清除所有补全提供者
 */
export function clearAllCompletionProviders() {
  for (const disposable of completionDisposables.values()) {
    disposable.dispose()
  }
  completionDisposables.clear()
}

/**
 * 清除缓存
 */
export function clearSchemaCache() {
  schemaCache.clear()
}

/**
 * SQL 语法验证（基于 sqlglot-rust）
 */
export async function validateSql(
  sql: string,
  dialect?: SqlDialect
): Promise<monaco.editor.IMarkerData[]> {
  const markers: monaco.editor.IMarkerData[] = []

  try {
    const result = await invoke<any>('validate_sql', {
      input: {
        sql,
        dialect: dialect || 'generic'
      }
    })

    if (!result.valid && result.errors && result.errors.length > 0) {
      result.errors.forEach((error: string) => {
        markers.push({
          severity: monaco.MarkerSeverity.Error,
          message: error,
          startLineNumber: 1,
          startColumn: 1,
          endLineNumber: 1,
          endColumn: sql.length + 1
        })
      })
    }
  } catch (error) {
    console.error('SQL validation failed:', error)
    // 降级到基础检查
    const basicMarkers = basicValidateSql(sql)
    markers.push(...basicMarkers)
  }

  return markers
}

/**
 * 基础 SQL 验证（降级方案）
 */
function basicValidateSql(sql: string): monaco.editor.IMarkerData[] {
  const markers: monaco.editor.IMarkerData[] = []
  const lines = sql.split('\n')

  lines.forEach((line, lineIndex) => {
    const lineNum = lineIndex + 1

    // 检查未闭合的括号
    const openParens = (line.match(/\(/g) || []).length
    const closeParens = (line.match(/\)/g) || []).length
    if (openParens !== closeParens) {
      markers.push({
        severity: monaco.MarkerSeverity.Warning,
        message: '括号可能未闭合',
        startLineNumber: lineNum,
        startColumn: 1,
        endLineNumber: lineNum,
        endColumn: line.length + 1
      })
    }

    // 检查字符串引号
    const singleQuotes = (line.match(/'/g) || []).length
    if (singleQuotes % 2 !== 0) {
      markers.push({
        severity: monaco.MarkerSeverity.Error,
        message: '字符串引号未闭合',
        startLineNumber: lineNum,
        startColumn: 1,
        endLineNumber: lineNum,
        endColumn: line.length + 1
      })
    }
  })

  return markers
}

/**
 * 格式化 SQL（基于 sqlglot-rust）
 */
export async function formatSql(
  sql: string,
  dialect?: SqlDialect
): Promise<string> {
  try {
    const result = await invoke<any>('format_sql', {
      input: {
        sql,
        dialect: dialect || 'generic'
      }
    })

    if (result.success) {
      return result.formatted_sql
    } else {
      // 静默返回原始 SQL，不打印警告
      return sql
    }
  } catch (error) {
    // 静默返回原始 SQL
    return sql
  }
}

/**
 * 转译 SQL（基于 sqlglot-rust）
 */
export async function transpileSql(
  sql: string,
  sourceDialect: SqlDialect,
  targetDialect: SqlDialect
): Promise<string> {
  try {
    const result = await invoke<any>('transpile_sql', {
      input: {
        sql,
        source_dialect: sourceDialect,
        target_dialect: targetDialect
      }
    })

    if (result.success) {
      return result.transpiled_sql
    } else {
      console.warn('SQL transpilation failed:', result.error)
      return sql
    }
  } catch (error) {
    console.error('Failed to transpile SQL:', error)
    return sql
  }
}

/**
 * 解析 SQL（基于 sqlglot-rust）
 */
export async function parseSql(
  sql: string,
  dialect?: SqlDialect
): Promise<{ success: boolean; statementsCount: number; error?: string }> {
  try {
    const result = await invoke<any>('parse_sql', {
      sql,
      dialect: dialect || 'generic'
    })

    return {
      success: result.success,
      statementsCount: result.statements_count,
      error: result.error
    }
  } catch (error) {
    console.error('Failed to parse SQL:', error)
    return {
      success: false,
      statementsCount: 0,
      error: String(error)
    }
  }
}
