/**
 * 视图引擎
 * 管理物化视图的创建、更新和订阅
 */

import { reactive } from 'vue'

import { DeltaProcessor } from './delta-processor'

import type { Delta, MaterializedView, ViewConfig, DataSource } from '../../types'

export class ViewEngine {
  private views = new Map<string, MaterializedView<unknown>>()
  private dependencies = new Map<string, Set<string>>()
  private dependents = new Map<string, Set<string>>()
  private subscribers = new Map<string, Set<(delta: Delta<unknown>) => void>>()
  private deltaProcessor = new DeltaProcessor()

  /**
   * 创建物化视图
   */
  createView<T>(config: ViewConfig<T>): MaterializedView<T> {
    const snapshot = reactive(config.initialData || []) as T[]

    const view: MaterializedView<T> = {
      name: config.name,
      version: 0,
      snapshot,
      size: 0,

      getById: (id: string) => {
        return view.snapshot.find(item => this.extractId(item) === id)
      },

      getByIndex: (index: number) => {
        return view.snapshot[index]
      },

      find: (predicate: (item: T) => boolean) => {
        return view.snapshot.find(predicate)
      },

      filter: (predicate: (item: T) => boolean) => {
        return view.snapshot.filter(predicate)
      },

      map: <R>(mapper: (item: T) => R): R[] => {
        return view.snapshot.map(mapper)
      },

      reduce: <R>(reducer: (acc: R, item: T) => R, initial: R): R => {
        return view.snapshot.reduce(reducer, initial)
      },

      onChange: (callback: (delta: Delta<T>) => void) => {
        return this.subscribe(config.name, callback as (delta: Delta<unknown>) => void)
      }
    }

    this.views.set(config.name, view as MaterializedView<unknown>)

    // 如果有数据源，开始监听
    if (config.source) {
      this.bindDataSource(config.name, config.source)
    }

    return view
  }

  /**
   * 绑定数据源
   */
  private bindDataSource<T>(viewName: string, source: DataSource<T>): void {
    const unsubscribe = source.subscribe((data) => {
      const view = this.views.get(viewName) as MaterializedView<T> | undefined
      if (!view) return

      const deltas = this.deltaProcessor.computeDiff(
        [...view.snapshot],
        data,
        item => this.extractId(item)
      )

      if (deltas.length > 0) {
        this.applyBatch(viewName, deltas)
      }
    })

    // 保存取消订阅函数
    this.subscriptions.set(viewName, unsubscribe)
  }

  private subscriptions = new Map<string, () => void>()

  /**
   * 应用单个增量
   */
  applyDelta<T>(viewName: string, delta: Delta<T>): void {
    const view = this.views.get(viewName) as MaterializedView<T> | undefined
    if (!view) {
      console.warn(`View not found: ${viewName}`)
      return
    }

    // 应用到快照
    const newSnapshot = this.deltaProcessor.applyToSnapshot([...view.snapshot], [delta])
    const mutableSnapshot = view.snapshot as T[]
    mutableSnapshot.length = 0
    mutableSnapshot.push(...newSnapshot)
    ;(view as any).version++
    ;(view as any).size = mutableSnapshot.length

    // 通知订阅者
    this.notifySubscribers(viewName, delta as Delta<unknown>)

    // 传播到依赖视图
    this.propagateChange(viewName, delta as Delta<unknown>)
  }

