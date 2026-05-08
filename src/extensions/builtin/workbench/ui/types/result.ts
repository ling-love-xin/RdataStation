export type FilterMode = 'quick' | 'sql' | 'duckdb'

export type ViewMode = 'grid' | 'text' | 'record' | 'chart'

export type ExportFormat = 'csv' | 'json' | 'insert' | 'xlsx' | 'parquet' | 'sql'

export interface QueryResult {
  columns: string[]
  rows: unknown[][]
  rowCount: number
  elapsedMs: number
  tempTable?: string
  affectedRows?: number
}

export interface ResultTab {
  readonly id: string
  title: string
  originalSql: string
  tableName: string
  connectionId: string
  duckdbTempTable: string
  isLoading: boolean
  columns: string[]
  rows: unknown[][]
  objectRows: Record<string, unknown>[]
  page: number
  pageSize: number
  originalRowCount: number
  displayedRowCount: number
  filterMode: FilterMode

  quickFilterExpression: string
  filteredRowCount: number

  sqlFilterExpression: string
  isSqlFilterLoading: boolean

  duckdbSql: string
  isDuckdbLoading: boolean
  isAnalysisActive: boolean

  executionTime: number
  timestamp: string

  dirtyRows: Set<number>
}

export interface ExecutionMeta {
  dbDuration: number
  duckdbDuration: number
  timestamp: string
}

export interface ResultData {
  columns: string[]
  rows: unknown[][]
}

export interface ColumnInsight {
  columnName: string
  dataType: string
  stats: ColumnStats
  histogram?: HistogramBucket[]
  sample: unknown[]
  quality: QualityScore
}

export interface ColumnStats {
  count: number
  nullCount: number
  distinctCount: number
  min?: number
  max?: number
  mean?: number
  median?: number
  stddev?: number
  skewness?: number
  kurtosis?: number
  minLength?: number
  maxLength?: number
  avgLength?: number
  mostFrequent?: Array<{ value: string; count: number }>
  minDate?: string
  maxDate?: string
  nullRatio: number
  distinctRatio: number
  emptyStringCount?: number
  zeroCount?: number
}

export interface HistogramBucket {
  min: number
  max: number
  count: number
  label: string
}

export interface QualityScore {
  overall: number
  grade: 'excellent' | 'good' | 'fair' | 'poor'
  dimensions: {
    completeness: number
    uniqueness: number
    typeConsistency: number
    distribution: number
  }
  summary: string
}

export interface TableProfile {
  tableName: string
  rowCount: number
  columns: TableColumnMeta[]
}

export interface TableColumnMeta {
  name: string
  dataType: string
  nullable: boolean
  isPrimaryKey: boolean
  defaultValue: string | null
  comment: string
}

export type MenuActionCategory = 'edit' | 'filter' | 'sort' | 'view' | 'export' | 'meta'

export interface MenuContext {
  value: unknown
  column: string
  columnIndex: number
  rowIndex: number
  tab: ResultTab
}

export interface ExportOptions {
  format: ExportFormat
  includeHeaders: boolean
  nullAs: string
  delimiter?: string
  quoteFields?: boolean
  prettyPrint?: boolean
  includeSchema?: boolean
}

export interface FilterPreset {
  id: string
  name: string
  filterMode: FilterMode
  expression: string
  createdAt: string
}

export interface CellUpdateInput {
  connId: string
  tableName: string
  columnName: string
  newValue: unknown
  rowIdentity: Record<string, unknown>
}

export interface CellUpdateResult {
  success: boolean
  affectedRows: number
  message: string
}

export type MultiRuleResult = Record<string, unknown>
