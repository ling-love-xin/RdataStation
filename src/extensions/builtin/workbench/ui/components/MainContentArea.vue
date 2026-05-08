<template>
  <div ref="containerRef" class="main-content-area">
    <!-- SQL Editor Area -->
    <div
      class="sql-editor-area"
      :style="{ height: `${sqlEditorHeight}px` }"
      :class="uiStore.isDark ? 'dockview-theme-dark' : 'dockview-theme-light'"
    >
      <DockviewVue ref="dockviewRef" class="dockview" @ready="onReady" />
    </div>

    <!-- Resizable Divider -->
    <div v-if="resultStore.showPanel" class="vertical-divider" @mousedown="startResize">
      <div class="divider-handle"></div>
    </div>

    <!-- Result Panel Area -->
    <div v-if="resultStore.showPanel" class="panel-area" :style="{ height: `${panelHeight}px` }">
      <div class="panel-content">
        <QueryResultPanel />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { DockviewVue, type DockviewReadyEvent } from 'dockview-vue'
import { ref } from 'vue'

import { panelRegistry } from '@/core/panel-registry'
import QueryResultPanel from '@/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue'
import { useResultStore } from '@/extensions/builtin/workbench/ui/stores/result-store'
import { useUiStore } from '@/shared/stores/ui'

const uiStore = useUiStore()
const resultStore = useResultStore()

const containerRef = ref<HTMLElement | null>(null)
const dockviewRef = ref<InstanceType<typeof DockviewVue> | null>(null)

const sqlEditorHeight = ref(400)
const panelHeight = ref(300)
const isResizing = ref(false)
const resizeStartY = ref(0)
const resizeStartHeight = ref(0)

// Event handlers
function onReady(event: DockviewReadyEvent) {
  const api = event.api

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
        direction: 'within',
      }
      api.addPanel(panelConfig)
    }
  })
}

// Panel management

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
  panelHeight.value = containerRef.value
    ? containerRef.value.offsetHeight - sqlEditorHeight.value - 4
    : 300
}

function stopResize() {
  isResizing.value = false
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}
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
  background-color: var(--primary-color, #165dff);
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
  color: var(--danger-color, #f53f3f);
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
