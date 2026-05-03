/**
 * 增量处理器
 * 负责计算差异、合并增量、应用变更
 */

import type { Delta, AddDelta, RemoveDelta, UpdateDelta } from '../../types'

export class DeltaProcessor {
  /**
   * 计算两个快照之间的差异
   */
  computeDiff<T>(
    oldSnapshot: T[],
    newSnapshot: T[],
    keyExtractor: (item: T) => string
  ): Delta<T>[] {
    const oldMap = new Map(oldSnapshot.map(item => [keyExtractor(item), item]))
    const newMap = new Map(newSnapshot.map(item => [keyExtractor(item), item]))

    const deltas: Delta<T>[] = []
    const processed = new Set<string>()

    // 检测新增和更新
    for (const [key, newItem] of newMap) {
      const oldItem = oldMap.get(key)

      if (!oldItem) {
        // 新增
        deltas.push({
          type: 'ADD',
          item: newItem,
          position: newSnapshot.indexOf(newItem),
          parentId: this.extractParentId(newItem)
        })
      } else if (!this.isEqual(oldItem, newItem)) {
        // 更新
        deltas.push({
          type: 'UPDATE',
          id: key,
          changes: this.extractChanges(oldItem, newItem),
          oldValues: this.extractChanges(newItem, oldItem)
        })
      }

      processed.add(key)
    }

    // 检测删除
    for (const [key, oldItem] of oldMap) {
      if (!processed.has(key)) {
        deltas.push({
          type: 'REMOVE',
          id: key,
          position: oldSnapshot.indexOf(oldItem),
          parentId: this.extractParentId(oldItem)
        })
      }
    }

    // 检测移动
    const moveDeltas = this.detectMoves(oldSnapshot, newSnapshot, keyExtractor)
    deltas.push(...moveDeltas)

    return this.sortDeltas(deltas)
  }

  /**
   * 合并多个增量
   */
  mergeDeltas<T>(deltas: Delta<T>[]): Delta<T>[] {
    const merged = new Map<string, Delta<T>>()

    for (const delta of deltas) {
      const key = this.getDeltaKey(delta)
      const existing = merged.get(key)

      if (existing) {
        merged.set(key, this.mergeTwoDeltas(existing, delta))
      } else {
        merged.set(key, delta)
      }
    }

    return Array.from(merged.values())
  }

  /**
   * 压缩增量
   */
  compressDeltas<T>(deltas: Delta<T>[]): Delta<T>[] {
    // 移除冗余的更新
    const compressed: Delta<T>[] = []
    const updates = new Map<string, UpdateDelta<T>>()

    for (const delta of deltas) {
      if (delta.type === 'UPDATE') {
        const existing = updates.get(delta.id)
        if (existing) {
          // 合并变更
          existing.changes = { ...existing.changes, ...delta.changes }
        } else {
          updates.set(delta.id, delta)
          compressed.push(delta)
        }
      } else {
        compressed.push(delta)
      }
    }

    return compressed
  }

  /**
   * 应用增量到快照
   */
  applyToSnapshot<T>(snapshot: T[], deltas: Delta<T>[]): T[] {
    const result = [...snapshot]
    const indexMap = new Map(snapshot.map((item, idx) => [this.extractId(item), idx]))

    // 按类型分组处理
    const removes = deltas.filter((d): d is RemoveDelta => d.type === 'REMOVE')
    const adds = deltas.filter((d): d is AddDelta<T> => d.type === 'ADD')
    const updates = deltas.filter((d): d is UpdateDelta<T> => d.type === 'UPDATE')
    const moves = deltas.filter(d => d.type === 'MOVE')

    // 1. 先处理删除（从后向前）
    removes
      .sort((a, b) => b.position - a.position)
      .forEach(delta => {
        if (delta.position >= 0 && delta.position < result.length) {
          result.splice(delta.position, 1)
        }
      })

    // 2. 处理更新
    updates.forEach(delta => {
      const idx = indexMap.get(delta.id)
      if (idx !== undefined && idx >= 0 && idx < result.length) {
        result[idx] = { ...result[idx], ...delta.changes }
      }
    })

    // 3. 处理新增
    adds.forEach(delta => {
      if (delta.position >= 0) {
        result.splice(delta.position, 0, delta.item)
      } else {
        result.push(delta.item)
      }
    })

    // 4. 处理移动
    moves.forEach(delta => {
      if ('from' in delta && 'to' in delta) {
        const from = delta.from as number
        const to = delta.to as number
        if (from >= 0 && from < result.length) {
          const [item] = result.splice(from, 1)
          result.splice(to, 0, item)
        }
      }
    })

    return result
  }

  // 辅助方法
  private isEqual<T>(a: T, b: T): boolean {
    return JSON.stringify(a) === JSON.stringify(b)
  }

  private extractChanges<T>(oldItem: T, newItem: T): Partial<T> {
    const changes: Partial<T> = {}
    for (const key in newItem) {
      if ((oldItem as any)[key] !== (newItem as any)[key]) {
        (changes as any)[key] = (newItem as any)[key]
      }
    }
    return changes
  }

  private extractId<T>(item: T): string {
    return (item as any).id || String(item)
  }

  private extractParentId<T>(item: T): string | null {
    return (item as any).parentId || null
  }

  private getDeltaKey<T>(delta: Delta<T>): string {
    switch (delta.type) {
      case 'ADD':
        return `ADD:${delta.parentId}:${(delta.item as any).id ?? Date.now()}`
      case 'REMOVE':
        return `REMOVE:${delta.id}`
      case 'UPDATE':
        return `UPDATE:${delta.id}`
      case 'MOVE':
        return `MOVE:${(delta as any).id}`
      case 'REORDER':
        return `REORDER:${(delta as any).parentId}`
      default:
        return `UNKNOWN:${Date.now()}`
    }
  }

  private mergeTwoDeltas<T>(a: Delta<T>, b: Delta<T>): Delta<T> {
    // 简化处理：返回最新的增量
    return b
  }

  private detectMoves<T>(
    oldSnapshot: T[],
    newSnapshot: T[],
    keyExtractor: (item: T) => string
  ): Delta<T>[] {
    // 简化实现：不检测移动，只检测增删改
    return []
  }

  private sortDeltas<T>(deltas: Delta<T>[]): Delta<T>[] {
    const order: Record<string, number> = { REMOVE: 0, MOVE: 1, UPDATE: 2, ADD: 3, REORDER: 4 }
    return deltas.sort((a, b) => (order[a.type] ?? 5) - (order[b.type] ?? 5))
  }
}

// 创建单例
export const deltaProcessor = new DeltaProcessor()
