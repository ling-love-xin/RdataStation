<template>
  <div class="query-result-panel">
    <!-- 顶部标签栏 -->
    <div v-if="tabs.length > 0" class="result-tabs">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        :class="['result-tab', { active: tab.id === activeTabId }]"
        @click="switchTab(tab.id)"
      >
        <span class="tab-title">{{ tab.title }}</span>
        <span class="tab-close" @click.stop="closeTab(tab.id)">&times;</span>
      </div>
    </div>

    <!-- 主内容区 -->
    <template v-if="hasActiveTab && activeTab">
      <!-- SQL 预览 + 模式切换条 -->
      <div class="toolbar-strip">
        <FilterModeSwitcher v-model="tab.filterMode" class="mode-switcher-inline" />
        <FilterPresetSelector
          :filter-mode="tab.filterMode"
          :current-expression="getCurrentExpression(tab)"
          @select="(e: any) => applyPreset(tab, e)"
          @save="(name: string, expr: string, mode: any) => saveFilterPreset(tab, name, expr, mode)"
        />
        <div class="strip-right">
          <QuickFilterInput
            v-if="tab.filterMode === 'quick'"
            :expression="tab.quickFilterExpression"
            :visible-count="tab.filteredRowCount"
            :total-count="tab.originalRowCount"
            @update:expression="(v: string) => (tab.quickFilterExpression = v)"
            @apply="(v: string) => applyQuickFilter(tab, v)"
            @clear="() => clearQuickFilter(tab)"
          />
          <SqlFilterInput
            v-if="tab.filterMode === 'sql'"
            :expression="tab.sqlFilterExpression"
            :loading="tab.isSqlFilterLoading"
            @update:expression="(v: string) => (tab.sqlFilterExpression = v)"
            @execute="() => executeSqlFilter(tab)"
          />
          <DuckDBAnalysisInput
            v-if="tab.filterMode === 'duckdb'"
            :sql="tab.duckdbSql"
            :loading="tab.isDuckdbLoading"
            @update:sql="(v: string) => (tab.duckdbSql = v)"
            @execute="() => executeDuckdbAnalysis(tab)"
            @clear="() => clearDuckdbAnalysis(tab)"
            @quick="(t: string) => quickDuckdbAction(tab, t)"
            @bridge-filter="() => handleBridgeFilter(tab)"
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
          <ResultGridView
            :tab="tab"
            :column-defs="columnDefs"
            :row-data="rowData"
            :default-col-def="defaultColDef"
            :pagination="pagination"
            :page-size="pageSize"
            :is-dark="uiStore.isDark"
            :loading="tab.isLoading"
            :empty-text="t('workbench.executeSqlToSeeResults')"
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
          <div v-if="currentView === 'chart'" class="chart-fill">
            <DataVisualizationPanel
              :data="(rowData as Record<string, unknown>[])"
              :columns="tab.columns"
            />
          </div>
          <ResultTextView
            v-if="currentView === 'text'"
            :tab="tab"
            :max-rows="10000"
            :empty-text="t('workbench.executeSqlToSeeResults')"
          />
          <div v-if="currentView === 'record'" class="record-view">
            <div class="record-nav">
              <NButton size="tiny" quaternary :disabled="selectedRecordIndex <= 0" @click="prevRecord">
                <ChevronLeft :size="14" />
              </NButton>
              <span class="record-nav-text">{{ selectedRecordIndex + 1 }} / {{ rowData.length }}</span>
              <NButton size="tiny" quaternary :disabled="selectedRecordIndex >= rowData.length - 1" @click="nextRecord">
                <ChevronRight :size="14" />
              </NButton>
            </div>
            <ResultRecordView
              :tab="tab"
              :selected-row-index="selectedRecordIndex"
              :empty-text="t('workbench.executeSqlToSeeResults')"
            />
          </div>
        </div>

        <!-- 右侧数值查看器 -->
        <div v-if="showValueViewer" class="value-viewer">
          <div class="viewer-header">
            <span class="viewer-title">{{ t('workbench.valueViewer') }}</span>
            <NButton size="tiny" quaternary @click="showValueViewer = false">
              <X :size="12" />
            </NButton>
          </div>
          <div class="viewer-content">
            <div class="viewer-field">
              <span class="field-label">{{ t('workbench.columnLabel') }}</span>
              <span class="field-val">{{ selectedCell?.column || '-' }}</span>
            </div>
            <div class="viewer-field">
              <span class="field-label">{{ t('workbench.rowLabel') }}</span>
              <span class="field-val">{{
                selectedCell?.row != null ? selectedCell.row + 1 : '-'
              }}</span>
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
          :title="t('workbench.openValueViewer')"
          @click="showValueViewer = true"
        >
          <PanelRight :size="14" />
        </NButton>
      </div>

      <!-- 底部状态栏（操作 + 行信息 + 分页） -->
      <div v-if="activeTab" class="result-statusbar">
        <div class="sbar-left">
          <span :class="['mode-badge', tab.filterMode]">{{ modeLabel(tab) }}</span>
          <NButton
            size="tiny"
            quaternary
            :title="t('resultPanel.refresh')"
            @click="handleRefresh(tab)"
          >
            <RotateCw :size="11" />
          </NButton>
          <NButton
            size="tiny"
            quaternary
            :disabled="!tabHasDirty(tab)"
            :title="t('resultPanel.save')"
            @click="handleSave(tab)"
          >
            <Save :size="11" />
          </NButton>
          <NButton
            size="tiny"
            quaternary
            :disabled="!tabHasDirty(tab)"
            :title="t('resultPanel.cancel')"
            @click="handleCancel(tab)"
          >
            <X :size="11" />
          </NButton>
          <NButton
            size="tiny"
            quaternary
            title="对比结果集"
            @click="showDiffModal = true"
          >
            <GitCompare :size="14" />
          </NButton>
          <NDropdown
            trigger="hover"
            :options="exportMenuOptions"
            @select="(k: string) => handleExport(k)"
          >
            <NButton size="tiny" quaternary :title="t('resultPanel.export')">
              <Download :size="11" />
            </NButton>
          </NDropdown>
        </div>
        <div class="sbar-center">
          <span class="row-info">{{ displayRowText }}</span>
          <span v-if="tab.executionTime" class="exec-time"
            >{{ (tab.executionTime / 1000).toFixed(3) }}s</span
          >
        </div>
        <div class="sbar-right">
          <NButton
            size="tiny"
            quaternary
            :disabled="!gridApi"
            :title="t('workbench.firstPage')"
            @click="firstPage"
          >
            <SkipBack :size="11" />
          </NButton>
          <NButton
            size="tiny"
            quaternary
            :disabled="!gridApi"
            :title="t('workbench.prevPage')"
            @click="prevPage"
          >
            <ChevronLeft :size="11" />
          </NButton>
          <span v-if="gridApi" class="page-indicator">{{ pageInfoText }}</span>
          <NButton
            size="tiny"
            quaternary
            :disabled="!gridApi"
            :title="t('workbench.nextPage')"
            @click="nextPage"
          >
            <ChevronRight :size="11" />
          </NButton>
          <NButton
            size="tiny"
            quaternary
            :disabled="!gridApi"
            :title="t('workbench.lastPage')"
            @click="lastPage"
          >
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

    <NModal
      v-model:show="showDiffModal"
      preset="dialog"
      title="结果集对比"
      :show-icon="false"
      style="width: 900px; max-height: 80vh;"
      :mask-closable="true"
    >
      <ResultDiffViewer />
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { ClientSideRowModelModule } from '@ag-grid-community/client-side-row-model'
import { ModuleRegistry } from '@ag-grid-community/core'
import '@ag-grid-community/styles/ag-grid.css'
import '@ag-grid-community/styles/ag-theme-alpine.css'
import { save } from '@tauri-apps/plugin-dialog'
import {
  Database,
  RotateCw,
  Save,
  X,
  Download,
  PanelRight,
  ChevronLeft,
  ChevronRight,
  SkipBack,
  SkipForward,
  AlignLeft,
  List,
  GitCompare,
  BarChart3,
} from 'lucide-vue-next'
import { createDiscreteApi, darkTheme, lightTheme, NButton, NDropdown, NModal } from 'naive-ui'
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { useInsightStore } from '@/extensions/builtin/workbench/ui/stores/insight-store'
import { useResultStore } from '@/extensions/builtin/workbench/ui/stores/result-store'
import { useSqlExecutionStore } from '@/extensions/builtin/workbench/ui/stores/sql-execution-store'
import type { ResultTab, ViewMode } from '@/extensions/builtin/workbench/ui/types/result'
import { useUiStore } from '@/shared/stores/ui'

