<template>
  <div class="scratchpad-panel" @click="handleGlobalClick">
    <div class="toolbar">
      <NButton size="small" :disabled="isLoading" @click="handleCreateFile">
        <template #icon>
          <NIcon><FilePlus /></NIcon>
        </template>
        {{ t('scratchpad.newFile') }}
      </NButton>
      <NButton size="small" :disabled="isLoading" @click="handleImportFile">
        <template #icon>
          <NIcon><Upload /></NIcon>
        </template>
        {{ t('scratchpad.import') }}
      </NButton>
      <NButton size="small" :disabled="isLoading" @click="handleAddReference">
        <template #icon>
          <NIcon><FolderSymlink /></NIcon>
        </template>
        {{ t('scratchpad.reference') }}
      </NButton>
      <NButton size="small" :disabled="isLoading" @click="loadFiles">
        <template #icon>
          <NIcon><RefreshCw /></NIcon>
        </template>
      </NButton>
      <NInput
        v-model:value="searchQuery"
        size="small"
        :placeholder="contentSearchMode ? t('scratchpad.searchContent') + '…' : t('scratchpad.search')"
        clearable
        class="search-input"
      />
      <NButton size="small" :type="contentSearchMode ? 'primary' : 'default'" @click="toggleSearchMode">
        <template #icon>
          <NIcon><Search /></NIcon>
        </template>
      </NButton>
    </div>

    <div v-if="isLoading" class="loading-state">
      <NSpin size="small" />
    </div>

    <div v-else-if="error" class="error-state">
      <span class="error-text">{{ error }}</span>
      <NButton size="tiny" @click="loadFiles">{{ t('navigator.retry') }}</NButton>
    </div>

    <div v-else class="tree-container">
      <div v-if="externalReferences.length > 0" class="tree-group">
        <div class="group-header" @click="toggleGroup('external')">
          <NIcon size="14">
            <component :is="groupExpanded.external ? ChevronDown : ChevronRight" />
          </NIcon>
          <NIcon size="14"><FolderSymlink /></NIcon>
          <span class="group-title">{{ t('scratchpad.externalReferences') }}</span>
          <span class="group-count">{{ externalReferences.length }}</span>
        </div>
        <div v-show="groupExpanded.external" class="group-content">
          <div
            v-for="ref in externalReferences"
            :key="ref.alias"
            class="ref-row"
            @click="handleRefClick(ref)"
            @contextmenu.prevent="showRefMenu($event, ref)"
          >
            <NIcon size="14">
              <FolderSymlink :class="{ 'ref-invalid': isRefInvalid(ref) }" />
            </NIcon>
            <span class="ref-name">{{ ref.alias }}</span>
            <span
              :class="['ref-path', { 'ref-path-invalid': isRefInvalid(ref) }]"
              :title="ref.path"
              >{{ ref.path }}</span
            >
            <span v-if="isRefInvalid(ref)" class="ref-badge">{{ t('scratchpad.invalid') }}</span>
          </div>
        </div>
      </div>

      <div class="tree-group">
        <div class="group-header" @click="toggleGroup('local')">
          <NIcon size="14">
            <component :is="groupExpanded.local ? ChevronDown : ChevronRight" />
          </NIcon>
          <NIcon size="14"><Folder /></NIcon>
          <span class="group-title">{{ t('scratchpad.localDrafts') }}</span>
          <span class="group-count">{{ filteredLocalEntries.length }}</span>
        </div>
        <div v-show="groupExpanded.local" class="group-content">
          <div v-if="filteredLocalEntries.length === 0" class="empty-hint">
            {{ contentSearchMode && searchQuery ? '无匹配结果' : t('scratchpad.noDrafts') }}
          </div>
          <ScratchpadTreeNode
            v-for="entry in filteredLocalEntries"
            :key="entry.path"
            :entry="entry"
            :depth="0"
            :expanded-keys="expandedKeys"
            :selected-key="selectedKey"
            :renaming-key="renamingKey"
            @select="handleSelect"
            @open="handleOpen"
            @contextmenu="showEntryMenu"
            @toggle-expand="handleToggleExpand"
            @start-rename="startRename"
            @finish-rename="finishRename"
            @cancel-rename="cancelRename"
          />
        </div>
      </div>

      <div class="tree-group">
        <div class="group-header" @click="toggleTrash">
          <NIcon size="14">
            <component :is="showTrash ? ChevronDown : ChevronRight" />
          </NIcon>
          <NIcon size="14"><Trash2 /></NIcon>
          <span class="group-title">{{ t('scratchpad.trash') }}</span>
          <span class="group-count">{{ trashEntries.length }}</span>
          <NButton
            v-if="showTrash && trashEntries.length > 0"
            size="tiny"
            type="error"
            quaternary
            @click.stop="handleEmptyTrash"
          >
            {{ t('scratchpad.emptyTrash') }}
          </NButton>
        </div>
        <div v-show="showTrash" class="group-content">
          <div v-if="trashEntries.length === 0" class="empty-hint">
            {{ t('scratchpad.noTrashItems') }}
          </div>
          <div
            v-for="item in trashEntries"
            :key="item.path"
            class="ref-row"
          >
            <NIcon size="14"><FileText /></NIcon>
            <span class="ref-name">{{ item.name }}</span>
            <span class="ref-path" :title="item.path">{{ item.path }}</span>
            <NButton
              size="tiny"
              quaternary
              type="primary"
              @click.stop="restoreTrashEntry(item.name)"
            >
              {{ t('scratchpad.restoreFromTrash') }}
            </NButton>
          </div>
        </div>
      </div>
    </div>

    <NModal v-model:show="showCreateModal" :title="t('scratchpad.newDraft')">
      <div class="modal-body">
        <NInput
          ref="createInputRef"
          v-model:value="newFileName"
          :placeholder="t('scratchpad.fileNamePlaceholder')"
          @keyup.enter="confirmCreate"
        />
        <div class="modal-actions">
          <NButton size="small" @click="showCreateModal = false">{{ t('common.cancel') }}</NButton>
          <NButton
            size="small"
            type="primary"
            :disabled="!newFileName.trim()"
            @click="confirmCreate"
            >{{ t('common.confirm') }}</NButton
          >
        </div>
      </div>
    </NModal>

    <NModal v-model:show="showRefModal" :title="t('scratchpad.addReference')">
      <div class="modal-body">
        <NInput
          v-model:value="newRefAlias"
          :placeholder="t('scratchpad.aliasPlaceholder')"
          class="modal-input"
        />
        <NInput
          v-model:value="newRefPath"
          :placeholder="t('scratchpad.pathPlaceholder')"
          class="modal-input"
        />
        <NButton size="small" @click="browseRefPath">
          <template #icon>
            <NIcon><FolderOpen /></NIcon>
          </template>
          {{ t('scratchpad.browse') }}
        </NButton>
        <div class="modal-actions">
          <NButton size="small" @click="showRefModal = false">{{ t('common.cancel') }}</NButton>
          <NButton
            size="small"
            type="primary"
            :disabled="!newRefAlias.trim() || !newRefPath.trim()"
            @click="confirmAddReference"
            >{{ t('common.confirm') }}</NButton
          >
        </div>
      </div>
    </NModal>

    <div
      v-if="contextMenu.visible"
      class="scratchpad-context-menu"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
    >
      <div
        v-for="item in contextMenu.items"
        :key="item.key"
        :class="['menu-item', { 'menu-item-danger': item.danger }]"
        @click="handleMenuAction(item.key)"
      >
        <NIcon v-if="item.icon" size="14"><component :is="item.icon" /></NIcon>
        <span>{{ item.label }}</span>
        <span v-if="item.shortcut" class="menu-shortcut">{{ item.shortcut }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  FilePlus,
  Folder,
  FolderOpen,
  RefreshCw,
  Upload,
  Search,
  ChevronDown,
  ChevronRight,
  FolderSymlink,
  FileText,
  Trash2,
  Pencil,
  Copy,
  ExternalLink,
  X,
  BarChart3,
} from 'lucide-vue-next'
import { NButton, NIcon, NInput, NSpin, NModal } from 'naive-ui'
import { ref, reactive, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'

import ScratchpadTreeNode from './ScratchpadTreeNode.vue'
import { useScratchpad } from '../composables/use-scratchpad'

import type { ScratchpadEntry, ExternalReference } from '../../types'

const { t } = useI18n()

const {
  response,
  isLoading,
  error,
  searchQuery,
  localEntries,
  externalReferences,
  scratchpadPath,
  loadFiles,
  createEntry,
  deleteEntry,
  renameEntry,
  loadFileContent,
  importFile,
  addReference,
  removeReference,
  isRefInvalid,
  openInExplorerAction,
  searchContent,
  trashEntries,
  loadTrashEntries,
  restoreTrashEntry,
  emptyTrashBin,
  analyzableFiles,
  loadAnalyzableFiles,
} = useScratchpad()

const fileMeta = computed(() => response.value?.file_meta ?? {})

const contentSearchMode = ref(false)
const contentResults = ref<Set<string>>(new Set())
const showTrash = ref(false)
let contentSearchTimer: ReturnType<typeof setTimeout> | null = null

function toggleSearchMode(): void {
  contentSearchMode.value = !contentSearchMode.value
  if (contentSearchMode.value) {
    searchQuery.value = ''
    contentResults.value = new Set()
  }
}

watch(searchQuery, async (val) => {
  if (!contentSearchMode.value) return
  if (contentSearchTimer) clearTimeout(contentSearchTimer)
  if (!val.trim()) {
    contentResults.value = new Set()
    return
  }
  contentSearchTimer = setTimeout(async () => {
    const results = await searchContent(val.trim())
    contentResults.value = new Set(results)
  }, 300)
})

const filteredLocalEntries = computed(() => {
  if (!contentSearchMode.value) return localEntries.value
  const results = contentResults.value
  const base = scratchpadPath.value || ''
  return localEntries.value.filter(e => {
    let rel = e.path
    if (base) {
      rel = e.path.replace(base.replace(/\\/g, '/'), '').replace(/^\//, '')
    }
    return results.has(rel)
  })
})

const expandedKeys = ref<Set<string>>(new Set())
const selectedKey = ref<string | null>(null)
const renamingKey = ref<string | null>(null)

const groupExpanded = reactive({ external: true, local: true })

const showCreateModal = ref(false)
const newFileName = ref('')
const createInputRef = ref<InstanceType<typeof NInput> | null>(null)

const showRefModal = ref(false)
const newRefAlias = ref('')
const newRefPath = ref('')

interface ContextMenuItem {
  key: string
  label: string
  icon?: typeof FileText
  danger?: boolean
  shortcut?: string
}

const contextMenu = reactive<{
  visible: boolean
  x: number
  y: number
  target: ScratchpadEntry | ExternalReference | null
  isRefTarget: boolean
  items: ContextMenuItem[]
}>({
  visible: false,
  x: 0,
  y: 0,
  target: null,
  isRefTarget: false,
  items: [],
})

let openDialog: typeof import('@tauri-apps/plugin-dialog').open | null = null
if (typeof window !== 'undefined' && (window as any).__TAURI__) {
  import('@tauri-apps/plugin-dialog')
    .then(m => {
      openDialog = m.open
    })
    .catch(() => {
      openDialog = null
    })
}

onMounted(async () => {
  await loadFiles()
  document.addEventListener('click', closeContextMenu)

  window.addEventListener('project-switched', loadFiles)
})

onUnmounted(() => {
  window.removeEventListener('project-switched', loadFiles)
})

onUnmounted(() => {
  document.removeEventListener('click', closeContextMenu)
})

function handleGlobalClick(): void {
  if (renamingKey.value) {
    cancelRename()
  }
}

function toggleGroup(group: 'external' | 'local'): void {
  groupExpanded[group] = !groupExpanded[group]
}

function toggleTrash(): void {
  showTrash.value = !showTrash.value
  if (showTrash.value) {
    loadTrashEntries()
  }
}

async function handleEmptyTrash(): Promise<void> {
  const confirmed = window.confirm(t('scratchpad.emptyTrashConfirm'))
  if (!confirmed) return
  await emptyTrashBin()
}

function handleSelect(entry: ScratchpadEntry): void {
  selectedKey.value = entry.path
}

function handleToggleExpand(entry: ScratchpadEntry): void {
  const next = new Set(expandedKeys.value)
  if (next.has(entry.path)) {
    next.delete(entry.path)
  } else {
    next.add(entry.path)
  }
  expandedKeys.value = next
}

function handleOpen(entry: ScratchpadEntry): void {
  closeContextMenu()
  if (entry.kind === 'folder') {
    handleToggleExpand(entry)
    return
  }
  openFileInEditor(entry)
}

async function openFileInEditor(entry: ScratchpadEntry): Promise<void> {
  const ext = entry.extension.toLowerCase()
  const langMap: Record<string, string> = {
    '.sql': 'sql',
    '.py': 'python',
    '.json': 'json',
    '.txt': 'plaintext',
    '.md': 'markdown',
  }
  const language = langMap[ext] || 'plaintext'

  const scratchpadBase = scratchpadPath.value || ''
  const relativePath = scratchpadBase
    ? entry.path.replace(scratchpadBase.replace(/\\/g, '/'), '').replace(/^\//, '')
    : entry.path

  const content = await loadFileContent(relativePath)
  if (content === null) {
    return
  }

  const metaForFile = fileMeta.value[relativePath]
  const lastConnectionId = metaForFile?.last_connection_id || ''

  window.dispatchEvent(
    new CustomEvent('open-sql-editor', {
      detail: {
        connectionId: lastConnectionId,
        databaseName: '',
        sql: content,
        scratchpadRelativePath: relativePath,
        scratchpadFileName: entry.name,
        language,
      },
    })
  )
}

async function handleCreateFile(): Promise<void> {
  newFileName.value = ''
  showCreateModal.value = true
  await nextTick()
  createInputRef.value?.focus()
}

async function confirmCreate(): Promise<void> {
  const name = newFileName.value.trim()
  if (!name) return
  showCreateModal.value = false
  await createEntry(name, false)
}

async function handleImportFile(): Promise<void> {
  if (!openDialog) {
    console.warn('[Scratchpad] Dialog plugin not available')
    return
  }
  try {
    const selected = await openDialog({
      multiple: false,
      title: t('scratchpad.selectFileToImport'),
      filters: [{ name: t('scratchpad.allFiles'), extensions: ['*'] }],
    })
    if (selected && typeof selected === 'string') {
      await importFile(selected)
    }
  } catch (e) {
    console.error('[Scratchpad] Import dialog error:', e)
  }
}

async function handleAddReference(): Promise<void> {
  newRefAlias.value = ''
  newRefPath.value = ''
  showRefModal.value = true
}

async function browseRefPath(): Promise<void> {
  if (!openDialog) return
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: t('scratchpad.selectRefDirectory'),
    })
    if (selected && typeof selected === 'string') {
      newRefPath.value = selected
    }
  } catch (e) {
    console.error('[Scratchpad] Browse error:', e)
  }
}

