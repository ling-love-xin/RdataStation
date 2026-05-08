/**
 * 配置系统类型定义与注册表
 *
 * ─── 三层优先级（单向 fallback）───
 *   Level 1: 项目覆盖值  (project-settings.json)
 *   Level 2: 全局默认值  (global-settings.json)
 *   Level 3: 系统硬编码   (DEFAULT_GLOBAL_CONFIG)
 * ─────────────────────────────
 *
 * ─── 新增配置项流程 ───
 *   1. 定义类型（Theme / Language 等）
 *   2. 在 CONFIG_REGISTRY 添加条目（含 key / default / writeType / rule）
 *   3. 追加到 GlobalConfig / ProjectConfig 接口
 *   4. 追加 zod inner schema + 注册到 VALUE_SCHEMAS
 *   5. ConfigKey / ConfigValueType / CONFIG_KEYS / OVERRIDE_RULES / SEED_KEYS 自动派生
 *
 * @module config
 * @version 1.1
 * @status Phase 1 complete / Phase 2 API ready
 */

import { z } from 'zod'

/** 主题枚举 */
type Theme = 'light' | 'dark' | 'system'

/** 语言枚举（对应 naive-ui locale + vue-i18n） */
type Language = 'zh-CN' | 'en'

/** 默认查询执行引擎 */
type DefaultEngine = 'native' | 'duckdb'

/** 编辑器设置（第 2 层嵌套，整体作为 JSON 值存取） */
interface EditorSettings {
  fontSize: number
  tabSize: number
  wordWrap: boolean
  minimap: boolean
  lineNumbers: boolean
  fontFamily: string
}

/** dockview-vue 序列化面板状态 */
interface SerializedPanelState {
  id: string
  component: string
  params?: Record<string, string>
  title: string
}

/** dockview-vue 序列化布局 */
interface SerializedDockviewLayout {
  panels: SerializedPanelState[]
  activePanel: string | null
}

/** 侧边栏序列化状态 */
interface SerializedSidebarState {
  collapsed: boolean
  width: number
  activeItem: string | null
}

/** 配置项可覆盖性规则 */
interface ConfigOverrideRule {
  /** Level 2 是否提供全局默认值 */
  globalDefault: boolean
  /** 是否允许项目覆盖 Level 2 值 */
  projectOverridable: boolean
  /** 是否仅项目级存在（不在 global-settings.json 中） */
  projectOnly: boolean
}

/** 全局配置对象（存储在 global-settings.json） */
interface GlobalConfig {
  theme: Theme
  language: Language
  editorSettings: EditorSettings
  defaultEngine: DefaultEngine
  recentProjects: string[]
}

/** 项目配置对象（存储在 project-{id}-settings.json，所有字段可选） */
interface ProjectConfig {
  theme?: Theme
  editorSettings?: Partial<EditorSettings>
  defaultEngine?: DefaultEngine
  dockviewLayout?: SerializedDockviewLayout
  sidebarState?: SerializedSidebarState
}

// ============================================
// 原子 zod schemas（供组合 + 写时校验复用）
// ============================================

const ThemeSchema = z.enum(['light', 'dark', 'system'])
const LanguageSchema = z.enum(['zh-CN', 'en'])
const DefaultEngineSchema = z.enum(['native', 'duckdb'])

const EditorSettingsSchema = z.object({
  fontSize: z.number().min(10).max(24),
  tabSize: z.union([z.literal(2), z.literal(4), z.literal(8)]),
  wordWrap: z.boolean(),
  minimap: z.boolean(),
  lineNumbers: z.boolean(),
  fontFamily: z.string().min(1),
})

const SerializedPanelStateSchema = z.object({
  id: z.string(),
  component: z.string(),
  params: z.record(z.string(), z.string()).optional(),
  title: z.string(),
})

const DockviewLayoutSchema = z.object({
  panels: z.array(SerializedPanelStateSchema),
  activePanel: z.string().nullable(),
})

const SidebarStateSchema = z.object({
  collapsed: z.boolean(),
  width: z.number().min(200).max(400),
  activeItem: z.string().nullable(),
})

const RecentProjectsSchema = z.array(z.string()).max(10)

// ============================================
// 配置注册表 —— 单一事实来源
// ============================================

