/**
 * SQL 编辑器增强服务
 * 提供智能代码补全、语法验证等功能
 * 基于 sqlglot-rust 后端实现 SQL 解析、验证、格式化和转译
 */

import { invoke } from '@tauri-apps/api/core'
import * as monaco from 'monaco-editor'

import type { SqlDialect } from '@/shared/types/sql'

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
      input: { conn_id: connectionId, sql: tablesSql, timeout_ms: 5000 },
    })

    const tableRows: string[][] = tablesResult?.result?.rows || []
    const tableNames = tableRows
      .map((r: any) => String(Array.isArray(r) ? r[0] : (r.name ?? r.table_name ?? '')))
      .filter(Boolean)

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
          input: { conn_id: connectionId, sql: colsSql, timeout_ms: 5000 },
        })

        const colRows: string[][] = colsResult?.result?.rows || []
        // SQLite PRAGMA table_info: [cid, name, type, notnull, dflt_value, pk]
        // information_schema.columns: [column_name]
        const colNames = colRows
          .map((r: any) =>
            String(Array.isArray(r) ? (isSqlite ? r[1] : r[0]) : (r.name ?? r.column_name ?? ''))
          )
          .filter(Boolean)

        tableSchemas.push({
          name: tableName,
          columns: colNames,
        })
      } catch (e) {
        console.warn(`Failed to get columns for ${tableName}:`, e)
      }
    }

    const dbSchema: DatabaseSchema = {
      tables: tableSchemas,
      views: [],
      functions: [],
    }

    // 缓存结果（带 TTL）
    schemaCache.set(cacheKey, {
      data: dbSchema,
      timestamp: Date.now(),
      ttl: CACHE_TTL,
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
        endColumn: word.endColumn,
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
            sortText: `a${String(index).padStart(3, '0')}`,
          })

          // 添加列名补全（带表名前缀）
          table.columns.forEach((col, colIndex) => {
            suggestions.push({
              label: `${table.name}.${col}`,
              kind: monaco.languages.CompletionItemKind.Field,
              insertText: `${table.name}.${col}`,
              detail: `Column of ${table.name}`,
              range,
              sortText: `b${String(index).padStart(3, '0')}${String(colIndex).padStart(3, '0')}`,
            })
          })
        })
      }

      return { suggestions }
    },
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
        dialect: dialect || 'generic',
      },
    })

    if (!result.valid && result.errors && result.errors.length > 0) {
      result.errors.forEach((error: string) => {
        markers.push({
          severity: monaco.MarkerSeverity.Error,
          message: error,
          startLineNumber: 1,
          startColumn: 1,
          endLineNumber: 1,
          endColumn: sql.length + 1,
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
        endColumn: line.length + 1,
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
        endColumn: line.length + 1,
      })
    }
  })

  return markers
}

/**
 * 格式化 SQL（基于 sqlglot-rust）
 */
export async function formatSql(sql: string, dialect?: SqlDialect): Promise<string> {
  try {
    const result = await invoke<any>('format_sql', {
      input: {
        sql,
        dialect: dialect || 'generic',
      },
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
        target_dialect: targetDialect,
      },
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
      dialect: dialect || 'generic',
    })

    return {
      success: result.success,
      statementsCount: result.statements_count,
      error: result.error,
    }
  } catch (error) {
    console.error('Failed to parse SQL:', error)
    return {
      success: false,
      statementsCount: 0,
      error: String(error),
    }
  }
}

/**
 * 分割多语句 SQL（基于 sqlglot-rust）
 */
export async function splitSql(sql: string, dialect?: SqlDialect): Promise<string[]> {
  try {
    return await invoke<string[]>('split_sql', {
      sql,
      dialect: dialect ? String(dialect) : null,
    })
  } catch {
    return [sql]
  }
}

/**
 * 注册 SQL 代码折叠提供器
 *
 * 识别可折叠区域：
 * - 多行括号子查询 (SELECT ... )
 * - CTE 块 WITH ... AS ( ... )
 * - BEGIN ... END 事务块
 * - 连续注释块
 */
