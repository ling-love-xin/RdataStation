<template>
  <div
    class="scratchpad-panel"
    @click="handleGlobalClick"
  >
    <div class="toolbar">
      <NButton size="small" :disabled="isLoading" @click="handleCreateFile">
        <template #icon>
          <NIcon><FilePlus /></NIcon>
        </template>
        新建
      </NButton>
      <NButton size="small" :disabled="isLoading" @click="handleImportFile">
        <template #icon>
          <NIcon><Upload /></NIcon>
        </template>
        导入
      </NButton>
      <NButton size="small" :disabled="isLoading" @click="handleAddReference">
        <template #icon>
          <NIcon><FolderSymlink /></NIcon>
        </template>
        引用
      </NButton>
      <NButton size="small" :disabled="isLoading" @click="loadFiles">
        <template #icon>
          <NIcon><RefreshCw /></NIcon>
        </template>
      </NButton>
      <NInput
        v-model:value="searchQuery"
        size="small"
        placeholder="搜索..."
        clearable
        class="search-input"
      />
    </div>

    <div v-if="isLoading" class="loading-state">
      <NSpin size="small" />
    </div>

    <div v-else-if="error" class="error-state">
      <span class="error-text">{{ error }}</span>
      <NButton size="tiny" @click="loadFiles">重试</NButton>
    </div>

    <div v-else class="tree-container">
      <div v-if="externalReferences.length > 0" class="tree-group">
        <div class="group-header" @click="toggleGroup('external')">
          <NIcon size="14">
            <component :is="groupExpanded.external ? ChevronDown : ChevronRight" />
          </NIcon>
          <NIcon size="14"><FolderSymlink /></NIcon>
          <span class="group-title">外部引用</span>
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
            >{{ ref.path }}</span>
            <span v-if="isRefInvalid(ref)" class="ref-badge">已失效</span>
          </div>
        </div>
      </div>

      <div class="tree-group">
        <div class="group-header" @click="toggleGroup('local')">
          <NIcon size="14">
            <component :is="groupExpanded.local ? ChevronDown : ChevronRight" />
          </NIcon>
          <NIcon size="14"><Folder /></NIcon>
          <span class="group-title">本地草稿</span>
          <span class="group-count">{{ localEntries.length }}</span>
        </div>
        <div v-show="groupExpanded.local" class="group-content">
          <div v-if="localEntries.length === 0" class="empty-hint">
            尚无草稿文件，点击 [新建] 或 [导入]
          </div>
          <ScratchpadTreeNode
            v-for="entry in localEntries"
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
    </div>

    <NModal v-model:show="showCreateModal" title="新建草稿">
      <div class="modal-body">
        <NInput
          ref="createInputRef"
          v-model:value="newFileName"
          placeholder="输入完整文件名，如 临时查询.sql"
          @keyup.enter="confirmCreate"
        />
        <div class="modal-actions">
          <NButton size="small" @click="showCreateModal = false">取消</NButton>
          <NButton size="small" type="primary" :disabled="!newFileName.trim()" @click="confirmCreate">确定</NButton>
        </div>
      </div>
    </NModal>

    <NModal v-model:show="showRefModal" title="添加外部引用">
      <div class="modal-body">
        <NInput
          v-model:value="newRefAlias"
          placeholder="引用别名，如 下载数据"
          class="modal-input"
        />
        <NInput
          v-model:value="newRefPath"
          placeholder="目录路径，如 /home/user/Downloads"
          class="modal-input"
        />
        <NButton size="small" @click="browseRefPath">
          <template #icon>
            <NIcon><FolderOpen /></NIcon>
          </template>
          浏览...
        </NButton>
        <div class="modal-actions">
          <NButton size="small" @click="showRefModal = false">取消</NButton>
          <NButton
            size="small"
            type="primary"
            :disabled="!newRefAlias.trim() || !newRefPath.trim()"
            @click="confirmAddReference"
          >确定</NButton>
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
  FilePlus, Folder, FolderOpen, RefreshCw, Upload,
  ChevronDown, ChevronRight, FolderSymlink,
  FileText, Trash2, Pencil, Copy, ExternalLink, X
} from 'lucide-vue-next'
import { NButton, NIcon, NInput, NSpin, NModal } from 'naive-ui'
import { ref, reactive, onMounted, onUnmounted, nextTick } from 'vue'

import ScratchpadTreeNode from './ScratchpadTreeNode.vue'
import { useScratchpad } from '../composables/use-scratchpad'

import type { ScratchpadEntry, ExternalReference } from '../../types'

