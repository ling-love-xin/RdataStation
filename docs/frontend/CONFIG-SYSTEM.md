# RdataStation 配置系统文档

> 版本：v1.0
> 更新日期：2026-05-07
> 状态：✅ Phase 1 全局配置链完成，Phase 2 项目配置链 API 就绪

---

## 目录

1. [概述](#概述)
2. [三层配置优先级](#三层配置优先级)
3. [配置项可覆盖性矩阵](#配置项可覆盖性矩阵)
4. [架构设计](#架构设计)
5. [Store 实例生命周期](#store-实例生命周期)
6. [文件结构](#文件结构)
7. [核心 API](#核心-api)
8. [初始化顺序](#初始化顺序)
9. [设置面板交互逻辑](#设置面板交互逻辑)
10. [项目覆盖 diff 模式](#项目覆盖-diff-模式)
11. [Schema 版本管理](#schema-版本管理)
12. [未来迁移路径](#未来迁移路径)
13. [验收标准](#验收标准)
14. [变更历史](#变更历史)

---

## 概述

RdataStation 配置系统采用**三层单向 fallback** 架构，通过 `tauri-plugin-store` 将配置持久化为 JSON 文件。所有读写操作统一经过 `useAppStore`（Pinia），组件不直接访问底层存储。

设计目标：
- **10 年生命周期**：接口稳定，向前兼容
- **迁移就绪**：`saveConfig()` 抽象层隔离存储实现，未来切 Rust 后端零改动前端
- **单层扁平**：配置对象嵌套 ≤ 2 层，兼容 JSON 文件与 SQLite 键值对

---

## 三层配置优先级

```
查询配置值的优先级（从高到低）：
  Level 1: 项目覆盖值     project-settings.json
       ↓ 如果不存在
  Level 2: 全局默认值     global-settings.json
       ↓ 如果不存在
  Level 3: 系统硬编码      DEFAULT_GLOBAL_CONFIG (src/stores/config.ts)
```

### 关键规则

| 规则 | 行为 |
|------|------|
| 单向 fallback | 只向下查找，不存在反向继承 |
| 显式删除 | `resetProjectOverride(key)` 删除后自动退回 Level 2 |
| 首次启动 seed | Level 3 值在首次启动时写入 Level 2，之后 Level 2 永远存在 |
| 手动 save | `Store.save()` 需手动调用，不会自动保存 |

---

## 配置项可覆盖性矩阵

| 配置键 | 全局级 | 项目可覆盖 | 说明 |
|--------|--------|-----------|------|
| `theme` | ✅ 提供默认 | ✅ 可覆盖 | 用户可能用不同主题区分多项目 |
| `language` | ✅ 提供默认 | ❌ 不可覆盖 | 语言是个人偏好，不随项目变化 |
| `editorSettings` | ✅ 提供默认 | ✅ 可覆盖 | 不同文件类型可能需要不同缩进 |
| `defaultEngine` | ✅ 提供默认 | ✅ 可覆盖 | 生产项目用原生，探索项目用 DuckDB |
| `dockviewLayout` | ❌ | ✅ 仅项目 | 每个项目独立布局 |
| `sidebarState` | ❌ | ✅ 仅项目 | 每个项目独立侧边栏状态 |
| `recentProjects` | ✅ 仅全局 | ❌ | 最近项目列表是个人历史 |

---

## 架构设计

```
┌──────────────────────────────────────────┐
│              useAppStore                  │
│        (单一数据入口 - Pinia)             │
│                                          │
│  globalStoreRef: shallowRef<Store>       │  ← Pinia devtools 可见
│  projectStoreRef: shallowRef<Store>      │
│                                          │
│  getConfig(key)  ──── 优先级合并────┐    │
│  saveConfig(k,v,scope) ←── 写入抽象层│    │
│  resetProjectOverride(key)          │    │
└──────────┬───────────────────────────┬───┘
           │                           │
    ┌──────▼──────┐            ┌──────▼────────┐
    │ globalStore  │            │ projectStore   │
    │ (永久)       │            │ (随项目生命周期)│
    │              │            │                │
    │ global-      │            │ project-{id}-  │
    │ settings.json│            │ settings.json  │
    └─────────────┘            └────────────────┘
```

### 单一数据入口

所有配置读写通过 `useAppStore`。组件读取：
```typescript
const appStore = useAppStore()
const isDark = appStore.isDark              // computed, 含优先级合并
const fontSize = appStore.effectiveEditorSettings.fontSize
```

组件写入：
```typescript
await appStore.setTheme('light')            // 内部走 saveConfig
await appStore.saveConfig('theme', 'light', 'global')
```

---

## Store 实例生命周期

| Store 实例 | 创建时机 | 释放时机 |
|-----------|---------|---------|
| `globalStore` | `main.ts` 中 `initialize()` 调用 | 应用关闭时 Tauri 自动清理 |
| `projectStore` | 用户打开或创建项目时 `openProject()` | 关闭项目或切换项目时 `closeProject()` |

### projectStore 切换协议

```
openProject(A)
  1. closeProject() → 保存并释放旧 projectStore
  2. Store.load(project-{A}-settings.json)
  3. projectStoreRef.value = 新 Store 实例
  4. projectConfig.value = 加载的项目覆盖值
```

---

## 文件结构

```
src/
├── stores/
│   ├── config.ts          # 类型定义 + 配置注册表 + 系统默认值
│   └── useAppStore.ts     # 核心 Pinia Store（唯一写入入口）
├── shared/
│   ├── stores/
│   │   └── ui.ts          # useUiStore（委派到 useAppStore）
│   └── styles/
│       ├── theme-tokens.ts # naive-ui 主题颜色令牌
│       ├── theme.css
│       ├── dockview-theme.css
│       └── ...
├── app/
│   ├── main.ts            # 初始化顺序（Pinia → 全局配置 → mounts）
│   └── App.vue            # 主题/语言应用入口
└── extensions/
    └── builtin/
        ├── settings/
        │   └── ui/components/SettingsPanel.vue  # 设置面板 UI
        └── workbench/
            └── ui/components/panels/SettingsPanel.vue
```

---

## 核心 API

### useAppStore

```
字段:
  globalConfig        Ref<GlobalConfig>   全局配置状态
  projectConfig       Ref<ProjectConfig>  项目覆盖状态
  initialized         Ref<boolean>        是否完成初始化
  projectOpen         Ref<boolean>        是否有项目打开
  projectPath         Ref<string|null>    当前项目路径

计算属性（含优先级合并）:
  effectiveTheme        Theme       project.theme ?? global.theme
  isDark                boolean     含 system 主题系统检测
  effectiveLanguage     Language    global.language
  effectiveEditorSettings  EditorSettings  三层合并: DEFAULT ← global ← project
  effectiveDefaultEngine   DefaultEngine    project ?? global
  recentProjects        string[]    global.recentProjects
  effectiveDockviewLayout  SerializedDockviewLayout | undefined
  effectiveSidebarState    SerializedSidebarState | undefined

核心方法:
  initialize()                      加载 global-settings.json → 内存
  openProject(path)                 加载 project-{id}-settings.json → 内存
  closeProject()                    保存并释放 projectStore
  saveConfig(key, value, scope) → SaveResult     写入抽象层
  resetProjectOverride(key)         删除项目覆盖，退回 Level 2
  hasProjectOverride(key) → bool    项目是否有覆盖值

便捷方法:
  setTheme(theme, scope?) → SaveResult
  setLanguage(lang) → SaveResult
  setEditorSettings(settings, scope?) → SaveResult
  setDefaultEngine(engine, scope?) → SaveResult
  addRecentProject(id)
  removeRecentProject(id)
  saveDockviewLayout(layout) → SaveResult
  saveSidebarState(state) → SaveResult
  applyTheme()                      同步 dark class 到 <html>
```

### SaveResult

```typescript
interface SaveResult {
  success: boolean
  key: ConfigKey
  scope: ConfigScope          // 'global' | 'project'
  error?: string              // 失败原因
}
```

---

## 初始化顺序

`main.ts` 严格执行顺序：

```
1. createPinia()               ← Pinia 实例化
2. app.use(pinia)
3. app.use(router)
4. async main():
   a. useAppStore().initialize()   ← 加载 global-settings.json
   b. applyTheme()                 ← 立即应用持久化主题
   c. extensionHost.activateExtensions()  ← 扩展系统
   d. panelRegistry.getAll()              ← 注册全局组件
   e. app.mount('#app')                  ← Vue 挂载
```

---

## 设置面板交互逻辑

### “应用”按钮模式

```
用户修改 → localTheme/localEditorSettings（本地响应式，不持久化）
         → 点击「应用所有设置」
         → useAppStore.setTheme/setEditorSettings/...
         → saveConfig() → tauri-plugin-store 写入 JSON
         → Vue 响应式自动更新所有绑定组件
```

### “恢复全局默认值”逻辑

```
用户点击「恢复全局默认值」
  → resetProjectOverride(THEME)
  → projectStore.delete('theme')
  → projectStore.save()
  → delete projectConfig.theme
  → localTheme = effectiveTheme  (自动退回 Level 2)
  → UI 上恢复按钮消失
```

---

## 项目覆盖 diff 模式

`setEditorSettings(settings, 'project')` 使用 diff 模式：
只存储与全局默认值不同的字段，而非完整对象。

```
全局 editorSettings: { fontSize: 14, tabSize: 2, wordWrap: true, ... }
项目覆盖后:          { fontSize: 16 }  ← 只存差异

好处：
  1. 全局值变更时，未覆盖的字段自动跟随
  2. project-settings.json 更小更可读
  3. 语义明确 —— 「覆盖」vs「等于全局值」可区分
```

---

## Schema 版本管理

`global-settings.json` 顶层写入 `_schemaVersion` 字段：

```
首次启动: _schemaVersion = undefined → 写入 1
后续启动: _schemaVersion = 1 → 跳过

未来 v2:
  if _schemaVersion < 2 → migrateConfig(old, new)
```

migrateConfig 函数（预留）：
```typescript
function migrateConfig(from: number, to: number) {
  switch (to) {
    case 2: /* 例如：editorSettings.fontFamily 改名 */ break
  }
}
```

---

## 未来迁移路径

### 迁移目标

| 组件 | 当前 | 目标 |
|------|------|------|
| 全局配置 | `global-settings.json` | `global.db` → `app_config` 表 |
| 项目配置 | `project-{id}-settings.json` | 项目内 `project.meta.sqlite` → `project_config` 表 |

### 不变的部分

- 配置键名 `theme` / `language` / `editorSettings` / ... 保持不变
- Pinia Store 公开接口（computed + 方法签名）不变
- 设置面板 UI 零改动

### 仅需修改的代码

**唯一修改点**：`saveConfig()` 内部实现。

```
当前:
  await store.set(key, value)
  await store.save()

目标:
  await invoke('set_config', { key, value, scope })  // Rust 后端
```

`initialize()` 和 `openProject()` 同理，替换 `Store.load` 为 `invoke`。

### 迁移成本

| 工作 | 工时 |
|------|------|
| Rust 建表 + 写 Tauri Command | 1-2 天 |
| 前端替换 saveConfig 内部 | ≤ 半天 |
| 总计 | ≤ 2.5 天 |

---

## 验收标准

| 验收项 | 验证方式 | 状态 |
|--------|---------|------|
| 配置持久化 | 修改设置 → 关闭应用 → 重新打开 → 设置仍存在 | ✅ |
| 优先级合并 | 设置全局字体 14 → 设置项目字体 16 → 项目显示 16 → 清除项目覆盖 → 显示 14 | ✅ API 就绪 |
| 设置面板联动 | 修改全局字体 → 设置面板显示新值 → 编辑器更新 | ✅ |
| 项目隔离 | 修改项目 A 主题为亮色 → 切换到项目 B → 项目 B 仍为暗色 | ⬜ 待 AppLayout 集成 |
| 迁移就绪 | saveConfig() 是唯一写入入口，未来替换内部即可 | ✅ |

---

## 变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.0 | 2026-05-07 | 初始版本。Phase 1 全局配置链完成。安装 tauri-plugin-store，创建 config.ts + useAppStore.ts，重构 useUiStore/App.vue/SettingsPanel。添加统一 ConfigRegistry + 种子驱动表 + diff 模式项目覆盖 + schema 版本管理 + SaveResult 返回类型。 |