async function confirmAddReference(): Promise<void> {
  const alias = newRefAlias.value.trim()
  const path = newRefPath.value.trim()
  if (!alias || !path) return
  showRefModal.value = false
  await addReference(alias, path)
}

function handleRefClick(ref: ExternalReference): void {
  selectedKey.value = ref.alias
}

function closeContextMenu(): void {
  contextMenu.visible = false
  contextMenu.target = null
}

const ANALYZABLE_EXTENSIONS = ['.csv', '.parquet', '.json', '.xlsx']

function isAnalyzableFile(entry: ScratchpadEntry): boolean {
  return ANALYZABLE_EXTENSIONS.includes(entry.extension.toLowerCase())
}

async function handleAnalyzeDuckDB(entry: ScratchpadEntry): Promise<void> {
  await loadAnalyzableFiles()
  const match = analyzableFiles.value.find(f => f.relative_path === entry.path)
  if (!match) return

  const scratchpadBase = scratchpadPath.value || ''
  const content = match.duckdb_query_hint

  window.dispatchEvent(
    new CustomEvent('open-sql-editor', {
      detail: {
        connectionId: '',
        databaseName: '',
        sql: content,
        scratchpadRelativePath: '',
        scratchpadFileName: `${entry.name} (DuckDB Preview)`,
        language: 'sql',
      },
    })
  )
}

