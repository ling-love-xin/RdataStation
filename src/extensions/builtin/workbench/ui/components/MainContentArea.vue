<template>
  <div ref="containerRef" class="main-content-area">
    <!-- SQL Editor Area -->
    <div
      class="sql-editor-area"
      :style="{ height: `${sqlEditorHeight}px` }"
      :class="uiStore.isDark ? 'dockview-theme-dark' : 'dockview-theme-light'"
    >
      <DockviewVue
        ref="dockviewRef"
        class="dockview"
        @ready="onReady"
      />
    </div>

    <!-- Resizable Divider -->
    <div
      v-if="showPanel"
      class="vertical-divider"
      @mousedown="startResize"
    >
      <div class="divider-handle"></div>
    </div>

    <!-- Result Panel Area -->
    <div
      v-if="showPanel"
      class="panel-area"
      :style="{ height: `${panelHeight}px` }"
    >
      <div class="panel-tabs">
        <div
          v-for="(result, index) in resultSets"
          :key="index"
          :class="['panel-tab', { active: activeResultIndex === index }]"
          @click="activeResultIndex = index"
        >
          {{ result.name || `Result ${index + 1}` }}
          <span class="close-btn" @click.stop="closeResult(index)">×</span>
        </div>
        <button class="add-tab-btn" @click="addNewResultTab">+</button>
      </div>
      <div class="panel-content">
        <QueryResultPanel
          v-if="activeResult"
          :result="activeResult"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { DockviewVue, type DockviewReadyEvent } from 'dockview-vue'
import { ref, computed, onMounted, onUnmounted } from 'vue'

import { panelRegistry } from '@/core/panel-registry'
import QueryResultPanel from '@/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue'
import { useUiStore } from '@/shared/stores/ui'

interface ResultSet {
  name: string
  data: unknown[]
  columns: string[]
  rowCount: number
  executionTime?: number
}

const uiStore = useUiStore()

const containerRef = ref<HTMLElement | null>(null)
const dockviewRef = ref<InstanceType<typeof DockviewVue> | null>(null)

// Panel visibility
const showPanel = ref(false)
const resultSets = ref<ResultSet[]>([])
const activeResultIndex = ref(0)

// Resize state
const sqlEditorHeight = ref(400)
const panelHeight = ref(300)
const isResizing = ref(false)
const resizeStartY = ref(0)
const resizeStartHeight = ref(0)

// Computed
const activeResult = computed(() => {
  if (activeResultIndex.value >= 0 && activeResultIndex.value < resultSets.value.length) {
    return resultSets.value[activeResultIndex.value]
  }
  return null
})

// Event handlers
function onReady(event: DockviewReadyEvent) {
  const api = event.api as any

  // Get all registered panels
  const panels = panelRegistry.getAll()
  console.log(`[MainContent] Creating ${panels.length} panels from registry`)

  // Filter panels by location (center/bottom)
  const centerPanels = panels.filter(p => p.location === 'center')
  const bottomPanels = panels.filter(p => p.location === 'bottom')

  // Create center panels (SQL Editor)
  let centerPanelId: string | null = null
  centerPanels.forEach((panel, index) => {
    const panelConfig: Record<string, unknown> = {
      id: `panel_${panel.id}`,
      component: panel.id,
      title: panel.name,
    }

    if (index === 0) {
      api.addPanel(panelConfig)
      centerPanelId = `panel_${panel.id}`
    } else if (centerPanelId) {
      panelConfig.position = {
        referencePanel: centerPanelId,
        direction: 'within'
      }
      api.addPanel(panelConfig)
    }
  })
}

// Panel management
function addNewResultTab() {
  const newIndex = resultSets.value.length + 1
  resultSets.value.push({
    name: `Result ${newIndex}`,
    data: [],
    columns: [],
    rowCount: 0
  })
  activeResultIndex.value = resultSets.value.length - 1
}

function closeResult(index: number) {
  resultSets.value.splice(index, 1)
  if (activeResultIndex.value >= resultSets.value.length) {
    activeResultIndex.value = Math.max(0, resultSets.value.length - 1)
  }
  if (resultSets.value.length === 0) {
    showPanel.value = false
  }
}