import DataVisualizationPanel from './DataVisualizationPanel.vue'
import DuckDBAnalysisInput from './result-panel/DuckDBAnalysisInput.vue'
import FilterModeSwitcher from './result-panel/FilterModeSwitcher.vue'
import FilterPresetSelector from './result-panel/FilterPresetSelector.vue'
import QuickFilterInput from './result-panel/QuickFilterInput.vue'
import ResultContextMenu from './result-panel/ResultContextMenu.vue'
import ResultDiffViewer from './result-panel/ResultDiffViewer.vue'
import SqlFilterInput from './result-panel/SqlFilterInput.vue'
import { useFilterPresets } from '../../composables/useFilterPresets'
import {
  reExecuteWithFilter as apiExecuteWithFilter,
  executeDuckdbAnalysis as apiDuckdbAnalysis,
  createDuckdbTempTable as apiCreateTempTable,
  saveCellUpdate as apiSaveCellUpdate,
  exportResultToFile as apiExportResult,
} from '../../services/result-analysis'

import type { GridApi } from '@ag-grid-community/core'

ModuleRegistry.registerModules([ClientSideRowModelModule])

// ─── Store ───────────────────────────────────────────────
const uiStore = useUiStore()
const resultStore = useResultStore()
const insightStore = useInsightStore()
const configProviderPropsRef = ref({ theme: uiStore.isDark ? darkTheme : lightTheme })
const { message } = createDiscreteApi(['message'], { configProviderProps: configProviderPropsRef })
watch(
  () => uiStore.isDark,
  v => {
    configProviderPropsRef.value = { theme: v ? darkTheme : lightTheme }
  }
)

