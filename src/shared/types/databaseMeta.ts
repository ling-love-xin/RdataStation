/**
 * 数据库元数据类型定义
 */

// ============================================================================
// 数据库元数据配置
// ============================================================================

export interface DatabaseMetaConfig {
  // 数据库类型
  dbType: string

  // 显示名称
  displayName?: string

  // 名称
  name?: string

  // 版本
  version?: string

  // 默认端口
  defaultPort?: number

  // URL 模板
  urlTemplate?: string

  // 支持的节点类型
  supportedNodeTypes: string[]

  // 节点类型定义
  nodeTypes?: NodeTypeConfig[]

  // 属性配置
  propertiesConfig: PropertiesConfig

  // 标签页配置
  tabsConfig: TabConfig[]

  // 标签分组配置
  tabGroups?: TabGroupConfig[]

  // 层级结构
  hierarchy?: Array<{ level: number; type: string }>

  // ID (用于标识)
  id?: string

  // 特性支持
  features?: Record<string, boolean>

  // 默认查询
  defaultQueries?: Record<string, string>

  // 图标配置
  icons?: Record<string, string>
}

export interface NodeTypeConfig {
  id: string
  label: string
  icon: string
  isContainer?: boolean
  children?: string[]
  showCount?: boolean
  parentTypes?: string[]
  actions?: NodeAction[] | string[]
  query?: string
  metadata?: string[]
}

export interface NodeAction {
  id: string
  label: string
  icon: string
  command?: string
  type?: string
}

export interface PropertiesConfig {
  generalFields?: PropertyField[]
  table?: NodePropertiesConfig
  view?: NodePropertiesConfig
  column?: NodePropertiesConfig
  index?: NodePropertiesConfig
  [key: string]: NodePropertiesConfig | PropertyField[] | undefined
}

export interface NodePropertiesConfig {
  tabs: TabConfig[]
  generalFields: PropertyField[]
}

export interface PropertyField {
  key: string
  label: string
  type: 'text' | 'number' | 'boolean' | 'datetime' | 'size'
  format?: string
  editable?: boolean
}

export interface TabConfig {
  id: string
  label: string
  icon?: string
  default?: boolean
  filter?: (nodeType: string) => boolean
}

export interface TabGroupConfig {
  id: string
  label: string
  icon: string
  filter: (nodeType: string) => boolean
  order: number
  expanded: boolean
}

// ============================================================================
// 导航器节点类型
// ============================================================================

export interface NavigatorNode {
  id: string
  name: string
  type:
    | 'connection'
    | 'database'
    | 'schema'
    | 'table'
    | 'view'
    | 'column'
    | 'index'
    | 'folder'
    | string
  connectionId?: string
  database?: string
  schema?: string
  parentId?: string | null
  children?: NavigatorNode[]
  isLoading?: boolean
  loading?: boolean
  isExpanded?: boolean
  expanded?: boolean
  state?: 'idle' | 'loading' | 'error' | 'loaded' | 'empty'
  meta?: Record<string, unknown>
  metadata?: Record<string, unknown>
  properties?: NodeProperties
}

export interface NodeProperties {
  schema?: string
  catalog?: string
  type?: string
  nullable?: boolean
  dataType?: string
  columnSize?: number
  decimalDigits?: number
  ordinalPosition?: number
  isPrimaryKey?: boolean
  isForeignKey?: boolean
  isUnique?: boolean
  isIndexed?: boolean
  defaultValue?: string
  autoIncrement?: boolean
  charset?: string
  collation?: string
  comment?: string
  engine?: string
  rowCount?: number
  dataSize?: number
  indexSize?: number
  createdAt?: string
  updatedAt?: string
  general?: Record<string, unknown>
  columns?: Record<string, unknown>[]
  indexes?: Record<string, unknown>[]
  ddl?: string
}

// ============================================================================
// 查询上下文
// ============================================================================

export interface QueryContext {
  connectionId: string
  database?: string
  schema?: string
  table?: string
}

// ============================================================================
// 数据库适配器类型
// ============================================================================

export interface DatabaseMetaAdapter {
  readonly driverId: string
  readonly driverName: string
  readonly features: DatabaseFeature[]

