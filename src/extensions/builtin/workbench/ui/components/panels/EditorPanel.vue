<template>
  <div :class="['editor-panel', `toolbar-position-top`]">
    <EditorToolbar
      v-if="showToolbar"
      :toolbar-position="'top'"
      :is-duck-db="false"
      :show-advanced="isAnalysisMode"
      @execute="handleExecute"
      @execute-new="handleExecuteNew"
      @format="handleFormat"
      @validate="handleValidate"
      @duckdb-execute="handleDuckDbExecute"
    />

    <div class="editor-body">
      <div class="tab-bar">
        <div
          v-for="tab in tabs"
          :key="tab.key"
          :class="['tab-item', { active: tab.isActive }]"
          :title="tab.key"
          @click="handleTabClick(tab.key)"
          @mousedown="closeOnMiddleClick($event, tab.key)"
          @contextmenu.prevent="onTabContextMenu($event, tab.key)"
        >
          <span class="tab-icon">
            <FileCode v-if="tab.language === 'sql'" :size="13" :stroke-width="1.5" />
            <File v-else :size="13" :stroke-width="1.5" />
          </span>
          <span class="tab-label">{{ tab.label }}</span>
          <span v-if="tab.isDirty" class="tab-dirty" />
          <span
            :class="['tab-close', { always: tab.isDirty }]"
            @click.stop="handleTabClose(tab.key)"
          >
            <X :size="12" :stroke-width="1.5" />
          </span>
        </div>
      </div>

      <Teleport to="body">
        <div
          v-if="contextMenu.visible"
          class="tab-context-menu"
          :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
          @click.stop
        >
          <div class="context-menu-item" @click="handleCloseFromMenu">关闭</div>
          <div class="context-menu-item" @click="handleCloseOthersFromMenu">关闭其他</div>
          <div class="context-menu-item" @click="handleCloseRightFromMenu">关闭右侧</div>
          <div class="context-menu-item" @click="handleCloseAllFromMenu">关闭所有</div>
        </div>
        <div
          v-if="contextMenu.visible"
          class="context-menu-backdrop"
          @click="closeContextMenu"
          @contextmenu.prevent="closeContextMenu"
        />
      </Teleport>

      <div class="editor-split">
        <div
          class="editor-area"
          :style="{ flex: hasResults ? `${splitRatio}` : '1 1 auto' }"
        >
          <div ref="editorContainerRef" class="monaco-container" />
          <EditorWelcome
            v-if="showWelcome"
            :visible="showWelcome"
            @connect="handleWelcomeConnect"
          />
        </div>

        <div
          v-if="hasResults"
          class="split-handle"
          @mousedown="startSplitDrag"
        />

        <div
          v-if="hasResults"
          class="result-area"
          :style="{ flex: `calc(1 - ${splitRatio})` }"
        >
          <ResultSubTab />
          <div ref="resultPanelHost" class="result-panel-host" />
        </div>
      </div>
    </div>

    <EditorStatusbar v-bind="statusbarProps" />
  </div>
</template>

<script setup lang="ts">
import { File, FileCode, X } from 'lucide-vue-next'
import * as monaco from 'monaco-editor'
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'

import { EditorManager } from '@/extensions/builtin/workbench/manager/EditorManager'
import { useUiStore } from '@/shared/stores/ui'
import { rdataDark, rdataLight } from '@/shared/styles/monaco-theme'

import EditorStatusbar from './EditorStatusbar.vue'
import EditorToolbar from './EditorToolbar.vue'
import EditorWelcome from './EditorWelcome.vue'
import ResultSubTab from './ResultSubTab.vue'

const uiStore = useUiStore()

const props = defineProps<{
  params: Record<string, unknown>
}>()

const editorContainerRef = ref<HTMLElement | null>(null)
const showWelcome = ref(true)
const cursorPosition = ref('Ln 1, Col 1')
const splitRatio = ref(0.55)
const isDragging = ref(false)

const currentFilePath = computed(() => String(props.params.filePath || ''))
const currentLanguage = computed(() => String(props.params.language || 'sql'))

const myFileInfo = computed(() => EditorManager.openFiles.get(currentFilePath.value))

const editorMode = computed(() => {
  const info = myFileInfo.value
  if (!info) return 'Plain Text'
  if (info.language === 'sql') return 'SQL'
  return info.language
})

const isDirty = computed(() => myFileInfo.value?.isDirty ?? false)

