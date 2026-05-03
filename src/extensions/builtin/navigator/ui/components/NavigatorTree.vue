<template>
  <div class="navigator-tree">
    <!-- 树形内容 -->
    <div class="tree-content" :style="{ height: `${contentHeight}px` }">
      <VirtualList
        v-if="virtualScroll"
        ref="virtualListRef"
        :items="filteredVisibleNodes"
        :item-height="28"
        :height="contentHeight"
        :get-item-key="getItemKey"
        @scroll="handleScroll"
      >
        <template #item="{ item }">
          <NavigatorNodeComponent
            :node="item"
            :level="item.level"
            :expanded="isExpanded(item.id)"
            :selected="isSelected(item.id)"
            :loading="isLoading(item.id)"
            :highlighted="isHighlighted(item)"
            :is-leaf="item.isLeaf"
            @click="handleNodeClick(item)"
            @dblclick="handleNodeDoubleClick(item)"
            @toggle="handleNodeToggle(item)"
            @select="handleNodeSelect(item)"
          />
        </template>
      </VirtualList>

      <div v-else class="tree-list">
        <NavigatorNodeComponent
          v-for="item in filteredVisibleNodes"
          :key="item.id"
          :node="item"
          :level="item.level"
          :expanded="isExpanded(item.id)"
          :selected="isSelected(item.id)"
          :loading="isLoading(item.id)"
          :highlighted="isHighlighted(item)"
          :is-leaf="item.isLeaf"
          @click="handleNodeClick(item)"
          @dblclick="handleNodeDoubleClick(item)"
          @toggle="handleNodeToggle(item)"
          @select="handleNodeSelect(item)"
        />
      </div>
    </div>

    <!-- 空状态 -->
    <div v-if="filteredVisibleNodes.length === 0 && !loading" class="empty-state">
      <slot name="empty">
        <Database :size="32" class="empty-icon" />
        <span class="empty-text">{{ emptyText }}</span>
      </slot>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <Loader2 :size="24" class="loading-icon spinning" />
      <span class="loading-text">加载中...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Database, Loader2 } from 'lucide-vue-next'
import { ref, computed, watch, onMounted, defineExpose } from 'vue'

import NavigatorNodeComponent from './NavigatorNode.vue'
import VirtualList from './VirtualList.vue'
import { metaNavigatorService } from '../../domain/services/meta-navigator-service'
import { useNavigator } from '../composables/useNavigator'

import type { NavigatorNode } from '../../types'

// 带 level 的节点类型
interface TreeNode extends NavigatorNode {
  level: number
  isLeaf: boolean
}

interface Props {
  viewName: string
  connectionId?: string
  database?: string
  schema?: string
  height?: number
  virtualScroll?: boolean
  searchable?: boolean
  lazyLoad?: boolean
  filterTypes?: string[]
  searchQuery?: string
}

const props = withDefaults(defineProps<Props>(), {
  height: 400,
  virtualScroll: true,
  searchable: true,
  lazyLoad: true,
  filterTypes: () => ['table', 'view', 'function', 'procedure'],
  searchQuery: ''
})

const emit = defineEmits<{
  'node-click': [node: NavigatorNode]
  'node-dblclick': [node: NavigatorNode]
  'node-expand': [node: NavigatorNode]
  'node-collapse': [node: NavigatorNode]
  'node-select': [node: NavigatorNode]
}>()

// 使用导航器
const {
  nodes,
  expandedKeys,
  selectedKeys,
  loading,
  visibleNodes,
  expand,
  collapse,
  select,
  toggle,
  refresh
} = useNavigator({
  viewName: props.viewName,
  connectionId: props.connectionId,
  lazyLoad: props.lazyLoad
})

const virtualListRef = ref()

// 过滤后的可见节点
const filteredVisibleNodes = computed<TreeNode[]>(() => {
  let result = visibleNodes.value as TreeNode[]

  // 按类型过滤
  if (props.filterTypes && props.filterTypes.length > 0) {
    result = result.filter((node: TreeNode) => {
      // 保留文件夹和连接节点
      if (node.type === 'folder' || node.type === 'connection' || node.type === 'database' || node.type === 'schema') {
        return true
      }
      return props.filterTypes!.includes(node.type)
    })
  }

  // 按搜索词过滤
  if (props.searchQuery) {
    const query = props.searchQuery.toLowerCase()
    result = result.filter((node: TreeNode) =>
      node.name.toLowerCase().includes(query)
    )
  }

  return result
})

