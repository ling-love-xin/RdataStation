type Theme = 'light' | 'dark' | 'system'
type Language = 'zh-CN' | 'en'
type DefaultEngine = 'native' | 'duckdb'

interface EditorSettings {
  fontSize: number
  tabSize: number
  wordWrap: boolean
  minimap: boolean
  lineNumbers: boolean
  fontFamily: string
}

interface SerializedPanelState {
  id: string
  component: string
  params?: Record<string, string>
  title: string
}

interface SerializedDockviewLayout {
  panels: SerializedPanelState[]
  activePanel: string | null
}

interface SerializedSidebarState {
  collapsed: boolean
  width: number
  activeItem: string | null
}

interface ConfigOverrideRule {
  globalDefault: boolean
  projectOverridable: boolean
  projectOnly: boolean
}

interface GlobalConfig {
  theme: Theme
  language: Language
  editorSettings: EditorSettings
  defaultEngine: DefaultEngine
  recentProjects: string[]
}

interface ProjectConfig {
  theme?: Theme
  editorSettings?: Partial<EditorSettings>
  defaultEngine?: DefaultEngine
  dockviewLayout?: SerializedDockviewLayout
  sidebarState?: SerializedSidebarState
}

const CONFIG_REGISTRY = {
  theme: {
    key: 'theme' as const,
    default: 'dark' as Theme,
    rule: { globalDefault: true, projectOverridable: true, projectOnly: false } satisfies ConfigOverrideRule,
  },
  language: {
    key: 'language' as const,
    default: 'zh-CN' as Language,
    rule: { globalDefault: true, projectOverridable: false, projectOnly: false } satisfies ConfigOverrideRule,
  },
  editorSettings: {
    key: 'editorSettings' as const,
    default: {
      fontSize: 14,
      tabSize: 2,
      wordWrap: true,
      minimap: true,
      lineNumbers: true,
      fontFamily: "'Cascadia Code', 'Fira Code', 'Consolas', monospace",
    } satisfies EditorSettings,
    rule: { globalDefault: true, projectOverridable: true, projectOnly: false } satisfies ConfigOverrideRule,
  },
  defaultEngine: {
    key: 'defaultEngine' as const,
    default: 'native' as DefaultEngine,
    rule: { globalDefault: true, projectOverridable: true, projectOnly: false } satisfies ConfigOverrideRule,
  },
  recentProjects: {
    key: 'recentProjects' as const,
    default: [] as string[],
    rule: { globalDefault: true, projectOverridable: false, projectOnly: false } satisfies ConfigOverrideRule,
  },
  dockviewLayout: {
    key: 'dockviewLayout' as const,
    default: {} as SerializedDockviewLayout,
    rule: { globalDefault: false, projectOverridable: false, projectOnly: true } satisfies ConfigOverrideRule,
  },
  sidebarState: {
    key: 'sidebarState' as const,
    default: {} as SerializedSidebarState,
    rule: { globalDefault: false, projectOverridable: false, projectOnly: true } satisfies ConfigOverrideRule,
  },
}

type ConfigKey = (typeof CONFIG_REGISTRY)[keyof typeof CONFIG_REGISTRY]['key']

const CONFIG_KEYS = {
  THEME: CONFIG_REGISTRY.theme.key,
  LANGUAGE: CONFIG_REGISTRY.language.key,
  EDITOR_SETTINGS: CONFIG_REGISTRY.editorSettings.key,
  DEFAULT_ENGINE: CONFIG_REGISTRY.defaultEngine.key,
  RECENT_PROJECTS: CONFIG_REGISTRY.recentProjects.key,
  DOCKVIEW_LAYOUT: CONFIG_REGISTRY.dockviewLayout.key,
  SIDEBAR_STATE: CONFIG_REGISTRY.sidebarState.key,
}

const OVERRIDE_RULES: Record<string, ConfigOverrideRule> = {}
for (const entry of Object.values(CONFIG_REGISTRY)) {
  OVERRIDE_RULES[entry.key] = entry.rule
}

const DEFAULT_GLOBAL_CONFIG: GlobalConfig = {
  theme: CONFIG_REGISTRY.theme.default,
  language: CONFIG_REGISTRY.language.default,
  editorSettings: { ...CONFIG_REGISTRY.editorSettings.default },
  defaultEngine: CONFIG_REGISTRY.defaultEngine.default,
  recentProjects: [...CONFIG_REGISTRY.recentProjects.default],
}

const DEFAULT_EDITOR_SETTINGS: EditorSettings = { ...CONFIG_REGISTRY.editorSettings.default }

const SCHEMA_VERSION = 1

const STORE_FILENAME_GLOBAL = 'global-settings.json'
const STORE_FILENAME_PROJECT = 'project-settings.json'

interface SeedEntry {
  key: string
  default: unknown
}

const GLOBAL_SEED_KEYS: SeedEntry[] = Object.values(CONFIG_REGISTRY)
  .filter((e) => e.rule.globalDefault)
  .map((e) => ({ key: e.key, default: e.default }))

const PROJECT_SEED_KEYS: SeedEntry[] = Object.values(CONFIG_REGISTRY)
  .filter((e) => e.rule.projectOnly || e.rule.projectOverridable)
  .map((e) => ({ key: e.key, default: e.default }))

export {
  CONFIG_KEYS,
  CONFIG_REGISTRY,
  OVERRIDE_RULES,
  DEFAULT_GLOBAL_CONFIG,
  DEFAULT_EDITOR_SETTINGS,
  SCHEMA_VERSION,
  STORE_FILENAME_GLOBAL,
  STORE_FILENAME_PROJECT,
  GLOBAL_SEED_KEYS,
  PROJECT_SEED_KEYS,
}

export type {
  ConfigKey,
  Theme,
  Language,
  DefaultEngine,
  EditorSettings,
  GlobalConfig,
  ProjectConfig,
  ConfigOverrideRule,
  SerializedDockviewLayout,
  SerializedSidebarState,
  SeedEntry,
}