// Resize handling
function startResize(event: MouseEvent) {
  isResizing.value = true
  resizeStartY.value = event.clientY
  resizeStartHeight.value = sqlEditorHeight.value

  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'row-resize'
  document.body.style.userSelect = 'none'
}

function handleMouseMove(event: MouseEvent) {
  if (!isResizing.value) return

  const deltaY = resizeStartY.value - event.clientY
  const newSqlEditorHeight = resizeStartHeight.value + deltaY

  // Constrain to min/max
  sqlEditorHeight.value = Math.max(100, Math.min(800, newSqlEditorHeight))
  panelHeight.value = containerRef.value ? containerRef.value.offsetHeight - sqlEditorHeight.value - 4 : 300
}

function stopResize() {
  isResizing.value = false
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

// Listen for SQL execution results
function handleSqlExecutionResult(event: CustomEvent) {
  const { result, results } = event.detail || {}

  if (results && Array.isArray(results) && results.length > 0) {
    // Multi-statement results
    results.forEach((r: any, index: number) => {
      resultSets.value.push({
        name: `Result ${resultSets.value.length + 1}`,
        data: r.rows || [],
        columns: r.columns || [],
        rowCount: r.rowCount || 0,
        executionTime: r.executionTime
      })
    })
    activeResultIndex.value = resultSets.value.length - 1
  } else if (result) {
    // Single result
    resultSets.value.push({
      name: `Result ${resultSets.value.length + 1}`,
      data: result.rows || [],
      columns: result.columns || [],
      rowCount: result.rowCount || 0,
      executionTime: result.executionTime
    })
    activeResultIndex.value = resultSets.value.length - 1
  }

  showPanel.value = true
}

onMounted(() => {
  window.addEventListener('sql-execution-result', handleSqlExecutionResult as any)
})

onUnmounted(() => {
  window.removeEventListener('sql-execution-result', handleSqlExecutionResult as any)
})
</script>

<style scoped>
.main-content-area {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background-color: var(--bg-primary, #1e1e1e);
}

.sql-editor-area {
  flex-shrink: 0;
  overflow: hidden;
}

.dockview {
  height: 100%;
  width: 100%;
}

.vertical-divider {
  height: 4px;
  background-color: var(--border-color, #3e3e42);
  cursor: row-resize;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s ease;
}

.vertical-divider:hover {
  background-color: var(--primary-color, #165DFF);
}

.divider-handle {
  width: 32px;
  height: 2px;
  background-color: var(--text-tertiary, #858585);
  border-radius: 1px;
}

.vertical-divider:hover .divider-handle {
  background-color: white;
}

.panel-area {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-secondary, #252526);
  overflow: hidden;
}

.panel-tabs {
  display: flex;
  align-items: center;
  height: 35px;
  background-color: var(--bg-tertiary, #2d2d30);
  border-bottom: 1px solid var(--border-color, #3e3e42);
  padding: 0 4px;
  gap: 2px;
  overflow-x: auto;
}

.panel-tab {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 12px;
  height: 30px;
  font-size: 12px;
  color: var(--text-secondary, #858585);
  background-color: transparent;
  border-radius: 4px 4px 0 0;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s ease;
}

.panel-tab:hover {
  color: var(--text-primary, #cccccc);
  background-color: var(--bg-hover, #3c3c3c);
}

.panel-tab.active {
  color: var(--text-primary, #cccccc);
  background-color: var(--bg-secondary, #252526);
}

.close-btn {
  font-size: 14px;
  line-height: 1;
  color: var(--text-tertiary, #858585);
  margin-left: 4px;
}

.close-btn:hover {
  color: var(--danger-color, #F53F3F);
}

.add-tab-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  color: var(--text-tertiary, #858585);
  background: none;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  margin-left: 4px;
}

.add-tab-btn:hover {
  color: var(--text-primary, #d4d4d4);
  background-color: var(--bg-hover, rgba(255, 255, 255, 0.1));
}

.panel-content {
  flex: 1;
  overflow: hidden;
}
</style>
