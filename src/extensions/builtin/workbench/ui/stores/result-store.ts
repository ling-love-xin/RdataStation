import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import {
  reExecuteWithFilter,
  executeDuckdbAnalysis as executeDuckdbApi,
  createDuckdbTempTable,
} from '../services/result-analysis'

import type {
  FilterMode,
  QueryResult,
  ResultTab,
  ExportFormat,
  ExportOptions,
} from '../types/result'

let tabCounter = 0

export const useResultStore = defineStore('result', () => {
  const tabs = ref<ResultTab[]>([])
  const activeTabId = ref<string | null>(null)
  const showPanel = ref(false)

  const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value) ?? null)
  const isAnyLoading = computed(() =>
    tabs.value.some(t => t.isSqlFilterLoading || t.isDuckdbLoading)
  )

  const activeTabPagedRows = computed(() => {
    const tab = activeTab.value
    if (!tab) return []
    const start = tab.page * tab.pageSize
    const end = start + tab.pageSize
    return tab.objectRows.slice(start, end)
  })

  const activeTabTotalPages = computed(() => {
    const tab = activeTab.value
    if (!tab || tab.displayedRowCount === 0) return 0
    return Math.ceil(tab.displayedRowCount / tab.pageSize)
  })

  function createTab(sql: string, connectionId: string): ResultTab {
    tabCounter++
    const id = `result_${Date.now()}_${tabCounter}`
    return {
      id,
      title: `结果 #${tabCounter}`,
      originalSql: sql || '',
      tableName: '',
      connectionId: connectionId || '',
      duckdbTempTable: '',
      isLoading: false,
      columns: [],
      rows: [],
      objectRows: [],
      page: 0,
      pageSize: 100,
      originalRowCount: 0,
      displayedRowCount: 0,
      filterMode: 'quick',
      quickFilterExpression: '',
      filteredRowCount: 0,
      sqlFilterExpression: '',
      isSqlFilterLoading: false,
      duckdbSql: '',
      isDuckdbLoading: false,
      isAnalysisActive: false,
      executionTime: 0,
      timestamp: '',
      dirtyRows: new Set(),
    }
  }

  function addTab(sql: string, connectionId: string): ResultTab {
    const tab = createTab(sql, connectionId)
    tabs.value.push(tab)
    activeTabId.value = tab.id
    showPanel.value = true
    return tab
  }

  function setTabResult(id: string, result: QueryResult): void {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab) return

    tab.columns = result.columns
    tab.rows = result.rows
    tab.objectRows = convertRowsToObjects(result.columns, result.rows)
    tab.tableName = extractTableName(tab.originalSql, tab.columns[0] ?? '')
    tab.page = 0
    tab.originalRowCount = result.rowCount
    tab.displayedRowCount = result.rowCount
    tab.executionTime = result.elapsedMs
    tab.timestamp = new Date().toLocaleString()
    tab.filterMode = 'quick'
    tab.quickFilterExpression = ''
    tab.isAnalysisActive = false

    if (result.tempTable) {
      tab.duckdbTempTable = result.tempTable
    }

    showPanel.value = true
  }

  function convertRowsToObjects(columns: string[], rows: unknown[][]): Record<string, unknown>[] {
    return rows.map(row => {
      const obj: Record<string, unknown> = {}
      columns.forEach((col, i) => {
        obj[col.replace(/\./g, '_')] = row[i]
      })
      return obj
    })
  }

  function closeTab(id: string): void {
    const idx = tabs.value.findIndex(t => t.id === id)
    if (idx === -1) return
    tabs.value.splice(idx, 1)
    if (activeTabId.value === id) {
      activeTabId.value = tabs.value[idx]?.id || tabs.value[idx - 1]?.id || null
    }
    if (tabs.value.length === 0) {
      showPanel.value = false
    }
  }

  function switchTab(id: string): void {
    activeTabId.value = id
  }

  function closeAllTabs(): void {
    tabs.value = []
    activeTabId.value = null
    showPanel.value = false
  }

  function removeTabResult(id: string): void {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab) return
    tab.columns = []
    tab.rows = []
    tab.originalRowCount = 0
    tab.displayedRowCount = 0
    tab.duckdbTempTable = ''
  }

  function setFilterMode(id: string, mode: FilterMode): void {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab) return
    tab.filterMode = mode
  }

  function applyQuickFilter(id: string, expression: string): void {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab) return
    tab.quickFilterExpression = expression
  }

  function clearFilter(id: string): void {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab) return
    tab.filterMode = 'quick'
    tab.quickFilterExpression = ''
    tab.sqlFilterExpression = ''
    tab.duckdbSql = ''
    tab.isAnalysisActive = false
    tab.displayedRowCount = tab.originalRowCount
    tab.filteredRowCount = tab.originalRowCount
  }

  async function executeSqlFilter(id: string, whereClause: string): Promise<void> {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab || !whereClause.trim()) return

    tab.isSqlFilterLoading = true
    try {
      const result = await reExecuteWithFilter(tab.connectionId, tab.originalSql, whereClause)
      tab.columns = result.columns
      tab.rows = result.rows
      tab.objectRows = convertRowsToObjects(result.columns, result.rows)
      tab.originalRowCount = result.rows.length
      tab.displayedRowCount = result.rows.length
      tab.executionTime = result.elapsed_ms
      if (result.temp_table) tab.duckdbTempTable = result.temp_table
    } finally {
      tab.isSqlFilterLoading = false
    }
  }

  async function executeDuckdbAnalysis(id: string, duckSql: string): Promise<void> {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab || !duckSql.trim()) return

    tab.isDuckdbLoading = true
    try {
      const hasTempTable = !!tab.duckdbTempTable
      const result = await executeDuckdbApi(
        tab.duckdbTempTable,
        duckSql,
        hasTempTable ? undefined : tab.columns,
        hasTempTable ? undefined : (tab.rows as unknown[][])
      )
      tab.columns = result.columns
      tab.rows = result.rows
      tab.objectRows = convertRowsToObjects(result.columns, result.rows)
      tab.displayedRowCount = result.rows.length
      tab.executionTime = result.elapsed_ms
      tab.isAnalysisActive = true
      tab.timestamp = new Date().toLocaleString()
    } finally {
      tab.isDuckdbLoading = false
    }
  }

  async function bridgeFilterFromDuckdb(
    id: string,
    visibleRows: Record<string, unknown>[]
  ): Promise<void> {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab || visibleRows.length === 0) return

    tab.isDuckdbLoading = true
    try {
      const rowsData: unknown[][] = visibleRows.map(row =>
        tab.columns.map(col => (row[col] ?? null) as unknown)
      )
      const tableName = await createDuckdbTempTable(tab.columns, rowsData)
      tab.duckdbTempTable = tableName
      tab.duckdbSql = `SELECT * FROM ${tableName} LIMIT 100`
    } finally {
      tab.isDuckdbLoading = false
    }
  }

  async function ensureDuckdbTable(id: string): Promise<void> {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab || tab.duckdbTempTable || tab.columns.length === 0 || tab.rows.length === 0) return

    try {
      const tableName = await createDuckdbTempTable(tab.columns, tab.rows)
      if (tableName) tab.duckdbTempTable = tableName
    } catch {
      /* silent */
    }
  }

  async function exportTab(
    id: string,
    format: ExportFormat,
    _options?: ExportOptions
  ): Promise<void> {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab) return

    if (format === 'csv') {
      const escapeCsv = (v: unknown): string => {
        if (v === null || v === undefined) return ''
        const s = String(v)
        if (s.includes(',') || s.includes('"') || s.includes('\n') || s.includes('\r')) {
          return `"${s.replace(/"/g, '""')}"`
        }
        return s
      }
      const header = tab.columns.map(c => escapeCsv(c)).join(',')
      const body = tab.rows.map(r => r.map(v => escapeCsv(v)).join(','))
      const csv = [header, ...body].join('\n')
      await navigator.clipboard.writeText(csv)
    } else if (format === 'json') {
      const data = tab.rows.map(r => {
        const obj: Record<string, unknown> = {}
        tab.columns.forEach((c, i) => {
          obj[c] = r[i]
        })
        return obj
      })
      await navigator.clipboard.writeText(JSON.stringify(data, null, 2))
    } else if (format === 'insert') {
      const tableName = tab.tableName || 'table_name'
      const colList = tab.columns.map(c => `\`${c}\``).join(', ')
      const values = tab.objectRows.map(row => {
        const vals = tab.columns.map(col => {
          const key = col.replace(/\./g, '_')
          const val = (row as Record<string, unknown>)[key]
          if (val === null || val === undefined) return 'NULL'
          if (typeof val === 'boolean') return val ? 'TRUE' : 'FALSE'
          if (typeof val === 'number') return String(val)
          return `'${String(val).replace(/'/g, "''")}'`
        })
        return `(${vals.join(', ')})`
      })
      const sql = `INSERT INTO \`${tableName}\` (${colList}) VALUES\n${values.join(',\n')};`
      await navigator.clipboard.writeText(sql)
    }
  }

  function markCellDirty(tabId: string, rowIndex: number): void {
    const tab = tabs.value.find(t => t.id === tabId)
    if (!tab) return
    tab.dirtyRows.add(rowIndex)
  }

  function resetDirtyCells(tabId: string): void {
    const tab = tabs.value.find(t => t.id === tabId)
    if (tab) tab.dirtyRows.clear()
  }

  function setPage(id: string, page: number): void {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab) return
    const maxPage = Math.max(0, Math.ceil(tab.displayedRowCount / tab.pageSize) - 1)
    tab.page = Math.max(0, Math.min(page, maxPage))
  }

  function setPageSize(id: string, size: number): void {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab || size < 10) return
    tab.pageSize = size
    tab.page = 0
  }

  function nextPage(id: string): void {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab) return
    const maxPage = Math.max(0, Math.ceil(tab.displayedRowCount / tab.pageSize) - 1)
    if (tab.page < maxPage) {
      tab.page++
    }
  }

  function prevPage(id: string): void {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab) return
    if (tab.page > 0) {
      tab.page--
    }
  }

  function extractTableName(sql: string, fallbackColumn: string): string {
    try {
      const fromMatch = sql.match(/\bFROM\s+`?(\w+)`?\s*(?:AS\s+\w+)?/i)
      if (fromMatch) return fromMatch[1]
      const joinMatch = sql.match(/\bJOIN\s+`?(\w+)`?\s/i)
      if (joinMatch) return joinMatch[1]
    } catch {
      // 正则失败则回退
    }
    return fallbackColumn ? `_result_${fallbackColumn}` : '_unknown'
  }

  async function saveCellUpdate(
    tabId: string,
    columnName: string,
    newValue: unknown,
    rowIndex: number
  ): Promise<boolean> {
    const tab = tabs.value.find(t => t.id === tabId)
    if (!tab || !tab.tableName || !tab.connectionId) return false

    const oldRow = tab.objectRows[rowIndex]
    if (!oldRow) return false

    const rowIdentity: Record<string, unknown> = {}
    for (const col of tab.columns) {
      const key = col.replace(/\./g, '_')
      if (key !== columnName.replace(/\./g, '_')) {
        rowIdentity[col] = (oldRow as Record<string, unknown>)[key] ?? null
      }
    }

    try {
      const { saveCellUpdate: apiSaveCellUpdate } = await import('../services/result-analysis')
      const result = await apiSaveCellUpdate({
        conn_id: tab.connectionId,
        table_name: tab.tableName,
        column_name: columnName,
        new_value: newValue,
        row_identity: rowIdentity,
      })

      if (result.success) {
        const fieldKey = columnName.replace(/\./g, '_')
        const newObjRow = { ...tab.objectRows[rowIndex], [fieldKey]: newValue }
        tab.objectRows[rowIndex] = newObjRow
        tab.dirtyRows.add(rowIndex)
        return true
      }
      return false
    } catch {
      return false
    }
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    showPanel,
    isAnyLoading,
    activeTabPagedRows,
    activeTabTotalPages,
    addTab,
    setTabResult,
    closeTab,
    switchTab,
    closeAllTabs,
    removeTabResult,
    setFilterMode,
    applyQuickFilter,
    clearFilter,
    executeSqlFilter,
    executeDuckdbAnalysis,
    bridgeFilterFromDuckdb,
    ensureDuckdbTable,
    exportTab,
    markCellDirty,
    resetDirtyCells,
    setPage,
    setPageSize,
    nextPage,
    prevPage,
    saveCellUpdate,
  }
})