const CONFIG_REGISTRY = {
  theme: {
    key: 'theme' as const,
    default: 'dark' as Theme,
    writeType: 'dark' as Theme,
    valueSchema: ThemeSchema,
    rule: {
      globalDefault: true,
      projectOverridable: true,
      projectOnly: false,
    } satisfies ConfigOverrideRule,
  },
  language: {
    key: 'language' as const,
    default: 'zh-CN' as Language,
    writeType: 'zh-CN' as Language,
    valueSchema: LanguageSchema,
    rule: {
      globalDefault: true,
      projectOverridable: false,
      projectOnly: false,
    } satisfies ConfigOverrideRule,
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
    writeType: {} as EditorSettings | Partial<EditorSettings>,
    valueSchema: EditorSettingsSchema,
    rule: {
      globalDefault: true,
      projectOverridable: true,
      projectOnly: false,
    } satisfies ConfigOverrideRule,
  },
  defaultEngine: {
    key: 'defaultEngine' as const,
    default: 'native' as DefaultEngine,
    writeType: 'native' as DefaultEngine,
    valueSchema: DefaultEngineSchema,
    rule: {
      globalDefault: true,
      projectOverridable: true,
      projectOnly: false,
    } satisfies ConfigOverrideRule,
  },
  recentProjects: {
    key: 'recentProjects' as const,
    default: [] as string[],
    writeType: [] as string[],
    valueSchema: RecentProjectsSchema,
    rule: {
      globalDefault: true,
      projectOverridable: false,
      projectOnly: false,
    } satisfies ConfigOverrideRule,
  },
  dockviewLayout: {
    key: 'dockviewLayout' as const,
    default: { panels: [], activePanel: null } satisfies SerializedDockviewLayout,
    writeType: {} as SerializedDockviewLayout,
    valueSchema: DockviewLayoutSchema,
    rule: {
      globalDefault: false,
      projectOverridable: false,
      projectOnly: true,
    } satisfies ConfigOverrideRule,
  },
  sidebarState: {
    key: 'sidebarState' as const,
    default: { collapsed: false, width: 280, activeItem: null } satisfies SerializedSidebarState,
    writeType: {} as SerializedSidebarState,
    valueSchema: SidebarStateSchema,
    rule: {
      globalDefault: false,
      projectOverridable: false,
      projectOnly: true,
    } satisfies ConfigOverrideRule,
  },
}

/** 联合类型：所有合法配置键（从注册表自动派生） */
type ConfigKey = (typeof CONFIG_REGISTRY)[keyof typeof CONFIG_REGISTRY]['key']

/** 配置键→值类型映射（从注册表 writeType 自动派生，新增键无需改动此处） */
type ConfigValueType<K extends ConfigKey> = (typeof CONFIG_REGISTRY)[K]['writeType']

/** 配置键→zod schema 查找表 */
type ValueSchemaLookup = {
  [K in ConfigKey]?: z.ZodType<unknown>
}

const VALUE_SCHEMAS: ValueSchemaLookup = {}
for (const entry of Object.values(CONFIG_REGISTRY)) {
  VALUE_SCHEMAS[entry.key] = entry.valueSchema as z.ZodType<unknown>
}

/** 从注册表派生的键名常量集合 */
const CONFIG_KEYS = {
  THEME: CONFIG_REGISTRY.theme.key,
  LANGUAGE: CONFIG_REGISTRY.language.key,
  EDITOR_SETTINGS: CONFIG_REGISTRY.editorSettings.key,
  DEFAULT_ENGINE: CONFIG_REGISTRY.defaultEngine.key,
  RECENT_PROJECTS: CONFIG_REGISTRY.recentProjects.key,
  DOCKVIEW_LAYOUT: CONFIG_REGISTRY.dockviewLayout.key,
  SIDEBAR_STATE: CONFIG_REGISTRY.sidebarState.key,
}

/** 可覆盖性规则查找表：key → ConfigOverrideRule */
const OVERRIDE_RULES: Record<string, ConfigOverrideRule> = {}
for (const entry of Object.values(CONFIG_REGISTRY)) {
  OVERRIDE_RULES[entry.key] = entry.rule
}

