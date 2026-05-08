<template>
  <div
    class="scratchpad-panel"
    :class="{ 'dragover-active': isDragOver }"
    :data-drop-hint="t('scratchpad.dragToImport')"
    @click="handleGlobalClick"
    @dragover.prevent="handleDragOver"
    @dragleave="handleDragLeave"
    @drop.prevent="handleDrop"
  >
    <div class="toolbar">
      <div class="toolbar-group">
        <NButton
          size="small"
          type="primary"
          :disabled="isLoading"
          @click="handleCreateFile"
        >
          <template #icon>
            <NIcon size="16"><FilePlus /></NIcon>
          </template>
          {{ t('scratchpad.newFile') }}
        </NButton>
        <NButton
          size="small"
          :disabled="isLoading"
          @click="handleImportFile"
        >
          <template #icon>
            <NIcon size="16"><Upload /></NIcon>
          </template>
          {{ t('scratchpad.import') }}
        </NButton>
        <NButton
          size="small"
          :disabled="isLoading"
          @click="handleAddReference"
        >
          <template #icon>
            <NIcon size="16"><FolderSymlink /></NIcon>
          </template>
          {{ t('scratchpad.reference') }}
        </NButton>
      </div>
      <div class="toolbar-group-right">
        <NButton size="small" quaternary @click="toggleSort('name')">
          <template #icon>
            <NIcon size="14"><component :is="sortIcon('name')" /></NIcon>
          </template>
          {{ t('scratchpad.sortByName') }}
        </NButton>
        <NButton size="small" quaternary @click="toggleSort('size')">
          <template #icon>
            <NIcon size="14"><component :is="sortIcon('size')" /></NIcon>
          </template>
          {{ t('scratchpad.sortBySize') }}
        </NButton>
        <NButton size="small" quaternary @click="toggleSort('modified')">
          <template #icon>
            <NIcon size="14"><component :is="sortIcon('modified')" /></NIcon>
          </template>
          {{ t('scratchpad.sortByModified') }}
        </NButton>
        <span class="toolbar-sep"></span>
        <NButton
          size="small"
          quaternary
          :disabled="isLoading"
          @click="loadFiles"
        >
          <template #icon>
            <NIcon size="16"><RefreshCw /></NIcon>
          </template>
        </NButton>
      </div>
    </div>

    <div class="search-bar">
      <NInput
        v-model:value="searchQuery"
        size="small"
        :placeholder="contentSearchMode ? t('scratchpad.searchContentHint') : t('scratchpad.searchNameHint')"
        clearable
        class="search-input"
      />
      <NButton
        size="small"
        :type="contentSearchMode ? 'primary' : 'default'"
        quaternary
        @click="toggleSearchMode"
      >
        <template #icon>
          <NIcon size="16"><Search /></NIcon>
        </template>
      </NButton>
    </div>

    <div v-if="contentSearchMode && contentAllMatches.length > 0" class="search-results">
      <div class="search-results-header">
        {{ t('scratchpad.searchFileResults') }}
        <span class="search-results-count">{{ t('scratchpad.matchesCount', { count: contentAllMatches.length }) }}</span>
      </div>
      <div
        v-for="[file, matches] in contentResults"
        :key="file"
        class="search-result-file"
      >
        <div class="search-result-file-name">
          <NIcon size="14"><FileText /></NIcon>
          <span>{{ file }}</span>
          <span class="search-result-match-count">{{ matches.length }}</span>
        </div>
        <div
          v-for="match in matches.slice(0, 5)"
          :key="`${match.file}:${match.line_number}`"
          class="search-result-line"
        >
          <span class="search-result-line-number">{{ match.line_number }}</span>
          <span class="search-result-line-content">{{ match.line_content }}</span>
        </div>
        <div v-if="matches.length > 5" class="search-result-more">
          ...还有 {{ matches.length - 5 }} 处匹配未显示
        </div>
      </div>
    </div>

    <div v-if="isLoading" class="loading-state">
      <NSpin size="small" />
    </div>

    <div v-else-if="error" class="error-state">
      <span class="error-text">{{ error }}</span>
      <NButton size="tiny" @click="loadFiles">{{ t('navigator.retry') }}</NButton>
    </div>

    <div v-else-if="!contentSearchMode || contentAllMatches.length === 0" class="tree-container">
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
            {{ contentSearchMode && searchQuery ? t('scratchpad.noMatchResults') : t('scratchpad.noDrafts') }}
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

    <NModal
      v-model:show="showPromoteConfirm"
      :title="t('scratchpad.promoteToResource')"
      preset="dialog"
      positive-text="提升并保留草稿"
      negative-text="提升并删除草稿"
      type="info"
      @positive-click="handlePromoteConfirm(false)"
      @negative-click="handlePromoteConfirm(true)"
    >
      <template v-if="promoteTarget">
        {{ t('scratchpad.promoteToResourceConfirm', { name: promoteTarget.name }) }}
      </template>
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
import { listen } from '@tauri-apps/api/event'
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
  GitBranch,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
} from 'lucide-vue-next'
import { NButton, NIcon, NInput, NSpin, NModal } from 'naive-ui'
import { ref, reactive, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'

import ScratchpadTreeNode from './ScratchpadTreeNode.vue'
import { useScratchpad } from '../composables/use-scratchpad'

import type { ScratchpadEntry, ExternalReference, SearchMatch } from '../../types'

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
  startWatching,
  stopWatching,
  promoteToResource,
} = useScratchpad()

