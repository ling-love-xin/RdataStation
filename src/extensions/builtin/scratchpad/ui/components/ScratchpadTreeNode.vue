<template>
  <div class="scratchpad-tree-node">
    <div
      :class="['node-row', { selected: isSelected }]"
      :style="{ paddingLeft: `${depth * 16 + 8}px` }"
      @click="handleClick"
      @dblclick="handleDoubleClick"
      @contextmenu.prevent="handleContextMenu"
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
      <input
        v-else
        ref="renameInputRef"
        v-model="renameValue"
        class="rename-input"
        @keyup.enter="commitRename"
        @keyup.escape="cancelRename"
        @blur="commitRename"
        @click.stop
      />

      <span v-if="entry.kind === 'file' && entry.size > 0" class="node-size">{{ formatSize(entry.size) }}</span>
      <span v-if="entry.kind === 'folder'" class="node-arrow" />
    </div>

    <div v-if="entry.kind === 'folder' && expanded" class="node-children">
      <ScratchpadTreeNode
        v-for="child in children"
        :key="child.path"
        :entry="child"
        :depth="depth + 1"
        :expanded-keys="expandedKeys"
        :selected-key="selectedKey"
        :renaming-key="renamingKey"
        @select="forwardSelect"
        @open="forwardOpen"
        @contextmenu="forwardContextmenu"
        @toggle-expand="forwardToggleExpand"
        @start-rename="forwardStartRename"
        @finish-rename="forwardFinishRename"
        @cancel-rename="forwardCancelRename"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  File, Folder, FolderOpen, Database, FileCode,
  Table2, FileText, Braces, BookOpen, ChevronDown, ChevronRight
} from 'lucide-vue-next'
import { NIcon } from 'naive-ui'
import { computed, ref, watch, nextTick } from 'vue'

import type { ScratchpadEntry } from '../../types'

interface Props {
  entry: ScratchpadEntry
  depth: number
  expandedKeys: Set<string>
  selectedKey: string | null
  renamingKey: string | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  select: [entry: ScratchpadEntry]
  open: [entry: ScratchpadEntry]
  contextmenu: [event: MouseEvent, entry: ScratchpadEntry]
  'toggle-expand': [entry: ScratchpadEntry]
  'start-rename': [entry: ScratchpadEntry]
  'finish-rename': [entry: ScratchpadEntry, newName: string]
  'cancel-rename': []
}>()

const expanded = computed(() => props.expandedKeys.has(props.entry.path))
const isSelected = computed(() => props.selectedKey === props.entry.path)
const isRenaming = computed(() => props.renamingKey === props.entry.path)

const renameValue = ref('')
const renameInputRef = ref<HTMLInputElement | null>(null)

const children = computed<ScratchpadEntry[]>(() => [])

const extensionIconMap: Record<string, typeof File> = {
  '.sql': Database,
  '.py': FileCode,
  '.csv': Table2,
  '.json': Braces,
  '.txt': FileText,
  '.md': BookOpen,
}

const fileIcon = computed(() => {
  const ext = props.entry.extension.toLowerCase()
  return extensionIconMap[ext] || File
})

watch(isRenaming, async (val) => {
  if (val) {
    renameValue.value = props.entry.name
    await nextTick()
    renameInputRef.value?.focus()
    renameInputRef.value?.select()
  }
})

function handleClick(): void {
  emit('select', props.entry)
}

function handleDoubleClick(): void {
  emit('open', props.entry)
}

function handleContextMenu(event: MouseEvent): void {
  emit('contextmenu', event, props.entry)
}

function handleToggleExpand(): void {
  emit('select', props.entry)
  emit('toggle-expand', props.entry)
}

function commitRename(): void {
  const trimmed = renameValue.value.trim()
  if (trimmed) {
    emit('finish-rename', props.entry, trimmed)
  } else {
    emit('cancel-rename')
  }
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
  gap: 2px;
  height: 28px;
  padding-right: 8px;
  cursor: pointer;
  border-radius: 4px;
  transition: background-color 0.1s;
}

.node-row:hover {
  background-color: var(--bg-tertiary, #e8e8e8);
}

.node-row.selected {
  background-color: var(--primary-color, #165DFF);
  color: #ffffff;
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
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rename-input {
  flex: 1;
  height: 22px;
  padding: 0 4px;
  font-size: 13px;
  border: 1px solid var(--primary-color, #165DFF);
  border-radius: 2px;
  outline: none;
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #333333);
}

.node-size {
  font-size: 11px;
  color: var(--text-tertiary, #999999);
  flex-shrink: 0;
}

.node-row.selected .node-size {
  color: rgba(255, 255, 255, 0.7);
}

.node-arrow {
  width: 12px;
  flex-shrink: 0;
}

.node-children {
  /* 子节点容器 */
}
</style>
