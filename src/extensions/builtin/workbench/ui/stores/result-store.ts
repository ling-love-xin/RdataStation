import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type FilterMode = 'quick' | 'sql' | 'duckdb'

export interface ResultData {
  columns: string[]
  rows: unknown[][]
}

export interface ExecutionMeta {
  dbDuration: number
  duckdbDuration: number
  timestamp: string
}

export interface ResultState {
  // 原始查询
  originalSql: string
  connectionId: string

  // DuckDB 临时表
  duckdbTempTable: string
  originalRowCount: number

  // 当前数据
  rowData: unknown[]
  columnDefs: string[]

  // 当前过滤模式
  filterMode: FilterMode

  // 模式1: 即时过滤
  quickFilterExpression: string

  // 模式2: SQL 过滤
  sqlFilterExpression: string
  isSqlFilterLoading: boolean

  // 模式3: DuckDB 分析
  duckdbSql: string
  isDuckdbLoading: boolean
  isAnalysisActive: boolean

  // 编辑状态
  dirtyRows: Set<number>

  // 性能
  executionTime: ExecutionMeta

  // 行数
  displayedRowCount: number
  filteredRowCount: number
}

export const useResultStore = defineStore('result', () => {
  const originalSql = ref('')
  const connectionId = ref('')
  const duckdbTempTable = ref('')
  const originalRowCount = ref(0)
  const rowData = ref<unknown[]>([])
  const columnDefs = ref<string[]>([])
  const filterMode = ref<FilterMode>('quick')
  const quickFilterExpression = ref('')
  const sqlFilterExpression = ref('')
  const isSqlFilterLoading = ref(false)
  const duckdbSql = ref('')
  const isDuckdbLoading = ref(false)
  const isAnalysisActive = ref(false)
  const dirtyRows = ref<Set<number>>(new Set())
  const executionTime = ref<ExecutionMeta>({ dbDuration: 0, duckdbDuration: 0, timestamp: '' })
  const displayedRowCount = ref(0)
  const filteredRowCount = ref(0)

  const hasDirtyRows = computed(() => dirtyRows.value.size > 0)
  const isQuickFilterActive = computed(() => quickFilterExpression.value.length > 0)

  function setResult(data: ResultData, meta: Partial<ExecutionMeta>) {
    columnDefs.value = data.columns
    rowData.value = data.rows
    originalRowCount.value = data.rows.length
    displayedRowCount.value = data.rows.length
    filteredRowCount.value = data.rows.length
    executionTime.value = { ...executionTime.value, ...meta }
    isAnalysisActive.value = false
    filterMode.value = 'quick'
    quickFilterExpression.value = ''
  }

  function setAnalysisResult(data: ResultData, meta: Partial<ExecutionMeta>) {
    columnDefs.value = data.columns
    rowData.value = data.rows
    displayedRowCount.value = data.rows.length
    executionTime.value = { ...executionTime.value, ...meta }
    isAnalysisActive.value = true
  }

  function setQuickFilter(expression: string) {
    quickFilterExpression.value = expression
  }

  function setSqlFilter(expression: string) {
    sqlFilterExpression.value = expression
  }

  function setDuckdbSql(sql: string) {
    duckdbSql.value = sql
  }

  function setTempTable(tableName: string) {
    duckdbTempTable.value = tableName
  }

  function resetToOriginal() {
    isAnalysisActive.value = false
    filterMode.value = 'quick'
    quickFilterExpression.value = ''
    sqlFilterExpression.value = ''
    duckdbSql.value = ''
  }

  function markDirty(rowIndex: number) {
    const newSet = new Set(dirtyRows.value)
    newSet.add(rowIndex)
    dirtyRows.value = newSet
  }

  function clearDirty() {
    dirtyRows.value = new Set()
  }

  return {
    originalSql, connectionId, duckdbTempTable, originalRowCount,
    rowData, columnDefs, filterMode,
    quickFilterExpression, sqlFilterExpression, isSqlFilterLoading,
    duckdbSql, isDuckdbLoading, isAnalysisActive,
    dirtyRows, executionTime, displayedRowCount, filteredRowCount,
    hasDirtyRows, isQuickFilterActive,
    setResult, setAnalysisResult, setQuickFilter, setSqlFilter,
    setDuckdbSql, setTempTable, resetToOriginal, markDirty, clearDirty,
  }
})