export function registerSqlFoldingProvider(): monaco.IDisposable {
  return monaco.languages.registerFoldingRangeProvider('sql', {
    provideFoldingRanges(model) {
      const ranges: monaco.languages.FoldingRange[] = []
      const lines = model.getLinesContent()

      // Track bracket-based folding: open parentheses on separate lines
      const bracketStack: number[] = []

      // Track BEGIN...END blocks
      const beginStack: number[] = []

      // Track comment blocks
      let commentStart: number | null = null

      for (let i = 0; i < lines.length; i++) {
        const line = lines[i]
        const trimmed = line.trimStart()
        const upperTrimmed = trimmed.toUpperCase()

        // Multi-line comment blocks (/* ... */)
        if (trimmed.includes('/*') && !trimmed.includes('*/')) {
          if (commentStart === null) {
            commentStart = i + 1
          }
        }
        if (commentStart !== null && trimmed.includes('*/')) {
          if (i + 1 > commentStart) {
            ranges.push({
              start: commentStart,
              end: i + 1,
              kind: monaco.languages.FoldingRangeKind.Comment.value,
            })
          }
          commentStart = null
        }

        // CTE start: WITH ... AS (
        if (upperTrimmed.startsWith('WITH ')) {
          bracketStack.push(i + 1)
        }

        // BEGIN transaction blocks
        if (upperTrimmed === 'BEGIN' || upperTrimmed === 'BEGIN TRANSACTION' || upperTrimmed.startsWith('START TRANSACTION')) {
          beginStack.push(i + 1)
        }
        if ((upperTrimmed === 'END' || upperTrimmed === 'COMMIT' || upperTrimmed === 'ROLLBACK') && beginStack.length > 0) {
          const start = beginStack.pop()!
          if (i + 1 > start) {
            ranges.push({ start, end: i + 1 })
          }
        }

        // Parenthesized subquery: line ending with ( or starting with (
        const openParens = (trimmed.match(/\(/g) || []).length
        const closeParens = (trimmed.match(/\)/g) || []).length
        const netParens = openParens - closeParens

        // Push stacking for multi-line parenthesized blocks
        for (let p = 0; p < openParens; p++) {
          bracketStack.push(i + 1)
        }
        for (let p = 0; p < closeParens; p++) {
          if (bracketStack.length > 0) {
            const start = bracketStack.pop()!
            if (i + 1 > start) {
              ranges.push({ start, end: i + 1 })
            }
          }
        }

        // Dedicated line that is just "(" — fold everything until matching ")"
        if (trimmed === '(') {
          bracketStack.push(i + 1)
        }
        if (trimmed === ')') {
          if (bracketStack.length > 0 && lines[bracketStack[bracketStack.length - 1] - 1]?.trim() === '(') {
            const start = bracketStack.pop()!
            if (i + 1 > start) {
              ranges.push({ start, end: i + 1 })
            }
          }
        }
      }

      return ranges
    },
  })
}

/**
 * 生成 DuckDB ATTACH 名称（与后端 `ext_{sanitized_name}` 保持一致）
 */
export function generateAttachName(connectionName: string): string {
  return `ext_${connectionName.replace(/[^a-zA-Z0-9_]/g, '_')}`
}

/**
 * 为 DuckDB 联邦查询重写 SQL：给无前缀表名加 ATTACH 前缀
 *
 * 例如：SELECT * FROM users WHERE id = 1
 *    → SELECT * FROM ext_MyConn.users WHERE id = 1
 *
 * 识别范围：FROM | JOIN | INSERT INTO 后的表名
 * 安全过滤：跳过 SQL 关键字、已有 '.' 前缀的表名
 */
export function rewriteDuckDBSQL(sql: string, attachName: string): string {
  if (!attachName) return sql

  const SQL_KEYWORDS = new Set([
    'WHERE', 'SET', 'VALUES', 'SELECT', 'ON', 'AS', 'AND', 'OR',
    'NOT', 'IN', 'IS', 'NULL', 'TRUE', 'FALSE', 'LIKE', 'BETWEEN',
    'EXISTS', 'CASE', 'WHEN', 'THEN', 'ELSE', 'END', 'GROUP', 'BY',
    'ORDER', 'HAVING', 'LIMIT', 'OFFSET', 'UNION', 'ALL', 'DISTINCT',
  ])

  const tableKeywords = [
    'FROM', 'JOIN', 'INNER JOIN', 'LEFT JOIN', 'RIGHT JOIN',
    'FULL JOIN', 'CROSS JOIN', 'NATURAL JOIN', 'LEFT OUTER JOIN',
    'RIGHT OUTER JOIN', 'FULL OUTER JOIN', 'INSERT INTO', 'UPDATE', 'INTO',
  ]

  let result = sql

  for (const keyword of tableKeywords) {
    const pattern = new RegExp(
      `(\\b${keyword}\\b)\\s+(?!${attachName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\.)([a-zA-Z_][a-zA-Z0-9_]*)\\b(?!\\s*\\.)`,
      'gi'
    )
    result = result.replace(pattern, (_match, kw, tableName) => {
      if (SQL_KEYWORDS.has(tableName.toUpperCase())) {
        return _match
      }
      return `${kw} ${attachName}.${tableName}`
    })
  }

  return result
}

export interface ParamInfo {
  name: string
  occurrences: number
}

export function detectParams(sql: string): ParamInfo[] {
  const paramMap = new Map<string, number>()
  const regex = /:([a-zA-Z_][a-zA-Z0-9_]*)/g
  let match: RegExpExecArray | null

  while ((match = regex.exec(sql)) !== null) {
    const name = match[1]
    paramMap.set(name, (paramMap.get(name) || 0) + 1)
  }

  const result: ParamInfo[] = []
  paramMap.forEach((occurrences, name) => {
    result.push({ name, occurrences })
  })

  return result
}

export function bindParams(sql: string, values: Record<string, string>): string {
  let result = sql
  for (const [name, value] of Object.entries(values)) {
    const escapedValue = value.replace(/'/g, "''")
    const regex = new RegExp(`:${name}\\b`, 'g')
    result = result.replace(regex, `'${escapedValue}'`)
  }
  return result
}
