import { Store } from '@tauri-apps/plugin-store'
import { defineStore } from 'pinia'
import { ref, computed, shallowRef } from 'vue'

import {
  CONFIG_KEYS,
  DEFAULT_GLOBAL_CONFIG,
  DEFAULT_EDITOR_SETTINGS,
  SCHEMA_VERSION,
  STORE_FILENAME_GLOBAL,
  STORE_FILENAME_PROJECT,
  GLOBAL_SEED_KEYS,
} from './config'

import type {
  ConfigKey,
  Theme,
  Language,
  DefaultEngine,
  EditorSettings,
  GlobalConfig,
  ProjectConfig,
  SerializedDockviewLayout,
  SerializedSidebarState,
} from './config'

type ConfigScope = 'global' | 'project'

interface SaveResult {
  success: boolean
  key: ConfigKey
  scope: ConfigScope
  error?: string
}

function buildProjectStoreFilename(projectPath: string): string {
  const normalized = projectPath.replace(/[/\\:]/g, '_')
  return `project-${normalized}-${STORE_FILENAME_PROJECT}`
}

function deepClone<T>(obj: T): T {
  return JSON.parse(JSON.stringify(obj))
}

export const useAppStore = defineStore('appConfig', () => {
  const globalConfig = ref<GlobalConfig>(deepClone(DEFAULT_GLOBAL_CONFIG))
  const projectConfig = ref<ProjectConfig>({})
  const initialized = ref(false)
  const projectOpen = ref(false)
  const projectPath = ref<string | null>(null)

  const globalStoreRef = shallowRef<Store | null>(null)
  const projectStoreRef = shallowRef<Store | null>(null)

  const effectiveTheme = computed<Theme>(() => {
    if (projectConfig.value.theme !== undefined) {
      return projectConfig.value.theme
    }
    return globalConfig.value.theme
  })

  const isDark = computed(() => {
    const t = effectiveTheme.value
    if (t === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return t === 'dark'
  })

  const effectiveLanguage = computed<Language>(() => globalConfig.value.language)

  const effectiveEditorSettings = computed<EditorSettings>(() => {
    const base = deepClone(DEFAULT_EDITOR_SETTINGS)
    Object.assign(base, globalConfig.value.editorSettings)
    if (projectConfig.value.editorSettings) {
      Object.assign(base, projectConfig.value.editorSettings)
    }
    return base
  })

  const effectiveDefaultEngine = computed<DefaultEngine>(() => {
    if (projectConfig.value.defaultEngine !== undefined) {
      return projectConfig.value.defaultEngine
    }
    return globalConfig.value.defaultEngine
  })

  const recentProjects = computed(() => globalConfig.value.recentProjects)

  const effectiveDockviewLayout = computed<SerializedDockviewLayout | undefined>(
    () => projectConfig.value.dockviewLayout
  )

  const effectiveSidebarState = computed<SerializedSidebarState | undefined>(
    () => projectConfig.value.sidebarState
  )

  function getConfig<T>(key: ConfigKey): T {
    return getConfigInternal(key) as T
  }

  function getConfigInternal(key: string): unknown {
    const rawGlobal = globalConfig.value as Record<string, unknown>
    const rawProject = projectConfig.value as Record<string, unknown>

    if (key === CONFIG_KEYS.DOCKVIEW_LAYOUT || key === CONFIG_KEYS.SIDEBAR_STATE) {
      return rawProject[key]
    }

    if (key === CONFIG_KEYS.LANGUAGE || key === CONFIG_KEYS.RECENT_PROJECTS) {
      const gv = rawGlobal[key]
      return gv !== undefined ? gv : (DEFAULT_GLOBAL_CONFIG as unknown as Record<string, unknown>)[key]
    }

    const pv = rawProject[key]
    if (pv !== undefined) {
      return pv
    }

    const gv = rawGlobal[key]
    if (gv !== undefined) {
      return gv
    }

    return (DEFAULT_GLOBAL_CONFIG as unknown as Record<string, unknown>)[key]
  }

  async function saveConfig(
    key: ConfigKey,
    value: unknown,
    scope: ConfigScope
  ): Promise<SaveResult> {
    const store = scope === 'global' ? globalStoreRef.value : projectStoreRef.value
    if (!store) {
      return { success: false, key, scope, error: `${scope} store not initialized` }
    }

    const rawTarget =
      scope === 'global'
        ? (globalConfig.value as Record<string, unknown>)
        : (projectConfig.value as Record<string, unknown>)

    const prevValue = rawTarget[key]

    try {
      await store.set(key, value)
      await store.save()
      rawTarget[key] = value
      return { success: true, key, scope }
    } catch (e) {
      rawTarget[key] = prevValue
      const message = e instanceof Error ? e.message : String(e)
      console.error(`[useAppStore] saveConfig failed: ${key}=${value} (${scope})`, e)
      return { success: false, key, scope, error: message }
    }
  }

  async function resetProjectOverride(key: ConfigKey): Promise<void> {
    const store = projectStoreRef.value
    if (!store) {
      return
    }

    try {
      await store.delete(key)
      await store.save()
      delete (projectConfig.value as Record<string, unknown>)[key]
    } catch (e) {
      console.error(`[useAppStore] resetProjectOverride failed: ${key}`, e)
    }
  }

  function hasProjectOverride(key: ConfigKey): boolean {
    return (projectConfig.value as Record<string, unknown>)[key] !== undefined
  }

  async function setTheme(theme: Theme, scope: ConfigScope = 'global'): Promise<SaveResult> {
    return saveConfig(CONFIG_KEYS.THEME, theme, scope)
  }

  async function setLanguage(language: Language): Promise<SaveResult> {
    return saveConfig(CONFIG_KEYS.LANGUAGE, language, 'global')
  }

  async function setEditorSettings(
    settings: Partial<EditorSettings>,
    scope: ConfigScope = 'global'
  ): Promise<SaveResult> {
    if (scope === 'global') {
      const current = deepClone(globalConfig.value.editorSettings)
      const merged = { ...current, ...settings }
      return saveConfig(CONFIG_KEYS.EDITOR_SETTINGS, merged, scope)
    }

    const current = deepClone(projectConfig.value.editorSettings ?? {})
    const globalCurrent = deepClone(globalConfig.value.editorSettings)
    const newMerged = { ...globalCurrent, ...current, ...settings }

    const diff: Partial<EditorSettings> = {}
    for (const entry of Object.entries(newMerged) as [keyof EditorSettings, unknown][]) {
      if (entry[1] !== globalCurrent[entry[0]]) {
        ;(diff as Record<string, unknown>)[entry[0] as string] = entry[1]
      }
    }

    return saveConfig(CONFIG_KEYS.EDITOR_SETTINGS, diff, scope)
  }

  async function setDefaultEngine(
    engine: DefaultEngine,
    scope: ConfigScope = 'global'
  ): Promise<SaveResult> {
    return saveConfig(CONFIG_KEYS.DEFAULT_ENGINE, engine, scope)
  }

  async function addRecentProject(projectId: string): Promise<void> {
    const list = [...globalConfig.value.recentProjects]
    const idx = list.indexOf(projectId)
    if (idx !== -1) {
      list.splice(idx, 1)
    }
    list.unshift(projectId)
    if (list.length > 10) {
      list.length = 10
    }
    await saveConfig(CONFIG_KEYS.RECENT_PROJECTS, list, 'global')
  }

  async function removeRecentProject(projectId: string): Promise<void> {
    const list = globalConfig.value.recentProjects.filter((id) => id !== projectId)
    await saveConfig(CONFIG_KEYS.RECENT_PROJECTS, list, 'global')
  }

  async function saveDockviewLayout(layout: SerializedDockviewLayout): Promise<SaveResult> {
    return saveConfig(CONFIG_KEYS.DOCKVIEW_LAYOUT, layout, 'project')
  }

  async function saveSidebarState(state: SerializedSidebarState): Promise<SaveResult> {
    return saveConfig(CONFIG_KEYS.SIDEBAR_STATE, state, 'project')
  }

  async function initialize(): Promise<void> {
    if (initialized.value) {
      return
    }

    try {
      const store = await Store.load(STORE_FILENAME_GLOBAL)
      globalStoreRef.value = store

      const storedVersion = await store.get<number>('_schemaVersion')
      const rawGlobal = globalConfig.value as Record<string, unknown>

      let needsSave = false

      for (const { key, default: defaultVal } of GLOBAL_SEED_KEYS) {
        const stored = await store.get<unknown>(key)
        if (stored !== null && stored !== undefined) {
          rawGlobal[key] = stored
        } else {
          await store.set(key, defaultVal)
          needsSave = true
        }
      }

      if (storedVersion !== SCHEMA_VERSION) {
        await store.set('_schemaVersion', SCHEMA_VERSION)
        needsSave = true
      }

      if (needsSave) {
        await store.save()
      }

      initialized.value = true
    } catch (e) {
      console.error('[useAppStore] Failed to initialize global store:', e)
      initialized.value = true
    }
  }

  async function openProject(p: string): Promise<void> {
    if (projectOpen.value) {
      await closeProject()
    }

    try {
      const filename = buildProjectStoreFilename(p)
      const store = await Store.load(filename)
      projectStoreRef.value = store

      const newConfig: ProjectConfig = {}
      const keys = [
        CONFIG_KEYS.THEME,
        CONFIG_KEYS.EDITOR_SETTINGS,
        CONFIG_KEYS.DEFAULT_ENGINE,
        CONFIG_KEYS.DOCKVIEW_LAYOUT,
        CONFIG_KEYS.SIDEBAR_STATE,
      ]

      for (const key of keys) {
        const stored = await store.get<unknown>(key)
        if (stored !== null && stored !== undefined) {
          ;(newConfig as Record<string, unknown>)[key] = stored
        }
      }

      projectConfig.value = newConfig
      projectPath.value = p
      projectOpen.value = true
    } catch (e) {
      console.error('[useAppStore] Failed to open project store:', e)
      projectOpen.value = true
      projectPath.value = p
    }
  }

  async function closeProject(): Promise<void> {
    if (!projectOpen.value) {
      return
    }

    const store = projectStoreRef.value
    if (store) {
      try {
        await store.save()
      } catch (e) {
        console.error('[useAppStore] Failed to save project store on close:', e)
      }
    }

    projectStoreRef.value = null
    projectConfig.value = {}
    projectPath.value = null
    projectOpen.value = false
  }

  function applyTheme() {
    const body = document.body
    const themeClass = isDark.value ? 'theme-dark' : 'theme-light'
    body.classList.remove('theme-dark', 'theme-light')
    body.classList.add(themeClass)
  }

  return {
    globalConfig,
    projectConfig,
    initialized,
    projectOpen,
    projectPath,

    globalStoreRef,
    projectStoreRef,

    effectiveTheme,
    isDark,
    effectiveLanguage,
    effectiveEditorSettings,
    effectiveDefaultEngine,
    recentProjects,
    effectiveDockviewLayout,
    effectiveSidebarState,

    getConfig,
    saveConfig,
    resetProjectOverride,
    hasProjectOverride,

    setTheme,
    setLanguage,
    setEditorSettings,
    setDefaultEngine,
    addRecentProject,
    removeRecentProject,
    saveDockviewLayout,
    saveSidebarState,

    initialize,
    openProject,
    closeProject,
    applyTheme,
  }
})