// ─── 多标签状态（从 store 读取）──────────────────────────
const tabs = computed(() => resultStore.tabs)
const activeTabId = computed(() => resultStore.activeTabId)
const hasActiveTab = computed(
  () => activeTabId.value !== null && tabs.value.some(t => t.id === activeTabId.value)
)
const activeTab = computed(() => {
  const id = resultStore.activeTabId
  if (!id) return undefined
  return resultStore.tabs.find(t => t.id === id)
})

const tab = computed(() => activeTab.value!)

// ─── AG Grid ─────────────────────────────────────────────
const gridApi = ref<GridApi | null>(null)
const showDiffModal = ref(false)
const gridContainerRef = ref<HTMLElement | null>(null)
const pageSize = ref(100)
const selectedRows = ref<unknown[]>([])

const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  type: 'cell' as 'cell' | 'header',
  value: null as unknown,
  column: '',
  sortDir: '',
})

interface DirtyCell {
  rowIndex: number
  colId: string
  oldValue: unknown
  newValue: unknown
}
const dirtyCells = ref<Map<string, DirtyCell>>(new Map())

function dirtyKey(rowIndex: number, colId: string): string {
  return `${rowIndex}:${colId}`
}

// ─── 过滤预设 ───────────────────────────────────────
const { addPreset } = useFilterPresets()

function getCurrentExpression(tab: ResultTab): string {
  switch (tab.filterMode) {
    case 'quick': return tab.quickFilterExpression
    case 'sql': return tab.sqlFilterExpression ?? ''
    default: return ''
  }
}

function applyPreset(tab: ResultTab, event: any): void {
  tab.filterMode = event.filterMode
  switch (tab.filterMode) {
    case 'quick':
      tab.quickFilterExpression = event.expression
      applyQuickFilter(tab, event.expression)
      break
    case 'sql':
      tab.sqlFilterExpression = event.expression
      executeSqlFilter(tab)
      break
  }
}

function saveFilterPreset(tab: ResultTab, name: string, expr: string, mode: any): void {
  addPreset(name, mode, expr)
  message.success('预设已保存')
}

// ─── Grid / Text / Record 视图切换 ──────────────────
const currentView = ref<ViewMode>('grid')
const showValueViewer = ref(false)
const selectedRecordIndex = ref(0)
interface SelectedCell {
  column: string
  row: number
  value: unknown
}
const selectedCell = ref<SelectedCell | null>(null)

const { t } = useI18n()

const viewModes = [
  { key: 'grid' as ViewMode, icon: Database, label: t('workbench.gridView') },
  { key: 'text' as ViewMode, icon: AlignLeft, label: t('workbench.textView') },
  { key: 'record' as ViewMode, icon: List, label: t('workbench.recordView') },
  { key: 'chart' as ViewMode, icon: BarChart3, label: t('workbench.chartView') },
]

function switchView(mode: ViewMode) {
  currentView.value = mode
  if (mode === 'record' && selectedRecordIndex.value >= rowData.value.length) {
    selectedRecordIndex.value = 0
  }
}

function prevRecord() {
  if (selectedRecordIndex.value > 0) selectedRecordIndex.value--
}
function nextRecord() {
  if (selectedRecordIndex.value < rowData.value.length - 1) selectedRecordIndex.value++
}

function firstPage() {
  gridApi.value?.paginationGoToFirstPage()
}
function prevPage() {
  gridApi.value?.paginationGoToPreviousPage()
}
function nextPage() {
  gridApi.value?.paginationGoToNextPage()
}
function lastPage() {
  gridApi.value?.paginationGoToLastPage()
}

function onFirstDataRendered(params: any) {
  params.api?.sizeColumnsToFit()
}

const displayRowText = computed(() => {
  if (!activeTab.value) return ''
  if (
    activeTab.value.filterMode === 'quick' &&
    activeTab.value.filteredRowCount !== activeTab.value.originalRowCount
  ) {
    return `${activeTab.value.originalRowCount} → ${activeTab.value.filteredRowCount} ${t('resultPanel.rows')}`
  }
  return `${activeTab.value.displayedRowCount} ${t('resultPanel.rows')}`
})

