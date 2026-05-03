<template>
  <div class="query-result-panel" :class="{ compact: compact }">
    <!-- 顶部标签栏 -->
    <div v-if="resultTabs.length > 0" class="result-tabs">
      <div
        v-for="tab in resultTabs"
        :key="tab.id"
        :class="['result-tab', { active: tab.id === activeTabId }]"
        @click="switchTab(tab.id)"
      >
        <span class="tab-title">{{ tab.title }}</span>
        <span class="tab-close" @click.stop="closeTab(tab.id)">&times;</span>
      </div>
    </div>

    <!-- 主内容区 -->
    <template v-if="activeTab">
      <!-- SQL 预览 + 模式切换条 -->
      <div class="toolbar-strip">
        <FilterModeSwitcher v-model="activeTab.filterMode" class="mode-switcher-inline" />
        <div class="strip-right">
          <QuickFilterInput
            v-if="activeTab.filterMode === 'quick'"
            :expression="activeTab.quickFilterExpression"
            :visible-count="activeTab.filteredRowCount"
            :total-count="activeTab.originalRowCount"
            @update:expression="(v: string) => activeTab.quickFilterExpression = v"
            @apply="(v: string) => applyQuickFilter(activeTab, v)"
            @clear="() => clearQuickFilter(activeTab)"
          />
          <SqlFilterInput
            v-if="activeTab.filterMode === 'sql'"
            :expression="activeTab.sqlFilterExpression"
            :loading="activeTab.isSqlFilterLoading"
            @update:expression="(v: string) => activeTab.sqlFilterExpression = v"
            @execute="() => executeSqlFilter(activeTab)"
          />
          <DuckDBAnalysisInput
            v-if="activeTab.filterMode === 'duckdb'"
            :sql="activeTab.duckdbSql"
            :loading="activeTab.isDuckdbLoading"
            @update:sql="(v: string) => activeTab.duckdbSql = v"
            @execute="() => executeDuckdbAnalysis(activeTab)"
            @clear="() => clearDuckdbAnalysis(activeTab)"
            @quick="(t: string) => quickDuckdbAction(activeTab, t)"
            @bridge-filter="() => handleBridgeFilter(activeTab)"
          />
        </div>
      </div>

      <!-- DBeaver 风格主体布局：左侧栏 + 内容区 + 右侧查看器 -->
      <div class="result-body">
        <!-- 左侧视图切换栏 -->
        <div class="view-sidebar">
          <button
            v-for="v in viewModes"
            :key="v.key"
            :class="['view-btn', { active: currentView === v.key }]"
            :title="v.label"
            @click="switchView(v.key)"
          >
            <component :is="v.icon" :size="18" />
          </button>
        </div>

        <!-- 中间表格区 -->
        <div ref="gridContainerRef" class="grid-area" @contextmenu.prevent="handleGridContextMenu">
          <div v-if="!(rowData.length > 0 && columnDefs.length > 0)" class="grid-empty">
            <div class="empty-icon"><Database :size="32" /></div>
            <div class="empty-text">{{ emptyText }}</div>
          </div>
          <div v-show="currentView === 'grid'" class="grid-fill">
            <AgGridVue
              v-if="activeTab"
              :key="activeTab.id + '_grid'"
              :class="gridThemeClass"
              :column-defs="columnDefs"
              :row-data="rowData"
              :default-col-def="defaultColDef"
              :pagination="pagination"
              :pagination-page-size="pageSize"
              :pagination-page-selector="[50, 100, 200, 500]"
              :enable-cell-text-selection="true"
              :row-selection="'multiple'"
              :suppress-row-click-selection="true"
              :animate-rows="true"
              :header-height="24"
              :row-height="22"
              :column-virtualisation="true"
              :row-buffer="20"
              :block-load-debounce-ms="50"
              :single-click-edit="false"
              :stop-editing-when-cells-lose-focus="true"
              :dom-layout="'normal'"
              :overlay-no-rows-template="''"
              :tooltip-show-delay="300"
              style="height: 100%; width: 100%;"
              @grid-ready="onGridReady"
              @cell-context-menu="onCellContextMenu"
              @row-clicked="onRowClicked"
              @selection-changed="onSelectionChanged"
              @cell-value-changed="onCellValueChanged"
              @first-data-rendered="onFirstDataRendered"
              @row-data-updated="onRowDataUpdated"
              @sort-changed="onSortChanged"
              @keydown="handleKeyDown"
            />
          </div>
          <!-- 文本视图 -->
          <div v-if="currentView === 'text'" class="text-view">
            <textarea :value="textViewContent" readonly class="text-view-area"></textarea>
          </div>
          <!-- 记录视图 -->
          <div v-if="currentView === 'record'" class="record-view">
            <div class="record-nav">
              <NButton size="tiny" quaternary :disabled="selectedRecordIndex <= 0" @click="prevRecord">
                <ChevronLeft :size="14" />
              </NButton>
              <span>{{ selectedRecordIndex + 1 }} / {{ rowData.length }}</span>
              <NButton size="tiny" quaternary :disabled="selectedRecordIndex >= rowData.length - 1" @click="nextRecord">
                <ChevronRight :size="14" />
              </NButton>
            </div>
            <div class="record-fields">
              <div v-for="col in activeTab.columns" :key="col" class="record-field">
                <span class="field-name">{{ col }}</span>
                <span class="field-value">{{ formatCellValue(rowData[selectedRecordIndex]?.[col]) }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 右侧数值查看器 -->
        <div v-if="showValueViewer" class="value-viewer">
          <div class="viewer-header">
            <span class="viewer-title">值查看器</span>
            <NButton size="tiny" quaternary @click="showValueViewer = false">
              <X :size="12" />
            </NButton>
          </div>
          <div class="viewer-content">
            <div class="viewer-field">
              <span class="field-label">列</span>
              <span class="field-val">{{ selectedCell?.column || '-' }}</span>
            </div>
            <div class="viewer-field">
              <span class="field-label">行</span>
              <span class="field-val">{{ selectedCell?.row != null ? selectedCell.row + 1 : '-' }}</span>
            </div>
            <textarea
              class="viewer-text"
              :value="selectedCell?.value != null ? String(selectedCell.value) : ''"
              readonly
              rows="8"
            />
          </div>
        </div>
        <NButton
          v-if="!showValueViewer && activeTab"
          size="tiny"
          quaternary
          class="viewer-toggle"
          title="打开值查看器"
          @click="showValueViewer = true"
        >
          <PanelRight :size="14" />
        </NButton>
      </div>

      <!-- 底部状态栏（操作 + 行信息 + 分页） -->
      <div class="result-statusbar">
        <div class="sbar-left">
          <span :class="['mode-badge', activeTab.filterMode]">{{ modeLabel(activeTab) }}</span>
          <NButton size="tiny" quaternary title="刷新" @click="handleRefresh(activeTab)">
            <RotateCw :size="11" />
          </NButton>
          <NButton size="tiny" quaternary :disabled="!tabHasDirty(activeTab)" title="保存" @click="handleSave(activeTab)">
            <Save :size="11" />
          </NButton>
          <NButton size="tiny" quaternary :disabled="!tabHasDirty(activeTab)" title="取消" @click="handleCancel(activeTab)">
            <X :size="11" />
          </NButton>
          <NDropdown trigger="hover" :options="exportMenuOptions" @select="(k: string) => handleExport(k)">
            <NButton size="tiny" quaternary title="导出">
              <Download :size="11" />
            </NButton>
          </NDropdown>
        </div>
        <div class="sbar-center">
          <span class="row-info">{{ displayRowText }}</span>
          <span v-if="activeTab.executionTime" class="exec-time">{{ (activeTab.executionTime / 1000).toFixed(3) }}s</span>
        </div>
        <div class="sbar-right">
          <NButton size="tiny" quaternary :disabled="!gridApi" title="第一页" @click="firstPage">
            <SkipBack :size="11" />
          </NButton>
          <NButton size="tiny" quaternary :disabled="!gridApi" title="上一页" @click="prevPage">
            <ChevronLeft :size="11" />
          </NButton>
          <span v-if="gridApi" class="page-indicator">{{ pageInfoText }}</span>
          <NButton size="tiny" quaternary :disabled="!gridApi" title="下一页" @click="nextPage">
            <ChevronRight :size="11" />
          </NButton>
          <NButton size="tiny" quaternary :disabled="!gridApi" title="最后一页" @click="lastPage">
            <SkipForward :size="11" />
          </NButton>
        </div>
      </div>
    </template>

    <ResultContextMenu
      :visible="contextMenu.visible"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :type="contextMenu.type"
      :value="contextMenu.value"
      :column="contextMenu.column"
      :sort-dir="contextMenu.sortDir"
      @action="handleContextAction"
      @close="closeContextMenu"
    />
  </div>
</template>

<script setup lang="ts">
import { ClientSideRowModelModule } from '@ag-grid-community/client-side-row-model'
import { ModuleRegistry } from '@ag-grid-community/core'
import { AgGridVue } from '@ag-grid-community/vue3'
import { Database, RotateCw, Save, X, Download, PanelRight, ChevronLeft, ChevronRight, SkipBack, SkipForward, AlignLeft, List } from 'lucide-vue-next'
import { createDiscreteApi, darkTheme, lightTheme, NButton, NDropdown } from 'naive-ui'
import { computed, ref, onMounted, onUnmounted, watch, reactive } from 'vue'

ModuleRegistry.registerModules([ClientSideRowModelModule])
import '@ag-grid-community/styles/ag-grid.css'
import '@ag-grid-community/styles/ag-theme-alpine.css'

const props = defineProps({
   compact: { type: Boolean, default: true },
   resultData: { type: Object, default: null }
 })

 watch(() => (props as any).resultData, (val: any) => {
   if (val && val.columns && val.rows) {
     const tab = addResultTab({
       columns: val.columns,
       rows: val.rows,
       originalSql: val.originalSql || '',
       connectionId: val.connectionId || '',
       elapsedMs: val.elapsedMs || 0
     })
     if (val.connectionId) tab.connectionId = val.connectionId
     if (val.originalSql) tab.originalSql = val.originalSql
   }
 }, { immediate: false })

import { useUiStore } from '@/shared/stores/ui'


import DuckDBAnalysisInput from './result-panel/DuckDBAnalysisInput.vue'
import FilterModeSwitcher from './result-panel/FilterModeSwitcher.vue'
import QuickFilterInput from './result-panel/QuickFilterInput.vue'
import ResultContextMenu from './result-panel/ResultContextMenu.vue'
import SqlFilterInput from './result-panel/SqlFilterInput.vue'
import {
  reExecuteWithFilter as apiExecuteWithFilter,
  executeDuckdbAnalysis as apiDuckdbAnalysis,
  createDuckdbTempTable as apiCreateTempTable,
} from '../../services/result-analysis'

interface ResultTab {
  id: string
  title: string
  originalSql: string
  connectionId: string
  duckdbTempTable: string
  columns: string[]
  rows: unknown[][]
  originalRowCount: number
  displayedRowCount: number
  filterMode: 'quick' | 'sql' | 'duckdb'
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

// ─── Store ───────────────────────────────────────────────
const uiStore = useUiStore()
const configProviderPropsRef = ref({ theme: uiStore.isDark ? darkTheme : lightTheme })
const { message } = createDiscreteApi(['message'], { configProviderProps: configProviderPropsRef })
watch(() => uiStore.isDark, (v) => { configProviderPropsRef.value = { theme: v ? darkTheme : lightTheme } })

// ─── 多标签状态 ──────────────────────────────────────────
const resultTabs = ref<ResultTab[]>([])
const activeTabId = ref<string | null>(null)
const activeTab = computed(() => resultTabs.value.find(t => t.id === activeTabId.value) || null)
const tabCounter = ref(0)

// ─── AG Grid ─────────────────────────────────────────────
const gridApi = ref<any>(null)
const gridContainerRef = ref<HTMLElement | null>(null)
const pageSize = ref(100)
const selectedRows = ref<unknown[]>([])

const contextMenu = ref({
  visible: false, x: 0, y: 0, type: 'cell' as 'cell' | 'header',
  value: null as any, column: '', sortDir: ''
})

// ─── Grid / Text / Record 视图切换 ──────────────────
type ViewMode = 'grid' | 'text' | 'record'
const currentView = ref<ViewMode>('grid')
const showValueViewer = ref(false)
const selectedRecordIndex = ref(0)
interface SelectedCell { column: string; row: number; value: any }
const selectedCell = ref<SelectedCell | null>(null)

const viewModes = [
  { key: 'grid' as ViewMode, icon: Database, label: '网格视图' },
  { key: 'text' as ViewMode, icon: AlignLeft, label: '文本视图' },
  { key: 'record' as ViewMode, icon: List, label: '记录视图' },
]

function switchView(mode: ViewMode) {
  currentView.value = mode
  if (mode === 'record' && selectedRecordIndex.value >= rowData.value.length) {
    selectedRecordIndex.value = 0
  }
}

function formatCellValue(val: any): string {
  if (val === null || val === undefined) return 'NULL'
  if (typeof val === 'object') { try { return JSON.stringify(val) } catch { return String(val) } }
  return String(val)
}

function prevRecord() { if (selectedRecordIndex.value > 0) selectedRecordIndex.value-- }
function nextRecord() { if (selectedRecordIndex.value < rowData.value.length - 1) selectedRecordIndex.value++ }

const textViewContent = computed(() => {
  if (!activeTab.value) return ''
  const cols = activeTab.value.columns
  return rowData.value.map((row: any, i: number) =>
    `[${i + 1}]\t${cols.map(c => String(row[c] ?? 'NULL')).join('\t')}`
  ).join('\n')
})

function firstPage() { gridApi.value?.paginationGoToFirstPage() }
function prevPage() { gridApi.value?.paginationGoToPreviousPage() }
function nextPage() { gridApi.value?.paginationGoToNextPage() }
function lastPage() { gridApi.value?.paginationGoToLastPage() }

function onFirstDataRendered(params: any) {
  params.api?.sizeColumnsToFit()
}

const displayRowText = computed(() => {
  if (!activeTab.value) return ''
  if (activeTab.value.filterMode === 'quick' && activeTab.value.filteredRowCount !== activeTab.value.originalRowCount) {
    return `${activeTab.value.originalRowCount} → ${activeTab.value.filteredRowCount} 行`
  }
  return `${activeTab.value.displayedRowCount} 行`
})

// ─── 导出菜单 ────────────────────────────────────────────
const exportMenuOptions = [
  { key: 'csv', label: '导出 CSV' },
  { key: 'json', label: '导出 JSON' },
  { key: 'insert', label: '导出 INSERT' },
]

// ─── 列定义 - 智能分辨 ───────────────────────────────────
const numericColPatterns = ['id', '_id', 'count', 'num', 'year', 'age', 'price', 'amount', 'total', 'qty', 'rate']

function isLikelyNumeric(colName: string): boolean {
  const lower = colName.toLowerCase()
  return numericColPatterns.some(p => lower.includes(p) || lower.endsWith(p))
}

function isLikelyDate(colName: string): boolean {
  const lower = colName.toLowerCase()
  return lower.includes('date') || lower.includes('time') || lower.endsWith('_at')
}

function isLikelyLongText(colName: string): boolean {
  const lower = colName.toLowerCase()
  return lower.includes('description') || lower.includes('content') || lower.includes('comment') || lower.includes('note') || lower.includes('text')
}

const columnDefs = computed(() => {
  if (!activeTab.value || activeTab.value.columns.length === 0) return [{ field: '__placeholder', headerName: '', hide: true }]
  const cols = activeTab.value.columns.map(col => ({
    field: col,
    headerName: col,
    headerTooltip: col,
    sortable: true,
    filter: true,
    resizable: true,
    minWidth: 80,
    width: isLikelyNumeric(col) ? 110 : isLikelyDate(col) ? 140 : isLikelyLongText(col) ? 200 : 130,
    flex: isLikelyLongText(col) ? 2 : isLikelyNumeric(col) ? 0 : 1,
    cellClass: isLikelyNumeric(col) ? 'text-right' : undefined,
    editable: true,
    cellRenderer: (params: any) => {
      const v = params.value
      if (v === null || v === undefined) return '<span class="null-value">NULL</span>'
      if (typeof v === 'object') {
        try { return JSON.stringify(v) } catch { return String(v) }
      }
      const str = String(v)
      if (str.length > 500) return str.substring(0, 200) + '...'
      return str
    },
    comparator: (a: any, b: any) => {
      if (a === null && b === null) return 0
      if (a === null) return 1; if (b === null) return -1
      if (typeof a === 'number' && typeof b === 'number') return a - b
      return String(a).localeCompare(String(b))
    }
  }))
  return [{
    field: '__rowNumber', headerName: '#', width: 55, pinned: 'left',
    sortable: false, filter: false, resizable: false,
    valueGetter: (p: any) => {
      const api = p.api
      if (!api || !api.paginationGetCurrentPage) return p.node.rowIndex + 1
      return api.paginationGetCurrentPage() * api.paginationGetPageSize() + p.node.rowIndex + 1
    },
    cellStyle: { textAlign: 'center', color: 'var(--text-tertiary)', fontSize: '11px', background: 'var(--bg-secondary)' }
  }, ...cols]
})

const rowData = computed(() => {
  if (!activeTab.value) return []
  return (activeTab.value.rows as any[]).map((row: any) => {
    if (Array.isArray(row)) {
      const obj: Record<string, unknown> = {}
      activeTab.value.columns.forEach((col, i) => { obj[col] = row[i] })
      return obj
    }
    return row
  })
})

const defaultColDef = {
  editable: true, sortable: true, filter: true, resizable: true,
  suppressMenu: false,
  filterParams: { maxNumConditions: 1 }
}

const gridThemeClass = computed(() => uiStore.isDark ? 'ag-theme-alpine-dark' : 'ag-theme-alpine')

const emptyText = computed(() => '执行 SQL 查看结果')

const pagination = computed(() => {
  if (!activeTab.value) return true
  return activeTab.value.rows.length < 50000
})

const pageInfoText = computed(() => {
  if (!gridApi.value || !gridApi.value.paginationGetCurrentPage) return ''
  const total = gridApi.value.paginationGetTotalPages()
  const current = gridApi.value.paginationGetCurrentPage() + 1
  return `${current}/${total} 页`
})

function modeLabel(tab: ResultTab): string {
  const map = { quick: '即时过滤', sql: 'SQL过滤', duckdb: 'DuckDB分析' }
  return map[tab.filterMode]
}

// ─── 标签管理 ───────────────────────────────────────────
function createResultTab(sql: string, connId: string): ResultTab {
  tabCounter.value++
  const id = `result_${Date.now()}_${tabCounter.value}`
  const tab = reactive<ResultTab>({
    id, title: `结果 #${tabCounter.value}`, originalSql: sql, connectionId: connId,
    duckdbTempTable: '', columns: [], rows: [], originalRowCount: 0, displayedRowCount: 0,
    filterMode: 'quick', quickFilterExpression: '', filteredRowCount: 0,
    sqlFilterExpression: '', isSqlFilterLoading: false,
    duckdbSql: '', isDuckdbLoading: false, isAnalysisActive: false,
    executionTime: 0, timestamp: '', dirtyRows: new Set() })
  return tab
}

function tabHasDirty(tab: ResultTab | null): boolean {
  return tab ? tab.dirtyRows.size > 0 : false
}

function addResultTab(data: { columns: string[], rows: unknown[][], originalSql?: string, connectionId?: string, elapsedMs?: number }) {
  const tab = createResultTab(data.originalSql || '', data.connectionId || '')
  tab.columns = data.columns
  tab.rows = data.rows
  tab.originalRowCount = data.rows.length
  tab.displayedRowCount = data.rows.length
  tab.executionTime = data.elapsedMs || 0
  tab.timestamp = new Date().toLocaleString()
  resultTabs.value.push(tab)
  activeTabId.value = tab.id
  return tab
}

function switchTab(id: string) {
  activeTabId.value = id
}

function closeTab(id: string) {
  const idx = resultTabs.value.findIndex(t => t.id === id)
  if (idx === -1) return
  resultTabs.value.splice(idx, 1)
  if (activeTabId.value === id) {
    activeTabId.value = resultTabs.value[idx]?.id || resultTabs.value[idx - 1]?.id || null
  }
}

// ─── 事件处理 ───────────────────────────────────────────
const handleResultUpdate = (event: CustomEvent) => {
  const detail = event.detail || {}
  const qr = detail.result
  if (!qr) return

  const columns = qr.columns || []
  const rows: unknown[][] = qr.rows || []
  const elapsedMs = detail.elapsedMs || 0
  const originalSql = detail.originalSql || ''
  const connectionId = detail.connectionId || ''

  // 查找是否已有该 SQL 的标签（同名标签复用）
  const existingTab = originalSql
    ? resultTabs.value.find(t => t.originalSql === originalSql && t.columns.length === 0)
    : null

  if (existingTab) {
    existingTab.columns = columns
    existingTab.rows = rows
    existingTab.originalRowCount = rows.length
    existingTab.displayedRowCount = rows.length
    existingTab.executionTime = elapsedMs
    existingTab.timestamp = new Date().toLocaleString()
    activeTabId.value = existingTab.id

    if (!existingTab.duckdbTempTable && columns.length > 0 && rows.length > 0) {
      ensureDuckdbTempTable(existingTab, columns, rows)
    }
  } else {
    const tab = addResultTab({ columns, rows, originalSql, connectionId, elapsedMs })
    if (columns.length > 0 && rows.length > 0) {
      ensureDuckdbTempTable(tab, columns, rows)
    }
  }
}

// Execute+：始终打开新标签
const handleResultNew = (event: CustomEvent) => {
  const detail = event.detail || {}
  const qr = detail.result
  if (!qr) return

  const columns = qr.columns || []
  const rows: unknown[][] = qr.rows || []
  const elapsedMs = detail.elapsedMs || 0
  const originalSql = detail.originalSql || ''
  const connectionId = detail.connectionId || ''

  const tab = addResultTab({ columns, rows, originalSql, connectionId, elapsedMs })
  if (columns.length > 0 && rows.length > 0) {
    ensureDuckdbTempTable(tab, columns, rows)
  }
}

onMounted(() => {
  window.addEventListener('query-result-updated', handleResultUpdate as (e: Event) => void)
  window.addEventListener('query-result-new', handleResultNew as (e: Event) => void)
  window.addEventListener('keydown', handleGlobalKeyDown)
})
onUnmounted(() => {
  window.removeEventListener('query-result-updated', handleResultUpdate as (e: Event) => void)
  window.removeEventListener('query-result-new', handleResultNew as (e: Event) => void)
  window.removeEventListener('keydown', handleGlobalKeyDown)
})

// ─── DuckDB 临时表 ──────────────────────────────────────
async function ensureDuckdbTempTable(tab: ResultTab, columns: string[], rows: unknown[][]) {
  try {
    const tableName = await apiCreateTempTable(columns, rows)
    if (tableName) tab.duckdbTempTable = tableName
  } catch { /* silent */ }
}

// ─── AG Grid 事件 ───────────────────────────────────────
function onGridReady(params: any) {
  gridApi.value = params.api
  setTimeout(() => params.api.sizeColumnsToFit(), 200)
}
function onRowDataUpdated(params: any) {
  if (params.api?.getDisplayedRowCount() > 0) params.api.sizeColumnsToFit()
}
function onSelectionChanged() {
  if (!gridApi.value) return
  selectedRows.value = gridApi.value.getSelectedRows()
}

/** 双击或单击行 → 切换到记录模式查看单行详情 */
function onRowClicked(event: any) {
  selectedRecordIndex.value = event?.rowIndex ?? 0
  currentView.value = 'record'
}
function onSortChanged() { /* optional */ }
function onCellValueChanged(event: any) {
  if (activeTab.value) {
    activeTab.value.dirtyRows.add(event.node.rowIndex)
  }
}

function onCellContextMenu(params: any) {
  closeContextMenu()
  setTimeout(() => {
    const e = window.event as MouseEvent
    contextMenu.value = { visible: true, x: e.clientX, y: e.clientY, type: 'cell', value: params.value, column: params.colDef.field, sortDir: '' }
  }, 10)
}

function handleGridContextMenu(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (target.closest('.ag-header-cell')) {
    const colId = target.closest('.ag-header-cell')?.getAttribute('col-id') || ''
    const sortModel = gridApi.value?.getSortModel() || []
    const sortEntry = sortModel.find((s: any) => s.colId === colId)
    closeContextMenu()
    setTimeout(() => {
      contextMenu.value = { visible: true, x: event.clientX, y: event.clientY, type: 'header', value: null, column: colId, sortDir: sortEntry?.sort || '' }
    }, 10)
  }
}
function closeContextMenu() { contextMenu.value = { ...contextMenu.value, visible: false } }

// ─── 模式1: 即时过滤 ───────────────────────────────────
function applyQuickFilter(tab: ResultTab, expr: string) {
  if (!gridApi.value) return
  gridApi.value.setQuickFilter(expr)
  tab.filteredRowCount = gridApi.value.getDisplayedRowCount()
  tab.displayedRowCount = tab.filteredRowCount
}
function clearQuickFilter(tab: ResultTab) {
  tab.quickFilterExpression = ''
  if (gridApi.value) {
    gridApi.value.setQuickFilter('')
    tab.filteredRowCount = tab.originalRowCount
    tab.displayedRowCount = tab.originalRowCount
  }
}

// ─── 模式2: SQL 过滤 ───────────────────────────────────
async function executeSqlFilter(tab: ResultTab) {
  const whereClause = tab.sqlFilterExpression.trim()
  if (!whereClause) return
  tab.isSqlFilterLoading = true
  try {
    const result = await apiExecuteWithFilter(tab.connectionId, tab.originalSql, whereClause)
    tab.columns = result.columns
    tab.rows = result.rows
    tab.originalRowCount = result.rows.length
    tab.displayedRowCount = result.rows.length
    tab.executionTime = result.elapsed_ms
    if (result.temp_table) tab.duckdbTempTable = result.temp_table
  } catch (e: any) { message.error(String(e)) } finally { tab.isSqlFilterLoading = false }
}

// ─── 模式3: DuckDB 分析 ─────────────────────────────────
async function executeDuckdbAnalysis(tab: ResultTab) {
  const sql = tab.duckdbSql.trim()
  if (!sql) return
  tab.isDuckdbLoading = true
  try {
    const hasTempTable = !!tab.duckdbTempTable
    const result = await apiDuckdbAnalysis(
      tab.duckdbTempTable, sql,
      hasTempTable ? undefined : tab.columns,
      hasTempTable ? undefined : tab.rows as unknown[][]
    )
    tab.columns = result.columns
    tab.rows = result.rows
    tab.displayedRowCount = result.rows.length
    tab.executionTime = result.elapsed_ms
    tab.isAnalysisActive = true
    tab.timestamp = new Date().toLocaleString()
  } catch (e: any) { message.error(String(e)) } finally { tab.isDuckdbLoading = false }
}
function clearDuckdbAnalysis(tab: ResultTab) {
  tab.isAnalysisActive = false
  tab.filterMode = 'quick'
  tab.quickFilterExpression = ''
  tab.sqlFilterExpression = ''
  tab.duckdbSql = ''
}
function quickDuckdbAction(tab: ResultTab, type: string) {
  const table = tab.duckdbTempTable || 'result_temp'
  if (type === 'count') tab.duckdbSql = `SELECT COUNT(*) FROM ${table}`
  else if (type === 'distinct') tab.duckdbSql = `SELECT DISTINCT * FROM ${table} LIMIT 100`
  else if (type === 'group') {
    const firstCol = tab.columns[0] || 'col1'
    tab.duckdbSql = `SELECT ${firstCol}, COUNT(*) FROM ${table} GROUP BY ${firstCol} ORDER BY 2 DESC`
  }
}

// ─── 桥接模式 ──────────────────────────────────────────
async function handleBridgeFilter(tab: ResultTab) {
  if (!gridApi.value) return
  const visibleRows: any[] = []
  gridApi.value.forEachNodeAfterFilter((node: any) => visibleRows.push(node.data))
  if (visibleRows.length === 0) return
  tab.isDuckdbLoading = true
  try {
    const rowsData: unknown[][] = visibleRows.map(row => tab.columns.map(col => row[col] ?? null))
    const tableName = await apiCreateTempTable(tab.columns, rowsData)
    tab.duckdbTempTable = tableName
    tab.duckdbSql = `SELECT * FROM ${tableName} LIMIT 100`
    message.success(`已写入 ${visibleRows.length} 行到 DuckDB 临时表`)
  } catch (e: any) { message.error(String(e)) } finally { tab.isDuckdbLoading = false }
}

// ─── 操作 ───────────────────────────────────────────────
function handleCopySql() {
  if (activeTab.value) navigator.clipboard.writeText(activeTab.value.originalSql)
}
function handleRefresh(tab: ResultTab) {
  window.dispatchEvent(new CustomEvent('query-result-refresh', { detail: { connectionId: tab.connectionId, sql: tab.originalSql } }))
}
function handleSave(tab: ResultTab) { tab.dirtyRows.clear() }
function handleCancel(tab: ResultTab) { tab.dirtyRows.clear() }
function handleExport(format: string) {
  if (!gridApi.value || !activeTab.value) return
  if (format === 'csv') {
    gridApi.value.exportDataAsCsv({ fileName: `result_${Date.now()}.csv`, columnSeparator: ',' })
  } else if (format === 'json') {
    const data = JSON.stringify(rowData.value, null, 2)
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url; a.download = `result_${Date.now()}.json`
    document.body.appendChild(a); a.click(); document.body.removeChild(a)
    URL.revokeObjectURL(url)
  } else if (format === 'insert') {
    copyRowsAsInsert()
  }
}

function copyRowsAsInsert() {
  if (!activeTab.value) return
  const cols = activeTab.value.columns
  const rows = gridApi.value?.getSelectedRows() || rowData.value
  const inserts = rows.map((row: any) => {
    const vals = cols.map(c => {
      const v = row[c]
      if (v === null || v === undefined) return 'NULL'
      if (typeof v === 'number') return String(v)
      return `'${String(v).replace(/'/g, "''")}'`
    }).join(', ')
    return `INSERT INTO result (${cols.join(', ')}) VALUES (${vals});`
  })
  navigator.clipboard.writeText(inserts.join('\n'))
}

// ─── 右键菜单操作 ───────────────────────────────────────
function handleContextAction(payload: Record<string, any>) {
  closeContextMenu()
  const tab = activeTab.value
  if (!tab) return
  const { action, column, value } = payload
  const col = column || ''

  switch (action) {
    case 'copyCell':
      if (value !== null && value !== undefined) navigator.clipboard.writeText(String(value))
      break
    case 'copyRow':
      if (gridApi.value) {
        const selected = gridApi.value.getSelectedRows()
        const rows = selected.length > 0 ? selected : rowData.value
        const text = rows.map((r: any) => tab.columns.map(c => String(r[c] ?? '')).join('\t')).join('\n')
        navigator.clipboard.writeText(tab.columns.join('\t') + '\n' + text)
      }
      break
    case 'copyRowJson':
      copyRowsAsJson()
      break
    case 'copyRowInsert':
      copyRowsAsInsert()
      break
    case 'filterByValue':
      if (col && value !== undefined) {
        tab.filterMode = 'quick'
        tab.quickFilterExpression = `${col} = ${typeof value === 'string' ? `'${String(value).replace(/'/g, "''")}'` : value}`
        applyQuickFilter(tab, tab.quickFilterExpression)
      }
      break
    case 'sqlFilterByValue':
      if (col && value !== undefined) {
        tab.filterMode = 'sql'
        tab.sqlFilterExpression = `${col} = ${typeof value === 'string' ? `'${String(value).replace(/'/g, "''")}'` : value}`
      }
      break
    case 'openColumnInsights':
      if (tab.duckdbTempTable) {
        window.dispatchEvent(new CustomEvent('open-column-insight', { detail: { column: col, tempTable: tab.duckdbTempTable } }))
      } else {
        message.warning('需先执行 DuckDB 分析')
      }
      break
    case 'sortAsc': if (gridApi.value) gridApi.value.applySortState([{ colId: col, sort: 'asc' }]); break
    case 'sortDesc': if (gridApi.value) gridApi.value.applySortState([{ colId: col, sort: 'desc' }]); break
    case 'sendSortToSql':
      tab.filterMode = 'sql'
      tab.sqlFilterExpression = `1=1 ORDER BY ${col} ${payload.sortDir === 'desc' ? 'DESC' : 'ASC'}`
      break
    case 'sendSortToDuckdb':
      tab.filterMode = 'duckdb'
      tab.duckdbSql = `SELECT * FROM result_temp ORDER BY ${col} ${payload.sortDir === 'desc' ? 'DESC' : 'ASC'} LIMIT 1000`
      break
    case 'hideColumn': if (gridApi.value) gridApi.value.setColumnsVisible([col], false); break
    case 'autoSizeColumn': if (gridApi.value) gridApi.value.autoSizeColumns([col]); break
    case 'autoSizeAll':
      if (gridApi.value) {
        const allCols: string[] = []
        gridApi.value.getColumns().forEach((c: any) => { if (c.getColDef().field !== '__rowNumber') allCols.push(c.getId()) })
        gridApi.value.autoSizeColumns(allCols)
      }
      break
    case 'columnSummary':
      if (tab.duckdbTempTable) {
        tab.filterMode = 'duckdb'
        tab.duckdbSql = `SELECT COUNT(*) as count, AVG("${col}") as avg, MIN("${col}") as min, MAX("${col}") as max, SUM("${col}") as sum FROM result_temp`
      }
      break
  }
}

