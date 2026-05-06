/**
 * Workbench Extension
 *
 * 提供工作台布局管理能力
 */

import EmptyWorkbenchPanel from './ui/components/panels/EmptyWorkbenchPanel.vue'
import OutputPanel from './ui/components/panels/OutputPanel.vue'
import PluginsPanel from './ui/components/panels/PluginsPanel.vue'
import SqlHistoryPanel from './ui/components/panels/SqlHistoryPanel.vue'

import type {
  ExtensionContext,
  ExtensionAPI,
  ExtensionModule,
  Disposable,
} from '../../core/types'

// Workbench 扩展特定的 API 接口
interface WorkbenchExtensionAPI extends ExtensionAPI {
  workbench: {
    openPanel(panelId: string, options?: { title?: string; component?: unknown }): void
    closePanel(panelId: string): void
    focusPanel(panelId: string): void
  }
}

interface PanelState {
  id: string
  title: string
  component?: unknown
  isActive: boolean
}

/**
 * 扩展激活函数
 */
const activate = (context: ExtensionContext): WorkbenchExtensionAPI => {
  console.log('[Workbench] Activating for project:', context.project.name)

  // TODO: 初始化 DDD 层
  // const infra = new WorkbenchInfrastructure(context)
  // const domain = new WorkbenchDomain(infra.repository)
  // const app = new WorkbenchApplication(domain, infra)
  // const ui = new WorkbenchUI(app, context)

  // 面板状态管理
  const panels = new Map<string, PanelState>()

  const openPanel = (panelId: string, options?: { title?: string; component?: unknown }): void => {
    panels.set(panelId, {
      id: panelId,
      title: options?.title || panelId,
      component: options?.component,
      isActive: true,
    })
    console.log(`[Workbench] Opened panel: ${panelId}`)
  }

  const closePanel = (panelId: string): void => {
    panels.delete(panelId)
    console.log(`[Workbench] Closed panel: ${panelId}`)
  }

  const focusPanel = (panelId: string): void => {
    const panel = panels.get(panelId)
    if (panel) {
      panel.isActive = true
      console.log(`[Workbench] Focused panel: ${panelId}`)
    }
  }

  // 注册空工作台面板（作为欢迎页，不注册到具体位置，由 WorkbenchView 动态控制）
  const emptyPanelDisposable = context.window.registerViewProvider('emptyWorkbench', {
    component: EmptyWorkbenchPanel,
    title: '欢迎',
    location: 'center',
    icon: 'Home',
    order: 0
  })

  // 注册SQL历史面板（右侧）
  const sqlHistoryDisposable = context.window.registerViewProvider('sqlHistory', {
    component: SqlHistoryPanel,
    title: 'SQL历史',
    location: 'right',
    icon: 'Clock',
    order: 2
  })

  // 注册输出面板（底部）
  const outputDisposable = context.window.registerViewProvider('output', {
    component: OutputPanel,
    title: '输出',
    location: 'bottom',
    icon: 'Terminal',
    order: 1
  })

  // 注册插件面板（左侧）
  const pluginsDisposable = context.window.registerViewProvider('plugins', {
    component: PluginsPanel,
    title: '插件',
    location: 'left',
    icon: 'Puzzle',
    order: 3
  })

  const disposables: Disposable[] = [
    emptyPanelDisposable,
    sqlHistoryDisposable,
    outputDisposable,
    pluginsDisposable,
    context.commands.registerCommand('workbench.openPanel', (...args: unknown[]) => openPanel(args[0] as string, args[1] as { title?: string; component?: unknown })),
    context.commands.registerCommand('workbench.closePanel', (...args: unknown[]) => closePanel(args[0] as string)),
    context.commands.registerCommand('workbench.focusPanel', (...args: unknown[]) => focusPanel(args[0] as string)),
  ]

  return {
    version: '1.0.0',
    project: context.project,
    commands: context.commands,
    window: context.window,
    workspace: context.workspace,
    database: context.database,
    sqlEditor: context.sqlEditor,
    events: context.events,
    configuration: context.configuration,
    utils: context.utils,

    workbench: {
      openPanel,
      closePanel,
      focusPanel,
    },

    dispose: () => {
      disposables.forEach(d => d.dispose())
      panels.clear()
    }
  }
}

const deactivate = (): void => {
  console.log('[Workbench] Deactivated')
}

const extension: ExtensionModule = {
  activate: activate as (context: ExtensionContext) => ExtensionAPI,
  deactivate,
}

export default extension