function showEntryMenu(event: MouseEvent, entry: ScratchpadEntry): void {
  event.preventDefault()
  event.stopPropagation()
  selectedKey.value = entry.path
  const pos = clampToViewport(event.clientX, event.clientY, 180, 240)
  contextMenu.x = pos.x
  contextMenu.y = pos.y
  contextMenu.isRefTarget = false
  contextMenu.target = entry
  contextMenu.items = [
    { key: 'open', label: t('scratchpad.open'), icon: FileText },
    ...(isAnalyzableFile(entry)
      ? [
          {
            key: 'analyze-duckdb',
            label: t('scratchpad.analyzeWithDuckDB'),
            icon: BarChart3,
          },
        ]
      : []),
    ...(entry.kind === 'folder'
      ? [
          {
            key: 'toggle-folder',
            label: expandedKeys.value.has(entry.path)
              ? t('scratchpad.collapse')
              : t('scratchpad.expand'),
            icon: ChevronRight,
          },
        ]
      : []),
    { key: 'rename', label: t('scratchpad.rename'), icon: Pencil, shortcut: 'F2' },
    { key: 'copy-path', label: t('scratchpad.copyPath'), icon: Copy },
    { key: 'delete', label: t('scratchpad.delete'), icon: Trash2, danger: true, shortcut: 'Del' },
  ]
  contextMenu.visible = true
}

