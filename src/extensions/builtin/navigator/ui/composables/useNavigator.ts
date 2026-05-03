/**
 * useNavigator Composable
 * 提供导航栏的核心功能和状态管理
 */

import { ref, computed, onMounted, onUnmounted, watch } from 'vue'

import { l1Cache, l2Cache } from '../../domain/cache'
import { viewEngine } from '../../domain/engine/view-engine'

import type { NavigatorNode, Delta } from '../../types'

export interface UseNavigatorOptions {
  viewName: string
  connectionId?: string
  lazyLoad?: boolean
  cacheEnabled?: boolean
}

export interface UseNavigatorReturn {
  // 状态
  nodes: import('vue').Ref<NavigatorNode[]>
  expandedKeys: import('vue').Ref<Set<string>>
  selectedKeys: import('vue').Ref<Set<string>>
  loading: import('vue').Ref<boolean>
  error: import('vue').Ref<Error | null>

  // 计算属性
  visibleNodes: import('vue').ComputedRef<(NavigatorNode & { level: number })[]>
  selectedNode: import('vue').ComputedRef<NavigatorNode | null>

  // 方法
  expand: (nodeId: string) => Promise<void>
  collapse: (nodeId: string) => void
  select: (nodeId: string) => void
  toggle: (nodeId: string) => Promise<void>
  refresh: () => Promise<void>
  search: (query: string) => NavigatorNode[]
}

