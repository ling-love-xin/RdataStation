<template>
  <div class="navigator-panel">
    <!-- 顶部工具栏 -->
    <div class="navigator-header">
      <div class="header-toolbar">
        <button class="toolbar-btn" title="新建连接" @click="handleNewConnection">
          <Plus :size="16" />
        </button>
        <button class="toolbar-btn" title="刷新" @click="handleRefresh">
          <RefreshCw :size="14" />
        </button>
        <div class="toolbar-divider" />
        <button class="toolbar-btn" title="全部折叠" @click="handleCollapseAll">
          <FolderMinus :size="14" />
        </button>
        <button class="toolbar-btn" title="全部展开" @click="handleExpandAll">
          <FolderPlus :size="14" />
        </button>
        <div class="toolbar-divider" />
        <button
          class="toolbar-btn"
          :class="{ active: isLinkedWithEditor }"
          title="关联编辑器"
          @click="handleLinkWithEditor"
        >
          <Link2 :size="14" />
        </button>
        <button class="toolbar-btn" title="设置" @click="handleSettings">
          <Settings :size="14" />
        </button>
      </div>
    </div>

    <!-- 搜索栏 -->
    <div class="navigator-search">
      <div class="search-input-wrapper">
        <Search :size="14" class="search-icon" />
        <input
          v-model="searchQuery"
          type="text"
          class="search-input"
          placeholder="搜索数据库对象..."
          @keyup.enter="handleSearch"
        />
        <button v-if="searchQuery" class="search-clear" @click="clearSearch">
          <X :size="12" />
        </button>
      </div>
    </div>

    <!-- 过滤器栏 -->
    <div class="navigator-filters">
      <div class="filter-scroll">
        <button
          v-for="filter in availableFilters"
          :key="filter.type"
          class="filter-btn"
          :class="{ active: activeFilters.includes(filter.type) }"
          :title="filter.label"
          @click="toggleFilter(filter.type)"
        >
          <component :is="filter.icon" :size="12" />
          <span>{{ filter.label }}</span>
        </button>
      </div>
    </div>

    <!-- 树形内容区 -->
    <div class="navigator-content">
      <VirtualList
        v-if="filteredConnections.length > 0"
        :items="treeItems"
        :item-height="28"
        :height="contentHeight"
        :get-item-key="getItemKey"
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

      <!-- 空状态 -->
      <div v-else class="empty-state">
        <div class="empty-content">
          <Database :size="40" class="empty-icon" />
          <p class="empty-title">没有数据库连接</p>
          <p class="empty-desc">点击上方 + 按钮新建连接</p>
          <button class="empty-btn" @click="handleNewConnection">
            新建连接
          </button>
        </div>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading" class="loading-overlay">
        <Loader2 class="loading-icon" :size="24" />
        <span>加载中...</span>
      </div>
    </div>

    <!-- 底部状态栏 -->
    <div class="navigator-footer">
      <span class="status-text">{{ statusText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Search,
  X,
  Database,
  RefreshCw,
  Loader2,
  Plus,
  FolderMinus,
  FolderPlus,
  Link2,
  Settings,
  Table,
  FileText,
  FunctionSquare,
  Workflow,
  LayoutGrid,
  Zap,
  List,
  MoreHorizontal
} from 'lucide-vue-next'
import { ref, computed, onMounted, watch, type Component } from 'vue'

import { useConnectionStore } from '@/extensions/builtin/connection/ui/stores/connection-store'

import { metaNavigatorService } from '../../domain/services/meta-navigator-service'
import NavigatorNodeComponent from '../components/NavigatorNode.vue'
import VirtualList from '../components/VirtualList.vue'

import type { NavigatorNode, ConnectionInfo } from '../../types'

// 带 level 的树节点类型
interface TreeNode extends NavigatorNode {
  level: number
  isLeaf: boolean
  status?: 'connected' | 'disconnected' | 'connecting'
}

// 过滤器定义
interface FilterDef {
  type: string
  label: string
  icon: Component
}

// 图标映射
const iconMap: Record<string, Component> = {
  table: Table,
  view: FileText,
  function: FunctionSquare,
  procedure: Workflow,
  index: LayoutGrid,
  trigger: Zap,
  column: List
}

// 连接 store
const connectionStore = useConnectionStore()

// 连接列表 - 从 store 获取真实连接
const connections = computed<ConnectionInfo[]>(() => {
  return connectionStore.connections.map(conn => ({
    id: conn.connId,
    name: conn.name,
    type: conn.dbType,
    status: conn.status === 'connected' ? 'connected' : 'disconnected',
    host: conn.url ? extractHostFromUrl(conn.url) : undefined,
    port: conn.url ? extractPortFromUrl(conn.url) : undefined
  }))
})

// 从 URL 提取主机
function extractHostFromUrl(url: string): string | undefined {
  try {
    const match = url.match(/@([^:]+):/)
    return match ? match[1] : undefined
  } catch {
    return undefined
  }
}

// 从 URL 提取端口
function extractPortFromUrl(url: string): number | undefined {
  try {
    const match = url.match(/@[^:]+:(\d+)\//)
    return match ? parseInt(match[1]) : undefined
  } catch {
    return undefined
  }
}

const searchQuery = ref('')
const loading = ref(false)
const isLinkedWithEditor = ref(true)
const expandedKeys = ref<Set<string>>(new Set())
const selectedKeys = ref<Set<string>>(new Set())
const loadingKeys = ref<Set<string>>(new Set())
const treeItems = ref<TreeNode[]>([])

// 动态过滤器 - 基于所有连接的元数据配置
const availableFilters = computed<FilterDef[]>(() => {
  const filterTypes = new Set<string>()

  for (const conn of connections.value) {
    const supportedTypes = metaNavigatorService.getSupportedNodeTypes(conn.type)
    for (const type of supportedTypes) {
      if (type !== 'connection' && type !== 'database' && type !== 'schema' && type !== 'column' && type !== 'folder') {
        filterTypes.add(type)
      }
    }
  }

  return Array.from(filterTypes).map(type => ({
    type,
    label: getFilterLabel(type),
    icon: iconMap[type] || MoreHorizontal
  }))
})

// 当前激活的过滤器
const activeFilters = ref<string[]>(['table', 'view'])

// 过滤后的连接
const filteredConnections = computed(() => {
  if (!searchQuery.value) return connections.value
  const query = searchQuery.value.toLowerCase()
  return connections.value.filter(conn =>
    conn.name.toLowerCase().includes(query) ||
    conn.host?.toLowerCase().includes(query)
  )
})

// 状态文本
const statusText = computed(() => {
  const total = connections.value.length
  const connected = connections.value.filter(c => c.status === 'connected').length
  return `${connected}/${total} 连接`
})

// 内容高度
const contentHeight = computed(() => {
  // 减去头部、搜索栏、过滤器栏和底部状态栏的高度
  return 400 // 临时固定值
})

// 方法
const isExpanded = (id: string) => expandedKeys.value.has(id)
const isSelected = (id: string) => selectedKeys.value.has(id)
const isLoading = (id: string) => loadingKeys.value.has(id)
const isHighlighted = (node: TreeNode) => {
  if (!searchQuery.value) return false
  return node.name.toLowerCase().includes(searchQuery.value.toLowerCase())
}

const getItemKey = (item: TreeNode, index: number) => {
  return item.id
}

const getFilterLabel = (type: string): string => {
  const labels: Record<string, string> = {
    table: '表',
    view: '视图',
    function: '函数',
    procedure: '存储过程',
    index: '索引',
    trigger: '触发器',
    column: '列'
  }
  return labels[type] || type
}

// 加载树数据
const loadTreeData = async () => {
  loading.value = true
  const result: TreeNode[] = []

  for (const conn of filteredConnections.value) {
    const connTree = await metaNavigatorService.generateConnectionTree(
      conn,
      expandedKeys.value,
      activeFilters.value
    )
    // 转换为TreeNode类型
    result.push(...connTree.map(node => node as TreeNode))
  }

  treeItems.value = result
  loading.value = false
}

// 处理节点点击
const handleNodeClick = (node: TreeNode) => {
  selectedKeys.value.clear()
  selectedKeys.value.add(node.id)
}

// 处理节点双击
const handleNodeDoubleClick = (node: TreeNode) => {
  if (node.type === 'table' || node.type === 'view') {
    // 打开表数据面板
    window.dispatchEvent(new CustomEvent('open-table-data', {
      detail: {
        connectionId: node.parentId,
        tableName: node.name
      }
    }))
  } else if (node.type === 'connection') {
    // 打开 SQL 编辑器
    window.dispatchEvent(new CustomEvent('open-sql-editor', {
      detail: {
        connectionId: node.id,
        databaseName: '',
        sql: ''
      }
    }))
  }
}

// 处理节点展开/折叠
const handleNodeToggle = async (node: TreeNode) => {
  if (expandedKeys.value.has(node.id)) {
    expandedKeys.value.delete(node.id)
  } else {
    expandedKeys.value.add(node.id)
    
    // 如果是连接节点且未连接，尝试连接
    if (node.type === 'connection' && node.status !== 'connected') {
      loadingKeys.value.add(node.id)
      try {
        const conn = connectionStore.connections.find(c => c.connId === node.id)
        if (conn && conn.url) {
          await connectionStore.connect(conn.dbType, conn.url, conn.name)
        }
      } catch (e) {
        console.error('连接失败:', e)
      } finally {
        loadingKeys.value.delete(node.id)
      }
    }
  }
  await loadTreeData()
}

// 处理节点选择
const handleNodeSelect = (node: TreeNode) => {
  selectedKeys.value.clear()
  selectedKeys.value.add(node.id)
}

// 处理搜索
const handleSearch = () => {
  // TODO: 实现搜索过滤
}

const clearSearch = () => {
  searchQuery.value = ''
}

// 处理新建连接
const handleNewConnection = () => {
  window.dispatchEvent(new CustomEvent('open-new-connection'))
}

// 处理刷新
const handleRefresh = async () => {
  await connectionStore.loadConnections()
  await loadTreeData()
}

// 处理全部折叠
const handleCollapseAll = () => {
  expandedKeys.value.clear()
  loadTreeData()
}

// 处理全部展开
const handleExpandAll = () => {
  for (const conn of connections.value) {
    expandedKeys.value.add(conn.id)
  }
  loadTreeData()
}

// 处理关联编辑器
const handleLinkWithEditor = () => {
  isLinkedWithEditor.value = !isLinkedWithEditor.value
}

// 处理设置
const handleSettings = () => {
  // TODO: 打开设置面板
}

// 切换过滤器
const toggleFilter = (type: string) => {
  const index = activeFilters.value.indexOf(type)
  if (index > -1) {
    activeFilters.value.splice(index, 1)
  } else {
    activeFilters.value.push(type)
  }
  loadTreeData()
}

// 监听过滤器和连接变化
watch([filteredConnections, activeFilters], () => {
  loadTreeData()
}, { deep: true })

// 组件挂载
onMounted(async () => {
  // 加载连接列表
  await connectionStore.loadConnections()
  
  // 默认展开前2个连接
  for (let i = 0; i < Math.min(2, connections.value.length); i++) {
    expandedKeys.value.add(connections.value[i].id)
  }
  loadTreeData()
})
</script>

<style scoped>
.navigator-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--bg-secondary);
}