  // 元数据查询方法
  getDatabases(connectionId: string): Promise<DatabaseInfo[]>
  getSchemas(connectionId: string, database?: string): Promise<SchemaInfo[]>
  getTables(connectionId: string, database?: string, schema?: string): Promise<TableInfo[]>
  getViews(connectionId: string, database?: string, schema?: string): Promise<ViewInfo[]>
  getColumns(
    connectionId: string,
    database: string,
    schema: string,
    table: string
  ): Promise<ColumnInfo[]>
  getIndexes(
    connectionId: string,
    database: string,
    schema: string,
    table: string
  ): Promise<IndexInfo[]>
  getForeignKeys(
    connectionId: string,
    database: string,
    schema: string,
    table: string
  ): Promise<ForeignKeyInfo[]>

  // 构建导航器节点
  buildConnectionNode(connectionId: string, name: string): NavigatorNode
  buildDatabaseNode(connectionId: string, database: DatabaseInfo): NavigatorNode
  buildSchemaNode(connectionId: string, database: string, schema: SchemaInfo): NavigatorNode
  buildTableNode(
    connectionId: string,
    database: string,
    schema: string,
    table: TableInfo
  ): NavigatorNode
  buildViewNode(
    connectionId: string,
    database: string,
    schema: string,
    view: ViewInfo
  ): NavigatorNode
  buildColumnNode(
    connectionId: string,
    database: string,
    schema: string,
    table: string,
    column: ColumnInfo
  ): NavigatorNode

  // 导航器加载方法
  getChildrenQuery(nodeType: string, context: QueryContext): string | null
  parseChildrenResult(nodeType: string, rows: unknown[], context: QueryContext): NavigatorNode[]

  // 获取配置
  getConfig(): DatabaseMetaConfig
}

export type DatabaseFeature =
  | 'schemas'
  | 'tables'
  | 'views'
  | 'procedures'
  | 'functions'
  | 'triggers'
  | 'indexes'
  | 'foreignKeys'
  | 'ssl'
  | 'sshTunnel'
  | 'httpProxy'

// ============================================================================
// 数据库对象信息
// ============================================================================

export interface DatabaseInfo {
  name: string
  charset?: string
  collation?: string
}

export interface SchemaInfo {
  name: string
  catalog?: string
  owner?: string
}

export interface TableInfo {
  name: string
  schema?: string
  catalog?: string
  type: 'table' | 'system_table' | 'temporary'
  engine?: string
  charset?: string
  collation?: string
  rowCount?: number
  dataSize?: number
  indexSize?: number
  comment?: string
  createdAt?: string
  updatedAt?: string
}

export interface ViewInfo {
  name: string
  schema?: string
  catalog?: string
  definition?: string
  isUpdatable?: boolean
  comment?: string
}

export interface ColumnInfo {
  name: string
  type: string
  nullable: boolean
  defaultValue?: string
  isPrimaryKey?: boolean
  isForeignKey?: boolean
  isAutoIncrement?: boolean
  isUnique?: boolean
  charset?: string
  collation?: string
  comment?: string
  ordinalPosition?: number
  columnSize?: number
  decimalDigits?: number
}

export interface IndexInfo {
  name: string
  tableName: string
  schema?: string
  isUnique: boolean
  isPrimary: boolean
  type?: string
  columns: IndexColumnInfo[]
}

export interface IndexColumnInfo {
  name: string
  order: 'asc' | 'desc'
  ordinalPosition: number
}

export interface ForeignKeyInfo {
  name: string
  tableName: string
  schema?: string
  columnName: string
  referencedTableName: string
  referencedSchema?: string
  referencedColumnName: string
  updateRule?: string
  deleteRule?: string
}

// ============================================================================
// 连接配置
// ============================================================================

export interface ConnectionConfig {
  id?: string
  name: string
  driver: string
  host: string
  port?: number
  database?: string
  username?: string
  password?: string
  properties?: Record<string, unknown>
}

// ============================================================================
// 驱动配置
// ============================================================================

export interface DriverConfig {
  id: string
  name: string
  icon: string
  features: DatabaseFeature[]
  defaultPort?: number
  fields: DriverField[]
}

export interface DriverField {
  name: string
  key?: string
  label: string
  type: 'text' | 'number' | 'password' | 'file' | 'select' | 'checkbox'
  field_type?: 'text' | 'number' | 'password' | 'file' | 'select' | 'checkbox'
  required?: boolean
  default?: unknown
  placeholder?: string
  options?: { label: string; value: unknown }[]
  option_type?: string
  optionType?: string
  description?: string
}
