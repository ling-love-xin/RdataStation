/**
 * 结果集分析服务
 *
 * 前后端对接层：所有与结果分析相关的 Tauri invoke 封装
 */
import { invoke } from '@tauri-apps/api/core'

// ==================== 接口定义 ====================

export interface ResultSet {
  columns: string[]
  rows: unknown[][]
  row_count: number
  elapsed_ms: number
  temp_table: string
}

export interface ColumnStats {
  column_name: string
  data_type: string
  total_count: number
  null_count: number
  unique_count: number | null
  numeric_stats: {
    min: number
    max: number
    avg: number
    median: number
    sum: number
    stddev: number | null
  } | null
  text_stats: {
    min_length: number
    max_length: number
    top_values: [string, number][]
  } | null
}

// ==================== API 方法 ====================

/**
 * SQL 过滤：拼接 WHERE 条件重新查询数据库
 * 后端自动将结果写入 DuckDB 临时表并返回表名
 */
export async function reExecuteWithFilter(
  connId: string,
  originalSql: string,
  whereClause?: string,
  orderClause?: string
): Promise<ResultSet> {
  return invoke<ResultSet>('re_execute_with_filter', {
    input: {
      conn_id: connId,
      original_sql: originalSql,
      where_clause: whereClause || null,
      order_clause: orderClause || null
    }
  })
}

/**
 * DuckDB 分析：对指定临时表执行分析 SQL
 * tempTable 为空时可用 columns/rows 创建临时表
 */
export async function executeDuckdbAnalysis(
  tempTable: string,
  sql: string,
  columns?: string[],
  rows?: unknown[][]
): Promise<ResultSet> {
  return invoke<ResultSet>('execute_duckdb_analysis', {
    input: {
      temp_table: tempTable,
      sql,
      columns: columns || null,
      rows: rows || null
    }
  })
}

/**
 * 列洞察：获取指定列的统计信息
 */
export async function getColumnInsights(
  tempTable: string,
  columnName: string
): Promise<ColumnStats> {
  return invoke<ColumnStats>('get_column_insights', {
    input: {
      temp_table: tempTable,
      column_name: columnName
    }
  })
}

/**
 * 创建 DuckDB 临时表并写入数据
 * 返回生成的临时表名
 */
export async function createDuckdbTempTable(
  columns: string[],
  rows: unknown[][]
): Promise<string> {
  return invoke<string>('create_duckdb_temp_table', {
    input: {
      columns,
      rows
    }
  })
}
