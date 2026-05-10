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

  let storeInitialized = false

  const initStore = async (projectPath: string): Promise<void> => {
    if (!projectPath) {
      console.warn('[Scratchpad] No project path, skipping store init')
      return
    }
    try {
      await invoke('init_scratchpad_store', { projectPath })
      storeInitialized = true
      console.log('[Scratchpad] Store initialized for:', projectPath)
    } catch (e) {
      console.error('[Scratchpad] Store init failed:', e)
    }
  }

  initStore(context.project.path).then(() => {
    console.log('[Scratchpad] Initial activation init complete')
  })

  const handleProjectSwitch = (event: Event): void => {
    const detail = (event as CustomEvent).detail as { project?: { path?: string } } | undefined
    const newPath = detail?.project?.path
    if (newPath) {
      console.log('[Scratchpad] Project switched to:', newPath)
      initStore(newPath).then(() => {
        console.log('[Scratchpad] Store re-initialized for project switch')
      })
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
      isInitialized: () => storeInitialized,
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
