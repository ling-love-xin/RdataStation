import ScratchpadPanel from './ui/components/ScratchpadPanel.vue'

import type {
  ExtensionContext,
  ExtensionAPI,
  ExtensionModule,
  Disposable,
} from '../../core/types'

interface ScratchpadExtensionAPI extends ExtensionAPI {
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  scratchpad: {}
}

const activate = (context: ExtensionContext): ScratchpadExtensionAPI => {
  console.log('[Scratchpad] Activating for project:', context.project.name)

  const panelDisposable = context.window.registerViewProvider('scratchpad', {
    component: ScratchpadPanel,
    title: '草稿箱',
    location: 'left',
    icon: 'FileText',
    order: 4
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

    scratchpad: {
      // 扩展特定的 API 预留
    },

    dispose: () => {
      disposables.forEach((d) => d.dispose())
    },
  }
}

const deactivate = (): void => {
  console.log('[Scratchpad] Deactivated')
}

const extension: ExtensionModule = {
  activate: activate as (context: ExtensionContext) => ExtensionAPI,
  deactivate,
}

export default extension