  /**
   * 批量应用增量
   */
  applyBatch<T>(viewName: string, deltas: Delta<T>[]): void {
    if (deltas.length === 0) return

    const view = this.views.get(viewName) as MaterializedView<T> | undefined
    if (!view) {
      console.warn(`View not found: ${viewName}`)
      return
    }

    // 合并和压缩增量
    const merged = this.deltaProcessor.mergeDeltas(deltas)
    const compressed = this.deltaProcessor.compressDeltas(merged)

    // 批量应用
    const newSnapshot = this.deltaProcessor.applyToSnapshot([...view.snapshot], compressed)
    const mutableSnapshot = view.snapshot as T[]
    mutableSnapshot.length = 0
    mutableSnapshot.push(...newSnapshot)
    ;(view as any).version += compressed.length
    ;(view as any).size = mutableSnapshot.length

    // 通知订阅者
    compressed.forEach(delta => {
      this.notifySubscribers(viewName, delta as Delta<unknown>)
    })

    // 传播到依赖视图
    compressed.forEach(delta => {
      this.propagateChange(viewName, delta as Delta<unknown>)
    })
  }

  /**
   * 添加视图依赖
   */
  addDependency(view: string, dependsOn: string): void {
    if (!this.dependencies.has(view)) {
      this.dependencies.set(view, new Set())
    }
    this.dependencies.get(view)!.add(dependsOn)

    if (!this.dependents.has(dependsOn)) {
      this.dependents.set(dependsOn, new Set())
    }
    this.dependents.get(dependsOn)!.add(view)
  }

  /**
   * 移除视图依赖
   */
  removeDependency(view: string, dependsOn: string): void {
    this.dependencies.get(view)?.delete(dependsOn)
    this.dependents.get(dependsOn)?.delete(view)
  }

  /**
   * 订阅视图变更
   */
  subscribe(viewName: string, callback: (delta: Delta<unknown>) => void): () => void {
    if (!this.subscribers.has(viewName)) {
      this.subscribers.set(viewName, new Set())
    }

    this.subscribers.get(viewName)!.add(callback)

    return () => {
      this.subscribers.get(viewName)?.delete(callback)
    }
  }

  /**
   * 获取视图
   */
  getView<T>(name: string): MaterializedView<T> | undefined {
    return this.views.get(name) as MaterializedView<T> | undefined
  }

  /**
   * 删除视图
   */
  deleteView(name: string): void {
    // 取消数据源订阅
    this.subscriptions.get(name)?.()
    this.subscriptions.delete(name)

    // 删除视图
    this.views.delete(name)
    this.subscribers.delete(name)

    // 清理依赖关系
    const deps = this.dependencies.get(name)
    if (deps) {
      deps.forEach(dep => {
        this.dependents.get(dep)?.delete(name)
      })
      this.dependencies.delete(name)
    }

    const dents = this.dependents.get(name)
    if (dents) {
      dents.forEach(dent => {
        this.dependencies.get(dent)?.delete(name)
      })
      this.dependents.delete(name)
    }
  }

  /**
   * 刷新视图
   */
  async refreshView(viewName: string): Promise<void> {
    const view = this.views.get(viewName)
    if (!view) return

    // 触发刷新逻辑
    // 实际实现中可能需要重新获取数据
    console.log(`Refreshing view: ${viewName}`)
  }

  // 私有方法
  private notifySubscribers(viewName: string, delta: Delta<unknown>): void {
    const callbacks = this.subscribers.get(viewName)
    if (callbacks) {
      callbacks.forEach(callback => {
        try {
          callback(delta)
        } catch (error) {
          console.error('Error in view subscriber:', error)
        }
      })
    }
  }

  private propagateChange(sourceView: string, delta: Delta<unknown>): void {
    const dependentViews = this.dependents.get(sourceView)
    if (!dependentViews) return

    dependentViews.forEach(dependentView => {
      // 转换增量并应用
      const transformedDelta = this.transformDelta(delta, sourceView, dependentView)
      this.applyDelta(dependentView, transformedDelta)
    })
  }

  private transformDelta(
    delta: Delta<unknown>,
    sourceView: string,
    targetView: string
  ): Delta<unknown> {
    // 根据视图转换逻辑转换增量
    // 这里简化处理，实际应根据 transform 配置转换
    return delta
  }

  private extractId<T>(item: T): string {
    return (item as any).id || String(item)
  }
}

// 创建单例
export const viewEngine = new ViewEngine()
