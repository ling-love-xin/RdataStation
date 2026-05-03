---
name: "frontend-enterprise-spec"
description: "RdataStation 企业级前端一体化规范，包含目录结构、布局设计、UI主题、组件规范、TS规范、Tauri桌面客户端规则、数据库工作台交互规范。Invoke when developing frontend components, layouts, or UI features for the RdataStation desktop database tool."
---

# RdataStation 企业级前端一体化规范

## 定位
适用于 **Tauri + Vue3 + TS + dockview-vue + Monaco** 桌面数据库工具
目标：专业、统一、无割裂、可长期维护、可团队协作

---

# 1. 目录结构（强制企业级）

```
src/
├── api/              # Tauri 通信 / 数据请求层
├── assets/
│   └── styles/       # 主题变量、全局样式
├── components/
│   ├── common/       # 全局通用组件（Button/Modal/Table）
│   └── layout/       # 布局辅助组件
├── composables/      # 业务 hooks（useSql、useConnection、useTheme）
├── config/           # 应用配置
├── constants/        # 枚举、常量
├── directives/       # 指令
├── layouts/          # 全局布局（MainLayout）
├── router/           # 路由
├── stores/
│   └── modules/      # Pinia 按业务模块化
├── types/            # TS 类型
├── utils/            # 工具函数
├── views/
│   ├── connection/   # 连接管理
│   ├── workbench/    # SQL 工作台（核心）
│   └── settings/     # 设置
├── App.vue
└── main.ts
```

---

# 2. 布局设计规范（frontend-design 强化）

## 2.1 全局布局结构

```
MainLayout
├── Header（36px，自定义标题栏 + 拖拽区）
├── AppContainer
│   └── 路由页面
└── StatusBar（底部状态栏）
```

## 2.2 Workbench 工作台布局

- **上部**：SQL 编辑器区域（默认 60%）
  - Query Tabs
  - Editor Toolbar
  - Monaco Editor
- **下部**：结果面板（可拖拽调整高度）
  - 结果表格 / 日志
- **右侧**：历史/搜索滑出面板

## 2.3 dockview-vue 规则

- 面板内边距：**12px**
- 间距：**8px**
- 分割线：**2px**
- 面板标题高度：**36px**
- 支持：拖拽、折叠、关闭、最大化、分栏
- **禁止过度嵌套、禁止面板混乱**

## 2.4 交互统一

- 双击标题栏 = 最大化/还原
- 面板拖拽实时响应
- 标签页支持关闭、固定、右键菜单
- 最小点击区域 **32px**
- 滚动条全局统一
- 全局无突兀跳转、无视觉割裂

---

# 3. UI 设计规范（ui-design 强化）

## 3.1 主题

- 双主题：dark / light
- 主色：**#165DFF**（专业数据蓝）
- 风格：克制、专业、长时间办公友好

## 3.2 颜色体系

```css
:root {
  /* 背景 3 级分层 */
  --bg-primary: #ffffff;      /* 主背景 */
  --bg-secondary: #f5f5f5;    /* 次级背景 */
  --bg-tertiary: #e8e8e8;     /* 第三级背景 */
  
  /* 文字 3 级梯度 */
  --text-primary: #333333;    /* 主文字 */
  --text-secondary: #666666;  /* 次级文字 */
  --text-tertiary: #999999;   /* 辅助文字 */
  
  /* 边框/分割线 */
  --border-color: #d9d9d9;
  
  /* 功能色 */
  --primary-color: #165DFF;
  --success-color: #00B42A;
  --warning-color: #FF7D00;
  --danger-color: #F53F3F;
}

.dark {
  --bg-primary: #1e1e1e;
  --bg-secondary: #252526;
  --bg-tertiary: #2d2d30;
  --text-primary: #cccccc;
  --text-secondary: #858585;
  --text-tertiary: #666666;
  --border-color: #3e3e42;
  --primary-color: #165DFF;
}
```

## 3.3 字体

- **界面**：Inter
- **代码/SQL**：JetBrains Mono

## 3.4 尺寸规范

- 基础单位：**4px**
- 标准间距：4/8/12/16/20/24/32
- 组件高度：**32px**
- 面板标题：**36px**
- 圆角：2px/4px/6px

## 3.5 组件规范

| 组件 | 规范 |
|------|------|
| 按钮 | 32px 高度，圆角 4px |
| 输入框 | 32px 高度 |
| 表格 | 行高 36px |
| 标签页 | 紧凑统一 |
| 图标 | 线性、2px 线条、16/20/24px |

## 3.6 无割裂原则

