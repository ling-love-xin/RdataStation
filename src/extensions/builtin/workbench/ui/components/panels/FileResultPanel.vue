<template>
  <div class="file-result-panel">
    <GridToolbar
      :row-count="metadata.totalRowCount"
      :elapsed-ms="metadata.elapsedMs"
      :is-duckdb-ready="metadata.duckdbTable !== null"
      @close="handleClose"
      @export-csv="handleExportCsv"
      @export-json="handleExportJson"
    />

    <div class="result-grid-wrapper">
      <div v-if="isGridReady" ref="gridContainerRef" class="ag-grid-container" />
      <div v-else class="grid-placeholder">
        <NSpin v-if="isLoading" size="medium" />
        <span v-else class="placeholder-text">点击结果 Tab 以加载数据</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NSpin } from 'naive-ui'
import { ref, onMounted, onBeforeUnmount, nextTick, shallowRef, watch } from 'vue'

import { EditorManager } from '@/extensions/builtin/workbench/manager/EditorManager'
import type { ResultSetMetadata, GridStateSnapshot } from '@/extensions/builtin/workbench/types/editor-types'

import GridToolbar from './GridToolbar.vue'

interface GridInstance {
  destroy(): void
  setGridOption(key: string, value: unknown): void
  getColumnState(): unknown[]
  getFilterModel(): unknown
  getSortModel(): unknown[]
}

interface PanelApi {
  isActive: boolean
}

const props = defineProps<{
  params: Record<string, unknown>
}>()

const emit = defineEmits<{
  close: []
}>()

const panelApi = (props.params.api as PanelApi) ?? null

const gridContainerRef = ref<HTMLElement | null>(null)
const gridInstance = shallowRef<GridInstance | null>(null)
const isGridReady = ref(false)
const isLoading = ref(false)

const metadata = ref<ResultSetMetadata>({
  id: String(props.params.resultSetId ?? ''),
  title: 'Result',
  columns: (props.params.columns as string[]) ?? [],
  totalRowCount: 0,
  elapsedMs: 0,
  affectedRows: 0,
  messages: '',
  sql: '',
  timestamp: Date.now(),
  dataSource: 'memory',
  duckdbTable: null,
  rows: (props.params.rows as unknown[][]) ?? [],
})

const savedGridState = ref<GridStateSnapshot | null>(null)

function saveGridState(): GridStateSnapshot | null {
  const inst = gridInstance.value
  if (!inst) return null
  try {
    return {
      columnState: inst.getColumnState() as Record<string, unknown>[],
      filterModel: inst.getFilterModel() as Record<string, unknown>,
      sortModel: inst.getSortModel() as Record<string, unknown>[],
    }
  } catch {
    return null
  }
}

async function createGrid(): Promise<void> {
  if (isGridReady.value) return
  isLoading.value = true

  const startTs = performance.now()

  await nextTick()

  const el = gridContainerRef.value
  if (!el) {
    isLoading.value = false
    return
  }

  try {
    const { createGrid: createAgGrid } = await import(
      '@/extensions/builtin/workbench/ui/composables/useGridConfig'
    )

    const instance = await createAgGrid(el, {
      columns: metadata.value.columns,
      rows: metadata.value.rows ?? [],
    })

    if (instance) {
      gridInstance.value = instance as unknown as GridInstance
      isGridReady.value = true

      const elapsed = Math.round(performance.now() - startTs)
      if (elapsed > 200) {
        console.warn(`[Perf] AG Grid create: ${elapsed}ms (>200ms target) | cols=${metadata.value.columns.length} rows=${(metadata.value.rows ?? []).length}`)
      } else {
        console.info(`[Perf] AG Grid create: ${elapsed}ms | cols=${metadata.value.columns.length} rows=${(metadata.value.rows ?? []).length}`)
      }

      if (savedGridState.value) {
        const state = savedGridState.value
        if (state.columnState) {
          instance.setGridOption('columnState', state.columnState)
        }
        if (state.filterModel) {
          instance.setGridOption('filterModel', state.filterModel)
        }
        if (state.sortModel) {
          instance.setGridOption('sortModel', state.sortModel)
        }
      }
    }
  } catch (err) {
    console.error('[FileResultPanel] Failed to create grid:', err)
  } finally {
    isLoading.value = false
  }
}

function destroyGrid(): void {
  if (isGridReady.value) {
    savedGridState.value = saveGridState()
  }

  if (gridInstance.value) {
    try {
      gridInstance.value.destroy()
    } catch {
      /* grid may already be destroyed */
    }
    gridInstance.value = null
  }
  isGridReady.value = false
}

onMounted(() => {
  if (panelApi?.isActive) {
    createGrid()
  }
})

watch(
  () => panelApi?.isActive,
  (active, prev) => {
    if (active === prev) return
    if (active) {
      const filePath = String(props.params.filePath ?? '')
      const resultSetId = String(props.params.resultSetId ?? '')
      if (filePath && resultSetId) {
        const info = EditorManager.openFiles.get(filePath)
        if (info) {
          const idx = info.resultSets.findIndex(rs => rs.id === resultSetId)
          if (idx >= 0) {
            EditorManager.setActiveResultIndex(filePath, idx)
          }
        }
      }
      createGrid()
    } else {
      destroyGrid()
    }
  },
)

onBeforeUnmount(() => {
  destroyGrid()
})

function handleClose(): void {
  emit('close')
}

async function handleExportCsv(): Promise<void> {
  const { exportCSV } = await import('@/extensions/builtin/workbench/ui/composables/useResultExport')
  if (gridInstance.value) {
    await exportCSV(gridInstance.value as unknown as Record<string, unknown>)
  }
}

async function handleExportJson(): Promise<void> {
  const { exportJSON } = await import('@/extensions/builtin/workbench/ui/composables/useResultExport')
  if (gridInstance.value) {
    await exportJSON(gridInstance.value as unknown as Record<string, unknown>)
  }
}
</script>

<style scoped>
.file-result-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.result-grid-wrapper {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.ag-grid-container {
  width: 100%;
  height: 100%;
}

.grid-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--n-text-color-disabled);
  font-size: 13px;
}
</style>