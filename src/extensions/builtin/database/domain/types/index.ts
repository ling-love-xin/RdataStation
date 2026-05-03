/**
 * 数据库导航器类型定义
 */

import type { NavigatorNode, NodeProperties } from '@/shared/types/databaseMeta'

/** 数据库连接信息 */
export interface DatabaseConnection {
  id: string
  name: string
  dbType: string
  host?: string
  port?: number
  filePath?: string
  databases: string[]
}

/** 表结构信息 */
export interface TableStructure {
  columns: ColumnInfo[]
  indexes: IndexInfo[]
  foreignKeys: ForeignKeyInfo[]
  triggers: TriggerInfo[]
}

/** 列信息 */
export interface ColumnInfo {
  name: string
  type: string
  nullable: boolean
  default?: string | null
  comment?: string
}

/** 索引信息 */
export interface IndexInfo {
  name: string
  type: string
  columns: string[]
}

/** 外键信息 */
export interface ForeignKeyInfo {
  name: string
  column: string
  refTable: string
  refColumn: string
}

/** 触发器信息 */
export interface TriggerInfo {
  name: string
  timing: string
  event: string
  definition: string
}

/** 数据库结构 */
export interface DatabaseStructure {
  tables: TableInfo[]
  views: ViewInfo[]
  procedures: ProcedureInfo[]
  functions: FunctionInfo[]
}

/** 表信息 */
export interface TableInfo {
  name: string
  comment?: string
  rowCount?: number
  engine?: string
}

/** 视图信息 */
export interface ViewInfo {
  name: string
  comment?: string
  definer?: string
  isUpdatable?: string
}

/** 存储过程信息 */
export interface ProcedureInfo {
  name: string
  comment?: string
}

/** 函数信息 */
export interface FunctionInfo {
  name: string
  comment?: string
}

/** 节点加载器接口 */
export interface NodeLoader {
  loadChildren(node: NavigatorNode): Promise<NavigatorNode[]>
  getNodeProperties(node: NavigatorNode): NodeProperties
}

/** 查询上下文 */
export interface QueryContext {
  connectionId: string
  database?: string
  schema?: string
  table?: string
}