/** 全局默认配置（Level 3 → seed 到 Level 2 首次启动） */
const DEFAULT_GLOBAL_CONFIG: GlobalConfig = {
  theme: CONFIG_REGISTRY.theme.default,
  language: CONFIG_REGISTRY.language.default,
  editorSettings: { ...CONFIG_REGISTRY.editorSettings.default },
  defaultEngine: CONFIG_REGISTRY.defaultEngine.default,
  recentProjects: [...CONFIG_REGISTRY.recentProjects.default],
}

/** 编辑器默认设置独立副本 */
const DEFAULT_EDITOR_SETTINGS: EditorSettings = { ...CONFIG_REGISTRY.editorSettings.default }

/** 配置 schema 主版本号（变更时追加迁移函数） */
const SCHEMA_VERSION = 1

/** tauri-plugin-store 文件名 */
const STORE_FILENAME_GLOBAL = 'global-settings.json'
const STORE_FILENAME_PROJECT = 'project-settings.json'

/** 最近项目列表最大条数 */
const RECENT_PROJECTS_MAX = 10

/** 侧边栏宽度限制 */
const SIDEBAR_WIDTH_MIN = 200
const SIDEBAR_WIDTH_MAX = 400

// ============================================
// 复合 zod schemas（供 loadStoreWithDefaults 校验）
// ============================================

const GlobalConfigSchema = z.object({
  _schemaVersion: z.number().optional(),
  theme: ThemeSchema,
  language: LanguageSchema,
  editorSettings: EditorSettingsSchema,
  defaultEngine: DefaultEngineSchema,
  recentProjects: RecentProjectsSchema,
})

const ProjectConfigSchema = z.object({
  theme: ThemeSchema.optional(),
  editorSettings: EditorSettingsSchema.partial().optional(),
  defaultEngine: DefaultEngineSchema.optional(),
  dockviewLayout: DockviewLayoutSchema.optional(),
  sidebarState: SidebarStateSchema.optional(),
})

// ============================================
// Schema 版本迁移
// ============================================

type MigrationFn = (data: Record<string, unknown>) => Record<string, unknown>

/**
 * 迁移函数注册表
 *
 * key: fromVersion（迁移前版本号）
 * value: 将数据迁移到 key+1 版本的纯函数
 *
 * @example
 * // SCHEMA_VERSION 从 1 升到 2 时：
 * // 1: (data) => {
 * //   // v2: 'locale' → 'language'
 * //   if ('locale' in data) {
 * //     data.language = data.locale
 * //     delete data.locale
 * //   }
 * //   return data
 * // }
 */
const MIGRATIONS: Record<number, MigrationFn> = {}

function migrateConfig(
  fromVersion: number,
  data: Record<string, unknown>
): Record<string, unknown> {
  let current = fromVersion
  let result = { ...data }

  while (current < SCHEMA_VERSION) {
    const migration = MIGRATIONS[current]
    if (migration) {
      result = migration(result)
      console.info(`[config] Migrated schema ${current} → ${current + 1}`)
    }
    current++
  }

  return result
}

/** 种子条目（initialize 驱动循环用） */
interface SeedEntry {
  key: ConfigKey
  default: unknown
}

/** 全局级种子键（globalDefault === true） */
const GLOBAL_SEED_KEYS: SeedEntry[] = Object.values(CONFIG_REGISTRY)
  .filter(e => e.rule.globalDefault)
  .map(e => ({ key: e.key, default: e.default }))

/** 项目级种子键（projectOnly || projectOverridable） */
const PROJECT_SEED_KEYS: SeedEntry[] = Object.values(CONFIG_REGISTRY)
  .filter(e => e.rule.projectOnly || e.rule.projectOverridable)
  .map(e => ({ key: e.key, default: e.default }))

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
  RECENT_PROJECTS_MAX,
  SIDEBAR_WIDTH_MIN,
  SIDEBAR_WIDTH_MAX,
  VALUE_SCHEMAS,
  EditorSettingsSchema,
  GlobalConfigSchema,
  ProjectConfigSchema,
  migrateConfig,
}

export type {
  ConfigKey,
  ConfigValueType,
  Theme,
  Language,
  DefaultEngine,
  EditorSettings,
  GlobalConfig,
  ProjectConfig,
  SerializedDockviewLayout,
  SerializedSidebarState,
}
