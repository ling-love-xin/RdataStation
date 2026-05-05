# RdataStation 前端布局架构文档

## 1. 布局概述

RdataStation 采用**100% Dockview 驱动的完整布局架构**，所有面板（左侧栏、中心区、右侧栏、底部栏）都通过 Dockview 统一管理。

### 1.1 布局结构

```
┌─────────────────────────────────────────────────────────────┐
│                      标题栏 (TitleBar)                       │
├──────────┬───────────────────────────────────────┬──────────┤
│          │                                       │          │
│  左栏    │              中心区                   │  右栏    │
│(Dockview)│             (Dockview)                │(Dockview)│
│          │                                       │          │
├──────────┴───────────────────────────────────────┴──────────┤
│                     底部栏 (Bottom Panel)                    │
│                         (Dockview)                          │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 区域说明

| 区域 | 技术实现 | 说明 |
|------|----------|------|
| 标题栏 | Vue 组件 | 窗口控制 + 拖拽区域 |
| 左栏 | Dockview 面板 | 数据库导航、分析资源管理、插件管理、设置 |
| 中心区 | Dockview 面板 | SQL 编辑器、查询结果等核心工作区 |
| 右栏 | Dockview 面板 | SQL 历史、输出、列洞察 |
| 底部栏 | Dockview 面板 | 预留功能 |
| 状态栏 | Vue 组件 | 显示执行状态、统计信息等 |

---

## 2. 核心组件

### 2.1 组件层级

```
WorkbenchLayout (Vue 根布局组件)
├── WorkbenchTitleBar (标题栏 Vue 组件)
├── DockviewLayout (Dockview 主布局，100% 面板管理)
└── WorkbenchStatusBar (状态栏 Vue 组件)
```

### 2.2 关键文件

| 文件路径 | 职责 |
|---------|------|
| `src/extensions/builtin/workbench/ui/components/WorkbenchLayout.vue` | 全局布局容器，整合所有组件 |
| `src/extensions/builtin/workbench/ui/components/WorkbenchTitleBar.vue` | 标题栏组件 |
| `src/extensions/builtin/workbench/ui/components/WorkbenchStatusBar.vue` | 状态栏组件 |
| `src/extensions/builtin/workbench/ui/components/DockviewLayout.vue` | Dockview 核心布局，面板的加载和布局管理 |
| `src/extensions/builtin/workbench/ui/stores/layout-store.ts` | 布局状态管理（Pinia） |
| `src/core/panel-registry.ts` | 面板注册表 |
| `src/core/window-api.ts` | 窗口 API，提供面板注册功能 |
| `src/core/vue-app-manager.ts` | Vue 应用管理器 |

---

## 3. 面板注册和加载

### 3.1 组件命名约定

为了避免与 HTML 内置元素名称冲突（如 `output`、`select` 等），所有通过 `window-api.ts` 注册的 Dockview 面板组件都会添加 `panel_` 前缀：

- **面板 ID**：`output`（原始 ID）
- **全局组件名**：`panel_output`（用于 Dockview 查找）

### 3.2 面板注册表

所有面板都通过 `window-api.ts.registerViewProvider` 注册，注册时指定：
- `id`：面板唯一 ID
- `component`：Vue 组件（或异步组件工厂）
- `title`：面板标题
- `location`：面板位置（`left`/`right`/`center`/`bottom`）
- `order`：面板顺序

### 3.3 注册流程

```
extension.ts 注册面板
  ↓
window-api.ts.registerViewProvider(id, config)
  ↓
构建全局组件名：`panel_${id}`
  ↓
调用 vue-app-manager.registerGlobalComponent()
  ↓
panel-registry 保存面板描述符
  ↓