function copyRowsAsJson() {
  if (!activeTab.value) return
  const rows = gridApi.value?.getSelectedRows() || rowData.value
  navigator.clipboard.writeText(JSON.stringify(rows, null, 2))
}

// ─── 键盘快捷键 ───────────────────────────────────────────
function handleKeyDown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    const tab = activeTab.value
    if (!tab) return
    if (tab.filterMode === 'sql') executeSqlFilter(tab)
    else if (tab.filterMode === 'duckdb') executeDuckdbAnalysis(tab)
  }
  if ((event.ctrlKey || event.metaKey) && event.key === 's') { event.preventDefault(); if (activeTab.value) handleSave(activeTab.value) }
  if ((event.ctrlKey || event.metaKey) && event.key === 'r') { event.preventDefault(); if (activeTab.value) handleRefresh(activeTab.value) }
}
function handleGlobalKeyDown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key === 'c') {
    if (document.activeElement?.closest('.ag-cell') && activeTab.value) {
      const selected = gridApi.value?.getSelectedRows() || []
      const rows = selected.length > 0 ? selected : rowData.value
      const text = rows.map((r: any) => activeTab.value.columns.map(c => String(r[c] ?? '')).join('\t')).join('\n')
      navigator.clipboard.writeText(activeTab.value.columns.join('\t') + '\n' + text)
    }
  }
}
</script>