// ─── 导出菜单 ────────────────────────────────────────────
const exportMenuOptions = computed(() => [
  { key: 'csv', label: t('workbench.exportCsv') },
  { key: 'json', label: t('workbench.exportJson') },
  { key: 'insert', label: t('workbench.exportInsert') },
  { key: 'parquet', label: t('workbench.exportParquet') },
  { key: 'xlsx', label: t('workbench.exportXlsx') },
])

// ─── 列定义 - 智能分辨 ───────────────────────────────────
const numericColPatterns = [
  'id',
  '_id',
  'count',
  'num',
  'year',
  'age',
  'price',
  'amount',
  'total',
  'qty',
  'rate',
]

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
  return (
    lower.includes('description') ||
    lower.includes('content') ||
    lower.includes('comment') ||
    lower.includes('note') ||
    lower.includes('text')
  )
}

const columnDefs = computed(() => {
  if (!activeTab.value || activeTab.value.columns.length === 0)
    return [{ field: '__placeholder', headerName: '', hide: true }]
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
        try {
          return JSON.stringify(v)
        } catch {
          return String(v)
        }
      }
      const str = String(v)
      if (str.length > 500) return str.substring(0, 200) + '...'
      return str
    },
    comparator: (a: any, b: any) => {
      if (a === null && b === null) return 0
      if (a === null) return 1
      if (b === null) return -1
      if (typeof a === 'number' && typeof b === 'number') return a - b
      return String(a).localeCompare(String(b))
    },
  }))
  return [
    {
      field: '__rowNumber',
      headerName: '#',
      width: 55,
      pinned: 'left',
      sortable: false,
      filter: false,
      resizable: false,
      valueGetter: (p: any) => {
        const api = p.api
        if (!api || !api.paginationGetCurrentPage) return p.node.rowIndex + 1
        return api.paginationGetCurrentPage() * api.paginationGetPageSize() + p.node.rowIndex + 1
      },
      cellStyle: {
        textAlign: 'center',
        color: 'var(--text-tertiary)',
        fontSize: '11px',
        background: 'var(--bg-secondary)',
      },
    },
    ...cols,
  ]
})

const rowData = computed(() => {
  const tab = activeTab.value
  if (!tab) return []
  return (tab.rows as unknown[][]).map((row: unknown[]) => {
    if (Array.isArray(row)) {
      const obj: Record<string, unknown> = {}
      tab.columns.forEach((col, i) => {
        obj[col] = row[i]
      })
      return obj
    }
    return row
  })
})

const defaultColDef = {
  editable: true,
  sortable: true,
  filter: true,
  resizable: true,
  suppressMenu: false,
  filterParams: { maxNumConditions: 1 },
}

const pagination = computed(() => {
  if (!activeTab.value) return true
  return activeTab.value.rows.length < 50000
})

const pageInfoText = computed(() => {
  if (!gridApi.value || !gridApi.value.paginationGetCurrentPage) return ''
  const total = gridApi.value.paginationGetTotalPages()
  const current = gridApi.value.paginationGetCurrentPage() + 1
  return `${current}/${total} ${t('resultPanel.page')}`
})

function modeLabel(tab: ResultTab): string {
  const map = {
    quick: t('resultPanel.instantFilter'),
    sql: t('resultPanel.sqlFilter'),
    duckdb: t('resultPanel.duckdbAnalysis'),
  }
  return map[tab.filterMode]
}

// ─── 标签管理（委托到 store）───────────────────────────
function tabHasDirty(tab: ResultTab | null): boolean {
  return tab ? tab.dirtyRows.size > 0 : false
}

function switchTab(id: string) {
  resultStore.switchTab(id)
}

function closeTab(id: string) {
  resultStore.closeTab(id)
}

// ─── 事件处理（使用 Pinia Store）──────────────────

const sqlExecutionStore = useSqlExecutionStore()

const handleResultUpdate = () => {
  const result = sqlExecutionStore.latestResult
  if (!result || !result.result) return

  const qr = result.result
  const columns = qr.columns || []
  const rows: unknown[][] = qr.rows || []
  const elapsedMs = qr.executionTime || 0
  const panelId = result.panelId || ''

  const existingTab = panelId
    ? resultStore.tabs.find(t => t.id === panelId && t.columns.length === 0)
    : null

  if (existingTab) {
    resultStore.setTabResult(existingTab.id, {
      columns,
      rows,
      rowCount: rows.length,
      elapsedMs,
    })
  } else {
    const tab = resultStore.addTab(panelId, '')
    resultStore.setTabResult(tab.id, {
      columns,
      rows,
      rowCount: rows.length,
      elapsedMs,
    })
  }
}