function showRefMenu(event: MouseEvent, ref: ExternalReference): void {
  event.preventDefault()
  event.stopPropagation()
  const pos = clampToViewport(event.clientX, event.clientY, 180, 100)
  contextMenu.x = pos.x
  contextMenu.y = pos.y
  contextMenu.isRefTarget = true
  contextMenu.target = ref
  contextMenu.items = [
    { key: 'open-ref-location', label: t('scratchpad.openLocation'), icon: ExternalLink },
    { key: 'remove-ref', label: t('scratchpad.removeReference'), icon: X, danger: true },
  ]
  contextMenu.visible = true
}

function clampToViewport(
  x: number,
  y: number,
  menuWidth: number,
  menuHeight: number
): { x: number; y: number } {
  const w = window.innerWidth
  const h = window.innerHeight
  return {
    x: Math.min(x, w - menuWidth),
    y: Math.min(y, h - menuHeight),
  }
}

async function handleMenuAction(key: string): Promise<void> {
  closeContextMenu()
  if (contextMenu.isRefTarget) {
    const ref = contextMenu.target as ExternalReference
    if (key === 'remove-ref') {
      await removeReference(ref.alias)
    } else if (key === 'open-ref-location') {
      await openInExplorerAction(ref.path)
    }
    return
  }

  const entry = contextMenu.target as ScratchpadEntry
  switch (key) {
    case 'open':
      openFileInEditor(entry)
      break
    case 'analyze-duckdb':
      handleAnalyzeDuckDB(entry)
      break
    case 'toggle-folder':
      handleToggleExpand(entry)
      break
    case 'rename':
      startRename(entry)
      break
    case 'copy-path':
      await navigator.clipboard.writeText(entry.path)
      break
    case 'delete':
      await deleteEntry(entry.path)
      break
  }
}

