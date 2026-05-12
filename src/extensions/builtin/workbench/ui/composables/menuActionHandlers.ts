import type { ResultTab } from '@/extensions/builtin/workbench/ui/types/result'

import type { GridApi } from 'ag-grid-community'

export interface MenuActionContext {
  tab: ResultTab
  column: string
  colId: string
  value: unknown
  gridApi: GridApi | null
  callbacks: MenuCallbacks
}

export interface MenuCallbacks {
  onExport: (format: string) => void
  onCopyCell: (value: unknown) => void
  onOpenInsight: (tab: ResultTab, column: string) => void
  onApplyQuickFilter: (tab: ResultTab, expression: string) => void
  onShowValueViewer: (value: unknown, column: string, rowIndex: number) => void
  onMessage: (text: string) => void
}

type MenuActionHandler = (ctx: MenuActionContext) => void

const menuActions: Record<string, MenuActionHandler> = {
  copyCellValue(ctx) {
    ctx.callbacks.onCopyCell(ctx.value)
  },

  copyRow(ctx) {
    const tab = ctx.tab
    let text = ''
    if (ctx.gridApi) {
      const selected = ctx.gridApi.getSelectedRows()
      if (selected.length > 0) {
        text = selected
          .map((r: Record<string, unknown>) => tab.columns.map(c => String(r[c] ?? '')).join('\t'))
          .join('\n')
      }
    }
    if (!text && ctx.value) {
      text = String(ctx.value)
    }
    if (text) navigator.clipboard.writeText(text)
  },

  copyTableHeader(ctx) {
    const tab = ctx.tab
    const header = tab.columns.map(c => `"${c.replace(/"/g, '""')}"`).join(',')
    const rows = ctx.gridApi?.getSelectedRows() || []
    const body = rows
      .map((r: Record<string, unknown>) => tab.columns.map(c => String(r[c] ?? '')).join('\t'))
      .join('\n')
    navigator.clipboard.writeText(header + '\n' + body)
  },

  exportCsv(ctx) {
    ctx.callbacks.onExport('csv')
  },

  exportJson(ctx) {
    ctx.callbacks.onExport('json')
  },

  exportInsert(ctx) {
    ctx.callbacks.onExport('insert')
  },

  openColumnInsights(ctx) {
    ctx.callbacks.onOpenInsight(ctx.tab, ctx.column)
  },

  quickFilterEquals(ctx) {
    const value = ctx.value
    if (value === null) {
      ctx.callbacks.onApplyQuickFilter(ctx.tab, `${ctx.column} IS NULL`)
    } else if (typeof value === 'string') {
      ctx.callbacks.onApplyQuickFilter(ctx.tab, `${ctx.column} = '${value.replace(/'/g, "''")}'`)
    } else {
      ctx.callbacks.onApplyQuickFilter(ctx.tab, `${ctx.column} = ${value}`)
    }
  },

  quickFilterNotEquals(ctx) {
    const value = ctx.value
    if (value === null) {
      ctx.callbacks.onApplyQuickFilter(ctx.tab, `${ctx.column} IS NOT NULL`)
    } else if (typeof value === 'string') {
      ctx.callbacks.onApplyQuickFilter(ctx.tab, `${ctx.column} != '${value.replace(/'/g, "''")}'`)
    } else {
      ctx.callbacks.onApplyQuickFilter(ctx.tab, `${ctx.column} != ${value}`)
    }
  },

  sortAsc(ctx) {
    if (ctx.gridApi) {
      const api = ctx.gridApi as unknown as {
        applySortState: (s: Array<{ colId: string; sort: string }>) => void
      }
      api.applySortState([{ colId: ctx.colId, sort: 'asc' }])
    }
  },

  sortDesc(ctx) {
    if (ctx.gridApi) {
      const api = ctx.gridApi as unknown as {
        applySortState: (s: Array<{ colId: string; sort: string }>) => void
      }
      api.applySortState([{ colId: ctx.colId, sort: 'desc' }])
    }
  },

  showValue(ctx) {
    ctx.callbacks.onShowValueViewer(ctx.value, ctx.column, -1)
  },

  autoResizeColumns(ctx) {
    if (ctx.gridApi) {
      const allCols: string[] = []
      const columns = ctx.gridApi.getColumns()
      if (columns) {
        columns.forEach(c => {
          if (c.getColDef().field !== '__rowNumber') allCols.push(c.getId())
        })
      }
      if (allCols.length > 0) ctx.gridApi.autoSizeColumns(allCols)
    }
  },
}

export function dispatchMenuAction(actionKey: string, ctx: MenuActionContext): void {
  const handler = menuActions[actionKey]
  if (handler) {
    handler(ctx)
  }
}