const handleResultNew = () => {
  const latest = sqlExecutionStore.consumeNewTabRequest()
  if (!latest || !latest.result) return

  const qr = latest.result
  const columns = qr.columns || []
  const rows: unknown[][] = qr.rows || []
  const elapsedMs = qr.executionTime || 0
  const panelId = latest.panelId || ''

  const tab = resultStore.addTab('', panelId)
  resultStore.setTabResult(tab.id, {
    columns,
    rows,
    rowCount: rows.length,
    elapsedMs,
  })
}

watch(
  () => sqlExecutionStore.executionResults.size,
  () => {
    handleResultUpdate()
  }
)

watch(
  () => sqlExecutionStore.newTabRequests.size,
  size => {
    if (size > 0) {
      handleResultNew()
    }
  }
)

onMounted(() => {
  window.addEventListener('keydown', handleGlobalKeyDown)
})
onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeyDown)
})

// ─── DuckDB 临时表（委托到 store）──────────────────────
async function ensureDuckdbTempTable(tabId: string) {
  await resultStore.ensureDuckdbTable(tabId)
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
function onSortChanged() {
  /* optional */
}
function onCellValueChanged(event: any) {
  if (!activeTab.value) return
  const { rowIndex, colDef, oldValue, newValue } = event
  const key = dirtyKey(rowIndex, colDef.field)
  const current = dirtyCells.value
  if (!current.has(key)) {
    const newMap = new Map(current)
    newMap.set(key, { rowIndex, colId: colDef.field, oldValue, newValue })
    dirtyCells.value = newMap
  } else {
    const existing = current.get(key)!
    if (existing.oldValue === newValue) {
      const newMap = new Map(current)
      newMap.delete(key)
      dirtyCells.value = newMap
    } else {
      const newMap = new Map(current)
      newMap.set(key, { ...existing, newValue })
      dirtyCells.value = newMap
    }
  }
}

function onCellContextMenu(params: any) {
  closeContextMenu()
  setTimeout(() => {
    const e = window.event as MouseEvent
    contextMenu.value = {
      visible: true,
      x: e.clientX,
      y: e.clientY,
      type: 'cell',
      value: params.value,
      column: params.colDef.field,
      sortDir: '',
    }
  }, 10)
}

function handleGridContextMenu(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (target.closest('.ag-header-cell')) {
    const colId = target.closest('.ag-header-cell')?.getAttribute('col-id') || ''
    const sortModel = (gridApi.value as Record<string, unknown>)?.getSortModel as
      | (() => Array<{ colId: string; sort: string }>)
      | undefined
    const sortEntry = sortModel?.().find(s => s.colId === colId)
    closeContextMenu()
    setTimeout(() => {
      contextMenu.value = {
        visible: true,
        x: event.clientX,
        y: event.clientY,
        type: 'header',
        value: null,
        column: colId,
        sortDir: sortEntry?.sort || '',
      }
    }, 10)
  }
}
function closeContextMenu() {
  contextMenu.value = { ...contextMenu.value, visible: false }
}

// ─── 模式1: 即时过滤 ───────────────────────────────────
function applyQuickFilter(tab: ResultTab, expr: string) {
  if (!gridApi.value) return
  ;(gridApi.value as unknown as { setQuickFilter: (v: string) => void }).setQuickFilter(expr)
  tab.filteredRowCount = gridApi.value.getDisplayedRowCount()
  tab.displayedRowCount = tab.filteredRowCount
}
function clearQuickFilter(tab: ResultTab) {
  tab.quickFilterExpression = ''
  if (gridApi.value) {
    ;(gridApi.value as unknown as { setQuickFilter: (v: string) => void }).setQuickFilter('')
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
  } catch (e: unknown) {
    message.error(String(e))
  } finally {
    tab.isDuckdbLoading = false
  }
}

// ─── 模式3: DuckDB 分析 ─────────────────────────────────
async function executeDuckdbAnalysis(tab: ResultTab) {
  const sql = tab.duckdbSql.trim()
  if (!sql) return
  tab.isDuckdbLoading = true
  try {
    const hasTempTable = !!tab.duckdbTempTable
    const result = await apiDuckdbAnalysis(
      tab.duckdbTempTable,
      sql,
      hasTempTable ? undefined : tab.columns,
      hasTempTable ? undefined : (tab.rows as unknown[][])
    )
    tab.columns = result.columns
    tab.rows = result.rows
    tab.displayedRowCount = result.rows.length
    tab.executionTime = result.elapsed_ms
    tab.isAnalysisActive = true
    tab.timestamp = new Date().toLocaleString()
  } catch (e: unknown) {
    message.error(String(e))
  } finally {
    tab.isSqlFilterLoading = false
  }
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
    message.success(`${t('resultPanel.rows')}: ${visibleRows.length} → DuckDB`)
  } catch (e: any) {
    message.error(String(e))
  } finally {
    tab.isDuckdbLoading = false
  }
}