const fileMeta = computed(() => response.value?.file_meta ?? {})

const contentSearchMode = ref(false)
const contentResults = ref<Map<string, SearchMatch[]>>(new Map())
const contentAllMatches = ref<SearchMatch[]>([])
const showTrash = ref(false)
const promoteTarget = ref<ScratchpadEntry | null>(null)
const showPromoteConfirm = ref(false)
const sortBy = ref<'name' | 'size' | 'modified'>('name')
const sortOrder = ref<'asc' | 'desc'>('asc')
let contentSearchTimer: ReturnType<typeof setTimeout> | null = null

function toggleSearchMode(): void {
  contentSearchMode.value = !contentSearchMode.value
  if (contentSearchMode.value) {
    searchQuery.value = ''
    contentResults.value = new Map()
    contentAllMatches.value = []
  }
}

function toggleSort(field: 'name' | 'size' | 'modified'): void {
  if (sortBy.value === field) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortBy.value = field
    sortOrder.value = 'asc'
  }
}

function sortIcon(field: 'name' | 'size' | 'modified'): typeof ArrowUpDown {
  if (sortBy.value !== field) return ArrowUpDown
  return sortOrder.value === 'asc' ? ArrowUp : ArrowDown
}

watch(searchQuery, async (val) => {
  if (!contentSearchMode.value) return
  if (contentSearchTimer) clearTimeout(contentSearchTimer)
  if (!val.trim()) {
    contentResults.value = new Map()
    contentAllMatches.value = []
    return
  }
  contentSearchTimer = setTimeout(async () => {
    const matches = await searchContent(val.trim())
    contentAllMatches.value = matches
    const grouped = new Map<string, SearchMatch[]>()
    for (const m of matches) {
      const list = grouped.get(m.file) || []
      list.push(m)
      grouped.set(m.file, list)
    }
    contentResults.value = grouped
  }, 300)
})

