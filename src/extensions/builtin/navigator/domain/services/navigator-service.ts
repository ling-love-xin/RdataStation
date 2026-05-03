/**
 * 导航栏服务
 * 处理数据获取、缓存和增量同步
 */

import { mockDataService } from './mock-data-service'
import { l1Cache, l2Cache } from '../cache'
import { viewEngine } from '../engine/view-engine'

import type { NavigatorNode, ConnectionInfo } from '../../types'

export class NavigatorService {
  private useMock = true // 开发阶段使用 Mock 数据

  /**
   * 获取连接列表
   */
  async getConnections(): Promise<ConnectionInfo[]> {
    if (this.useMock) {
      return mockDataService.getConnections()
    }

    // TODO: 从后端 API 获取
    return []
  }

  /**
   * 加载连接的数据库列表
   */
  async loadConnectionDatabases(connectionId: string): Promise<void> {
    const cacheKey = `navigator:${connectionId}:databases`

    // 尝试从缓存加载
    const cached = l1Cache.get(cacheKey) as NavigatorNode[] | undefined
    if (cached) {
      this.updateView(connectionId, cached)
      return
    }

    // 从服务加载
    const databases = this.useMock
      ? await mockDataService.getRootNodes(connectionId)
      : await this.fetchFromAPI(connectionId)

    // 更新缓存
    l1Cache.set(cacheKey, databases)
    await l2Cache.set(cacheKey, databases)

    // 更新视图
    this.updateView(connectionId, databases)
  }

  /**
   * 加载子节点
   */
  async loadChildren(nodeId: string, connectionId: string): Promise<NavigatorNode[]> {
    const cacheKey = `navigator:children:${nodeId}`

    // 尝试从缓存加载
    const cached = l1Cache.get(cacheKey) as NavigatorNode[] | undefined
    if (cached) {
      return cached
    }

    // 从服务加载
    const children = this.useMock
      ? await mockDataService.getChildren(nodeId)
      : await this.fetchChildrenFromAPI(nodeId)

    // 更新缓存
    l1Cache.set(cacheKey, children)
    await l2Cache.set(cacheKey, children)

    return children
  }

  /**
   * 展开节点
   */
  async expandNode(nodeId: string, connectionId: string): Promise<void> {
    const children = await this.loadChildren(nodeId, connectionId)

    // 应用增量到视图
    children.forEach((child, index) => {
      viewEngine.applyDelta(`navigator:${connectionId}`, {
        type: 'ADD',
        item: child,
        position: index,
        parentId: nodeId
      })
    })

    // 保存展开状态
    await this.saveExpandedState(connectionId, nodeId, true)
  }

  /**
   * 折叠节点
   */
  async collapseNode(nodeId: string, connectionId: string): Promise<void> {
    // 获取该节点的所有子节点
    const children = await this.getChildrenFromView(nodeId, connectionId)

    // 移除所有子节点
    children.forEach(child => {
      viewEngine.applyDelta(`navigator:${connectionId}`, {
        type: 'REMOVE',
        id: child.id,
        position: -1,
        parentId: nodeId
      })
    })

    // 保存展开状态
    await this.saveExpandedState(connectionId, nodeId, false)
  }

  /**
   * 搜索节点
   */
  async searchNodes(connectionId: string, query: string): Promise<NavigatorNode[]> {
    if (this.useMock) {
      return mockDataService.searchNodes(connectionId, query)
    }

    // TODO: 实现搜索 API
    return []
  }

  /**
   * 刷新节点
   */
  async refreshNode(nodeId: string, connectionId: string): Promise<void> {
    // 清除缓存
    l1Cache.delete(`navigator:children:${nodeId}`)
    await l2Cache.delete(`navigator:children:${nodeId}`)

    // 如果是展开状态，重新加载
    const isExpanded = await this.isNodeExpanded(connectionId, nodeId)
    if (isExpanded) {
      // 先移除现有子节点
      const existingChildren = await this.getChildrenFromView(nodeId, connectionId)
      existingChildren.forEach(child => {
        viewEngine.applyDelta(`navigator:${connectionId}`, {
          type: 'REMOVE',
          id: child.id,
          position: -1,
          parentId: nodeId
        })
      })

      // 重新加载
      await this.expandNode(nodeId, connectionId)
    }
  }

  /**
   * 获取展开状态
   */
  async getExpandedNodes(connectionId: string): Promise<string[]> {
    const state = await l2Cache.get<string[]>(`navigator:expanded:${connectionId}`)
    return state || []
  }

  /**
   * 保存展开状态
   */
  private async saveExpandedState(connectionId: string, nodeId: string, expanded: boolean): Promise<void> {
    const key = `navigator:expanded:${connectionId}`
    const current = await l2Cache.get<string[]>(key) || []

    if (expanded) {
      if (!current.includes(nodeId)) {
        current.push(nodeId)
      }
    } else {
      const index = current.indexOf(nodeId)
      if (index > -1) {
        current.splice(index, 1)
      }
    }

    await l2Cache.set(key, current)
  }

  /**
   * 检查节点是否展开
   */
  private async isNodeExpanded(connectionId: string, nodeId: string): Promise<boolean> {
    const expanded = await this.getExpandedNodes(connectionId)
    return expanded.includes(nodeId)
  }

  /**
   * 从视图获取子节点
   */
  private getChildrenFromView(nodeId: string, connectionId: string): NavigatorNode[] {
    const view = viewEngine.getView<NavigatorNode>(`navigator:${connectionId}`)
    if (!view) return []

    return view.snapshot.filter(node => node.parentId === nodeId)
  }

  /**
   * 更新视图
   */
  private updateView(connectionId: string, nodes: NavigatorNode[]): void {
    const viewName = `navigator:${connectionId}`

    // 检查视图是否存在
    let view = viewEngine.getView<NavigatorNode>(viewName)
    if (!view) {
      // 创建新视图
      view = viewEngine.createView({
        name: viewName,
        initialData: []
      })
    }

    // 清空现有数据
    view.snapshot.forEach((node, index) => {
      viewEngine.applyDelta(viewName, {
        type: 'REMOVE',
        id: node.id,
        position: index,
        parentId: null
      })
    })

    // 添加新数据
    nodes.forEach((node, index) => {
      viewEngine.applyDelta(viewName, {
        type: 'ADD',
        item: node,
        position: index,
        parentId: null
      })
    })
  }

  /**
   * 从 API 获取数据
   */
  private async fetchFromAPI(connectionId: string): Promise<NavigatorNode[]> {
    // TODO: 实现真实的 API 调用
    return []
  }

  /**
   * 从 API 获取子节点
   */
  private async fetchChildrenFromAPI(nodeId: string): Promise<NavigatorNode[]> {
    // TODO: 实现真实的 API 调用
    return []
  }
}

// 创建单例
export const navigatorService = new NavigatorService()