// 空状态文本
const emptyText = computed(() => {
  if (props.searchQuery) {
    return '无搜索结果'
  }
  if (props.filterTypes && props.filterTypes.length === 0) {
    return '请选择至少一个对象类型'
  }
  return '暂无数据'
})

// 计算内容高度
const contentHeight = computed(() => {
  return props.height
})

// 方法
const isExpanded = (id: string) => expandedKeys.value.has(id)
const isSelected = (id: string) => selectedKeys.value.has(id)
const isLoading = (id: string) => false // TODO: 实现加载状态
const isHighlighted = (node: TreeNode) => {
  if (!props.searchQuery) return false
  return node.name.toLowerCase().includes(props.searchQuery.toLowerCase())
}

const getItemKey = (item: TreeNode, index: number) => {
  return item.id || String(index)
}

// 事件处理
const handleNodeClick = (node: TreeNode) => {
  emit('node-click', node)
}

const handleNodeDoubleClick = (node: TreeNode) => {
  emit('node-dblclick', node)
}

const handleNodeToggle = async (node: TreeNode) => {
  if (isExpanded(node.id)) {
    collapse(node.id)
    emit('node-collapse', node)
  } else {
    await expand(node.id)
    emit('node-expand', node)
  }
}

const handleNodeSelect = (node: TreeNode) => {
  select(node.id)
  emit('node-select', node)
}

const handleScroll = (scrollTop: number) => {
  // 可以在这里处理滚动事件
}

// 加载数据
const loadData = async () => {
  if (!props.connectionId) return

  loading.value = true
  try {
    // 使用 metaNavigatorService 生成树形结构
    const mockConnections = [{
      id: props.connectionId,
      name: 'Connection',
      type: 'mysql',
      status: 'connected' as const
    }]
    
    const treeNodes = await metaNavigatorService.generateConnectionTree(
      mockConnections[0],
      expandedKeys.value,
      props.filterTypes || []
    )

    // 更新节点
    nodes.value = treeNodes.map(({ level, isLeaf, ...node }) => {
      // 确保节点包含path和depth属性
      return {
        ...node,
        path: (node as any).path || '',
        depth: (node as any).depth || 0
      } as NavigatorNode
    })

    // 默认展开第一层
    treeNodes.forEach((node) => {
      if (!node.isLeaf) {
        expandedKeys.value.add(node.id)
      }
    })
  } catch (err) {
    console.error('Failed to load navigator data:', err)
  } finally {
    loading.value = false
  }
}

// 暴露方法
const collapseAll = () => {
  expandedKeys.value.clear()
}

const expandAll = () => {
  const expandAllNodes = (nodeList: TreeNode[]) => {
    for (const node of nodeList) {
      expandedKeys.value.add(node.id)
      if (node.children) {
        expandAllNodes(node.children as TreeNode[])
      }
    }
  }
  expandAllNodes(nodes.value as TreeNode[])
}

defineExpose({
  refresh: async () => {
    await loadData()
  },
  collapseAll,
  expandAll
})

// 监听连接、数据库、schema变化
watch(() => props.connectionId, async (newId, oldId) => {
  if (newId !== oldId) {
    expandedKeys.value.clear()
    selectedKeys.value.clear()
    await loadData()
  }
})

watch(() => props.database, async () => {
  expandedKeys.value.clear()
  await loadData()
})

watch(() => props.schema, async () => {
  expandedKeys.value.clear()
  await loadData()
})

// 初始加载
onMounted(() => {
  if (props.connectionId) {
    loadData()
  }
})
</script>

<style scoped>
.navigator-tree {
  display: flex;
  flex-direction: column;
  flex: 1;
  height: 100%;
  background-color: var(--bg-secondary);
  overflow: hidden;
}

.tree-content {
  flex: 1;
  overflow: hidden;
}

.tree-list {
  height: 100%;
  overflow-y: auto;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-tertiary);
}

.empty-icon {
  margin-bottom: 12px;
  opacity: 0.5;
}

.empty-text {
  font-size: 13px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-tertiary);
}

.loading-icon {
  margin-bottom: 12px;
  color: var(--primary-color);
}

.loading-text {
  font-size: 13px;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