export function useNavigator(options: UseNavigatorOptions): UseNavigatorReturn {
  // 状态
  const nodes = ref<NavigatorNode[]>([])
  const expandedKeys = ref<Set<string>>(new Set())
  const selectedKeys = ref<Set<string>>(new Set())
  const loading = ref(false)
  const error = ref<Error | null>(null)

  // 获取视图
  const view = computed(() => viewEngine.getView<NavigatorNode>(options.viewName))

  // 订阅视图变更
  let unsubscribe: (() => void) | null = null

  onMounted(() => {
    if (view.value) {
      // 初始化数据
      nodes.value = [...view.value.snapshot]

      // 订阅增量更新
      unsubscribe = view.value.onChange((delta) => {
        applyDeltaToNodes(delta)
      })
    }

    // 从缓存加载初始数据
    if (options.cacheEnabled !== false) {
      loadFromCache()
    }
  })

  onUnmounted(() => {
    unsubscribe?.()
  })

  // 监听连接变化
  watch(() => options.connectionId, async (newId, oldId) => {
    if (newId !== oldId) {
      // 清空当前数据
      nodes.value = []
      expandedKeys.value.clear()
      selectedKeys.value.clear()

      // 从缓存加载新数据
      if (options.cacheEnabled !== false) {
        await loadFromCache()
      }
    }
  })

  // 计算可见节点（扁平化）
  const visibleNodes = computed(() => {
    const result: Array<NavigatorNode & { level: number }> = []

    const traverse = (nodeList: NavigatorNode[], level: number) => {
      for (const node of nodeList) {
        result.push({ ...node, level })

        if (expandedKeys.value.has(node.id) && node.children) {
          traverse(node.children, level + 1)
        }
      }
    }

    traverse(nodes.value, 0)
    return result
  })

  // 计算选中节点
  const selectedNode = computed(() => {
    const selectedId = Array.from(selectedKeys.value)[0]
    if (!selectedId) return null

    return findNode(nodes.value, selectedId)
  })

  // 展开节点
  const expand = async (nodeId: string): Promise<void> => {
    expandedKeys.value.add(nodeId)

    // 懒加载子节点
    if (options.lazyLoad) {
      const node = findNode(nodes.value, nodeId)
      if (node && !node.children && !node.isLeaf) {
        await loadChildren(nodeId)
      }
    }

    // 保存展开状态到缓存
    if (options.cacheEnabled !== false) {
      await saveExpandedState()
    }
  }

  // 折叠节点
  const collapse = (nodeId: string): void => {
    expandedKeys.value.delete(nodeId)

    // 保存展开状态到缓存
    if (options.cacheEnabled !== false) {
      saveExpandedState()
    }
  }

  // 选择节点
  const select = (nodeId: string): void => {
    selectedKeys.value.clear()
    selectedKeys.value.add(nodeId)
  }

  // 切换展开/折叠
  const toggle = async (nodeId: string): Promise<void> => {
    if (expandedKeys.value.has(nodeId)) {
      collapse(nodeId)
    } else {
      await expand(nodeId)
    }
  }

  // 刷新
  const refresh = async (): Promise<void> => {
    loading.value = true
    error.value = null

    try {
      await viewEngine.refreshView(options.viewName)

      // 重新加载展开节点的子节点
      for (const nodeId of expandedKeys.value) {
        await loadChildren(nodeId)
      }
    } catch (err) {
      error.value = err as Error
    } finally {
      loading.value = false
    }
  }

  // 搜索
  const search = (query: string): NavigatorNode[] => {
    const results: NavigatorNode[] = []
    const lowerQuery = query.toLowerCase()

    const searchInNodes = (nodeList: NavigatorNode[]) => {
      for (const node of nodeList) {
        if (node.name.toLowerCase().includes(lowerQuery)) {
          results.push(node)
        }

        if (node.children) {
          searchInNodes(node.children)
        }
      }
    }

    searchInNodes(nodes.value)
    return results
  }

  // 从缓存加载
  const loadFromCache = async (): Promise<void> => {
    if (!options.connectionId) return

    try {
      // 尝试从 L1 缓存加载
      const l1Data = l1Cache.get(`navigator:${options.connectionId}`) as NavigatorNode[] | undefined
      if (l1Data) {
        nodes.value = l1Data
        return
      }

      // 尝试从 L2 缓存加载
      const l2Data = await l2Cache.get(`navigator:${options.connectionId}`) as NavigatorNode[] | undefined
      if (l2Data) {
        nodes.value = l2Data
        // 提升到 L1
        l1Cache.set(`navigator:${options.connectionId}`, l2Data)
        return
      }
    } catch (err) {
      console.warn('Failed to load from cache:', err)
    }
  }

  // 保存到缓存
  const saveToCache = async (): Promise<void> => {
    if (!options.connectionId) return

    try {
      // 保存到 L1
      l1Cache.set(`navigator:${options.connectionId}`, nodes.value)

      // 保存到 L2
      await l2Cache.set(`navigator:${options.connectionId}`, nodes.value)
    } catch (err) {
      console.warn('Failed to save to cache:', err)
    }
  }

  // 加载展开状态
  const loadExpandedState = async (): Promise<void> => {
    if (!options.connectionId) return

    try {
      const state = await l2Cache.get<string[]>(`navigator:expanded:${options.connectionId}`)
      if (state) {
        expandedKeys.value = new Set(state)
      }
    } catch (err) {
      console.warn('Failed to load expanded state:', err)
    }
  }

  // 保存展开状态
  const saveExpandedState = async (): Promise<void> => {
    if (!options.connectionId) return

    try {
      await l2Cache.set(`navigator:expanded:${options.connectionId}`, Array.from(expandedKeys.value))
    } catch (err) {
      console.warn('Failed to save expanded state:', err)
    }
  }

  // 加载子节点
  const loadChildren = async (nodeId: string): Promise<void> => {
    loading.value = true

    try {
      // 从缓存加载
      const cached = await l2Cache.get<NavigatorNode[]>(`navigator:children:${nodeId}`)
      if (cached) {
        updateNodeChildren(nodeId, cached)
        return
      }

      // TODO: 从 API 加载
      // const children = await api.fetchChildren(nodeId)
      // updateNodeChildren(nodeId, children)
      // await l2Cache.set(`navigator:children:${nodeId}`, children)
    } catch (err) {
      console.error('Failed to load children:', err)
    } finally {
      loading.value = false
    }
  }

  // 应用增量到节点列表
  const applyDeltaToNodes = (delta: Delta<NavigatorNode>): void => {
    switch (delta.type) {
      case 'ADD':
        addNode(delta.item, delta.parentId)
        break
      case 'REMOVE':
        removeNode(delta.id)
        break
      case 'UPDATE':
        updateNode(delta.id, delta.changes)
        break
    }

    // 保存到缓存
    if (options.cacheEnabled !== false) {
      saveToCache()
    }
  }

  // 辅助方法
  const addNode = (node: NavigatorNode, parentId: string | null): void => {
    if (!parentId) {
      nodes.value.push(node)
      return
    }

    const parent = findNode(nodes.value, parentId)
    if (parent) {
      if (!parent.children) {
        parent.children = []
      }
      parent.children.push(node)
    }
  }

  const removeNode = (nodeId: string): void => {
    const removeFromList = (list: NavigatorNode[]): boolean => {
      const index = list.findIndex(n => n.id === nodeId)
      if (index > -1) {
        list.splice(index, 1)
        return true
      }

      for (const node of list) {
        if (node.children && removeFromList(node.children)) {
          return true
        }
      }

      return false
    }

    removeFromList(nodes.value)
  }

  const updateNode = (nodeId: string, changes: Partial<NavigatorNode>): void => {
    const node = findNode(nodes.value, nodeId)
    if (node) {
      Object.assign(node, changes)
    }
  }

  const updateNodeChildren = (nodeId: string, children: NavigatorNode[]): void => {
    const node = findNode(nodes.value, nodeId)
    if (node) {
      node.children = children
    }
  }

  const findNode = (list: NavigatorNode[], id: string): NavigatorNode | null => {
    for (const node of list) {
      if (node.id === id) return node
      if (node.children) {
        const found = findNode(node.children, id)
        if (found) return found
      }
    }
    return null
  }

  return {
    nodes,
    expandedKeys,
    selectedKeys,
    loading,
    error,
    visibleNodes,
    selectedNode,
    expand,
    collapse,
    select,
    toggle,
    refresh,
    search
  }
}
