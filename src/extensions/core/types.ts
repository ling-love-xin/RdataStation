/**
 * 扩展系统核心类型定义
 *
 * 定义扩展生命周期、上下文、API 等核心接口
 */

import type { EventBus } from './event-bus'

// ============================================================================
// 扩展基础类型
// ============================================================================

/** 扩展唯一标识 */
export type ExtensionId = string

/** 扩展版本 */
export type ExtensionVersion = string

/** 可释放资源 */
export interface Disposable {
  dispose(): void
}

// ============================================================================
// 扩展上下文
// ============================================================================

/** 项目信息 */
export interface ProjectInfo {
  id: string
  name: string
  path: string
  description?: string
  createdAt: string
  updatedAt: string
}

/** 命令注册表 */
export interface CommandRegistry {
  registerCommand(id: string, handler: (...args: unknown[]) => unknown): Disposable
  executeCommand(id: string, ...args: unknown[]): Promise<unknown>
}

/** 面板注册表 */
export interface PanelRegistry {
  register(panel: PanelDescriptor): Disposable
}

/** 面板描述符 */
export interface PanelDescriptor {
  id: string
  name: string
  component: unknown // Vue component
  location: 'left' | 'right' | 'bottom' | 'center'
  icon?: string
  order?: number
}

/** 窗口 API */
export interface WindowAPI {
  registerViewProvider(
    id: string,
    provider: {
      component: unknown
      title: string
      location: 'left' | 'right' | 'bottom' | 'center'
      icon?: string
      order?: number
    }
  ): Disposable
  showNotification(message: string, type?: 'info' | 'warning' | 'error'): void
}

/** 工作区 API */
export interface WorkspaceAPI {
  getWorkspacePath(): string | undefined
  openFile(path: string): Promise<void>
}

/** 数据库 API */
export interface DatabaseAPI {
  executeQuery(connId: string, sql: string): Promise<unknown>
  getConnection(connId: string): unknown
  registerConnectionProvider(provider: ConnectionProvider): Disposable
}

/** SQL 编辑器 API */
export interface SqlEditorAPI {
  openEditor(connId?: string): Promise<void>
  getCurrentEditor(): unknown
}

/** 配置 API */
export interface ConfigurationAPI {
  get<T>(key: string, defaultValue?: T): T
  set<T>(key: string, value: T): Promise<void>
}

/** 工具 API */
export interface UtilsAPI {
  formatDate(date: Date): string
  formatBytes(bytes: number): string
  debounce<T extends (...args: unknown[]) => unknown>(fn: T, delay: number): T
}

/** 数据库驱动贡献 */
export interface DatabaseDriverContribution {
  id: string
  name: string
  icon: string
  features: DatabaseFeature[]
  defaultPort?: number
  connectionSchema: ConnectionSchema
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

export interface ConnectionSchema {
  fields: ConnectionField[]
}

export interface ConnectionField {
  name: string
  label: string
  type: 'text' | 'number' | 'password' | 'file' | 'select' | 'checkbox'
  required?: boolean
  default?: unknown
  placeholder?: string
  options?: { label: string; value: unknown }[]
}

/** 连接提供者 */
export interface ConnectionProvider {
  readonly driverId: string
  connect(config: unknown): Promise<Connection>
}

/** 连接 */
export interface Connection {
  readonly id: string
  readonly state: 'connecting' | 'connected' | 'error' | 'closed'
  disconnect(): Promise<void>
  execute(sql: string): Promise<QueryResult>
}

/** 查询结果 */
export interface QueryResult {
  columns: string[]
  rows: unknown[][]
  rowCount: number
  executionTime: number
}

/** 扩展上下文 */
export interface ExtensionContext {
  /** 当前项目信息 */
  project: ProjectInfo

  /** 事件总线（插件间通信） */
  events: EventBus

  /** 命令注册表 */
  commands: CommandRegistry

  /** 面板注册表 */
  window: WindowAPI

  /** 工作区 API */
  workspace: WorkspaceAPI

