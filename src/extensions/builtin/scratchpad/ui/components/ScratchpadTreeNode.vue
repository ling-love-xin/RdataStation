<template>
  <div class="scratchpad-tree-node">
    <div
      :class="['node-row', { selected: isSelected }]"
      :style="{ paddingLeft: `${depth * 16 + 8}px` }"
      :draggable="entry.kind === 'file' && !isRenaming"
      @click="handleClick"
      @dblclick="handleDoubleClick"
      @contextmenu.prevent="handleContextMenu"
      @dragstart="handleDragStart"
    >
      <span v-if="entry.kind === 'folder'" class="folder-toggle" @click.stop="handleToggleExpand">
        <NIcon size="14">
          <component :is="expanded ? ChevronDown : ChevronRight" />
        </NIcon>
      </span>
      <span v-else class="folder-toggle-spacer" />

      <NIcon size="14" class="node-icon">
        <component :is="entry.kind === 'folder' ? (expanded ? FolderOpen : Folder) : fileIcon" />
      </NIcon>

      <span v-if="renamingKey !== entry.path" class="node-name">{{ entry.name }}</span>
      <div v-else class="rename-wrapper">
        <input
          ref="renameInputRef"
          v-model="renameValue"
          class="rename-input"
          :disabled="renamingSaving"
          @keyup.enter="commitRename"
          @keyup.escape="cancelRename"
          @blur="commitRename"
          @click.stop
        />
        <span v-if="renamingSaving" class="rename-spinner" />
      </div>

      <span v-if="entry.kind === 'file' && entry.size > 0" class="node-size">{{
        formatSize(entry.size)
      }}</span>
      <span v-if="modifiedTime" class="node-time">{{ modifiedTime }}</span>
      <span v-if="entry.kind === 'folder'" class="node-arrow" />
    </div>

    <div v-if="entry.kind === 'folder' && expanded" class="node-children">
      <ScratchpadTreeNode
        v-for="child in childEntries"
        :key="child.path"
        :entry="child"
        :depth="depth + 1"
        :expanded-keys="expandedKeys"
        :selected-key="selectedKey"
        :selected-keys="selectedKeys"
        :renaming-key="renamingKey"
        @select="forwardSelect"
        @open="forwardOpen"
        @contextmenu="forwardContextmenu"
        @toggle-expand="forwardToggleExpand"
        @start-rename="forwardStartRename"
        @finish-rename="forwardFinishRename"
        @cancel-rename="forwardCancelRename"
        @drag-start="forwardDragStart"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  File,
  Folder,
  FolderOpen,
  Database,
  FileCode,
  Table2,
  FileText,
  Braces,
  BookOpen,
  ChevronDown,
  ChevronRight,
} from 'lucide-vue-next'
import { NIcon } from 'naive-ui'
import { computed, ref, watch, nextTick } from 'vue'

import type { ScratchpadEntry } from '../../types'

interface Props {
  entry: ScratchpadEntry
  depth: number
  expandedKeys: Set<string>
  selectedKey: string | null
  selectedKeys?: Set<string>
  renamingKey: string | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  select: [entry: ScratchpadEntry, event?: MouseEvent]
  open: [entry: ScratchpadEntry]
  contextmenu: [event: MouseEvent, entry: ScratchpadEntry]
  'toggle-expand': [entry: ScratchpadEntry]
  'start-rename': [entry: ScratchpadEntry]
  'finish-rename': [entry: ScratchpadEntry, newName: string]
  'cancel-rename': []
  'drag-start': [event: DragEvent, entry: ScratchpadEntry]
}>()

const expanded = computed(() => props.expandedKeys.has(props.entry.path))
const isSelected = computed(() => {
  if (props.selectedKeys) return props.selectedKeys.has(props.entry.path)
  return props.selectedKey === props.entry.path
})
const isRenaming = computed(() => props.renamingKey === props.entry.path)

const renameValue = ref('')
const renameInputRef = ref<HTMLInputElement | null>(null)
const renamingSaving = ref(false)

const childEntries = computed(() => props.entry.children || [])

const extensionIconMap: Record<string, typeof File> = {
  '.sql': Database,
  '.py': FileCode,
  '.csv': Table2,
  '.json': Braces,
  '.txt': FileText,
  '.md': BookOpen,
  '.duckdb': Database,
  '.parquet': Table2,
}

