/**
 * 导航栏类型定义
 */

export type NodeType =
  | 'project'
  | 'connection'
  | 'database'
  | 'schema'
  | 'table'
  | 'view'
  | 'procedure'
  | 'function'
  | 'column'
  | 'column-folder'
  | 'index'
  | 'trigger'
  | 'folder'
  | string

export interface NavigatorNode {
  id: string
  type: NodeType
  name: string
  parentId: string | null
  path: string
  depth: number
  isLeaf?: boolean
  children?: NavigatorNode[]
  childrenCount?: number
  state?: NodeState
  metadata?: NodeMetadata
  derived?: NodeDerived
}

export interface NodeState {
  expanded: boolean
  selected: boolean
  loading: boolean
  error: Error | null
  highlighted: boolean
  visible: boolean
}

export interface NodeMetadata {
  rowCount?: number
  size?: string
  engine?: string
  charset?: string
  createdAt?: string
  updatedAt?: string
  comment?: string
  dataType?: string
  nullable?: boolean
  defaultValue?: string
  isPrimaryKey?: boolean
  isForeignKey?: boolean
  isIndexed?: boolean
  isUnique?: boolean
  childCount?: number
  [key: string]: unknown
}

export interface NodeDerived {
  fullPath: string
  displayName: string
  iconType: string
  badgeCount: number
  hasChildren: boolean
  isLeaf: boolean
}

export interface ConnectionInfo {
  id: string
  name: string
  type: string
  host?: string
  port?: number
  database?: string
  username?: string
  status: 'connected' | 'disconnected' | 'connecting' | 'error'
  latency?: number
  lastConnectedAt?: string
}

export interface SearchOptions {
  query: string
  types?: NodeType[]
  connectionId?: string
  database?: string
  schema?: string
  fuzzy?: boolean
  limit?: number
}

export interface SearchResult {
  node: NavigatorNode
  score: number
  highlights?: string[]
  path: string[]
}

export interface ViewConfig<T> {
  name: string
  source?: DataSource<T>
  transform?: ViewTransform<T, unknown>
  indexBy?: keyof T | ((item: T) => string)
  initialData?: T[]
}

export interface DataSource<T> {
  subscribe(callback: (data: T[]) => void): () => void
  fetch?(): Promise<T[]>
}

export interface ViewTransform<T, R> {
  name: string
  transform(snapshot: T[]): R[]
  transformDelta?(delta: import('./delta').Delta<T>, currentSnapshot: R[]): import('./delta').Delta<R>
}

export interface MaterializedView<T> {
  readonly name: string
  readonly version: number
  readonly snapshot: readonly T[]
  readonly size: number

  getById(id: string): T | undefined
  getByIndex(index: number): T | undefined
  find(predicate: (item: T) => boolean): T | undefined
  filter(predicate: (item: T) => boolean): T[]
  map<R>(mapper: (item: T) => R): R[]
  reduce<R>(reducer: (acc: R, item: T) => R, initial: R): R

  onChange(callback: (delta: import('./delta').Delta<T>) => void): () => void
}
