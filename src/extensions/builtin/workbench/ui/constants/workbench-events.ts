/**
 * Workbench 全局事件常量枚举
 * 所有标题栏、工具栏、命令面板派发的事件必须在此定义
 * 使用常量替代硬编码字符串，避免拼写错误
 */

export enum WorkbenchEvent {
  // 文件操作
  NewQuery = 'workbench:new-query',
  NewConnection = 'workbench:new-connection',
  OpenProject = 'workbench:open-project',
  Save = 'workbench:save',

  // 编辑操作
  Undo = 'workbench:undo',
  Redo = 'workbench:redo',
  Cut = 'workbench:cut',
  Copy = 'workbench:copy',
  Paste = 'workbench:paste',
  Find = 'workbench:find',
  Replace = 'workbench:replace',

  // 视图操作
  CommandPalette = 'workbench:command-palette',
  ToggleSidebar = 'workbench:toggle-sidebar',
  TogglePanel = 'workbench:toggle-panel',

  // 连接操作
  ManageConnections = 'workbench:manage-connections',
  Disconnect = 'workbench:disconnect',

  // 运行操作
  ExecuteSql = 'workbench:execute-sql',
  ExecuteScript = 'workbench:execute-script',
  StopExecution = 'workbench:stop-execution',

  // 工具操作
  PluginManagement = 'workbench:plugin-management',
  OpenSettings = 'workbench:open-settings',
  KeyboardShortcuts = 'workbench:keyboard-shortcuts',

  // 帮助操作
  OpenDocs = 'workbench:open-docs',
  CheckUpdates = 'workbench:check-updates',
  About = 'workbench:about',

  // 其他
  OpenHistory = 'workbench:open-history',
  OpenTerminal = 'workbench:open-terminal',
}

/**
 * 派发 Workbench 全局事件
 * @param event - 事件枚举值
 * @param detail - 可选的事件详情数据
 */
export function dispatchWorkbenchEvent(event: WorkbenchEvent, detail?: unknown): void {
  window.dispatchEvent(new CustomEvent(event, { detail }))
}

/**
 * 监听 Workbench 全局事件
 * @param event - 事件枚举值
 * @param handler - 事件处理函数
 * @returns 清理函数，用于移除监听
 */
export function listenWorkbenchEvent(
  event: WorkbenchEvent,
  handler: (e: CustomEvent) => void
): () => void {
  const wrappedHandler = (e: Event) => handler(e as CustomEvent)
  window.addEventListener(event, wrappedHandler)
  return () => window.removeEventListener(event, wrappedHandler)
}