// ─── 操作 ───────────────────────────────────────────────
function handleCopySql() {
  if (activeTab.value) navigator.clipboard.writeText(activeTab.value.originalSql)
}
function handleRefresh(tab: ResultTab) {
  sqlExecutionStore.requestRefresh(tab.id)
}
async function handleSave(tab: ResultTab) {
  const cells = dirtyCells.value
  if (cells.size === 0) return

  let successCount = 0
  let failCount = 0

  for (const [, cell] of cells) {
    try {
      const result = await apiSaveCellUpdate({
        conn_id: tab.connectionId,
        table_name: tab.tableName,
        column_name: cell.colId,
        new_value: cell.newValue,
        row_identity: buildRowIdentity(tab, cell.rowIndex, cell.colId),
      })
      if (result.success) {
        const row = tab.objectRows[cell.rowIndex] as Record<string, unknown>
        row[cell.colId.replace(/\./g, '_')] = cell.newValue
        successCount++
      } else {
        failCount++
      }
    } catch {
      failCount++
    }
  }

  dirtyCells.value = new Map()
  if (failCount > 0) {
    message.warning(t('resultPanel.savePartial', { success: successCount, fail: failCount }))
  } else {
    message.success(t('resultPanel.saveSuccess', { count: successCount }))
  }
}

function buildRowIdentity(tab: ResultTab, rowIndex: number, excludeCol: string): Record<string, unknown> {
  const oldRow = tab.objectRows[rowIndex]
  if (!oldRow) return {}
  const identity: Record<string, unknown> = {}
  for (const col of tab.columns) {
    const key = col.replace(/\./g, '_')
    if (key !== excludeCol.replace(/\./g, '_')) {
      identity[col] = (oldRow as Record<string, unknown>)[key] ?? null
    }
  }
  return identity
}

async function handleCancel(tab: ResultTab) {
  const cells = dirtyCells.value
  if (cells.size === 0) return

  for (const [, cell] of cells) {
    const row = tab.objectRows[cell.rowIndex] as Record<string, unknown>
    row[cell.colId.replace(/\./g, '_')] = cell.oldValue
  }
  tab.objectRows = [...tab.objectRows]
  dirtyCells.value = new Map()
  message.info(t('resultPanel.changesReverted'))
}
async function handleExport(format: string) {
  const tab = activeTab.value
  if (!tab) return

  if (format === 'csv') {
    if (!gridApi.value) return
    gridApi.value.exportDataAsCsv({ fileName: `result_${Date.now()}.csv`, columnSeparator: ',' })
    return
  }
  if (format === 'json') {
    const data = JSON.stringify(rowData.value, null, 2)
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `result_${Date.now()}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    return
  }
  if (format === 'insert') {
    copyRowsAsInsert()
    return
  }

  // DuckDB COPY TO: parquet / xlsx
  const ext = format === 'parquet' ? 'parquet' : 'xlsx'
  const filterLabel = format === 'parquet' ? 'Parquet' : 'Excel'
  const filePath = await save({
    defaultPath: `result_${Date.now()}.${ext}`,
    filters: [
      { name: `${filterLabel} 文件`, extensions: [ext] },
    ],
  })
  if (!filePath) return

  let tempTable = tab.duckdbTempTable
  if (!tempTable) {
    try {
      tempTable = await apiCreateTempTable({
        conn_id: tab.connectionId,
        columns: tab.columns,
        rows: tab.rows,
      })
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      message.error(`创建临时表失败: ${msg}`)
      return
    }
  }

  try {
    await apiExportResult({
      temp_table: tempTable,
      file_path: filePath,
      format,
    })
    message.success(`已导出到 ${filePath}`)
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    message.error(`导出失败: ${msg}`)
  }
}

function copyRowsAsInsert() {
  if (!activeTab.value) return
  const cols = activeTab.value.columns
  const rows = gridApi.value?.getSelectedRows() || rowData.value
  const inserts = rows.map((row: any) => {
    const vals = cols
      .map(c => {
        const v = row[c]
        if (v === null || v === undefined) return 'NULL'
        if (typeof v === 'number') return String(v)
        return `'${String(v).replace(/'/g, "''")}'`
      })
      .join(', ')
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
        const text = rows
          .map((r: any) => tab.columns.map(c => String(r[c] ?? '')).join('\t'))
          .join('\n')
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
        insightStore.loadColumnInsight(tab.duckdbTempTable, col)
      } else {
        message.warning(t('resultPanel.needDuckdbFirst'))
      }
      break
    case 'openColumnVisualization':
      if (tab.duckdbTempTable) {
        insightStore.autoOpenVisualization = true
        insightStore.loadColumnInsight(tab.duckdbTempTable, col)
      } else {
        message.warning(t('resultPanel.needDuckdbFirst'))
      }
      break
    case 'sortAsc':
      if (gridApi.value) {
        const api = gridApi.value as unknown as {
          applySortState: (s: Array<{ colId: string; sort: string }>) => void
        }
        api.applySortState([{ colId: col, sort: 'asc' }])
      }
      break
    case 'sortDesc':
      if (gridApi.value) {
        const api = gridApi.value as unknown as {
          applySortState: (s: Array<{ colId: string; sort: string }>) => void
        }
        api.applySortState([{ colId: col, sort: 'desc' }])
      }
      break
    case 'sendSortToSql':
      tab.filterMode = 'sql'
      tab.sqlFilterExpression = `1=1 ORDER BY ${col} ${payload.sortDir === 'desc' ? 'DESC' : 'ASC'}`
      break
    case 'sendSortToDuckdb':
      tab.filterMode = 'duckdb'
      tab.duckdbSql = `SELECT * FROM result_temp ORDER BY ${col} ${payload.sortDir === 'desc' ? 'DESC' : 'ASC'} LIMIT 1000`
      break
    case 'hideColumn':
      if (gridApi.value) gridApi.value.setColumnsVisible([col], false)
      break
    case 'autoSizeColumn':
      if (gridApi.value) gridApi.value.autoSizeColumns([col])
      break
    case 'autoSizeAll':
      if (gridApi.value) {
        const allCols: string[] = []
        const columns = gridApi.value.getColumns()
        if (columns) {
          columns.forEach(c => {
            if (c.getColDef().field !== '__rowNumber') allCols.push(c.getId())
          })
        }
        if (allCols.length > 0) gridApi.value.autoSizeColumns(allCols)
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
  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault()
    if (activeTab.value) handleSave(activeTab.value)
  }
  if ((event.ctrlKey || event.metaKey) && event.key === 'r') {
    event.preventDefault()
    if (activeTab.value) handleRefresh(activeTab.value)
  }
}
function handleGlobalKeyDown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key === 'c') {
    if (document.activeElement?.closest('.ag-cell') && activeTab.value) {
      const tab = activeTab.value
      const selected = gridApi.value?.getSelectedRows() || []
      const rows = selected.length > 0 ? selected : rowData.value
      const text = rows
        .map((r: Record<string, unknown>) => tab.columns.map(c => String(r[c] ?? '')).join('\t'))
        .join('\n')
      navigator.clipboard.writeText(tab.columns.join('\t') + '\n' + text)
    }
  }
}
</script>

