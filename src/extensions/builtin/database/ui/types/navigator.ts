export interface GlobalConnection {
  id: string
  name: string
  driver: string
  host: string | null
  port: number | null
  database: string | null
  tags: string[]
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface FilterConfig {
  showTables: boolean
  showViews: boolean
  showSystemSchemas: boolean
  showColumns: boolean
}

export interface DatabaseInfo {
  name: string
  schemas: SchemaInfo[]
}

export interface IndexInfo {
  name: string
  columns: string[]
  isUnique: boolean
  isPrimary: boolean
  type: 'btree' | 'hash' | 'gist' | 'spatial' | 'other'
}

export interface TriggerInfo {
  name: string
  event: 'INSERT' | 'UPDATE' | 'DELETE' | 'TRUNCATE'
  timing: 'BEFORE' | 'AFTER' | 'INSTEAD OF'
  function: string
  enabled: boolean
}

export interface ConstraintInfo {
  name: string
  type: 'PRIMARY KEY' | 'FOREIGN KEY' | 'UNIQUE' | 'CHECK' | 'NOT NULL'
  columns: string[]
  referencedTable?: string
  referencedColumns?: string[]
}

export interface ProcedureInfo {
  name: string
  language: string
  returnType: string
  parameters: Array<{ name: string; type: string; mode: 'IN' | 'OUT' | 'INOUT' }>
  definition: string
}

export interface FunctionInfo {
  name: string
  language: string
  returnType: string
  parameters: Array<{ name: string; type: string }>
  isAggregate: boolean
  definition: string
}

export interface SequenceInfo {
  name: string
  currentValue: number
  minValue: number
  maxValue: number
  increment: number
  cacheSize: number
  isCycled: boolean
}

export interface SchemaInfo {
  name: string
  tables: TableInfo[]
  views: ViewInfo[]
  indexes: IndexInfo[]
  triggers: TriggerInfo[]
  procedures: ProcedureInfo[]
  functions: FunctionInfo[]
  sequences: SequenceInfo[]
}

export interface TableInfo {
  name: string
  type: string
  columns: ColumnInfo[]
}

export interface ViewInfo {
  name: string
  type: string
  columns: ColumnInfo[]
}

export interface ColumnInfo {
  name: string
  dataType: string
  nullable?: boolean
  defaultValue?: string
  isPrimaryKey?: boolean
}

export interface ConnectionInfo {
  id: string
  name: string
  driver: string
  tags?: string[]
}

export interface SearchObjectResult {
  connectionId: string
  databaseName: string
  schemaName: string
  objectName: string
  objectType: 'table' | 'view' | 'column'
}

export interface NavigatorState {
  expanded_keys: string[]
  selected_keys: string[]
  filter_config: FilterConfig | null
}

export interface ContextMenuNodeData {
  nodeKey: string
  keyParts: string[]
  connectionId: string
  dbName: string
  schemaName: string
  tableName: string
}

export const NODE_TYPE_MAP: Record<string, string> = {
  conn: 'connection',
  db: 'database',
  schema: 'schema',
  table: 'table',
  view: 'view',
  col: 'column',
  tables: 'folder',
  views: 'folder'
}

export const NODE_TYPE_ICONS: Record<string, string> = {
  connection: 'Database',
  database: 'Database',
  schema: 'Folder',
  table: 'Table',
  view: 'FileText',
  column: 'Columns',
  folder: 'FolderOpen'
}

export const NODE_TYPE_COLORS: Record<string, string> = {
  connection: '#4f46e5',
  database: '#3b82f6',
  schema: '#10b981',
  table: '#3b82f6',
  view: '#8b5cf6',
  column: '#6b7280',
  folder: '#f59e0b'
}
