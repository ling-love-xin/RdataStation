# SQL 编辑器模块完整设计

> 版本：v1.3
> 创建日期：2026-05-09
> 状态：📐 设计文档
> 关联：[SQL-EDITOR.md](./SQL-EDITOR.md) · [优化计划](./SQL-EDITOR-OPTIMIZATION-PLAN.md)

---

## 📖 目录

- [1. 模块定位与目标](#1-模块定位与目标)
- [2. 总体架构](#2-总体架构)
- [3. 组件树与职责](#3-组件树与职责)
- [4. 数据流架构](#4-数据流架构)
- [5. 状态管理矩阵](#5-状态管理矩阵)
- [6. 配置系统设计](#6-配置系统设计)
- [7. 接口契约](#7-接口契约)
- [8. 功能模块设计](#8-功能模块设计)
- [9. 未实现功能技术方案](#9-未实现功能技术方案)
- [10. 文件结构清单](#10-文件结构清单)

---

## 1. 模块定位与目标

### 1.1 定位

SQL 编辑器模块是 RdataStation 的核心交互模块，对标 DBeaver / DataGrip 的 SQL 编辑器体验，提供完整的 SQL 编写、执行、结果分析闭环。

### 1.2 设计目标

```
用户体验目标:
  ├── 零等待感知 ─── 编辑器 < 100ms 启动、SQL 执行实时反馈
  ├── 上下文智能 ─── 连接感知的补全、方言、高亮、错误定位
  ├── 无损操作   ─── 自动保存草稿、历史追溯、参数化复用
  └── 视觉一致   ─── 全局主题 + 编辑器 6 项设置跨会话保持

架构目标:
  ├── 组件轻量 ─── 单文件 < 400 行、职责单一
  ├── 通信规范 ─── Pinia Store + provide/inject、零全局 CustomEvent
  ├── 配置统一 ─── 单一 CONFIG_REGISTRY → useAppStore → tauri-plugin-store
  └── 类型安全 ─── 消除 as any、统一接口契约
```

### 1.3 数据库支持矩阵

| 数据库 | 语法高亮 | 代码补全 | 执行 | 方言转换 | DuckDB 加速 | EXPLAIN |
| ------ | -------- | -------- | ---- | -------- | ----------- | ------- |
| MySQL | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| PostgreSQL | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| SQLite | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| DuckDB | ✅ | ✅ | ✅ | ✅ | N/A | ✅ |

---

## 2. 总体架构

### 2.1 分层架构

```
┌──────────────────────────────────────────────────────────────────┐
│                        UI Layer (Vue 3)                          │
│  ┌───────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ SqlEditorPanel│  │QueryResult   │  │ Settings Panels (x2) │  │
│  │ (编排层 ~947L)│  │Panel         │  │ → 统一到 config.ts  │  │
│  └───────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
│          │                 │                      │              │
│  ┌───────┴─────────────────┴──────────────────────┴───────────┐  │
│  │                    Composables Layer                        │  │
│  │  useSqlExecution / useMonacoEditor / useConnectionBinding   │  │
│  │  useDialectSync / useEditorPersistence / useResultTabs     │  │
│  └──────────────────────────┬─────────────────────────────────┘  │
├─────────────────────────────┼────────────────────────────────────┤
│                     Pinia Store Layer                             │
│  ┌──────────────────────────┼─────────────────────────────────┐  │
│  │  sql-execution-store  │  result-store  │  useAppStore      │  │
│  │  layout-store         │  workbench-store                   │  │
│  └──────────────────────────┼─────────────────────────────────┘  │
├─────────────────────────────┼────────────────────────────────────┤
│                     Service Layer                                 │
│  ┌──────────────────────────┼─────────────────────────────────┐  │
│  │  sql-editor-service  │  sql-dialect-highlight               │  │
│  │  sql-snippets        │  sql-history-service                 │  │
│  │  result-analysis     │  query.ts (Tauri IPC)                │  │
│  └──────────────────────────┼─────────────────────────────────┘  │
├─────────────────────────────┼────────────────────────────────────┤
│                   Tauri IPC Bridge                                │
│  invoke('execute_sql') / invoke('cancel_sql_query') / ...        │
├─────────────────────────────┼────────────────────────────────────┤
│                    Rust Backend                                   │
│  ConnectionManager → Database trait → MySQL/PostgreSQL/SQLite/   │
│  DuckDB implementations                                           │
└──────────────────────────────────────────────────────────────────┘
```

### 2.2 关键设计决策

| 决策 | 理由 |
| ---- | ---- |
| 1:n 编辑器↔结果面板（非 n:n） | 每个编辑器独立持有结果区域，避免多编辑器的结果混乱（DBeaver 风格） |
| Pinia Store 通信（非 CustomEvent） | 显式数据流、DevTools 可调试、无内存泄漏风险 |
| composable 封装（非 Mixin） | 类型安全、按需导入、可独立测试 |
| tauri-plugin-store 持久化 | 跨会话 JSON 文件持久化，未来可迁移至 Rust SQLite |
| 三层配置优先级 | global-settings.json → project-settings.json → 系统硬编码 |

---

## 3. 组件树与职责

### 3.1 完整组件树

```
WorkbenchView.vue (dockview-vue 容器)
│
├── NavigatorPanel.vue ───────────────── 数据库导航树
├── SqlEditorPanel.vue ───────────────── 编排层 (核心)
│   ├── EditorToolbar.vue ────────────── 工具栏
│   │   ├── 执行组: Execute / Execute+ / DuckDB / EXPLAIN
│   │   ├── 编辑组: Format / Validate / Transpile
│   │   └── 功能组: History / Star(收藏) / Map(Minimap) / Settings
│   ├── EditorWelcome.vue ────────────── 欢迎页 (空编辑器时)
│   ├── Monaco Editor ────────────────── SQL 编辑区 (通过 useMonacoEditor)
│   ├── TranspileModal.vue ───────────── 方言转换弹窗
│   ├── ParamBindingModal.vue ────────── 参数绑定弹窗
│   ├── Settings Popover ─────────────── 编辑器设置 (NPopover)
│   ├── QueryResultPanel.vue ─────────── 结果展示面板
│   │   ├── MultiTabResults.vue ─────── 多 Tab 结果管理
│   │   ├── OutputPanel.vue ─────────── 输出/消息视图
│   │   ├── DataVisualizationPanel.vue ─ 图表视图 (ECharts)
│   │   ├── ColumnInsightPanel.vue ──── 列洞察
│   │   └── (AG Grid) ───────────────── 数据表格
│   └── EditorStatusbar.vue ──────────── 状态栏
│       ├── 光标位置 + 选中信息
│       ├── 执行状态 + 耗时
│       ├── 连接的 NPopselect
│       └── 事务指示器 (TX)
│
├── SettingsPanel.vue ─────────────────── 工作台设置面板 (需重构)
├── SqlHistoryPanel.vue ──────────────── SQL 执行历史
├── SnippetPanel.vue ─────────────────── SQL 代码片段面板
└── TableDataPanel.vue ───────────────── 表数据视图
```

### 3.2 组件职责矩阵

| 组件 | 行数 | 职责 | 依赖 Composable |
| ---- | ---- | ---- | --------------- |
| **SqlEditorPanel** | ~947 | 编排所有子组件 + 协调 composables | useMonacoEditor, useSqlExecution, useConnectionBinding, useDialectSync, useEditorPersistence |
| **EditorToolbar** | ~200 | 工具栏按钮 + 分组折叠 + 位置切换 | (纯展示 + emit) |
| **EditorStatusbar** | ~180 | 状态信息展示 + 连接选择器 + 事务指示 | (纯展示 + emit) |
| **EditorWelcome** | ~80 | 空编辑器欢迎页 + 最近连接 | (纯展示 + emit) |
| **QueryResultPanel** | ~900 | 结果展示 + 三模式过滤 + Inline Edit + 导出 | useResultTabs, useGridConfig, useGridKeyboard, useFilterModes, useFilterPresets, useResultExport |
| **TranspileModal** | ~60 | 方言转换弹窗 | (纯展示 + emit) |
| **ParamBindingModal** | ~100 | 参数绑定表单 | (纯展示 + emit) |
| **SqlHistoryPanel** | ~300 | 历史列表 + 搜索 + 收藏 + 双击重执行 | (事件驱动) |
| **SnippetPanel** | ~200 | 代码片段列表 + 搜索 + 插入/删除 | (事件驱动) |
| **SettingsPanel** | ~500 | 工作台全局设置 (需重构) | 无 (当前用 localStorage) |

---

## 4. 数据流架构

### 4.1 执行链路

```
┌──────────┐
│ 用户点击  │ Execute / Ctrl+Enter
│  Execute   │
└─────┬─────┘
      │
      ▼
┌─────────────────────────────────────────────────────────────────┐
│ SqlEditorPanel.handleExecute()                                  │
│   1. getSelectedText() || getEditorValue()                      │
│   2. detectParams(sql) ──→ 有 :param? ──→ ParamBindingModal    │
│   3. ensureConnection(connId)                                   │
│   4. useSqlExecution.executeSql(sql, runtimeConnId)              │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│ useSqlExecution.executeSql()                                    │
│   1. scheduleParse() ──→ 更新状态栏语句数                        │
│   2. queryService.executeSql(sql, connId)                       │
│      └─→ invoke('execute_sql', { sql, conn_id })                │
│          └─→ [Rust] ConnectionManager → Database::query()       │
│              └─→ QueryResult { batches: Vec<RecordBatch> }      │
│   3. storeResult() ──→ resultStore.addTab() + setTabResult()    │
│   4. clearErrorMarkers() ──→ 清除 Monaco 错误标记                │
│   5. (on error) setErrorMarker() ──→ 解析错误位置 → Monaco 标记  │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 DuckDB 加速链路

```
handleDuckDbExecute()
  → ensureConnection(connId)
  → executeDuckDBAccelerated()
    → rewriteDuckDBSQL(sql, attachName)  // FROM users → FROM ext_MyConn.users
    → appDataDir() → 获取扩展目录
    → queryService.executeDuckDBAccelerated({ sql, connId, dbType, dataDir })
      → invoke('execute_duckdb_accelerated', ...)
        → [Rust] DuckDBEngine
          → init_extensions()
          → ATTACH 'url' AS ext_name (TYPE mysql/postgres/sqlite)
          → execute(user SQL)
          → DETACH IF EXISTS (best-effort)
          → Arrow RecordBatch → JSON
    → storeResult()
```

### 4.3 配置读写链路

```
┌─ 读链路 ─────────────────────────────────────────────────────────┐
│                                                                  │
│  appStore.initialize()                                           │
│    → Store.load('global-settings.json')                          │
│    → zod 逐字段校验 → globalConfig.value                         │
│    → Store.load('project-{id}-settings.json')                    │
│    → zod 逐字段校验 → projectConfig.value                        │
│                                                                  │
│  effectiveEditorSettings (computed)                              │
│    = DEFAULT_EDITOR_SETTINGS                                     │
│    ⋈ globalConfig.editorSettings                                 │
│    ⋈ projectConfig.editorSettings (projectOverridable: true)     │
│                                                                  │
│  SqlEditorPanel.onMounted                                       │
│    → read effectiveEditorSettings → init refs                    │
│    → createEditor() → setFontSize/setTabSize/... (立即应用)       │
│                                                                  │
│  watch(effectiveEditorSettings)  ← 外部配置变更自动同步           │
└──────────────────────────────────────────────────────────────────┘

┌─ 写链路 ─────────────────────────────────────────────────────────┐
│                                                                  │
│  用户调整设置 → applyXxx()                                       │
│    → Monaco updateOptions()  (实时生效)                           │
│    → persistEditorSettings()                                     │
│      → appStore.saveConfig('editorSettings', payload, 'global')  │
│        → Store.set('editorSettings', payload)                    │
│        → Store.save() → global-settings.json                     │
│          { "editorSettings": { "fontSize": 16, ... } }            │
│        → globalConfig.value.editorSettings = payload  (内存同步)  │
│                                                                  │
│  SettingsPanel (全局设置页)                                       │
│    → appStore.saveBatch([...])                                   │
│      → Theme + Language + EditorSettings + DefaultEngine         │
│    → appStore.applyTheme()                                       │
└──────────────────────────────────────────────────────────────────┘
```

---

## 5. 状态管理矩阵

### 5.1 Store 职责分工

| Store | 文件 | 作用域 | 职责 |
| ----- | ---- | ------ | ---- |
| **useAppStore** | `@/stores/useAppStore` | 全局 | 主题、语言、编辑器设置、默认引擎、最近项目、布局 |
| **resultStore** | `workbench/stores/result-store` | 工作台 | 结果 Tab 管理、分页、三模式过滤 |
| **sqlExecutionStore** | `workbench/stores/sql-execution-store` | 工作台 | 执行结果分发、新 Tab 请求、刷新请求 |
| **layoutStore** | `workbench/stores/layout-store` | 工作台 | 面板布局、分割位置 |
| **insightStore** | `workbench/stores/insight-store` | 工作台 | 列洞察数据 |
| **workbenchStore** | `workbench/stores/workbench-store` | 工作台 | 面板管理、编辑器状态 |

### 5.2 通信机制选择

| 场景 | 机制 | 原因 |
| ---- | ---- | ---- |
| 执行结果 → 结果面板 | **Pinia Store** (resultStore) | 生产者/消费者解耦，支持多编辑器 |
| 编辑器 → 工具栏 | **Props + Emits** | 父子组件直接通信 |
| 编辑器 → 状态栏 | **Composable 共享 ref** | 同一编排层内共享 |
| 历史面板 → 编辑器 | **CustomEvent** (`sql-history-re-execute`) | 跨 dockview 面板通信（临时方案，待迁移至 Pinia） |
| 片段面板 → 编辑器 | **CustomEvent** (`insert-snippet`) | 同上 |
| 全局设置 → 所有编辑器 | **Pinia Store** (useAppStore) + watch | 一对多广播 |

### 5.3 待迁移的 CustomEvent

> 当前 2 个残留 CustomEvent 由 Phase 3 双通道过渡时期遗留，建议迁移：

| 事件 | 当前方式 | 建议迁移至 |
| ---- | -------- | ---------- |
| `sql-history-re-execute` | CustomEvent | `sqlExecutionStore.reExecuteRequest` |
| `insert-snippet` | CustomEvent | provide/inject 或 `sqlExecutionStore.insertSnippetRequest` |

---

## 6. 配置系统设计

### 6.1 CONFIG_REGISTRY 现状与扩展

当前 `config.ts` 定义了 7 个配置键。Workbench SettingsPanel 有 6 个 Tab 的设置全部游离在外。

#### 6.1.1 已有键 (7 个)

```typescript
CONFIG_REGISTRY = {
  theme,             // 'light' | 'dark' | 'system'
  language,          // 'zh-CN' | 'en'
  editorSettings,    // { fontSize, tabSize, wordWrap, minimap, lineNumbers, fontFamily }
  defaultEngine,     // 'native' | 'duckdb'
  recentProjects,    // string[]
  dockviewLayout,    // SerializedDockviewLayout   (projectOnly)
  sidebarState,      // SerializedSidebarState     (projectOnly)
}
```

#### 6.1.2 待新增键 (6 个 — 覆盖 Workbench SettingsPanel)

```typescript
// 连接池设置
connectionPool: {
  key: 'connectionPool',
  default: {
    maxConnections: 10,
    minIdleConnections: 2,
    connectionTimeout: 30,        // 秒
    idleTimeout: 300,             // 秒
    autoReconnect: true,
    healthCheck: true,
    healthCheckInterval: 60,      // 秒
  },
  rule: { globalDefault: true, projectOverridable: false, projectOnly: false },
}

// 历史设置
historySettings: {
  key: 'historySettings',
  default: {
    maxHistoryItems: 100,
    retentionDays: 30,
    enableHistory: true,
    includeSQL: true,
    enableUndo: true,
  },
  rule: { globalDefault: true, projectOverridable: false, projectOnly: false },
}

// 监控设置
monitoringSettings: {
  key: 'monitoringSettings',
  default: {
    enableMonitoring: true,
    updateInterval: 5,           // 秒
    enableAlerts: true,
    alertOnDisconnect: true,
    alertOnSlowQuery: true,
    slowQueryThreshold: 1000,    // 毫秒
  },
  rule: { globalDefault: true, projectOverridable: false, projectOnly: false },
}

// 性能设置
performanceSettings: {
  key: 'performanceSettings',
  default: {
    virtualScrollBuffer: 5,
    maxCacheSize: 100,           // MB
    cacheExpireMinutes: 60,
    enableLazyLoad: true,
    enablePreload: true,
  },
  rule: { globalDefault: true, projectOverridable: false, projectOnly: false },
}

// 外观设置 (UI 字体/紧凑模式，不包含编辑器字体和主题)
appearanceSettings: {
  key: 'appearanceSettings',
  default: {
    uiFontSize: 13,              // UI 字体大小 (非编辑器)
    compactMode: false,
  },
  rule: { globalDefault: true, projectOverridable: false, projectOnly: false },
}

// 结果面板设置
resultSettings: {
  key: 'resultSettings',
  default: {
    pageSize: 200,               // 默认分页大小
    defaultViewMode: 'grid',     // 'grid' | 'text' | 'chart'
    nullDisplay: 'NULL',
    dateFormat: 'YYYY-MM-DD HH:mm:ss',
  },
  rule: { globalDefault: true, projectOverridable: true, projectOnly: false },
}
```

#### 6.1.3 GlobalConfig / ProjectConfig 接口扩展

```typescript
interface GlobalConfig {
  theme: Theme
  language: Language
  editorSettings: EditorSettings
  defaultEngine: DefaultEngine
  recentProjects: string[]
  // 新增
  connectionPool: ConnectionPoolSettings
  historySettings: HistorySettings
  monitoringSettings: MonitoringSettings
  performanceSettings: PerformanceSettings
  appearanceSettings: AppearanceSettings
  resultSettings: ResultSettings
}

interface ProjectConfig {
  theme?: Theme
  editorSettings?: Partial<EditorSettings>
  defaultEngine?: DefaultEngine
  dockviewLayout?: SerializedDockviewLayout
  sidebarState?: SerializedSidebarState
  // 新增
  resultSettings?: Partial<ResultSettings>
}
```

### 6.2 两个 SettingsPanel 统一方案

```
现状:
  settings/.../SettingsPanel.vue  ──→ useAppStore.saveConfig()  ✅ 正确
  workbench/.../SettingsPanel.vue ──→ localStorage.setItem()     ❌ 游离

目标:
  ┌─────────────────────────────────────────────────┐
  │  全局设置页 (settings/SettingsPanel.vue)        │
  │  ├── 外观 (主题/语言/编辑器/UI字体/紧凑模式)    │
  │  ├── 编辑器 (字号/缩进/换行/行号/Minimap/字体)  │
  │  ├── 结果面板 (分页大小/默认视图/NULL显示/日期) │
  │  └── 默认引擎                                    │
  └─────────────────────────────────────────────────┘
           ↓ 所有保存通过 useAppStore.saveConfig()
  
  ┌─────────────────────────────────────────────────┐
  │  工作台设置页 (workbench/SettingsPanel.vue)     │
  │  ├── 连接池 (转入 config.ts CONFIG_REGISTRY)    │
  │  ├── 历史 (转入 config.ts CONFIG_REGISTRY)      │
  │  ├── 监控 (转入 config.ts CONFIG_REGISTRY)      │
  │  ├── 性能 (转入 config.ts CONFIG_REGISTRY)      │
  │  ├── 快捷键 (从 CONFIG_REGISTRY 读取)           │
  │  └── 外观 → 改为只读展示 + "在全局设置中修改"   │
  └─────────────────────────────────────────────────┘
           ↓ 所有保存通过 useAppStore.saveConfig()
```

---

## 7. 接口契约

### 7.1 Tauri 命令接口（18 条）

| 命令 | 输入类型 | 输出类型 | 前端入口 |
| ---- | -------- | -------- | -------- |
| `execute_sql` | `{ sql, conn_id }` | `QueryResult` | ✅ `queryService.executeSql()` |
| `execute_transaction` | `{ sqls, conn_id }` | `QueryResult[]` | ✅ `queryService.executeTransaction()` |
| `begin_transaction` | `{ conn_id }` | `void` | ✅ `queryService.beginTransaction()` |
| `commit_transaction` | `{ conn_id }` | `void` | ✅ `queryService.commitTransaction()` |
| `rollback_transaction` | `{ conn_id }` | `void` | ✅ `queryService.rollbackTransaction()` |
| `get_transaction_status` | `{ conn_id }` | `TransactionStatus` | ✅ `queryService.getTransactionStatus()` |
| `cancel_sql_query` | `{ conn_id }` | `void` | ✅ `queryService.cancelQuery()` |
| `execute_duckdb_accelerated` | `DuckDBAcceleratedInput` | `DuckDBAcceleratedResponse` | ✅ `queryService.executeDuckDBAccelerated()` |
| `get_sql_history` | `{ conn_id? }` | `SqlHistoryItem[]` | ✅ `queryService.getSqlHistory()` |
| `search_sql_history` | `{ query }` | `SqlHistoryItem[]` | ✅ `queryService.searchSqlHistory()` |
| `clear_sql_history` | `{ conn_id? }` | `void` | ✅ `queryService.clearSqlHistory()` |
| `remove_sql_history` | `{ id }` | `void` | ✅ `queryService.removeSqlHistory()` |
| `register_external_database` | `RegisterExternalDbInput` | `void` | ⚠️ **缺 UI** |
| `create_external_table` | `CreateExternalTableInput` | `void` | ⚠️ **缺 UI** |
| `parse_sql` | `{ sql }` | `ParseResult` | ✅ `parseSql()` |
| `format_sql` | `{ sql, dialect }` | `FormatResult` | ✅ `formatSql()` |
| `transpile_sql` | `{ sql, from, to }` | `TranspileResult` | ✅ `transpileSql()` |
| `validate_sql` | `{ sql, dialect }` | `ValidateResult` | ✅ `validateSql()` |
| `split_sql` | `{ sql }` | `SplitResult` | ✅ `splitSql()` |

### 7.2 Composable 接口

```typescript
// useMonacoEditor — Monaco 编辑器生命周期
function useMonacoEditor(options: {
  containerRef: Ref<HTMLElement | undefined>
  panelId: string
  initialValue?: string
  onContentChange?: (value: string) => void
  onCursorChange?: (pos: { lineNumber: number; column: number }) => void
  onSelectionChange?: (info: { lines: number; chars: number } | null) => void
}): {
  editor: Readonly<Ref<monaco.editor.IStandaloneCodeEditor | null>>
  getValue: () => string
  setValue: (value: string) => void
  getSelectedText: () => string
  insertText: (text: string) => void
  focus: () => void
  showWelcome: Ref<boolean>
  cursorPosition: Ref<string>
  selectedTextInfo: Ref<string>
  setFontSize: (size: number) => void
  setTabSize: (size: number) => void
  setWordWrap: (enabled: boolean) => void
  setLineNumbers: (enabled: boolean) => void
  setMinimap: (enabled: boolean) => void
  setFontFamily: (family: string) => void
  registerContextActions: (actions: IActionDescriptor[]) => IDisposable[]
}

// useSqlExecution — SQL 执行逻辑
function useSqlExecution(options: {
  panelId: string
  getEditorValue: () => string
  getSelectedText: () => string
  runtimeConnId: Ref<string>
  currentDatabaseType?: Ref<string | null>
  currentConnectionName?: Ref<string>
}): {
  executing: Ref<boolean>
  lastExecutionTime: Ref<number | null>
  hasResults: Ref<boolean>
  currentResultData: Ref<ResultData | null>
  inTransaction: Ref<boolean>
  statementCount: Ref<number>
  executeSql: (sql: string, connId: string) => Promise<ExecuteResult>
  execute: () => Promise<void>
  executeNew: () => Promise<void>
  executeBatch: () => Promise<void>
  explain: () => Promise<void>
  duckdbExecute: () => Promise<void>
  cancelExecution: () => void
  beginTransaction: () => Promise<void>
  commitTransaction: () => Promise<void>
  rollbackTransaction: () => Promise<void>
  storeResult: (result: ExecuteResult) => void
  scheduleParse: () => void
  checkForParams: (sql: string) => string[] | null
  buildBoundSql: (sql: string, params: Record<string, string>) => string
}

// useConnectionBinding — 连接管理
function useConnectionBinding(options: {
  panelId: string
  initialConnectionId?: string
}): {
  selectedConnection: Ref<string>
  runtimeConnId: Ref<string>
  connectionInfoText: Ref<string>
  popselectOptions: Ref<Array<{ label: string; value: string }>>
  isDuckDbConnection: Ref<boolean>
  currentDatabase: Ref<string>
  currentConnectionName: Ref<string>
  databaseType: Ref<string | null>
  ensureConnection: (connId: string) => Promise<boolean>
  onConnectionSelected: (connId: string) => void
}
```

### 7.3 结果面板 Composables

```typescript
// useResultTabs — 结果 Tab 管理
function useResultTabs(panelId: string): {
  tabs: Ref<ResultTab[]>
  activeTabId: Ref<string | null>
  activeTab: ComputedRef<ResultTab | null>
  addTab: (sql: string, connectionId: string) => void
  removeTab: (tabId: string) => void
  setActiveTab: (tabId: string) => void
}

// useResultExport — 结果导出
function useResultExport(activeTab: Ref<ResultTab | null>): {
  exportCSV: () => void
  exportJSON: () => void
  exportInsert: () => void
  exportParquet: () => Promise<void>
  exportXLSX: () => Promise<void>
  copyRowsAsInsert: () => void
}

// useFilterModes — 三模式过滤
function useFilterModes(activeTab: Ref<ResultTab | null>): {
  filterMode: Ref<FilterMode>
  quickFilter: Ref<string>
  sqlFilter: Ref<string>
  duckdbSql: Ref<string>
  applyQuickFilter: () => void
  applySqlFilter: () => Promise<void>
  applyDuckdbFilter: () => Promise<void>
  resetFilter: () => void
}
```

---

## 8. 功能模块设计

### 8.1 SQL 编辑模块

```
Monaco Editor 配置:
  language: 'sql'
  theme: 跟随 isDark (vs-dark / vs)
  fontSize:      14 (可配置, 10-28)
  tabSize:       2  (可配置, 1-8)
  wordWrap:      true (可配置)
  lineNumbers:   true (可配置)
  minimap:       true (可配置)
  fontFamily:    'Cascadia Code', 'Fira Code', 'Consolas', monospace (可配置)
  foldingStrategy: 'auto'
  showFoldingControls: 'always'
  renderWhitespace: 'selection'
  bracketPairColorization: { enabled: true }
  autoClosingBrackets: 'always'
  autoClosingQuotes: 'always'
  suggest: { showKeywords: true, showSnippets: true }

注册提供器 (Disposable 管理):
  ├── CompletionItemProvider  → 数据库 schema 补全 (TTL 5min)
  ├── FoldingRangeProvider    → SQL 语义折叠
  └── ContextActions × 2      → "执行选中 SQL" / "复制为 VALUES"
```

### 8.2 SQL 执行模块

```
执行模式:
  ├── 单语句执行 ──── Ctrl+Enter
  │   └── 检测 :param? → ParamBindingModal → bindParams() → execute
  ├── 新标签执行 ──── Ctrl+Shift+Enter
  │   └── 执行 → resultStore.addTab() → 新建 Tab (不覆盖现有)
  ├── 批量执行 ────── 工具栏 ListChecks 按钮
  │   └── splitSql() → forEach → addTab() → 汇总成功/失败
  ├── EXPLAIN ─────── 工具栏 FileSearch 按钮
  │   └── "EXPLAIN " + sql → execute → 新 Tab "执行计划"
  └── DuckDB 加速 ──── 工具栏 ⚡ 按钮
      └── rewriteDuckDBSQL() → executeDuckDBAccelerated()

性能策略:
  ├── 500ms 防抖解析 → statementCount 展示
  ├── 1000ms 防抖保存 → localStorage 草稿
  └── Abort 双通道取消 → JS AbortController + Rust CancellationToken
```

### 8.3 结果展示模块

```
QueryResultPanel:
  ├── 视图模式切换 ── grid (表格) / chart (图表) / text (输出)
  │   └── grid: AG Grid (虚拟滚动, 排序, 筛选, 分页 200条/页)
  │   └── chart: ECharts (bar/line/pie/scatter)
  │   └── text: 原始文本输出
  ├── 三模式过滤
  │   ├── quick: AG Grid 客户端快速过滤
  │   ├── sql:   WHERE 子句服务器端过滤
  │   └── duckdb: DuckDB 临时表分析
  ├── 导出 (5 种格式)
  │   ├── CSV    (ag-Grid exportDataAsCsv)
  │   ├── JSON   (Blob 下载)
  │   ├── INSERT (手动生成)
  │   ├── Parquet (DuckDB COPY TO)
  │   └── XLSX   (DuckDB COPY TO)
  ├── Inline Edit
  │   └── 双击 → 编辑 → dirty cells → save_cell_update → UPDATE
  └── Column Insight
      └── 列统计 (nullRatio, distinctRatio, min/max/mean/stddev, histogram)
```

---

## 9. 未实现功能技术方案

### 9.1 P0: 统一配置体系

**目标**：Workbench SettingsPanel 接入 `useAppStore`。

**步骤**：

```
1. config.ts 新增 6 个 CONFIG_REGISTRY 条目 (见 6.1.2)
2. GlobalConfig 接口新增 6 个字段
3. zod schema 新增 6 个子 schema
4. useAppStore 新增 6 个 effectiveXxx computed
5. workbench/SettingsPanel.vue:
   - 替换 localStorage 为 useAppStore.saveConfig()
   - 外观 Tab 删除 theme/fontSize (与全局设置重复)
   - 快捷键 Tab: 配置键 shortcutBindings 实现真正可编辑
6. 删除 workbench SettingsPanel 中 100+ 行硬编码 resetSettings
```

### 9.2 P1: DuckDB 外部数据源 UI

**目标**：为 `register_external_database` 和 `create_external_table` 提供前端入口。

**技术方案**：

```
触发方式:
  NavigatorPanel.vue → DuckDB 连接右键菜单
    ├── "注册外部数据库..." → ExternalDbDialog.vue
    └── "创建外部表..."     → CreateExternalTableDialog.vue

ExternalDbDialog.vue:
  ┌────────────────────────────────────┐
  │ 注册外部数据库                      │
  │                                    │
  │ 数据库类型: [MySQL ▼]              │
  │ 连接 URL:   [mysql://...      ]    │
  │ 数据库名:   [mydb              ]    │
  │                                    │
  │         [取消]  [注册]             │
  └────────────────────────────────────┘
    → invoke('register_external_database', {
        db_type: 'mysql',
        url: 'mysql://user:pass@host:3306/db',
        database: 'mydb'
      })

注册成功后:
  → ATTACH 自动执行
  → 外部表出现在导航树 DuckDB 连接下
  → 用户可直接 SELECT * FROM external_db.table
```

### 9.3 P1: 执行计划可视化

**目标**：将 EXPLAIN 文本转为交互式树形图。

**技术方案**：

```
解析层 (sql-editor-service.ts):
  new function: parseExplainResult(dbType, rawText) → ExplainTreeNode[]
  
  ExplainTreeNode {
    operation: string      // 'Seq Scan' | 'Index Scan' | 'Hash Join' | 'Sort' | ...
    cost: number           // 预估耗时
    rows: number           // 预估行数
    actualTime?: number    // 实际耗时 (EXPLAIN ANALYZE)
    children: ExplainTreeNode[]
  }

渲染层 (ExplainVisualization.vue):
  使用 ECharts tree series:
  - 节点大小 = 相对 cost
  - 节点颜色 = 性能分级 (绿→黄→红)
  - SCAN 节点标记扫描类型 (seq → 红色警告)
  - 悬浮 Tooltip 显示详细信息

触发入口:
  QueryResultPanel → 当 Tab 标题为 "执行计划" 时
    → viewMode 新增 'explain-tree'
    → 点击左侧树状图图标切换
```

### 9.4 P1: 批量导入 SQL 文件

**目标**：支持拖放/选择 `.sql` 文件到编辑器。

**技术方案**：

```
触发方式 (二选一或都支持):
  A. EditorToolbar 新增 "打开 SQL 文件" 按钮
     → Tauri dialog.open({ filters: [{ name: 'SQL', extensions: ['sql'] }] })
  B. 编辑器区域 @drop / @dragover 事件
     → 接受 .sql 文件

流程:
  选择文件 → Tauri fs.readTextFile(path)
    → editor.setValue(content)
    → 可选: 弹出确认框 "是否立即执行?"
      ├── 是 → splitSql() → 逐条 executeBatch()
      └── 否 → 仅加载到编辑器
```

### 9.5 P2: SQL 对比视图

**技术方案**：

```
使用 Monaco Diff Editor:
  monaco.editor.createDiffEditor(container, {
    readOnly: true,
    renderSideBySide: true,      // side-by-side 模式
    originalEditable: false,
  })

触发场景:
  SqlHistoryPanel → 选中两条历史记录 → 右键 "对比 SQL"
    → 打开 DiffEditorPanel (新 dockview Tab)
    → setModel({ original: sql1, modified: sql2 })

组件:
  DiffEditorPanel.vue:
    - 接收 originalSql + modifiedSql props
    - 使用 useMonacoEditor 的 createDiffEditor 变体
    - 左上角标题: "原 SQL (时间戳)" ← → "新 SQL (时间戳)"
```

### 9.6 P2: 编辑器主题微调

**技术方案**：

```
配置扩展 (EditorSettings):
  interface EditorSettings {
    // 现有
    fontSize: number
    tabSize: number
    wordWrap: boolean
    minimap: boolean
    lineNumbers: boolean
    fontFamily: string
    // 新增
    colorTheme: 'vs-dark' | 'vs' | 'hc-black' | 'custom'
    cursorStyle: 'line' | 'block' | 'underline'
    cursorBlinking: 'blink' | 'smooth' | 'phase' | 'expand' | 'solid'
    renderIndentGuides: boolean
    matchBrackets: 'always' | 'near' | 'never'
  }

预设主题 (settings/.../SettingsPanel.vue 新增):
  - Monokai (默认 dark)
  - Solarized Light
  - One Dark Pro
  - GitHub Light
  - 自定义 (颜色拾取器逐 token 配置)

实现:
  预设主题通过 monaco.editor.defineTheme() 注册
  自定义颜色存储到 editorSettings.customColors: Record<string, string>
  编辑器启动时根据 colorTheme 选择对应主题名
```

---

## 10. 文件结构清单

### 10.1 完整文件树

```
src/
├── stores/
│   ├── config.ts                      # CONFIG_REGISTRY (7 键 → 目标 13 键)
│   └── useAppStore.ts                 # 全局配置 Store (effectiveXxx computed)
│
├── extensions/builtin/workbench/
│   ├── ui/
│   │   ├── components/panels/
│   │   │   ├── SqlEditorPanel.vue     # 编排层 (~947L)
│   │   │   ├── EditorToolbar.vue      # 工具栏
│   │   │   ├── EditorStatusbar.vue    # 状态栏
│   │   │   ├── EditorWelcome.vue      # 欢迎页
│   │   │   ├── QueryResultPanel.vue   # 结果面板
│   │   │   ├── MultiTabResults.vue    # 多 Tab 结果
│   │   │   ├── OutputPanel.vue        # 输出视图
│   │   │   ├── DataVisualizationPanel.vue  # ECharts 图表
│   │   │   ├── ColumnInsightPanel.vue # 列洞察
│   │   │   ├── TranspileModal.vue     # 方言转换弹窗
│   │   │   ├── ParamBindingModal.vue  # 参数绑定弹窗
│   │   │   ├── SqlHistoryPanel.vue    # 执行历史
│   │   │   ├── SnippetPanel.vue       # 代码片段
│   │   │   ├── SettingsPanel.vue      # 工作台设置 (需重构)
│   │   │   └── SnippetPanel.vue       # 片段管理
│   │   │
│   │   ├── composables/
│   │   │   ├── useMonacoEditor.ts     # Monaco 生命周期
│   │   │   ├── useSqlExecution.ts     # SQL 执行逻辑
│   │   │   ├── useConnectionBinding.ts # 连接管理
│   │   │   ├── useDialectSync.ts      # 方言同步
│   │   │   ├── useEditorPersistence.ts # 草稿持久化
│   │   │   ├── useResultTabs.ts       # 结果 Tab
│   │   │   ├── useResultExport.ts     # 结果导出
│   │   │   ├── useResultDiff.ts       # 结果对比
│   │   │   ├── useGridConfig.ts       # AG Grid 配置
│   │   │   ├── useGridKeyboard.ts     # Grid 键盘
│   │   │   ├── useFilterModes.ts      # 三模式过滤
│   │   │   ├── useFilterPresets.ts    # 过滤预设
│   │   │   └── menuActionHandlers.ts  # 右键菜单
│   │   │
│   │   ├── stores/
│   │   │   ├── result-store.ts        # 结果 Tab + 过滤
│   │   │   ├── sql-execution-store.ts # 执行结果分发
│   │   │   ├── layout-store.ts        # 布局
│   │   │   ├── insight-store.ts       # 列洞察
│   │   │   └── workbench-store.ts     # 工作台状态
│   │   │
│   │   ├── types/
│   │   │   └── result.ts              # QueryResult / ResultTab / FilterMode / ...
│   │   │
│   │   └── views/
│   │       └── WorkbenchView.vue      # dockview 容器 + 全局快捷键
│   │
│   └── services/
│       ├── sql-editor-service.ts      # 补全/验证/格式化/解析/错误标记/折叠/参数
│       ├── sql-dialect-highlight.ts   # 方言 Monarch tokenizer
│       ├── sql-snippets.ts            # 代码模板库
│       └── sql-history-service.ts     # 执行历史
│
├── extensions/builtin/query/ui/services/
│   └── query.ts                       # Tauri IPC 封装 (18 命令)
│
├── extensions/builtin/settings/ui/components/
│   └── SettingsPanel.vue              # 全局设置页 (已接入 useAppStore)
│
└── shared/
    ├── types/
    │   ├── index.ts                   # Connection 等共享类型
    │   └── sql.ts                     # SqlDialect / DatabaseType
    └── locales/
        ├── zh-CN.json                 # 中文本地化
        └── en.json                    # 英文本地化
```

### 10.2 待新增文件

| 文件 | 用途 | 优先级 |
| ---- | ---- | ------ |
| `ExternalDbDialog.vue` | DuckDB 外部数据库注册弹窗 | P1 |
| `CreateExternalTableDialog.vue` | DuckDB 外部表创建弹窗 | P1 |
| `ExplainVisualization.vue` | 执行计划树形图组件 | P1 |
| `DiffEditorPanel.vue` | SQL 对比视图 | P2 |

---

## 附录

### 版本历史

| 版本 | 日期 | 说明 |
| ---- | ---- | ---- |
| v1.3 | 2026-05-09 | 遗漏修复：SQLite rows_to_arrow NULL 类型误判、useSqlExecution parseTimer 清理、WorkbenchTitleBar 桩动作 |
| v1.2 | 2026-05-09 | 驱动层审计修复：rows_to_arrow NULL 类型误判 × 2、PostgreSQL query_with_params、MySQL affected_rows、_db 命名 |
| v1.1 | 2026-05-09 | 审计修复：Rust unwrap × 2、runtimeConnId 类型、handleExplain 重构、QueryResultPanel any × 10 |
| v1.0 | 2026-05-09 | 初始设计文档，覆盖完整架构 + 未实现功能方案 |

### 审计修复历史 (2026-05-09)

| # | 修复项 | 文件 | 变更 |
|---|--------|------|------|
| 1 | `task.progress.lock().unwrap()` | [cache_warming_commands.rs:499](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/commands/cache_warming_commands.rs#L499) | → `lock().map_err(\|e\| ...)?` |
| 2 | `results.into_iter().next().unwrap()` | [stream_engine.rs:45](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/dbi/engine/stream_engine.rs#L45) | → `.next().ok_or_else(\|\| ...)?` |
| 3 | `runtimeConnId: Ref<string \| null>` | [useConnectionBinding.ts:19](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/composables/useConnectionBinding.ts#L19) | → `Ref<string>('')` |
| 4 | `binding.runtimeConnId as unknown as Ref<string>` | [SqlEditorPanel.vue:377](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/SqlEditorPanel.vue#L377) | → 移除不安全双重转换 |
| 5 | `handleExplain()` 手动 invoke | [SqlEditorPanel.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/SqlEditorPanel.vue) | → 委托 `executeExplain(t('...'))` |
| 6 | 新增 `executeExplain()` | [useSqlExecution.ts](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/composables/useSqlExecution.ts) | 统一执行链路 + 新建 Tab + 自定义标题 |
| 7 | 新增 `title?: string` 到 ExecutionResult | [sql-execution-store.ts](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/stores/sql-execution-store.ts) | 支持 EXPLAIN Tab 自定义标题 |
| 8 | QueryResultPanel `any` × 10 | [QueryResultPanel.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue) | → AG Grid 事件类型 + PresetSelectEvent + Record<string,unknown> |
| **Lint** | 审计前：**5 errors, 484 warnings** | 审计后：**0 errors, 469 warnings** | -15 warnings |

### 驱动层修复历史 (2026-05-09)

| # | 修复项 | 严重度 | 文件 | 变更 |
|---|--------|:--:|------|------|
| 1 | `postgres_rows_to_arrow()` 首行 NULL → 类型误判 → 数据丢失 | 🔴 P0 | [postgres.rs:L275-L363](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/native/postgres.rs#L275) | 两阶段重构：Phase 1 扫描非 NULL 值检测类型，Phase 2 统一收集 |
| 2 | `mysql_rows_to_arrow()` 首行 NULL → 类型误判 → 数据丢失 | 🔴 P0 | [mysql.rs:L355-L410](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/native/mysql.rs#L355) | 同上两阶段重构 |
| 3 | PostgreSQL 缺少 `query_with_params()` | 🟠 P1 | [postgres.rs:L86](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/native/postgres.rs#L86) | 新增 36 行实现，遵循 MySQL 参数绑定模式 |
| 4 | MySQL `affected_rows` 空写结果不一致 | 🟡 P2 | [mysql.rs:L53](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/native/mysql.rs#L53) | `None` → `if is_read_only { None } else { Some(0) }` |
| 5 | PostgreSQL `list_tables` `_db` 命名误导 | 🟡 P2 | [postgres.rs:L198](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/native/postgres.rs#L198) | `_db: &str` → `db: &str` |
| **Cargo** | 审计前：28 errors (预存) | 审计后：1 error (预存，非驱动) | 驱动层 0 新错误 |

### 遗漏修复历史 (Round 8, 2026-05-09)

| # | 修复项 | 严重度 | 文件 | 变更 |
|---|--------|:--:|------|------|
| 1 | `sqlite_rows_to_arrow()` 首行 NULL → 类型误判 → 数据丢失（与 postgres/mysql 同根因） | 🔴 P0 | [sqlite.rs:L401-L407](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/native/sqlite.rs#L401) | 移除 `Null` 分支中的 `detected_type = Some(DataType::Utf8)`，NULL 值不再错误地覆盖类型检测 |
| 2 | `useSqlExecution` parseTimer 未清理 → 组件卸载后定时器回调访问已销毁 ref | 🟡 P2 | [useSqlExecution.ts:L437-L442](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/composables/useSqlExecution.ts#L437) | 新增 `cleanup()` 函数，清除 parseTimer |
| 3 | `SqlEditorPanel` onBeforeUnmount 未调用 composable 清理 | 🟡 P2 | [SqlEditorPanel.vue:L862](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/SqlEditorPanel.vue#L862) | onBeforeUnmount 中添加 `cleanupExecution()` 调用 |
| 4 | `WorkbenchTitleBar` 6 个桩菜单动作 `console.log(...)` → 生产环境无意义输出 | 🟡 P2 | [WorkbenchTitleBar.vue:L274](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/WorkbenchTitleBar.vue#L274) | 统一替换为 `const notImplemented = (): void => {}` |
| **Lint** | 审计前：**0 errors, 469 warnings** | 修复后：**0 errors, 449 warnings** | -20 warnings |
| **Cargo** | 审计前：19 errors (预存) | 修复后：19 errors (全部预存，非本次变更) | 0 新错误 |

### 相关文档

| 文档 | 路径 |
| ---- | ---- |
| SQL 编辑器当前文档 | [SQL-EDITOR.md](./SQL-EDITOR.md) |
| 架构优化计划 | [SQL-EDITOR-OPTIMIZATION-PLAN.md](./SQL-EDITOR-OPTIMIZATION-PLAN.md) |
| 前端架构文档 | [ARCHITECTURE.md](./ARCHITECTURE.md) |
| 前端组件规范 | [COMPONENTS.md](./COMPONENTS.md) |
| 项目规则 | `.trae/rules/` |