DockviewLayout 使用 `panel_${panel.id}` 作为 component name 加载
```

---

## 4. DockviewLayout 初始化

```typescript
const onReady = (event: DockviewReadyEvent) => {
  const api = event.api
  layoutStore.setDockviewApi(api)

  // 获取所有面板并按位置分组
  const panels = panelRegistry.getAll()
  const leftPanels = panels.filter(p => p.location === 'left')
  const rightPanels = panels.filter(p => p.location === 'right')
  const centerPanels = panels.filter(p => p.location === 'center')
  const bottomPanels = panels.filter(p => p.location === 'bottom')

  // 1. 创建第一个中心面板作为锚点
  if (centerPanels.length > 0) {
    const firstCenterPanel = centerPanels[0]
    api.addPanel({
      id: `panel_${firstCenterPanel.id}`,
      component: `panel_${firstCenterPanel.id}`,
      title: firstCenterPanel.name,
      minimumWidth: 400
    })
  }

  // 2. 创建左侧边栏
  if (leftPanels.length > 0) {
    // 以中心面板为参考，向左添加面板
  }

  // 3. 创建右侧边栏
  if (rightPanels.length > 0) {
    // 以中心面板为参考，向右添加面板
  }

  // 4. 创建底部面板
  if (bottomPanels.length > 0) {
    // 以中心面板为参考，向下添加面板
  }
}
```

---

## 5. 布局状态管理

### 5.1 LayoutStore 状态

```typescript
// 可见性状态
menuBarVisible: boolean
leftActivityBarVisible: boolean
rightActivityBarVisible: boolean
primarySideBarVisible: boolean
secondarySideBarVisible: boolean
panelVisible: boolean
statusBarVisible: boolean

// 展开状态
primarySideBarExpanded: boolean
secondarySideBarExpanded: boolean

// 尺寸状态
primarySideBarWidth: number
secondarySideBarWidth: number
panelHeight: number

// 面板配置
panelConfigs: Map<string, PanelConfig>
floatingPanels: any[]

// Dockview 布局数据（完全托管）
layoutData: any
```

### 5.2 PanelConfig 接口

```typescript
interface PanelConfig {
  location: 'left' | 'right' | 'center' | 'bottom' | 'floating'
  groupId?: string
  isVisible: boolean
  order: number
}
```

### 5.3 核心方法

| 方法 | 说明 |
|------|------|
| `setDockviewApi()` | 设置 Dockview API 引用 |
| `setLayoutData()` | 设置布局数据（由 dockview 完全托管） |
| `updatePanelConfig()` | 更新面板位置/可见性配置 |
| `activatePanel()` | 激活指定面板 |
| `resetLayout()` | 重置布局为默认值 |
| `saveLayoutConfig()` | 保存布局配置到 localStorage |
| `loadLayoutConfig()` | 从 localStorage 加载布局配置 |

---

## 6. Dockview 集成

### 6.1 工作范围

Dockview **负责整个布局的面板管理**，包括左栏、中心区、右栏、底部栏。

### 6.2 面板事件监听

```typescript
function setupDockviewListeners(api: DockviewApi) {
  // 面板激活变化
  api.onDidActivePanelChange((panel) => {
    console.log('Active panel changed:', panel?.id)
  })

  // 面板关闭
  api.onDidRemovePanel((panelId) => {
    console.log('Panel closed:', panelId)
    layoutStore.updatePanelConfig(panelId, { isVisible: false })
  })

  // 布局变化（保存到 store）
  api.onDidAddGroup(() => saveLayoutToStore())
  api.onDidRemoveGroup(() => saveLayoutToStore())
  api.onDidActiveGroupChange(() => saveLayoutToStore())
}
```

---

## 7. 布局持久化

### 7.1 保存布局

布局数据（包括面板位置、尺寸、分组等）会在布局变化时自动保存到 `localStorage`，键为 `rdata_station_layout_config`。

### 7.2 加载布局

应用启动时会尝试从 `localStorage` 恢复布局配置，有效期为 30 天。

### 7.3 布局配置接口

```typescript
interface LayoutConfig {
  visibility: { ... }
  selection: { ... }
  sizes: { ... }
  panelConfigs: Record<string, PanelConfig>
  layoutData: any  // Dockview 完整布局数据
  timestamp: number
}
```

---

## 8. 尺寸约束

| 区域 | 最小宽度 | 最大宽度 | 默认宽度 |
|------|----------|----------|----------|
| 左侧栏/右侧栏 | 200px | 600px | 300px |

---

## 9. 相关文档

- [组件规范文档](./COMPONENTS.md)
- [前端架构文档](./ARCHITECTURE.md)
