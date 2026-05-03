<template>
  <div
    class="virtual-tree-node"
    :class="{
      'is-expanded': node.isExpanded,
      'is-selected': isSelected,
      'is-loading': node.isLoading,
      'is-favorite': isFavorite
    }"
    :style="{ paddingLeft: `${node.level * 16 + 8}px` }"
    @click="handleClick"
    @dblclick="handleDblClick"
    @contextmenu.prevent="handleContextMenu"
  >
    <span class="expand-icon" @click.stop="handleExpand">
      <ChevronRight v-if="!node.isExpanded && !node.isLeaf" :size="14" />
      <ChevronDown v-if="node.isExpanded" :size="14" />
      <span v-if="node.isLeaf" class="leaf-spacer" />
      <Loader2 v-if="node.isLoading" :size="14" class="loading-icon" />
    </span>

    <component :is="iconConfig.icon" :size="14" class="node-icon" :style="{ color: iconColor }" />

    <Star v-if="isFavorite" :size="12" class="favorite-icon" />

    <span class="node-label" :class="{ 'is-highlight': isHighlighted }">
      <template v-if="isHighlighted">
        <span v-for="(part, index) in labelParts" :key="index" :class="{ 'highlight-match': part.isMatch }">
          {{ part.text }}
        </span>
      </template>
      <template v-else>
        {{ node.label }}
      </template>
    </span>

    <span v-if="node.connectionStatus === 'connected'" class="status-dot connected" title="已连接"></span>
    <span v-else-if="node.type === 'connection'" class="status-dot disconnected" title="未连接"></span>

    <span v-if="node.connectionTags?.length" class="connection-tags">
      <span v-for="tag in node.connectionTags" :key="tag" class="tag">{{ tag }}</span>
    </span>
  </div>
</template>

<script setup lang="ts">
import { ChevronRight, ChevronDown, Loader2, Star } from 'lucide-vue-next'
import { computed } from 'vue'

import { getNodeIcon } from '../config/node-icons'

import type { VirtualTreeNode } from '../types/virtual-tree'

interface Props {
  node: VirtualTreeNode
  isSelected: boolean
  searchQuery?: string
  favoriteKeys?: Set<string>
}

const props = withDefaults(defineProps<Props>(), {
  searchQuery: '',
  favoriteKeys: () => new Set()
})

const emit = defineEmits<{
  expand: [node: VirtualTreeNode]
  select: [node: VirtualTreeNode]
  'context-menu': [node: VirtualTreeNode, event: MouseEvent]
  'dblclick': [node: VirtualTreeNode]
}>()

const iconConfig = computed(() => getNodeIcon(props.node.type))

const iconColor = computed(() => {
  if (props.node.connectionStatus === 'connected') {
    return '#00B42A'
  }
  return iconConfig.value.color || 'var(--text-secondary)'
})

const isFavorite = computed(() => props.favoriteKeys.has(props.node.key))

const isHighlighted = computed(() => {
  return props.searchQuery && props.node.label.toLowerCase().includes(props.searchQuery.toLowerCase())
})

const labelParts = computed(() => {
  if (!props.searchQuery) return [{ text: props.node.label, isMatch: false }]
  
  const label = props.node.label
  const query = props.searchQuery.toLowerCase()
  const labelLower = label.toLowerCase()
  const parts: Array<{ text: string; isMatch: boolean }> = []
  
  let lastIndex = 0
  let index = labelLower.indexOf(query)
  
  while (index !== -1) {
    if (index > lastIndex) {
      parts.push({ text: label.slice(lastIndex, index), isMatch: false })
    }
    parts.push({ text: label.slice(index, index + query.length), isMatch: true })
    lastIndex = index + query.length
    index = labelLower.indexOf(query, lastIndex)
  }
  
  if (lastIndex < label.length) {
    parts.push({ text: label.slice(lastIndex), isMatch: false })
  }
  
  return parts
})

function handleClick() {
  emit('select', props.node)
}

function handleDblClick() {
  emit('dblclick', props.node)
}

function handleExpand() {
  if (!props.node.isLeaf) {
    emit('expand', props.node)
  }
}

function handleContextMenu(event: MouseEvent) {
  emit('context-menu', props.node, event)
}
</script>

<style scoped>
.virtual-tree-node {
  display: flex;
  align-items: center;
  height: 28px;
  cursor: pointer;
  user-select: none;
  font-size: 13px;
  color: var(--text-primary);
  transition: background-color 0.1s;
}

.virtual-tree-node:hover {
  background-color: var(--bg-tertiary);
}

.virtual-tree-node.is-selected {
  background-color: var(--primary-color);
  color: white;
}

.virtual-tree-node.is-selected .node-icon {
  color: white;
}

.expand-icon {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 2px;
  flex-shrink: 0;
}

.leaf-spacer {
  width: 14px;
}

.node-icon {
  margin-right: 6px;
  flex-shrink: 0;
  color: var(--text-secondary);
}

.node-icon.icon-connected {
  color: #22c55e;
}

.virtual-tree-node.is-selected .node-icon {
  color: white;
}

.favorite-icon {
  color: #f59e0b;
  margin-right: 4px;
  flex-shrink: 0;
}

.highlight-match {
  background-color: rgba(255, 255, 0, 0.3);
  color: inherit;
  font-weight: 600;
}

.dark .highlight-match {
  background-color: rgba(255, 255, 0, 0.2);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-left: 6px;
  flex-shrink: 0;
}

.status-dot.connected {
  background-color: #22c55e;
  box-shadow: 0 0 4px rgba(34, 197, 94, 0.5);
}

.status-dot.disconnected {
  background-color: var(--text-tertiary);
  opacity: 0.5;
}

.node-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.connection-tags {
  display: flex;
  gap: 4px;
  margin-left: 8px;
  flex-shrink: 0;
}

.tag {
  font-size: 10px;
  padding: 1px 6px;
  background: var(--bg-tertiary);
  border-radius: 3px;
  color: var(--text-secondary);
}

.loading-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
