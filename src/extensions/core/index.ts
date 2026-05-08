/**
 * 扩展系统核心模块
 */

export { EventBus, eventBus } from './event-bus'
export type {
  Disposable,
  ExtensionId,
  ExtensionVersion,
  ProjectInfo,
  CommandRegistry,
  PanelRegistry,
  PanelDescriptor,
  WindowAPI,
  WorkspaceAPI,
  DatabaseAPI,
  SqlEditorAPI,
  ConfigurationAPI,
  UtilsAPI,
  ExtensionContext,
  ExtensionAPI,
  ExtensionModule,
  ExtensionMetadata,
  ExtensionRegistry,
} from './types'

export { ConnectionEvents, QueryEvents, ProjectEvents, NavigatorEvents } from './types'
