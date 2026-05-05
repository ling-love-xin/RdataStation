import type {
  ExtensionContext,
  ExtensionAPI,
  ExtensionModule,
  Disposable,
} from '../../core/types'
import AnalyticsResourceManager from './ui/components/AnalyticsResourceManager.vue'

interface AnalyticsResourceExtensionAPI extends ExtensionAPI {
  analyticsResource: {
    // 可以在这里添加扩展特定的 API
  }
}

const activate = (context: ExtensionContext): AnalyticsResourceExtensionAPI => {
  console.log('[Analytics Resource] Activating for project:', context.project.name)

  // 注册面板
  const panelDisposable = context.window.registerViewProvider('analytics-resource-manager', {
    component: AnalyticsResourceManager,
    title: '分析资源管理器',
    location: 'left',
    icon: '📊'
  })

  const disposables: Disposable[] = [
    panelDisposable,
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

    analyticsResource: {
      // 扩展特定的 API
    },

    dispose: () => {
      disposables.forEach((d) => d.dispose())
    },
  }
}

const deactivate = (): void => {
  console.log('[Analytics Resource] Deactivated')
}

const extension: ExtensionModule = {
  activate: activate as (context: ExtensionContext) => ExtensionAPI,
  deactivate,
}

export default extension;
