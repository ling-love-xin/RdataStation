/**
 * 虚拟树控制器 - 重构版
 *
 * 支持：
 * - 动态层级深度（7-8层）
 * - 健壮的 key 编码（base64）
 * - 缓存感知加载
 * - 完善的错误处理
 */

import { ref, computed, shallowRef } from 'vue'

import { NodeKeyEncoder } from '../types/virtual-tree'

import type { VirtualTreeNode } from '../types/virtual-tree'

export interface UseVirtualTreeOptions {
  /** 加载子节点的回调 */
  onLoadChildren: (node: VirtualTreeNode) => Promise<VirtualTreeNode[]>
  /** 选择节点的回调 */
  onSelect?: (node: VirtualTreeNode) => void
  /** 展开节点的回调 */
  onExpand?: (node: VirtualTreeNode) => void
}

export function useVirtualTree(options: UseVirtualTreeOptions) {
  const { onLoadChildren, onSelect, onExpand } = options

  // 扁平化的节点数组（纯对象，无响应式）
  const flatNodes = shallowRef<VirtualTreeNode[]>([])

  // 节点映射表（用于快速查找）
  const nodeMap = new Map<string, VirtualTreeNode>()

  // 根节点 keys
  const rootKeys = shallowRef<string[]>([])

  // 选中的节点 key
  const selectedKey = ref<string | null>(null)

  // 当前选中的节点
  const selectedNode = computed<VirtualTreeNode | null>(() => {
    if (!selectedKey.value) return null
    return nodeMap.get(selectedKey.value) || null
  })

  // 正在加载的节点 key 集合（防止重复加载）
  const loadingKeys = new Set<string>()

  /**
   * 添加根节点
   */
  function addRootNode(node: VirtualTreeNode) {
    nodeMap.set(node.key, node)
    rootKeys.value = [...rootKeys.value, node.key]
    flatNodes.value = [...flatNodes.value, node]
  }

  /**
   * 获取指定节点的所有子节点
   */
  function getChildren(parentKey: string): VirtualTreeNode[] {
    const parentIndex = flatNodes.value.findIndex(n => n.key === parentKey)
    if (parentIndex === -1) return []

    const parentLevel = flatNodes.value[parentIndex].level
    const children: VirtualTreeNode[] = []

    let i = parentIndex + 1
    while (i < flatNodes.value.length) {
      if (flatNodes.value[i].level <= parentLevel) break
      children.push(flatNodes.value[i])
      i++
    }

    return children
  }

  /**
   * 更新根节点列表
   */
  function setRootNodes(nodes: VirtualTreeNode[]) {
    try {
      // 保存所有节点的展开状态（不仅仅是根节点）
      const expandedNodes = new Map<string, VirtualTreeNode>()
      nodeMap.forEach((node, key) => {
        if (node.isExpanded && !node.isLeaf) {
          expandedNodes.set(key, node)
        }
      })

      // 清除旧数据
      nodeMap.clear()
      rootKeys.value = []
      loadingKeys.clear()

      // 添加新节点
      nodes.forEach(node => {
        nodeMap.set(node.key, node)
        rootKeys.value.push(node.key)
      })

      flatNodes.value = [...nodes]

      // 恢复已展开状态标记（子节点需要重新加载）
      expandedNodes.forEach((oldNode, key) => {
        const newNode = nodeMap.get(key)
        if (newNode) {
          newNode.isExpanded = true
          newNode.isLoaded = oldNode.isLoaded
          newNode.childCount = oldNode.childCount
        }
      })
    } catch (error) {
      console.error('setRootNodes failed:', error)
    }
  }

  /**
   * 展开/收起节点
   */
  async function toggleNode(node: VirtualTreeNode) {
    if (node.isLeaf) return

    if (node.isLoading) return

    const realNode = nodeMap.get(node.key)
    if (!realNode) return

    if (realNode.isExpanded) {
      realNode.isExpanded = false
      removeChildren(node.key)
    } else {
      realNode.isExpanded = true

      // 如果子节点已加载过，直接展开不重新加载
      if (realNode.isLoaded && realNode.childCount > 0) {
        // 已加载过，直接展开
        flatNodes.value = [...flatNodes.value]
        return
      }

      if (!loadingKeys.has(node.key)) {
        loadingKeys.add(node.key)
        realNode.isLoading = true
        try {
          const children = await onLoadChildren(realNode)
          if (children && children.length > 0) {
            insertChildren(node.key, children)
            realNode.childCount = children.length
            realNode.isLoaded = true
          } else {
            // 没有子节点，标记为叶子
            realNode.isLeaf = true
            realNode.childCount = 0
            realNode.isLoaded = true
          }
        } catch (error) {
          console.error('加载子节点失败:', error)
          realNode.isExpanded = false
        } finally {
          realNode.isLoading = false
          loadingKeys.delete(node.key)
        }
      }
    }

    flatNodes.value = [...flatNodes.value]
  }

  /**
   * 插入子节点到扁平数组中的正确位置
   */
  function insertChildren(parentKey: string, children: VirtualTreeNode[]) {
    if (!children || children.length === 0) return

    const parentIndex = flatNodes.value.findIndex(n => n.key === parentKey)
    if (parentIndex === -1) return

    // 设置子节点的 parentId 并添加到 nodeMap
    children.forEach(child => {
      child.parentId = parentKey
      nodeMap.set(child.key, child)
    })

    // 找到插入位置（父节点之后，跳过已有的子节点）
    let insertIndex = parentIndex + 1
    const parentLevel = flatNodes.value[parentIndex].level
    while (insertIndex < flatNodes.value.length) {
      const existingNode = flatNodes.value[insertIndex]
      if (existingNode.level <= parentLevel) {
        break
      }
      insertIndex++
    }

    // 插入子节点
    const newNodes = [
      ...flatNodes.value.slice(0, insertIndex),
      ...children,
      ...flatNodes.value.slice(insertIndex),
    ]
    flatNodes.value = newNodes
  }

  /**
   * 移除子节点（收起时）
   */
  function removeChildren(parentKey: string) {
    const parentIndex = flatNodes.value.findIndex(n => n.key === parentKey)
    if (parentIndex === -1) return

    const parentLevel = flatNodes.value[parentIndex].level

    let endIndex = parentIndex + 1
    while (endIndex < flatNodes.value.length) {
      if (flatNodes.value[endIndex].level <= parentLevel) {
        break
      }
      endIndex++
    }

    const removedNodes = flatNodes.value.slice(parentIndex + 1, endIndex)
    removedNodes.forEach(node => nodeMap.delete(node.key))

    const newNodes = [
      ...flatNodes.value.slice(0, parentIndex + 1),
      ...flatNodes.value.slice(endIndex),
    ]
    flatNodes.value = newNodes
  }

  /**
   * 选择节点
   */
  function selectNode(node: VirtualTreeNode) {
    selectedKey.value = node.key
    onSelect?.(node)
  }

  /**
   * 更新节点
   */
  function updateNode(key: string, updates: Partial<VirtualTreeNode>) {
    const node = nodeMap.get(key)
    if (node) {
      Object.assign(node, updates)
      flatNodes.value = [...flatNodes.value]
    }
  }

  /**
   * 刷新指定节点的子节点
   */
  async function refreshNode(nodeKey: string) {
    const node = nodeMap.get(nodeKey)
    if (!node) return

    // 移除现有子节点
    removeChildren(nodeKey)
    node.isLoaded = false
    node.childCount = 0

    // 重新加载
    if (node.isExpanded) {
      node.isLoading = true
      loadingKeys.add(nodeKey)
      try {
        const children = await onLoadChildren(node)
        if (children && children.length > 0) {
          insertChildren(nodeKey, children)
          node.childCount = children.length
          node.isLoaded = true
        }
      } catch (error) {
        console.error('刷新节点失败:', error)
      } finally {
        node.isLoading = false
        loadingKeys.delete(nodeKey)
      }
    }

    flatNodes.value = [...flatNodes.value]
  }

  /**
   * 清除指定连接的节点
   */
  function clearConnection(connectionId: string) {
    const keysToRemove: string[] = []
    nodeMap.forEach((node, key) => {
      const parts = NodeKeyEncoder.decode(key)
      if (parts.length > 1 && parts[1] === connectionId) {
        keysToRemove.push(key)
      }
    })

    keysToRemove.forEach(key => nodeMap.delete(key))
    flatNodes.value = flatNodes.value.filter(n => !keysToRemove.includes(n.key))
    rootKeys.value = rootKeys.value.filter(k => !keysToRemove.includes(k))
  }

  /**
   * 清除所有节点
   */
  function clearAll() {
    nodeMap.clear()
    rootKeys.value = []
    flatNodes.value = []
    selectedKey.value = null
    loadingKeys.clear()
  }

  return {
    flatNodes,
    rootKeys,
    selectedKey,
    selectedNode,
    addRootNode,
    setRootNodes,
    toggleNode,
    selectNode,
    updateNode,
    refreshNode,
    clearConnection,
    clearAll,
  }
}