  /** 数据库 API */
  database: DatabaseAPI

  /** SQL 编辑器 API */
  sqlEditor: SqlEditorAPI

  /** 配置 API */
  configuration: ConfigurationAPI

  /** 工具 API */
  utils: UtilsAPI

  /** 扩展存储路径 */
  extensionPath: string

  /** 订阅资源释放 */
  subscribe(disposable: Disposable): void
}

// ============================================================================
// 扩展 API
// ============================================================================

/** 扩展 API 基础接口 */
export interface ExtensionAPI {
  /** API 版本 */
  version: ExtensionVersion

  /** 当前项目 */
  project: ProjectInfo

  /** 命令注册表 */
  commands: CommandRegistry

  /** 窗口 API */
  window: WindowAPI

  /** 工作区 API */
  workspace: WorkspaceAPI

  /** 数据库 API */
  database: DatabaseAPI

  /** SQL 编辑器 API */
  sqlEditor: SqlEditorAPI

  /** 事件总线 */
  events: EventBus

  /** 配置 API */
  configuration: ConfigurationAPI

  /** 工具 API */
  utils: UtilsAPI

  /** 释放资源 */
  dispose?(): void
}

// ============================================================================
// 扩展模块
// ============================================================================

/** 扩展模块定义 */
export interface ExtensionModule {
  /** 扩展激活 */
  activate(context: ExtensionContext): ExtensionAPI | Promise<ExtensionAPI>

  /** 扩展停用 */
  deactivate?(): void | Promise<void>
}

/** 扩展元数据 */
export interface ExtensionMetadata {
  /** 扩展 ID */
  id: ExtensionId

  /** 扩展名称 */
  name: string

  /** 扩展版本 */
  version: ExtensionVersion

  /** 扩展描述 */
  description?: string

  /** 作者 */
  author?: string

  /** 依赖的其他扩展 */
  extensionDependencies?: ExtensionId[]

  /** 激活事件 */
  activationEvents?: string[]
}

// ============================================================================
// 扩展注册表
// ============================================================================

/** 扩展注册表 */
export interface ExtensionRegistry {
  /** 注册扩展 */
  register(id: ExtensionId, module: ExtensionModule, metadata: ExtensionMetadata): void

  /** 获取扩展 */
  get(id: ExtensionId): ExtensionModule | undefined

  /** 激活扩展 */
  activate(id: ExtensionId, context: ExtensionContext): Promise<ExtensionAPI>

  /** 停用扩展 */
  deactivate(id: ExtensionId): Promise<void>

  /** 获取所有已激活的扩展 */
  getActivatedExtensions(): Map<ExtensionId, ExtensionAPI>
}

// ============================================================================
// 预定义事件名称
// ============================================================================

/** 连接相关事件 */
export const ConnectionEvents = {
  /** 连接状态变化 */
  CHANGED: 'connection:changed',
  /** 连接创建 */
  CREATED: 'connection:created',
  /** 连接删除 */
  DELETED: 'connection:deleted',
  /** 连接测试完成 */
  TESTED: 'connection:tested',
} as const

/** 查询相关事件 */
export const QueryEvents = {
  /** 查询执行 */
  EXECUTED: 'query:executed',
  /** 查询取消 */
  CANCELLED: 'query:cancelled',
  /** 查询错误 */
  ERROR: 'query:error',
} as const

/** 项目相关事件 */
export const ProjectEvents = {
  /** 项目打开 */
  OPENED: 'project:opened',
  /** 项目关闭 */
  CLOSED: 'project:closed',
  /** 项目切换 */
  SWITCHED: 'project:switched',
} as const

/** 导航器相关事件 */
export const NavigatorEvents = {
  /** 刷新导航器 */
  REFRESH: 'navigator:refresh',
  /** 节点展开 */
  NODE_EXPANDED: 'navigator:nodeExpanded',
  /** 节点折叠 */
  NODE_COLLAPSED: 'navigator:nodeCollapsed',
} as const