- 区域过渡自然
- 标题栏与主体视觉连续
- 无边框窗口无缝融合
- **禁止突兀色块、乱阴影、乱边框**

---

# 4. 桌面客户端规范（desktop-app-design 强化）

## 4.1 Tauri 窗口

- 无边框窗口
- 最小尺寸：**1080×720**
- 支持最小化、最大化、关闭、缩放、拖拽
- 主题自动同步窗口样式

## 4.2 标题栏

- 高度：**36px**
- 全局 `data-tauri-drag-region`
- 右侧窗口控制按钮
- 与界面视觉完全融合，不脱节

## 4.3 桌面交互

- 双击标题栏最大化
- 窗口边缘缩放
- 右键系统菜单
- 平滑动画、无闪烁
- 符合原生桌面体验

---

# 5. 组件开发规范

## 5.1 SFC 结构

```vue
<template>
  <!-- 模板 -->
</template>

<script setup lang="ts">
// 逻辑
</script>

<style scoped>
/* 样式 */
</style>
```

## 5.2 Props 必须强类型

```typescript
interface Props {
  id: string
  label?: string
  type?: 'primary' | 'secondary'
}

defineProps<Props>()
```

## 5.3 Emits 必须显式

```typescript
defineEmits<{
  run: [sql: string]
  cancel: []
}>()
```

## 5.4 拆分原则

- 单文件 **< 300 行**
- 页面私有组件放在 `views/xxx/components`
- 通用组件下沉到 `components/common`

---

# 6. TypeScript 规范

- **禁止 any**
- 接口 `IXxx` 格式
- 枚举 PascalCase
- 全局类型放入 `types/`
- 所有请求/状态必须标注类型

---

# 7. 业务分层规范（核心架构）

| 层级 | 职责 | 位置 |
|------|------|------|
| 页面 | 只做展示 | `views/` |
| 逻辑 | 业务 hooks | `composables/` |
| 状态 | Pinia stores | `stores/` |
| 请求 | API 层 | `api/` |
| 工具 | 工具函数 | `utils/` |
| UI | 组件 | `components/` |

**禁止在页面写请求，禁止在组件写复杂业务。**

---

# 8. 数据库工作台专属规则

## 8.1 SQL 编辑器

- Monaco 主题统一
- 格式化、执行、取消、explain 统一布局
- 错误面板自动吸附
- 多行选中、语法高亮规范

## 8.2 结果表格

- 虚拟滚动、筛选、排序、导出
- 大数据量不卡顿
- 空状态、加载状态、错误状态统一

## 8.3 连接管理

- 连接列表、测试、删除、编辑统一交互
- 数据库树结构支持展开/加载/刷新

## 8.4 工作台体验

- 执行状态实时显示
- 耗时、行数在状态栏展示
- 日志、消息、通知统一

---

# 9. 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 文件 | kebab-case | `sql-editor.vue` |
| 组件 | base-xxx.vue | `base-button.vue` |
| hooks | use-xxx.ts | `use-sql.ts` |
| 页面 | index.vue | `index.vue` |
| 变量 | camelCase | `currentTab` |
| 常量 | UPPER_SNAKE_CASE | `MAX_RETRY` |
| 路由 | kebab-case | `/sql-editor` |

---

# 10. AI 输出约束（强制执行）

1. **严格遵循本一体化规范**
2. 不破坏业务逻辑
3. 输出可直接替换、可编译代码
4. 结构、UI、交互、主题全局统一
5. 无视觉割裂、无风格混乱
6. 符合 Tauri 桌面数据库工具专业气质
7. 企业级可维护、可扩展

---

# 11. 常用代码模板

## 11.1 组件模板

```vue
<template>
  <div class="component-name">
    <!-- 内容 -->
  </div>
</template>

<script setup lang="ts">
interface Props {
  // 定义 props
}

defineProps<Props>()

defineEmits<{
  // 定义 emits
}>()
</script>

<style scoped>
.component-name {
  /* 样式 */
}
</style>
```

## 11.2 Composable 模板

```typescript
import { ref, computed } from 'vue'

export function useXxx() {
  const state = ref('')
  
  const computedValue = computed(() => {
    return state.value
  })
  
  function action() {
    // 动作
  }
  
  return {
    state,
    computedValue,
    action
  }
}
```

## 11.3 Store 模板

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useXxxStore = defineStore('xxx', () => {
  // State
  const items = ref<Item[]>([])
  
  // Getters
  const count = computed(() => items.value.length)
  
  // Actions
  function add(item: Item) {
    items.value.push(item)
  }
  
  return {
    items,
    count,
    add
  }
})
```