function startRename(entry: ScratchpadEntry): void {
  renamingKey.value = entry.path
}

async function finishRename(entry: ScratchpadEntry, newName: string): Promise<void> {
  renamingKey.value = null
  if (newName && newName !== entry.name) {
    await renameEntry(entry.path, newName)
  }
}

function cancelRename(): void {
  renamingKey.value = null
}

function handleKeydown(event: KeyboardEvent): void {
  if ((event.ctrlKey || event.metaKey) && event.key === 'n') {
    event.preventDefault()
    handleCreateFile()
    return
  }
  if (!selectedKey.value) return
  if (event.key === 'F2') {
    event.preventDefault()
    const entry = localEntries.value.find(e => e.path === selectedKey.value)
    if (entry) startRename(entry)
  } else if (event.key === 'Delete') {
    event.preventDefault()
    const entry = localEntries.value.find(e => e.path === selectedKey.value)
    if (entry) deleteEntry(entry.path)
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.scratchpad-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px;
  border-bottom: 1px solid var(--color-border, #4a5458);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
}

.tree-container {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.loading-state,
.error-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  gap: 8px;
}

.error-text {
  color: var(--brand-danger, #d63031);
  font-size: 13px;
}

.tree-group {
  margin-bottom: 2px;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 28px;
  padding: 0 8px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary, #9ca3af);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.group-header:hover {
  background-color: var(--color-bg-tertiary, #2d3436);
}

.group-title {
  flex: 1;
}

.group-count {
  font-size: 11px;
  color: var(--color-text-muted, #6b7280);
}

.ref-row {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 28px;
  padding: 0 8px 0 24px;
  font-size: 13px;
  cursor: pointer;
}

.ref-row:hover {
  background-color: var(--color-bg-tertiary, #2d3436);
}

.ref-name {
  color: var(--color-text-primary, #e5e7eb);
  font-weight: 500;
}

.ref-path {
  font-size: 11px;
  color: var(--color-text-muted, #6b7280);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.ref-path-invalid {
  color: var(--brand-danger, #d63031);
  text-decoration: line-through;
}

.ref-badge {
  font-size: 10px;
  color: var(--brand-danger, #d63031);
  background: rgba(214, 48, 49, 0.1);
  padding: 1px 4px;
  border-radius: 2px;
  flex-shrink: 0;
}

.ref-invalid {
  color: var(--brand-danger, #d63031);
}

.empty-hint {
  padding: 12px 24px;
  font-size: 12px;
  color: var(--color-text-muted, #6b7280);
  text-align: center;
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  min-width: 360px;
}

.modal-input {
  width: 100%;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.scratchpad-context-menu {
  position: fixed;
  z-index: 1000;
  background: var(--color-bg-elevated, #3d4446);
  border: 1px solid var(--color-border, #4a5458);
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  min-width: 180px;
  padding: 4px 0;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 30px;
  padding: 0 12px;
  font-size: 13px;
  cursor: pointer;
  transition: background-color 0.1s;
}

.menu-item:hover {
  background-color: var(--color-bg-tertiary, #2d3436);
}

.menu-item-danger {
  color: var(--brand-danger, #d63031);
}

.menu-item-danger:hover {
  background-color: rgba(245, 63, 63, 0.08);
}

.menu-shortcut {
  margin-left: auto;
  font-size: 11px;
  color: var(--color-text-muted, #6b7280);
}

.menu-item-danger .menu-shortcut {
  color: rgba(245, 63, 63, 0.6);
}
</style>