const {
  isLoading,
  error,
  searchQuery,
  localEntries,
  externalReferences,
  loadFiles,
  createEntry,
  deleteEntry,
  renameEntry,
  importFile,
  addReference,
  removeReference,
  isRefInvalid,
  openInExplorerAction,
  getFileSize,
} = useScratchpad()

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
  import('@tauri-apps/plugin-dialog').then((m) => { openDialog = m.open }).catch(() => { openDialog = null })
}

onMounted(() => {
  loadFiles()
  document.addEventListener('click', closeContextMenu)
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
  const MAX_MB = 50

  try {
    const size = await getFileSize(entry.path)
    if (size !== null && size > MAX_MB * 1024 * 1024) {
      console.warn(`[Scratchpad] File too large: ${entry.name} (${(size / (1024 * 1024)).toFixed(1)} MB > ${MAX_MB} MB limit)`)
      return
    }
  } catch {
    // 文件可能已被删除，忽略
  }

  const editorMap: Record<string, string> = {
    '.sql': 'sql-editor',
    '.py': 'code-editor',
    '.csv': 'data-preview',
    '.json': 'code-editor',
    '.txt': 'code-editor',
    '.md': 'code-editor',
  }
  const editorType = editorMap[entry.extension.toLowerCase()] || 'code-editor'
  console.log(`[Scratchpad] Opening ${entry.name} in ${editorType}`, entry.path)
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
      title: '选择要导入的文件',
      filters: [{ name: '所有文件', extensions: ['*'] }],
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
      title: '选择要引用的目录',
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

function showEntryMenu(event: MouseEvent, entry: ScratchpadEntry): void {
  event.preventDefault()
  event.stopPropagation()
  selectedKey.value = entry.path
  const pos = clampToViewport(event.clientX, event.clientY, 180, 210)
  contextMenu.x = pos.x
  contextMenu.y = pos.y
  contextMenu.isRefTarget = false
  contextMenu.target = entry
  contextMenu.items = [
    { key: 'open', label: '打开', icon: FileText },
    ...(entry.kind === 'folder'
      ? [{ key: 'toggle-folder', label: expandedKeys.value.has(entry.path) ? '折叠' : '展开', icon: ChevronRight }]
      : []),
    { key: 'rename', label: '重命名', icon: Pencil, shortcut: 'F2' },
    { key: 'copy-path', label: '复制路径', icon: Copy },
    { key: 'delete', label: '删除', icon: Trash2, danger: true, shortcut: 'Del' },
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
    { key: 'open-ref-location', label: '打开位置', icon: ExternalLink },
    { key: 'remove-ref', label: '移除引用', icon: X, danger: true },
  ]
  contextMenu.visible = true
}

function clampToViewport(
  x: number, y: number,
  menuWidth: number, menuHeight: number,
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
    const entry = localEntries.value.find((e) => e.path === selectedKey.value)
    if (entry) startRename(entry)
  } else if (event.key === 'Delete') {
    event.preventDefault()
    const entry = localEntries.value.find((e) => e.path === selectedKey.value)
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
  border-bottom: 1px solid var(--border-color, #d9d9d9);
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
  color: var(--danger-color, #F53F3F);
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
  color: var(--text-secondary, #666666);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.group-header:hover {
  background-color: var(--bg-tertiary, #e8e8e8);
}

.group-title {
  flex: 1;
}

.group-count {
  font-size: 11px;
  color: var(--text-tertiary, #999999);
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
  background-color: var(--bg-tertiary, #e8e8e8);
}

.ref-name {
  color: var(--text-primary, #333333);
  font-weight: 500;
}

.ref-path {
  font-size: 11px;
  color: var(--text-tertiary, #999999);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.ref-path-invalid {
  color: var(--danger-color, #F53F3F);
  text-decoration: line-through;
}

.ref-badge {
  font-size: 10px;
  color: var(--danger-color, #F53F3F);
  background: rgba(245, 63, 63, 0.1);
  padding: 1px 4px;
  border-radius: 2px;
  flex-shrink: 0;
}

.ref-invalid {
  color: var(--danger-color, #F53F3F);
}

.empty-hint {
  padding: 12px 24px;
  font-size: 12px;
  color: var(--text-tertiary, #999999);
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
  background: var(--bg-primary, #ffffff);
  border: 1px solid var(--border-color, #d9d9d9);
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
  background-color: var(--bg-tertiary, #e8e8e8);
}

.menu-item-danger {
  color: var(--danger-color, #F53F3F);
}

.menu-item-danger:hover {
  background-color: rgba(245, 63, 63, 0.08);
}

.menu-shortcut {
  margin-left: auto;
  font-size: 11px;
  color: var(--text-tertiary, #999999);
}

.menu-item-danger .menu-shortcut {
  color: rgba(245, 63, 63, 0.6);
}
</style>