const statusbarProps = computed(() => ({
  cursorPosition: cursorPosition.value,
  selectedTextInfo: '',
  editorMode: editorMode.value,
  executing: EditorManager.isExecuting,
  canCancel: EditorManager.isExecuting,
  lastExecutionTime: null,
  connectionInfoText: myFileInfo.value?.connectionId || '',
  popselectOptions: [],
  selectedConnection: myFileInfo.value?.connectionId || '',
  inTransaction: false,
  statementCount: 0,
  isDirty: isDirty.value,
}))

const showToolbar = computed(() => {
  const info = myFileInfo.value
  if (!info) return false
  return info.type === 'analysis' || info.language === 'sql'
})

const isAnalysisMode = computed(() => myFileInfo.value?.type === 'analysis')

const hasResults = computed(() => {
  const info = myFileInfo.value
  return info ? info.resultSets.length > 0 : false
})

const tabs = computed(() =>
  Array.from(EditorManager.openFiles.entries()).map(([path, info]) => ({
    key: path,
    label: info.fileName,
    isDirty: info.isDirty,
    isActive: path === EditorManager.activeFilePath,
    language: info.language,
  }))
)

const contextMenu = ref<{ visible: boolean; x: number; y: number; filePath: string }>({
  visible: false, x: 0, y: 0, filePath: '',
})

function onTabContextMenu(e: MouseEvent, filePath: string) {
  e.preventDefault()
  contextMenu.value = { visible: true, x: e.clientX, y: e.clientY, filePath }
}

function closeContextMenu() {
  contextMenu.value.visible = false
}

function handleCloseFromMenu() {
  const fp = contextMenu.value.filePath
  if (fp) EditorManager.closeFile(fp)
  closeContextMenu()
}

function handleCloseOthersFromMenu() {
  const fp = contextMenu.value.filePath
  for (const [path] of EditorManager.openFiles) {
    if (path !== fp) EditorManager.closeFile(path)
  }
  closeContextMenu()
}

function handleCloseRightFromMenu() {
  const fp = contextMenu.value.filePath
  const paths = Array.from(EditorManager.openFiles.keys())
  const idx = paths.indexOf(fp)
  if (idx >= 0) {
    for (let i = idx + 1; i < paths.length; i++) {
      EditorManager.closeFile(paths[i])
    }
  }
  closeContextMenu()
}

function handleCloseAllFromMenu() {
  for (const [path] of EditorManager.openFiles) {
    EditorManager.closeFile(path)
  }
  closeContextMenu()
}

function handleTabClick(filePath: string) {
  if (filePath !== EditorManager.activeFilePath) {
    EditorManager.switchToFile(filePath)
  }
}

function closeOnMiddleClick(e: MouseEvent, filePath: string) {
  if (e.button === 1) {
    e.preventDefault()
    handleTabClose(filePath)
  }
}

function handleTabClose(name: string) {
  EditorManager.closeFile(name)
}

function handleExecute() {
  EditorManager.executeCurrentSQL()
}

function handleExecuteNew() {
  EditorManager.executeNewTabSQL()
}

function handleFormat() {
  EditorManager.formatSQL()
}

function handleValidate() {
  EditorManager.validateSQL()
}

function handleDuckDbExecute() {
  EditorManager.executeDuckDBAccelerated()
}

function handleWelcomeConnect() {
  showWelcome.value = false
}

function startSplitDrag(e: MouseEvent) {
  e.preventDefault()
  isDragging.value = true
  document.addEventListener('mousemove', onSplitDrag)
  document.addEventListener('mouseup', stopSplitDrag)
}

function onSplitDrag(e: MouseEvent) {
  if (!isDragging.value) return
  const container = (e.target as HTMLElement).closest('.editor-split')
  if (!container) return
  const rect = container.getBoundingClientRect()
  const ratio = (e.clientY - rect.top) / rect.height
  splitRatio.value = Math.min(0.9, Math.max(0.1, ratio))
}

function stopSplitDrag() {
  isDragging.value = false
  document.removeEventListener('mousemove', onSplitDrag)
  document.removeEventListener('mouseup', stopSplitDrag)
}