<style scoped>
.query-result-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--bg-primary);
}
.query-result-panel.compact .result-tabs {
  height: 24px;
  font-size: 10px;
}
.query-result-panel.compact .result-tab {
  padding: 0 6px;
}

/* ─── 标签栏 ─────────────────────────────────────────── */
.result-tabs {
  display: flex;
  align-items: stretch;
  height: 26px;
  flex-shrink: 0;
  background: var(--bg-tertiary, #2d2d30);
  border-bottom: 1px solid var(--border-color, #3e3e42);
  overflow-x: auto;
}
.result-tab {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 0 8px;
  font-size: 11px;
  cursor: pointer;
  color: var(--text-secondary, #888);
  white-space: nowrap;
  border-right: 1px solid var(--border-color, #3e3e42);
  user-select: none;
}
.result-tab:hover {
  background: var(--bg-hover, #333);
  color: var(--text-primary);
}
.result-tab.active {
  background: var(--bg-primary);
  color: var(--text-primary);
  border-bottom: 2px solid var(--primary-color, #0078d4);
}
.tab-close {
  font-size: 13px;
  line-height: 1;
  opacity: 0.5;
  margin-left: 2px;
}
.tab-close:hover {
  opacity: 1;
}

/* ─── 顶条 ───────────────────────────────────────────── */
.toolbar-strip {
  display: flex;
  align-items: center;
  padding: 1px 4px;
  gap: 4px;
  flex-shrink: 0;
  min-height: 26px;
  background: var(--bg-secondary, #252526);
  border-bottom: 1px solid var(--border-color, #333);
}
.toolbar-strip .strip-right {
  flex: 1;
  min-width: 0;
}

/* ─── 主体布局 ────────────────────────────────────────── */
.result-body {
  flex: 1;
  min-height: 0;
  display: flex;
  position: relative;
}

/* 左侧栏 */
.view-sidebar {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding: 2px;
  flex-shrink: 0;
  background: var(--bg-tertiary, #2d2d30);
  border-right: 1px solid var(--border-color, #3e3e42);
}
.view-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: var(--text-secondary, #888);
  cursor: pointer;
  transition: all 0.12s;
}
.view-btn:hover {
  background: var(--bg-hover, #3c3c3c);
  color: var(--text-primary);
}
.view-btn.active {
  background: var(--primary-color);
  color: #fff;
}

/* 中间网格 */
.grid-area {
  flex: 1;
  min-width: 0;
  position: relative;
  overflow: hidden;
}
:deep(.ag-theme-alpine),
:deep(.ag-theme-alpine-dark) {
  height: 100% !important;
  font-size: 11px;
}
:deep(.ag-root-wrapper) {
  border: none;
}
:deep(.ag-header) {
  min-height: 24px !important;
}
:deep(.ag-header-cell) {
  font-size: 10px;
  font-weight: 600;
  padding: 0 4px !important;
}
:deep(.ag-header-cell-label) {
  padding: 0;
}
:deep(.ag-row) {
  font-size: 11px;
  min-height: 22px !important;
}
:deep(.ag-cell) {
  padding: 0 4px !important;
  line-height: 22px !important;
}
:deep(.null-value) {
  color: #999;
  font-style: italic;
  font-size: 10px;
}
:deep(.text-right) {
  text-align: right;
  font-family: monospace;
}
:deep(.ag-pinned-left-cols-container) {
  border-right: 1px solid var(--border-color, #444);
}
:deep(.ag-row-even) {
  background: var(--bg-row-even, rgba(128, 128, 128, 0.03));
}
:deep(.ag-row-odd) {
  background: transparent;
}
:deep(.ag-row:hover) {
  background: var(--bg-hover, rgba(255, 255, 255, 0.04)) !important;
}
/* 自动列宽 */
:deep(.ag-cell) {
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 文本视图样式由 ResultTextView.vue 管理 */

/* 记录视图 */
.record-view {
  height: 100%;
  overflow-y: auto;
  padding: 6px;
  display: flex;
  flex-direction: column;
}
.record-nav {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
  font-size: 11px;
  flex-shrink: 0;
}
.record-nav-text {
  font-family: monospace;
  font-size: 11px;
  color: var(--text-secondary, #999);
  min-width: 60px;
  text-align: center;
}

/* 右侧值查看器 */
.value-viewer {
  width: 220px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--border-color, #3e3e42);
  background: var(--bg-secondary, #252526);
}
.viewer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 6px;
  border-bottom: 1px solid var(--border-color, #333);
}
.viewer-title {
  font-size: 11px;
  font-weight: 600;
}
.viewer-content {
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow: auto;
}
.viewer-field {
  display: flex;
  gap: 6px;
  font-size: 10px;
}
.field-label {
  font-weight: 600;
  color: var(--text-secondary);
  min-width: 24px;
}
.field-val {
  font-family: monospace;
}
.viewer-text {
  width: 100%;
  min-height: 100px;
  border: 1px solid var(--border-color, #333);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-family: monospace;
  font-size: 10px;
  padding: 4px;
  resize: none;
}
.viewer-toggle {
  position: absolute;
  top: 4px;
  right: 2px;
  z-index: 10;
}

/* ─── 底部状态栏（操作/行信息/分页合并） ──────────────── */
.result-statusbar {
  display: flex;
  align-items: center;
  height: 24px;
  padding: 0 4px;
  gap: 4px;
  flex-shrink: 0;
  font-size: 10px;
  color: var(--text-secondary, #999);
  background: var(--bg-tertiary, #2d2d30);
  border-top: 1px solid var(--border-color, #3e3e42);
}
.sbar-left,
.sbar-center,
.sbar-right {
  display: flex;
  align-items: center;
  gap: 1px;
}
.sbar-center {
  flex: 1;
  justify-content: center;
  gap: 6px;
}
.sbar-right {
  gap: 1px;
}
.mode-badge {
  padding: 0 4px;
  border-radius: 2px;
  font-size: 9px;
  font-weight: 600;
  line-height: 16px;
}
.mode-badge.quick {
  background: #2d6a4f33;
  color: #52c41a;
}
.mode-badge.sql {
  background: #1a5a8a33;
  color: #1890ff;
}
.mode-badge.duckdb {
  background: #613a8a33;
  color: #b37feb;
}
.separator {
  color: var(--border-color, #444);
}
.row-info {
  font-family: monospace;
}
.exec-time {
  color: var(--primary-color);
  font-family: monospace;
}
.page-indicator {
  font-family: monospace;
  margin: 0 2px;
}

.analysis-notice {
  display: flex;
  align-items: center;
  height: 20px;
  padding: 0 6px;
  background: #f5eec033;
  border-bottom: 1px solid #d4c78e66;
  font-size: 10px;
  color: #b8a44c;
  flex-shrink: 0;
}
.empty-icon {
  opacity: 0.4;
}
.empty-text {
  font-size: 13px;
  color: var(--text-secondary, #888);
}
</style>
