/**
 * 结果集分析服务 + 洞察 API
 *
 * 前后端对接层：所有与结果分析相关的 Tauri invoke 封装
 */
import { invoke } from '@tauri-apps/api/core'

// ==================== 基础接口 ====================

export interface ResultSet {
  columns: string[]
  rows: unknown[][]
  row_count: number
  elapsed_ms: number
  temp_table: string
}

// ==================== 洞察体系类型 ====================

export interface ColumnInsightFull {
  stats: ColumnStats
  sample: unknown[]
  histogram: DistributionBin[] | null
}

export interface ColumnStats {
  column_name: string
  data_type: string
  total_count: number
  null_count: number
  null_rate: number
  unique_count: number | null
  stats_detail: ColumnStatsDetail
}

export type ColumnStatsDetail =
  | NumericStatsDetail
  | TextStatsDetail
  | DateTimeStatsDetail
  | BooleanStatsDetail
  | UnknownStatsDetail

export interface NumericStatsDetail {
  kind: 'Numeric'
  min: number
  max: number
  avg: number
  median: number
  p25: number
  p75: number
  sum: number
  stddev: number | null
  skewness: number | null
  kurtosis: number | null
  is_extreme: ExtremeValue[]
}

export interface ExtremeValue {
  value: number
  kind: string
}

export interface TextStatsDetail {
  kind: 'Text'
  min_length: number
  max_length: number
  top_values: TextFrequency[]
}

export interface TextFrequency {
  value: string
  count: number
  ratio: number
}

export interface DateTimeStatsDetail {
  kind: 'DateTime'
  earliest: string
  latest: string
  span_days: number
  monthly_distribution: TextFrequency[]
}

export interface BooleanStatsDetail {
  kind: 'Boolean'
  true_count: number
  false_count: number
  true_ratio: number
}

export interface UnknownStatsDetail {
  kind: 'Unknown'
}

export interface DistributionBin {
  label: string
  count: number
  ratio: number
}

// ==================== API 方法 ====================

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

export async function getColumnInsightFull(
  tempTable: string,
  columnName: string
): Promise<ColumnInsightFull> {
  return invoke<ColumnInsightFull>('get_column_insight_full', {
    input: {
      temp_table: tempTable,
      column_name: columnName
    }
  })
}

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

// ═══════════════════ 持久化 API ═══════════════════

export interface SaveInsightSnapshotInput {
  temp_table: string
  column_name: string
  conn_id?: string
  db_name?: string
  schema_name?: string
  table_name?: string
}

export interface InsightVersionEntry {
  snapshot_id: string
  column_name: string
  data_type: string | null
  stats_json: string
  version_id: string
  parent_version_id: string | null
  checksum: string
  created_at: string
}

export async function saveColumnInsightSnapshot(
  input: SaveInsightSnapshotInput
): Promise<string> {
  return invoke<string>('save_column_insight_snapshot', {
    input
  })
}

export async function getColumnInsightHistory(
  columnName: string
): Promise<InsightVersionEntry[]> {
  return invoke<InsightVersionEntry[]>('get_column_insight_history', {
    input: {
      column_name: columnName
    }
  })
}
