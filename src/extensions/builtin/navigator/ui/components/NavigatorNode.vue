<template>
  <div
    class="navigator-node"
    :class="{
      'is-expanded': props.expanded,
      'is-selected': props.selected,
      'is-loading': props.loading,
      'is-highlighted': props.highlighted
    }"
    :style="{ paddingLeft: `${props.level * 16}px` }"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    @contextmenu.prevent="handleContextMenu"
  >
    <!-- 展开/折叠图标 -->
    <div class="node-expand-icon" @click.stop="handleToggle">
      <ChevronRight
        v-if="!props.isLeaf"
        :size="14"
        :class="{ 'is-expanded': props.expanded }"
      />
    </div>

    <!-- 节点图标 -->
    <div class="node-icon">
      <slot name="icon" :node="props.node">
        <component :is="getNodeIcon(props.node)" :size="14" />
      </slot>
    </div>

    <!-- 节点标签 -->
    <div class="node-label">
      <slot name="label" :node="props.node">
        <span class="node-name">{{ props.node.name }}</span>
        <span v-if="props.node.derived?.badgeCount" class="node-badge">
          {{ props.node.derived.badgeCount }}
        </span>
      </slot>
    </div>

    <!-- 节点后缀 -->
    <div class="node-suffix">
      <slot name="suffix" :node="props.node">
        <span v-if="props.node.metadata?.dataType" class="node-type">
          {{ props.node.metadata.dataType }}
        </span>
      </slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Database,
  Table,
  FileText,
  Columns,
  Key,
  Hash,
  Folder,
  FolderOpen,
  ChevronRight,
  Plug
} from 'lucide-vue-next'

import type { NavigatorNode, NodeType } from '../../types'

interface Props {
  node: NavigatorNode
  level: number
  expanded: boolean
  selected: boolean
  loading: boolean
  highlighted: boolean
  isLeaf: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  click: [node: NavigatorNode]
  dblclick: [node: NavigatorNode]
  toggle: [node: NavigatorNode]
  select: [node: NavigatorNode]
  contextmenu: [node: NavigatorNode, event: MouseEvent]
}>()

// 获取节点图标
const getNodeIcon = (node: NavigatorNode) => {
  const iconMap: Record<NodeType, any> = {
    project: Folder,
    connection: Plug,
    database: Database,
    schema: Folder,
    table: Table,
    view: FileText,
    procedure: Folder,
    function: Folder,
    column: Columns,
    index: Key,
    trigger: Hash,
    folder: props.expanded ? FolderOpen : Folder
  }

  return iconMap[node.type] || Folder
}

// 处理点击
const handleClick = () => {
  emit('click', props.node)
  emit('select', props.node)
}

// 处理双击
const handleDoubleClick = () => {
  emit('dblclick', props.node)
}

// 处理展开/折叠
const handleToggle = () => {
  emit('toggle', props.node)
}

// 处理右键菜单
const handleContextMenu = (event: MouseEvent) => {
  emit('contextmenu', props.node, event)
}
</script>

<style scoped>
.navigator-node {
  display: flex;
  align-items: center;
  height: 28px;
  padding: 0 8px;
  cursor: pointer;
  user-select: none;
  transition: background-color 0.2s;
}

.navigator-node:hover {
  background-color: var(--bg-hover);
}

.navigator-node.is-selected {
  background-color: var(--primary-light);
  color: var(--primary-color);
}

.navigator-node.is-highlighted {
  background-color: var(--warning-light);
}

.node-expand-icon {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 4px;
  cursor: pointer;
}

.node-expand-icon svg {
  transition: transform 0.2s;
}

.node-expand-icon svg.is-expanded {
  transform: rotate(90deg);
}

.node-icon {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 6px;
  color: var(--text-secondary);
}

.node-label {
  flex: 1;
  display: flex;
  align-items: center;
  min-width: 0;
  overflow: hidden;
}

.node-name {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-badge {
  margin-left: 6px;
  padding: 0 4px;
  font-size: 10px;
  background-color: var(--bg-hover);
  border-radius: 4px;
  color: var(--text-secondary);
}

.node-suffix {
  margin-left: 8px;
  display: flex;
  align-items: center;
}

.node-type {
  font-size: 11px;
  color: var(--text-tertiary);
  font-style: italic;
}
</style>