const fileIcon = computed(() => {
  const name = props.entry.name
  const ext = name.includes('.') ? '.' + name.split('.').pop()?.toLowerCase() : ''
  return extensionIconMap[ext] || File
})

const modifiedTime = computed(() => {
  const ts = props.entry.modified_at
  if (!ts) return ''
  const date = new Date(ts)
  if (isNaN(date.getTime())) return ''
  const now = Date.now()
  const diff = now - date.getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return ''
  if (minutes < 60) return `${minutes}m`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h`
  const days = Math.floor(hours / 24)
  if (days < 7) return `${days}d`
  return ''
})

watch(isRenaming, async val => {
  if (val) {
    renameValue.value = props.entry.name
    await nextTick()
    renameInputRef.value?.focus()
    renameInputRef.value?.select()
  }
})

function handleClick(event: MouseEvent): void {
  emit('select', props.entry, event)
}

function handleDoubleClick(): void {
  emit('open', props.entry)
}

function handleContextMenu(event: MouseEvent): void {
  emit('contextmenu', event, props.entry)
}

function handleDragStart(event: DragEvent): void {
  if (props.entry.kind !== 'file') return
  emit('drag-start', event, props.entry)
}

function handleToggleExpand(): void {
  emit('select', props.entry)
  emit('toggle-expand', props.entry)
}

function commitRename(): void {
  const trimmed = renameValue.value.trim()
  if (!trimmed) {
    emit('cancel-rename')
    return
  }
  renamingSaving.value = true
  emit('finish-rename', props.entry, trimmed)
}

function cancelRename(): void {
  emit('cancel-rename')
}

function forwardSelect(entry: ScratchpadEntry): void {
  emit('select', entry)
}

function forwardOpen(entry: ScratchpadEntry): void {
  emit('open', entry)
}

function forwardContextmenu(event: MouseEvent, entry: ScratchpadEntry): void {
  emit('contextmenu', event, entry)
}

function forwardToggleExpand(entry: ScratchpadEntry): void {
  emit('toggle-expand', entry)
}

function forwardStartRename(entry: ScratchpadEntry): void {
  emit('start-rename', entry)
}

function forwardFinishRename(entry: ScratchpadEntry, newName: string): void {
  emit('finish-rename', entry, newName)
}

function forwardCancelRename(): void {
  emit('cancel-rename')
}

function forwardDragStart(event: DragEvent, entry: ScratchpadEntry): void {
  emit('drag-start', event, entry)
}

function formatSize(bytes: number): string {
  if (bytes === 0) return ''
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<style scoped>
.scratchpad-tree-node {
  user-select: none;
}

.node-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  height: 32px;
  padding-right: var(--spacing-sm);
  cursor: pointer;
  border-radius: var(--border-radius-sm);
  transition: background-color 0.1s;
}

.node-row:hover {
  background-color: var(--color-bg-tertiary);
}

.node-row.selected {
  background-color: var(--brand-accent);
  color: var(--color-selected-text);
}

.folder-toggle,
.folder-toggle-spacer {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.folder-toggle {
  cursor: pointer;
}

.node-icon {
  flex-shrink: 0;
}

.node-name {
  flex: 1;
  font-size: var(--font-size-md);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rename-input {
  flex: 1;
  height: 24px;
  padding: 0 var(--spacing-xs);
  font-size: var(--font-size-md);
  border: 1px solid var(--brand-accent);
  border-radius: var(--border-radius-sm);
  outline: none;
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
}

.rename-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.rename-wrapper {
  display: flex;
  align-items: center;
  flex: 1;
  gap: var(--spacing-xs);
}

.rename-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--color-border-subtle);
  border-top-color: var(--brand-accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.node-size {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.node-row.selected .node-size {
  opacity: 0.7;
}

.node-time {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  flex-shrink: 0;
  min-width: 28px;
  text-align: right;
}

.node-row.selected .node-time {
  opacity: 0.7;
}

.node-arrow {
  width: 12px;
  flex-shrink: 0;
}

.node-children {
  /* 子节点容器 */
}
</style>