<style scoped>
.query-result-panel { display: flex; flex-direction: column; height: 100%; min-height: 0; background: var(--bg-primary); }
.query-result-panel.compact .result-tabs { height: 24px; font-size: 10px; }
.query-result-panel.compact .result-tab { padding: 0 6px; }

/* ─── 标签栏 ─────────────────────────────────────────── */
.result-tabs {
  display: flex; align-items: stretch; height: 26px; flex-shrink: 0;
  background: var(--bg-tertiary, #2d2d30);
  border-bottom: 1px solid var(--border-color, #3e3e42); overflow-x: auto;
}
.result-tab {
  display: flex; align-items: center; gap: 3px;
  padding: 0 8px; font-size: 11px; cursor: pointer;
  color: var(--text-secondary, #888); white-space: nowrap;
  border-right: 1px solid var(--border-color, #3e3e42); user-select: none;
}
.result-tab:hover { background: var(--bg-hover, #333); color: var(--text-primary); }
.result-tab.active { background: var(--bg-primary); color: var(--text-primary); border-bottom: 2px solid var(--primary-color, #0078d4); }
.tab-close { font-size: 13px; line-height: 1; opacity: 0.5; margin-left: 2px; }
.tab-close:hover { opacity: 1; }

/* ─── 顶条 ───────────────────────────────────────────── */
.toolbar-strip {
  display: flex; align-items: center; padding: 1px 4px; gap: 4px;
  flex-shrink: 0; min-height: 26px;
  background: var(--bg-secondary, #252526);
  border-bottom: 1px solid var(--border-color, #333);
}
.toolbar-strip .strip-right { flex: 1; min-width: 0; }

/* ─── 主体布局 ────────────────────────────────────────── */
.result-body { flex: 1; min-height: 0; display: flex; position: relative; }

/* 左侧栏 */
.view-sidebar {
  display: flex; flex-direction: column; gap: 1px; padding: 2px;
  flex-shrink: 0; background: var(--bg-tertiary, #2d2d30);
  border-right: 1px solid var(--border-color, #3e3e42);
}
.view-btn {
  display: flex; align-items: center; justify-content: center;
  width: 24px; height: 24px; border: none; border-radius: 3px;
  background: transparent; color: var(--text-secondary, #888);
  cursor: pointer; transition: all 0.12s;
}
.view-btn:hover { background: var(--bg-hover, #3c3c3c); color: var(--text-primary); }
.view-btn.active { background: var(--primary-color); color: #fff; }

/* 中间网格 */
.grid-area { flex: 1; min-width: 0; position: relative; overflow: hidden; }
.grid-empty {
  position: absolute; inset: 0; z-index: 1;
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  background: var(--bg-primary); gap: 8px; color: var(--text-tertiary, #666);
  pointer-events: none;
}
.grid-fill { height: 100%; }
:deep(.ag-theme-alpine), :deep(.ag-theme-alpine-dark) { height: 100% !important; font-size: 11px; }
:deep(.ag-root-wrapper) { border: none; }
:deep(.ag-header) { min-height: 24px !important; }
:deep(.ag-header-cell) { font-size: 10px; font-weight: 600; padding: 0 4px !important; }
:deep(.ag-header-cell-label) { padding: 0; }
:deep(.ag-row) { font-size: 11px; min-height: 22px !important; }
:deep(.ag-cell) { padding: 0 4px !important; line-height: 22px !important; }
:deep(.null-value) { color: #999; font-style: italic; font-size: 10px; }
:deep(.text-right) { text-align: right; font-family: monospace; }
:deep(.ag-pinned-left-cols-container) { border-right: 1px solid var(--border-color, #444); }
:deep(.ag-row-even) { background: var(--bg-row-even, rgba(128,128,128,0.03)); }
:deep(.ag-row-odd) { background: transparent; }
:deep(.ag-row:hover) { background: var(--bg-hover, rgba(255,255,255,0.04)) !important; }
/* 自动列宽 */
:deep(.ag-cell) { overflow: hidden; text-overflow: ellipsis; }

/* 文本视图 */
.text-view { height: 100%; padding: 6px; }
.text-view-area {
  width: 100%; height: 100%; border: none; resize: none;
  background: var(--bg-primary); color: var(--text-primary);
  font-family: monospace; font-size: 11px; line-height: 1.6;
}

/* 记录视图 */
.record-view { height: 100%; overflow-y: auto; padding: 6px; }
.record-nav { display: flex; align-items: center; gap: 8px; margin-bottom: 6px; font-size: 11px; }
.record-fields { display: flex; flex-direction: column; gap: 4px; }
.record-field { display: flex; align-items: flex-start; gap: 8px; padding: 4px 6px; border-radius: 3px; }
.record-field:nth-child(even) { background: var(--bg-row-even, rgba(128,128,128,0.03)); }
.field-name {
  font-size: 10px; font-weight: 700; color: var(--text-secondary);
  text-transform: uppercase; min-width: 120px; flex-shrink: 0;
  line-height: 20px;
}
.field-value {
  font-size: 11px; font-family: monospace; line-height: 20px;
  white-space: pre-wrap; word-break: break-all; color: var(--text-primary);
}

/* 右侧值查看器 */
.value-viewer { width: 220px; flex-shrink: 0; display: flex; flex-direction: column; border-left: 1px solid var(--border-color, #3e3e42); background: var(--bg-secondary, #252526); }
.viewer-header { display: flex; align-items: center; justify-content: space-between; padding: 4px 6px; border-bottom: 1px solid var(--border-color, #333); }
.viewer-title { font-size: 11px; font-weight: 600; }
.viewer-content { padding: 6px; display: flex; flex-direction: column; gap: 6px; overflow: auto; }
.viewer-field { display: flex; gap: 6px; font-size: 10px; }
.field-label { font-weight: 600; color: var(--text-secondary); min-width: 24px; }
.field-val { font-family: monospace; }
.viewer-text { width: 100%; min-height: 100px; border: 1px solid var(--border-color, #333); background: var(--bg-primary); color: var(--text-primary); font-family: monospace; font-size: 10px; padding: 4px; resize: none; }
.viewer-toggle { position: absolute; top: 4px; right: 2px; z-index: 10; }

/* ─── 底部状态栏（操作/行信息/分页合并） ──────────────── */
.result-statusbar {
  display: flex; align-items: center; height: 24px; padding: 0 4px; gap: 4px;
  flex-shrink: 0; font-size: 10px; color: var(--text-secondary, #999);
  background: var(--bg-tertiary, #2d2d30);
  border-top: 1px solid var(--border-color, #3e3e42);
}
.sbar-left, .sbar-center, .sbar-right { display: flex; align-items: center; gap: 1px; }
.sbar-center { flex: 1; justify-content: center; gap: 6px; }
.sbar-right { gap: 1px; }
.mode-badge { padding: 0 4px; border-radius: 2px; font-size: 9px; font-weight: 600; line-height: 16px; }
.mode-badge.quick { background: #2d6a4f33; color: #52c41a; }
.mode-badge.sql { background: #1a5a8a33; color: #1890ff; }
.mode-badge.duckdb { background: #613a8a33; color: #b37feb; }
.separator { color: var(--border-color, #444); }
.row-info { font-family: monospace; }
.exec-time { color: var(--primary-color); font-family: monospace; }
.page-indicator { font-family: monospace; margin: 0 2px; }

.analysis-notice { display: flex; align-items: center; height: 20px; padding: 0 6px; background: #f5eec033; border-bottom: 1px solid #d4c78e66; font-size: 10px; color: #b8a44c; flex-shrink: 0; }
.empty-icon { opacity: 0.4; }
.empty-text { font-size: 13px; color: var(--text-secondary, #888); }
</style>