onMounted(async () => {
  await nextTick()
  const el = editorContainerRef.value
  if (!el) return

  const theme = uiStore.theme === 'dark' ? 'rdata-dark' : 'rdata-light'
  monaco.editor.defineTheme('rdata-dark', rdataDark)
  monaco.editor.defineTheme('rdata-light', rdataLight)

  const ed = monaco.editor.create(el, {
    value: '',
    language: currentLanguage.value,
    theme,
    automaticLayout: true,
    minimap: { enabled: true, scale: 1, showSlider: 'mouseover' },
    fontSize: 14,
    lineNumbers: 'on',
    scrollBeyondLastLine: false,
    wordWrap: 'off',
    tabSize: 2,
    renderWhitespace: 'selection',
  })

  ed.onDidChangeCursorPosition((e) => {
    cursorPosition.value = `Ln ${e.position.lineNumber}, Col ${e.position.column}`
  })

  ed.onDidChangeModelContent(() => {
    showWelcome.value = false
  })

  const fp = currentFilePath.value
  EditorManager.setEditor(ed)

  const myFileInfo = EditorManager.openFiles.get(fp)
  if (myFileInfo) {
    ed.setModel(myFileInfo.model)
  }
})

onUnmounted(() => {
})

watch(
  () => EditorManager.activeFileInfo,
  (info) => {
    if (info) {
      showWelcome.value = false
    }
  },
  { immediate: true }
)

watch(
  () => uiStore.theme,
  (theme) => {
    monaco.editor.setTheme(theme === 'dark' ? 'rdata-dark' : 'rdata-light')
  }
)
</script>

<style scoped>
.editor-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.editor-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.tab-bar {
  flex-shrink: 0;
  display: flex;
  align-items: stretch;
  height: 34px;
  background: var(--tab-bg, #1e1e1e);
  border-bottom: 1px solid var(--tab-border, #2d2d2d);
  overflow-x: auto;
  overflow-y: hidden;
}

.tab-bar::-webkit-scrollbar {
  height: 3px;
}

.tab-bar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 100%;
  min-width: 0;
  max-width: 180px;
  padding: 0 10px;
  font-size: 12px;
  color: var(--tab-inactive-fg, #858585);
  background: var(--tab-inactive-bg, #2d2d2d);
  border-right: 1px solid var(--tab-border, #252526);
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
  transition: background 0.12s ease, color 0.12s ease;
  position: relative;
}

.tab-item:last-of-type {
  border-right: none;
}

.tab-item:hover {
  background: var(--tab-hover-bg, #353535);
  color: var(--tab-hover-fg, #cccccc);
}

.tab-item.active {
  background: var(--tab-active-bg, #1e1e1e);
  color: var(--tab-active-fg, #e0e0e0);
  border-bottom: 2px solid var(--tab-accent, #0066b8);
}

.tab-item.active .tab-label {
  font-weight: 500;
}

.tab-icon {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  color: inherit;
  opacity: 0.65;
}

.tab-item.active .tab-icon {
  opacity: 0.9;
}

.tab-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.tab-dirty {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--tab-dirty, #f0c040);
  flex-shrink: 0;
  animation: dirty-pulse 2s ease-in-out infinite;
}

@keyframes dirty-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 4px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.1s ease, background 0.1s ease;
}

.tab-item:hover .tab-close,
.tab-item .tab-close.always {
  opacity: 0.6;
}

.tab-close:hover {
  opacity: 1 !important;
  background: var(--tab-close-hover-bg, rgba(255, 255, 255, 0.12));
  color: var(--tab-close-hover-fg, #ffffff);
}

.tab-item.active .tab-close {
  opacity: 0;
}

.tab-item.active:hover .tab-close,
.tab-item.active .tab-close.always {
  opacity: 0.5;
}

.tab-item.active .tab-close:hover {
  opacity: 1 !important;
}

.editor-split {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-area {
  min-height: 60px;
  overflow: hidden;
  position: relative;
}

.monaco-container {
  width: 100%;
  height: 100%;
}

.split-handle {
  height: 4px;
  cursor: row-resize;
  background: var(--n-border-color);
  flex-shrink: 0;
}

.split-handle:hover {
  background: var(--n-color-target);
}

.result-area {
  min-height: 80px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.result-panel-host {
  flex: 1;
  overflow: hidden;
}
</style>

<style>
.tab-context-menu {
  position: fixed;
  z-index: 10001;
  background: var(--menu-bg, #252526);
  border: 1px solid var(--menu-border, #3c3c3c);
  border-radius: 6px;
  padding: 4px 0;
  min-width: 160px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.48);
  backdrop-filter: blur(8px);
}

.context-menu-item {
  padding: 7px 16px;
  font-size: 12px;
  color: var(--menu-fg, #cccccc);
  cursor: pointer;
  user-select: none;
  transition: background 0.08s ease;
}

.context-menu-item:hover {
  background: var(--menu-hover-bg, #094771);
  color: var(--menu-hover-fg, #ffffff);
}

.context-menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10000;
}
</style>
