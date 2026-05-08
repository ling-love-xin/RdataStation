import { computed, shallowRef, watch, type ComputedRef, type Ref } from 'vue'

import type { ResultTab } from '../types/result'
import type { ColDef } from '@ag-grid-community/core'

export function useGridConfig(activeTab: ComputedRef<ResultTab | null>, isDark: Ref<boolean>) {
  const columnDefs = shallowRef<ColDef[]>([])

  let columnsKey = ''

  function buildColumnDefs(tab: ResultTab): ColDef[] {
    const rowNumberCol: ColDef = {
      headerName: '#',
      field: '__rowNumber',
      width: 70,
      sortable: false,
      filter: false,
      cellStyle: { color: '#999' },
    }

    const dataCols: ColDef[] = tab.columns.map((col: string) => {
      const field = col.replace(/\./g, '_')
      return {
        field,
        headerName: col,
        headerTooltip: col,
        sortable: true,
        filter: true,
        resizable: true,
        minWidth: 80,
        flex: 1,
        cellRendererSelector: (params: { value: unknown }) => {
          if (params.value === null || params.value === undefined) {
            return { component: 'nullCellRenderer' }
          }
          return undefined
        },
        cellStyle: (params: { value: unknown }) => {
          if (params.value === null || params.value === undefined) {
            return { color: '#999', fontStyle: 'italic' }
          }
          return undefined
        },
      }
    })

    return [rowNumberCol, ...dataCols]
  }

  function refreshColumns(tab: ResultTab | null): void {
    if (!tab || tab.columns.length === 0) {
      const key = '__empty'
      if (columnsKey !== key) {
        columnsKey = key
        columnDefs.value = [
          {
            headerName: '#',
            field: '__rowNumber',
            width: 70,
            sortable: false,
            filter: false,
          },
        ]
      }
      return
    }

    const key = tab.columns.join('|')
    if (columnsKey !== key) {
      columnsKey = key
      columnDefs.value = buildColumnDefs(tab)
    }
  }

  watch(
    () => {
      const tab = activeTab.value
      if (!tab) return '__none'
      return tab.columns.join('|')
    },
    key => {
      if (key !== columnsKey && key !== '__none') {
        refreshColumns(activeTab.value)
      }
    },
    { immediate: true }
  )

  const defaultColDef: ColDef = {
    sortable: true,
    resizable: true,
  }

  const pagination = computed(() => {
    const tab = activeTab.value
    if (!tab) return false
    return tab.displayedRowCount > 100 ? ('bottom' as const) : false
  })

  const rowData = computed<Record<string, unknown>[]>(() => {
    const tab = activeTab.value
    if (!tab) return []
    return tab.objectRows
  })

  const paginationPageSize = computed(() => {
    const tab = activeTab.value
    if (!tab) return 100
    return tab.pageSize
  })

  const COLUMN_STATE_PREFIX = 'rds_grid_cols_'

  function saveColumnState(tabId: string, api: unknown): void {
    try {
      const gridApi = api as {
        getColumnState: () => unknown[]
      }
      const state = gridApi.getColumnState()
      const key = `${COLUMN_STATE_PREFIX}${tabId}`
      localStorage.setItem(key, JSON.stringify(state))
    } catch {
      // localStorage may be unavailable
    }
  }

  function restoreColumnState(tabId: string, api: unknown): void {
    try {
      const key = `${COLUMN_STATE_PREFIX}${tabId}`
      const raw = localStorage.getItem(key)
      if (!raw) return
      const state: unknown[] = JSON.parse(raw)
      if (!Array.isArray(state) || state.length === 0) return
      const gridApi = api as {
        applyColumnState: (params: { state: unknown[]; applyOrder: boolean }) => void
      }
      gridApi.applyColumnState({ state, applyOrder: true })
    } catch {
      // Silently ignore
    }
  }

  function clearColumnState(tabId: string): void {
    try {
      const key = `${COLUMN_STATE_PREFIX}${tabId}`
      localStorage.removeItem(key)
    } catch {
      // localStorage may be unavailable
    }
  }

  return {
    columnDefs,
    defaultColDef,
    pagination,
    paginationPageSize,
    rowData,
    saveColumnState,
    restoreColumnState,
    clearColumnState,
  }
}
