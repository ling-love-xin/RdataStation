/**
 * Dockview API 类型定义 (v6.0+)
 *
 * 从 dockview-vue 和 dockview-core 重新导出类型
 * 提供完整的类型安全支持
 */

// 直接从 dockview-vue 重新导出所有核心类型
export * from 'dockview-vue'

import type {
  IDockviewApi,
  IGroupPanelViewApi,
  IGroupviewPanelApi,
} from 'dockview-vue'

export type DockviewApi = IDockviewApi
export type DockviewPanelInstance = IGroupviewPanelApi
export type DockviewGroupInstance = IGroupPanelViewApi
