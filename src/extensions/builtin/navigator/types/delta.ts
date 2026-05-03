/**
 * 增量变更类型定义
 */

export type DeltaType = 'ADD' | 'REMOVE' | 'UPDATE' | 'MOVE' | 'REORDER'

export interface AddDelta<T> {
  type: 'ADD'
  item: T
  position: number
  parentId: string | null
}

export interface RemoveDelta {
  type: 'REMOVE'
  id: string
  position: number
  parentId: string | null
}

export interface UpdateDelta<T> {
  type: 'UPDATE'
  id: string
  changes: Partial<T>
  oldValues?: Partial<T>
}

export interface MoveDelta {
  type: 'MOVE'
  id: string
  from: number
  to: number
  oldParentId: string
  newParentId: string
}

export interface ReorderDelta {
  type: 'REORDER'
  parentId: string
  newOrder: string[]
}

export type Delta<T> =
  | AddDelta<T>
  | RemoveDelta
  | UpdateDelta<T>
  | MoveDelta
  | ReorderDelta

/**
 * 获取增量的 key
 */
export function getDeltaKey<T>(delta: Delta<T>): string {
  switch (delta.type) {
    case 'ADD':
      return `ADD:${delta.parentId}:${(delta.item as any).id ?? Date.now()}`
    case 'REMOVE':
      return `REMOVE:${delta.id}`
    case 'UPDATE':
      return `UPDATE:${delta.id}`
    case 'MOVE':
      return `MOVE:${delta.id}`
    case 'REORDER':
      return `REORDER:${delta.parentId}`
    default:
      return `UNKNOWN:${Date.now()}`
  }
}

/**
 * 检查是否为某种类型的增量
 */
export function isAddDelta<T>(delta: Delta<T>): delta is AddDelta<T> {
  return delta.type === 'ADD'
}

export function isRemoveDelta<T>(delta: Delta<T>): delta is RemoveDelta {
  return delta.type === 'REMOVE'
}

export function isUpdateDelta<T>(delta: Delta<T>): delta is UpdateDelta<T> {
  return delta.type === 'UPDATE'
}

export function isMoveDelta<T>(delta: Delta<T>): delta is MoveDelta {
  return delta.type === 'MOVE'
}

export function isReorderDelta<T>(delta: Delta<T>): delta is ReorderDelta {
  return delta.type === 'REORDER'
}