const filteredLocalEntries = computed(() => {
  let entries = localEntries.value
  if (contentSearchMode.value) {
    const results = contentResults.value
    const base = scratchpadPath.value || ''
    entries = entries.filter(e => {
      let rel = e.path
      if (base) {
        rel = e.path.replace(base.replace(/\\/g, '/'), '').replace(/^\//, '')
      }
      return results.has(rel)
    })
  }
  return [...entries].sort((a, b) => {
    let cmp = 0
    if (sortBy.value === 'name') {
      cmp = a.name.localeCompare(b.name)
    } else if (sortBy.value === 'size') {
      cmp = a.size - b.size
    } else if (sortBy.value === 'modified') {
      const da = a.modified_at || ''
      const db = b.modified_at || ''
      cmp = da.localeCompare(db)
    }
    return sortOrder.value === 'asc' ? cmp : -cmp
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

const isDragOver = ref(false)
let dragLeaveTimer: ReturnType<typeof setTimeout> | null = null

function handleDragOver(event: DragEvent): void {
  if (dragLeaveTimer) {
    clearTimeout(dragLeaveTimer)
    dragLeaveTimer = null
  }
  isDragOver.value = true
}

function handleDragLeave(event: DragEvent): void {
  dragLeaveTimer = setTimeout(() => {
    isDragOver.value = false
  }, 100)
}

async function handleDrop(event: DragEvent): Promise<void> {
  isDragOver.value = false
  const files = event.dataTransfer?.files
  if (!files || files.length === 0) return

  for (let i = 0; i < files.length; i++) {
    const filePath = (files[i] as unknown as { path?: string }).path
    if (filePath) {
      await importFile(filePath)
    }
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
    { key: 'promote', label: t('scratchpad.promoteToResource'), icon: GitBranch },
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
    case 'promote':
      promoteTarget.value = entry
      showPromoteConfirm.value = true
      break
  }
}

async function handlePromoteConfirm(removeAfter: boolean): Promise<void> {
  showPromoteConfirm.value = false
  const entry = promoteTarget.value
  if (!entry) return
  const path = entry.path
  await promoteToResource(path, removeAfter)
  promoteTarget.value = null
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
  if (!selectedKey.value) {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault()
      const entries = filteredLocalEntries.value
      if (entries.length > 0) {
        selectedKey.value = event.key === 'ArrowDown' ? entries[0].path : entries[entries.length - 1].path
        scrollToSelected()
      }
    }
    return
  }

  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    const entries = filteredLocalEntries.value
    const idx = entries.findIndex(e => e.path === selectedKey.value)
    if (idx === -1) return
    const nextIdx = event.key === 'ArrowDown'
      ? Math.min(idx + 1, entries.length - 1)
      : Math.max(idx - 1, 0)
    selectedKey.value = entries[nextIdx].path
    scrollToSelected()
    return
  }

  if (event.key === 'Enter') {
    event.preventDefault()
    const entry = localEntries.value.find(e => e.path === selectedKey.value)
    if (entry) handleOpen(entry)
    return
  }

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

function scrollToSelected(): void {
  nextTick(() => {
    const el = document.querySelector('.node-row.selected')
    el?.scrollIntoView({ block: 'nearest' })
  })
}

let unlisten: (() => void) | null = null

onMounted(async () => {
  document.addEventListener('keydown', handleKeydown)
  await startWatching()
  try {
    const unlistenFn = await listen('scratchpad-changed', () => {
      loadFiles()
    })
    unlisten = unlistenFn
  } catch {
    // event listener setup failed, watcher still works for in-app changes
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  if (unlisten) {
    unlisten()
    unlisten = null
  }
  stopWatching()
})
</script>

<style scoped>
.scratchpad-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg-primary);
  overflow: hidden;
  position: relative;
}

.scratchpad-panel.dragover-active::after {
  content: attr(data-drop-hint);
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--brand-accent-soft);
  border: 2px dashed var(--brand-accent);
  color: var(--brand-accent);
  font-size: 14px;
  font-weight: 600;
  z-index: 100;
  pointer-events: none;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  padding: 8px 8px 4px;
  flex-shrink: 0;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.toolbar-group-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.toolbar-sep {
  width: 1px;
  height: 18px;
  background: var(--color-border-subtle);
  margin: 0 4px;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px 8px;
  border-bottom: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
}

.tree-container {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
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
  color: var(--brand-danger);
  font-size: 13px;
}

.tree-group {
  margin-bottom: 2px;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 8px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  transition: background-color 0.1s;
}

.group-header:hover {
  background-color: var(--color-bg-tertiary);
}

.group-title {
  flex: 1;
}

.group-count {
  font-size: 11px;
  color: var(--color-text-muted);
}

.ref-row {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 8px 0 24px;
  font-size: 13px;
  cursor: pointer;
  transition: background-color 0.1s;
}

.ref-row:hover {
  background-color: var(--color-bg-tertiary);
}

.ref-name {
  color: var(--color-text-primary);
  font-weight: 500;
}

.ref-path {
  font-size: 11px;
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.ref-path-invalid {
  color: var(--brand-danger);
  text-decoration: line-through;
}

.ref-badge {
  font-size: 10px;
  color: var(--brand-danger);
  background: var(--brand-accent-soft);
  padding: 1px 6px;
  border-radius: var(--border-radius-sm, 4px);
  flex-shrink: 0;
}

.ref-invalid {
  color: var(--brand-danger);
}

.empty-hint {
  padding: 12px 24px;
  font-size: 12px;
  color: var(--color-text-muted);
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
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-md, 6px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  min-width: 180px;
  padding: 4px 0;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 32px;
  padding: 0 12px;
  font-size: 13px;
  cursor: pointer;
  transition: background-color 0.1s;
  color: var(--color-text-primary);
}

.menu-item:hover {
  background-color: var(--color-bg-tertiary);
}

.menu-item-danger {
  color: var(--brand-danger);
}

.menu-item-danger:hover {
  background-color: var(--brand-accent-soft);
}

.menu-shortcut {
  margin-left: auto;
  font-size: 11px;
  color: var(--color-text-muted);
}

.menu-item-danger .menu-shortcut {
  opacity: 0.6;
}

.search-results {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.search-results-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 12px 8px;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  border-bottom: 1px solid var(--color-border-subtle);
}

.search-results-count {
  font-size: 11px;
  font-weight: 400;
  color: var(--color-text-muted);
}

.search-result-file {
  margin: 2px 0;
}

.search-result-file-name {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 12px;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: background-color 0.1s;
}

.search-result-file-name:hover {
  background-color: var(--color-bg-tertiary);
}

.search-result-match-count {
  font-size: 11px;
  color: var(--color-text-muted);
  background: var(--color-bg-tertiary);
  padding: 1px 6px;
  border-radius: var(--border-radius-sm, 4px);
}

.search-result-line {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 24px;
  padding: 0 12px 0 32px;
  font-size: 12px;
  font-family: var(--font-mono, 'Consolas', 'Courier New', monospace);
}

.search-result-line-number {
  color: var(--color-text-muted);
  min-width: 32px;
  text-align: right;
  flex-shrink: 0;
}

.search-result-line-content {
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.search-result-more {
  padding: 2px 12px 4px 32px;
  font-size: 11px;
  color: var(--color-text-muted);
  font-style: italic;
}
</style>
