import { invoke } from '@tauri-apps/api/core'

import ScratchpadPanel from './ui/components/ScratchpadPanel.vue'

import type { ExtensionContext, ExtensionAPI, ExtensionModule, Disposable } from '../../core/types'

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
    order: 4,
  })

  const initStore = async (projectPath: string) => {
    try {
      await invoke('init_scratchpad_store', { projectPath })
      console.log('[Scratchpad] Store initialized for:', projectPath)
    } catch (e) {
      console.error('[Scratchpad] Store init failed:', e)
    }
  }

  initStore(context.project.path)

  const handleProjectSwitch = (event: Event) => {
    const detail = (event as CustomEvent).detail as { project?: { path?: string } } | undefined
    const newPath = detail?.project?.path
    if (newPath) {
      console.log('[Scratchpad] Project switched to:', newPath)
      initStore(newPath)
    }
  }

  window.addEventListener('project-switched', handleProjectSwitch)

  const disposables: Disposable[] = [
    panelDisposable,
    {
      dispose: () => window.removeEventListener('project-switched', handleProjectSwitch),
    },
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
      disposables.forEach(d => d.dispose())
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