/* 顶部工具栏 */
.navigator-header {
  display: flex;
  align-items: center;
  height: 36px;
  padding: 0 8px;
  background-color: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
}

.header-toolbar {
  display: flex;
  align-items: center;
  gap: 2px;
  flex: 1;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.toolbar-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.toolbar-btn.active {
  background-color: var(--primary-light);
  color: var(--primary-color);
}

.toolbar-divider {
  width: 1px;
  height: 16px;
  background-color: var(--border-color);
  margin: 0 4px;
}

/* 搜索栏 */
.navigator-search {
  padding: 8px;
  border-bottom: 1px solid var(--border-color);
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 10px;
  color: var(--text-tertiary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 32px;
  padding: 0 28px 0 32px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
  transition: all 0.2s;
}

.search-input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

.search-clear {
  position: absolute;
  right: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
}

.search-clear:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

/* 过滤器栏 */
.navigator-filters {
  padding: 6px 8px;
  border-bottom: 1px solid var(--border-color);
  overflow: hidden;
}

.filter-scroll {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.filter-scroll::-webkit-scrollbar {
  display: none;
}

.filter-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  flex-shrink: 0;
}

.filter-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.filter-btn.active {
  background-color: var(--primary-light);
  border-color: var(--primary-color);
  color: var(--primary-color);
}

/* 内容区 */
.navigator-content {
  flex: 1;
  overflow: hidden;
  position: relative;
}

/* 空状态 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 20px;
}

.empty-content {
  text-align: center;
  color: var(--text-secondary);
}

.empty-icon {
  margin-bottom: 12px;
  color: var(--text-tertiary);
}

.empty-title {
  margin: 0 0 4px 0;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.empty-desc {
  margin: 0 0 16px 0;
  font-size: 12px;
  color: var(--text-secondary);
}

.empty-btn {
  padding: 8px 16px;
  border: 1px solid var(--primary-color);
  border-radius: 4px;
  background-color: var(--primary-color);
  color: white;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.empty-btn:hover {
  background-color: var(--primary-hover);
}

/* 加载状态 */
.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  background-color: rgba(var(--bg-primary), 0.8);
  color: var(--text-secondary);
  font-size: 12px;
}

.loading-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* 底部状态栏 */
.navigator-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 28px;
  padding: 0 12px;
  background-color: var(--bg-tertiary);
  border-top: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-secondary);
}

.status-text {
  font-weight: 500;
}
</style>
