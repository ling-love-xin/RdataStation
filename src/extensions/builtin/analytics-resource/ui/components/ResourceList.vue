<template>
  <div
    ref="containerRef"
    class="resource-list"
    @scroll="handleScroll"
    @contextmenu.prevent="handleContextMenu"
  >
    <div class="virtual-spacer" :style="{ height: `${totalHeight}px` }">
      <div
        class="virtual-content"
        :style="{ transform: `translateY(${offsetY}px)` }"
      >
        <div
          v-for="item in visibleItems"
          :key="item.id"
          :class="['resource-item', { selected: isSelected(item.id) }]"
          @click="handleClick(item, $event)"
          @dblclick="handleDoubleClick(item)"
        >
          <span class="resource-icon">
            {{ getResourceIcon(item.resource_type) }}
          </span>

          <div class="resource-info">
            <div class="resource-name">{{ item.name }}</div>
            <div class="resource-meta">
              {{ getResourceMeta(item) }}
            </div>
          </div>

          <span :class="['scope-tag', item.scope]">
            {{ getScopeLabel(item.scope) }}
          </span>
        </div>
      </div>
    </div>

    <div v-if="items.length === 0" class="empty-state">
      <span class="empty-icon">{{ emptyIcon }}</span>
      <p>{{ emptyText }}</p>
    </div>

    <ContextMenu
      ref="contextMenuRef"
      :items="contextMenuItems"
      @select="handleContextMenuSelect"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import type { AnalyticsResource } from '../../types'
import ContextMenu, { type ContextMenuItem } from './ContextMenu.vue'

const props = withDefaults(defineProps<{
  items: AnalyticsResource[]
  selectedIds: string[]
  itemHeight?: number
  emptyIcon?: string
  emptyText?: string
}>(), {
  itemHeight: 72,
  emptyIcon: '📭',
  emptyText: '暂无资源',
})

const emit = defineEmits<{
  select: [id: string, multiSelect: boolean]
  open: [resource: AnalyticsResource]
  delete: [id: string]
  edit: [resource: AnalyticsResource]
  copy: [resource: AnalyticsResource]
}>()

const containerRef = ref<HTMLDivElement | null>(null)
const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)

const containerHeight = ref(0)
const scrollTop = ref(0)

const totalHeight = computed(() => props.items.length * props.itemHeight)

const visibleRange = computed(() => {
  const start = Math.floor(scrollTop.value / props.itemHeight)
  const visibleCount = Math.ceil(containerHeight.value / props.itemHeight) + 2
  const startIndex = Math.max(0, start - 1)
  const endIndex = Math.min(props.items.length, startIndex + visibleCount + 2)
  return { startIndex, endIndex }
})

const offsetY = computed(() => visibleRange.value.startIndex * props.itemHeight)

const visibleItems = computed(() => {
  return props.items.slice(visibleRange.value.startIndex, visibleRange.value.endIndex)
})

const contextMenuItems = computed<ContextMenuItem[]>(() => [
  { id: 'open', label: '打开', icon: '📖', action: () => emit('open', props.items.find(i => i.id === selectedIds.value[0])!) },
  { id: 'edit', label: '编辑', icon: '✏️', action: () => emit('edit', props.items.find(i => i.id === selectedIds.value[0])!) },
  { id: 'copy', label: '复制', icon: '📋', action: () => emit('copy', props.items.find(i => i.id === selectedIds.value[0])!) },
  { id: 'separator', label: '---', disabled: true },
  { id: 'delete', label: '删除', icon: '🗑️', danger: true, action: () => emit('delete', selectedIds.value[0]) },
])

const selectedIds = computed(() => props.selectedIds)

function isSelected(id: string) {
  return selectedIds.value.includes(id)
}

function getResourceIcon(type: string) {
  switch (type) {
    case 'connection': return '🔌'
    case 'table': return '📊'
    case 'file': return '📄'
    default: return '📦'
  }
}

function getScopeLabel(scope: string) {
  switch (scope) {
    case 'global': return '🌍 全局'
    case 'project': return '📂 项目'
    case 'session': return '📌 会话'
    default: return scope
  }
}

function getResourceMeta(resource: AnalyticsResource) {
  const meta: string[] = []
  if (resource.row_count !== undefined && resource.row_count !== null) {
    meta.push(`${resource.row_count.toLocaleString()} 行`)
  }
  if (resource.column_count !== undefined && resource.column_count !== null) {
    meta.push(`${resource.column_count} 列`)
  }
  if (resource.file_size !== undefined && resource.file_size !== null) {
    meta.push(`${formatFileSize(resource.file_size)}`)
  }
  return meta.join(' · ')
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}

function handleClick(item: AnalyticsResource, event: MouseEvent) {
  const multiSelect = event.shiftKey || event.ctrlKey || event.metaKey
  emit('select', item.id, multiSelect)
}

function handleDoubleClick(item: AnalyticsResource) {
  emit('open', item)
}

function handleContextMenu(event: MouseEvent) {
  contextMenuRef.value?.open(event)
}

function handleContextMenuSelect(item: ContextMenuItem) {
  // ContextMenu handles the action
}

function handleScroll() {
  if (containerRef.value) {
    scrollTop.value = containerRef.value.scrollTop
  }
}

function updateContainerHeight() {
  if (containerRef.value) {
    containerHeight.value = containerRef.value.clientHeight
  }
}

onMounted(() => {
  updateContainerHeight()
  window.addEventListener('resize', updateContainerHeight)
})

onUnmounted(() => {
  window.removeEventListener('resize', updateContainerHeight)
})

watch(() => props.items, () => {
  updateContainerHeight()
})
</script>

<style scoped>
.resource-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
  position: relative;
}

.virtual-spacer {
  position: relative;
}

.virtual-content {
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
}

.resource-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 8px;
  margin-bottom: 8px;
  background: var(--color-background-elevated, #fff);
  cursor: pointer;
  transition: all 0.2s;
  height: 56px;
  box-sizing: border-box;
}

.resource-item:hover {
  border-color: var(--color-primary, #165dff);
}

.resource-item.selected {
  border-color: var(--color-primary, #165dff);
  background: var(--color-primary-lighter, #e8f0ff);
}

.resource-icon {
  font-size: 24px;
  flex-shrink: 0;
}

.resource-info {
  flex: 1;
  min-width: 0;
}

.resource-name {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.resource-meta {
  font-size: 12px;
  color: var(--text-secondary, #666);
}

.scope-tag {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  flex-shrink: 0;
}

.scope-tag.global {
  background: var(--tag-global-bg, #e3f2fd);
  color: var(--tag-global-text, #1976d2);
}

.scope-tag.project {
  background: var(--tag-project-bg, #e8f5e9);
  color: var(--tag-project-text, #388e3c);
}

.scope-tag.session {
  background: var(--tag-session-bg, #fff3e0);
  color: var(--tag-session-text, #f57c00);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: var(--text-secondary, #666);
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}
</style>
