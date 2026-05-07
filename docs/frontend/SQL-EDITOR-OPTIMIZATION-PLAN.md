# SQL 编辑器优化开发计划

> 版本：v1.0
> 创建日期：2026-05-08
> 状态：⏳ 待确认
> 依赖：无外部依赖，纯架构优化

---

## 📖 目录

- [优化目标](#优化目标)
- [现状分析](#现状分析)
- [Phase 1：类型与持久化统一](#phase-1类型与持久化统一)
- [Phase 2：组件拆分 + Composable 抽取](#phase-2组件拆分--composable-抽取)
- [Phase 3：通信机制重构](#phase-3通信机制重构)
- [Phase 4：体验增强](#phase-4体验增强)
- [文件变更清单](#文件变更清单)
- [验收标准](#验收标准)
- [风险控制](#风险控制)

---

## 优化目标

1. **组件轻量化**：将 1600+ 行的 `SqlEditorPanel.vue` 拆分为多个独立组件 + composables，单一文件不超过 300 行
2. **类型安全强化**：消除所有 `as any` 强制转换，统一连接/Dockview params/方言类型定义
3. **通信规范化**：用 Pinia Store + provide/inject 替代 7+ 个全局 CustomEvent
4. **持久化统一**：创建 `useEditorPersistence.ts` 统一管理 localStorage 读写
5. **可测试性提升**：每个 composable 可独立进行单元测试

---

## 现状分析

| 问题 | 严重程度 | 影响 |
|------|----------|------|
| SqlEditorPanel.vue 1600+ 行，职责过多 | 🔴 P0 | 维护困难、新功能风险高 |
| 7+ 个 CustomEvent 在 window 上广播 | 🔴 P0 | 隐式耦合、调试困难、内存泄漏风险 |
| `(conn as any).dbType` 松散类型遍布 | 🟡 P1 | 类型安全缺失、IDE 提示失效 |
| localStorage key 散落三处，无统一管理 | 🟡 P1 | 持久化逻辑碎片化 |
| 方言高亮每次完整重建 Monarch tokenizer | 🟢 P2 | 切换连接有微小性能开销 |
| 执行中的查询无法取消 | 🟢 P2 | 用户体验欠缺 |

---

## Phase 1：类型与持久化统一

> 🎯 目标：打好基础，不改变任何运行时行为，后续 Phase 的铺路石
> ⏱ 预估复杂度：低风险，2-3 天

### 1.1 统一方言类型定义

**问题**：`SqlDialect` 重复定义在 `sql-editor-service.ts` 和 `sql-dialect-highlight.ts`

**操作**：
- 新增 `src/shared/types/sql.ts`，定义 `SqlDialect`、`DatabaseType` 并统一导出
- 两个 service 文件改为 `import type { SqlDialect } from '@/shared/types/sql'`
- 删除各自文件中的重复定义

### 1.2 统一连接类型定义

**问题**：`Connection` 类型中 `dbType` 字段缺失，导致到处 `(conn as any).dbType`

**操作**：
- 在 `src/shared/types/index.ts` 中完善 `Connection` 接口，新增 `dbType: DatabaseType` 字段
- 确保 `connectionStore.connections` 类型为 `Connection[]`
- 逐步替换 `SqlEditorPanel.vue` 中的 `(conn as any).dbType` 为 `conn.dbType`

### 1.3 统一 Dockview Params 类型

**问题**：Dockview 嵌套 `params.params` 结构导致到处都是 `(props.params as any)?.params?.connectionId`

**操作**：
- 新增类型 `SqlEditorParams`：
  ```typescript
  interface SqlEditorParams {
    connectionId?: string
    databaseName?: string
    initialSql?: string
    panelId?: string
    schema?: string
  }
  ```
- 在 `SqlEditorPanel.vue` 中新增一个 `resolvedParams` computed 统一解包

### 1.4 统一 Tauri 响应类型

**操作**：
- 新增 `ExecuteSqlResponse` 接口，明确 `result` 结构
- 替换 `executeSingleStatement()` 中的 `result.result || result` 防御性写法

### 1.5 创建 useEditorPersistence.ts

**操作**：新增文件 `src/extensions/builtin/workbench/ui/composables/useEditorPersistence.ts`

**职责**：
- 统一 localStorage key 前缀 `rdata:workbench:`
- 提供 `draft.save()` / `draft.load()` / `draft.remove()`（替代散落的 `sql-draft-${panelId}` key）
- 提供 `clearOrphanDrafts()` 清理已关闭面板的过期草稿（7天 TTL）

---

## Phase 2：组件拆分 + Composable 抽取

> 🎯 目标：将大组件拆分为可独立维护的单元
> ⏱ 预估复杂度：中风险，3-4 天

### 2.1 拆分方案总览

```
SqlEditorPanel.vue              ← 仅保留布局编排 (~80行)
├── EditorToolbar.vue            ← 工具栏组件 (~120行)
├── EditorStatusbar.vue          ← 状态栏组件 (~80行)
├── EditorWelcome.vue            ← 欢迎页水印 (~40行)
├── TranspileModal.vue           ← 方言转换弹窗 (~50行)
├── composables/
│   ├── useEditorPersistence.ts  ← Phase 1 创建，Phase 2 集成
│   ├── useMonacoEditor.ts       ← Monaco 初始化/销毁/主题/快捷键 (~250行)
│   ├── useSqlExecution.ts       ← 执行逻辑 (单/多语句/Explain/DuckDB) (~200行)
│   ├── useConnectionBinding.ts  ← 连接选择/ensure/waitFor (~150行)
│   └── useDialectSync.ts        ← 方言监听 + 高亮补全注册/注销 (~60行)
```

### 2.2 步骤 2.1：抽取小组件（低风险先练手）

| 序号 | 操作 | 来源文件 | 目标文件 |
|------|------|----------|----------|
| 1 | 提取欢迎页水印 | SqlEditorPanel.vue:L155-L167 | `EditorWelcome.vue` |
| 2 | 提取方言转换弹窗 | SqlEditorPanel.vue:L108-L131 | `TranspileModal.vue` |

**EditorWelcome.vue 接口**：
```typescript
interface Props {
  visible: boolean
}
// 无 emits，纯展示组件
```

**TranspileModal.vue 接口**：
```typescript
interface Props {
  visible: boolean
  dialectOptions: Array<{ label: string; key: SqlDialect }>
}
interface Emits {
  (e: 'close'): void
  (e: 'transpile', targetDialect: SqlDialect): void
}
```

### 2.3 步骤 2.2：抽取 useMonacoEditor composable

**来源**：SqlEditorPanel.vue 中 `initEditor()` 全部逻辑 + `onUnmounted` 销毁逻辑 + 主题/内容 watch

**文件**：`src/extensions/builtin/workbench/ui/composables/useMonacoEditor.ts`

**API 设计**：
```typescript
export function useMonacoEditor(options: {
  containerRef: Ref<HTMLElement | undefined>
  panelId: string
  initialValue?: string
  language?: string
  theme?: string
  onContentChange?: (value: string) => void
  onCursorChange?: (position: { lineNumber: number; column: number }) => void
  onSelectionChange?: (info: { lines: number; chars: number } | null) => void
}) {
  // 返回
  return {
    editor: Readonly<Ref<monaco.editor.IStandaloneCodeEditor | null>>
    getValue: () => string
    setValue: (value: string) => void
    getSelectedText: () => string
    insertText: (text: string) => void
    focus: () => void
    showWelcome: Ref<boolean>
    cursorPosition: Ref<string>
    selectedTextInfo: Ref<string>
    // 快捷键注册
    registerCommand: (keybinding: number, handler: () => void) => void
  }
}
```

### 2.4 步骤 2.3：抽取编辑器子组件

**EditorToolbar.vue**

```typescript
interface Props {
  toolbarPosition: 'top' | 'left' | 'right'
  isDuckDb: boolean
}
interface Emits {
  (e: 'execute'): void
  (e: 'executeNew'): void
  (e: 'duckdbExecute'): void
  (e: 'explain'): void
  (e: 'format'): void
  (e: 'validate'): void
  (e: 'settings'): void
  (e: 'togglePosition'): void
}
```

**EditorStatusbar.vue**

```typescript
interface Props {
  cursorPosition: string
  selectedTextInfo: string
  editorMode: string
  executing: boolean
  lastExecutionTime: number | null
  connectionInfoText: string
  popselectOptions: Array<{ label: string; value: string }>
  selectedConnection: string
}
interface Emits {
  (e: 'connectionChange', connId: string): void
}
```

### 2.5 步骤 2.4：抽取业务 composables

**useSqlExecution.ts**

```typescript
export function useSqlExecution(options: {
  panelId: string
  getEditorValue: () => string
  getSelectedText: () => string
  selectedConnection: Ref<string>
  runtimeConnId: Ref<string>
  getCurrentDialect: () => SqlDialect
}) {
  return {
    executing: Ref<boolean>
    lastExecutionTime: Ref<number | null>
    hasResults: Ref<boolean>
    currentResultData: Ref<ResultData | null>
    execute: () => Promise<void>
    executeNew: () => Promise<void>
    explain: () => Promise<void>
    duckdbExecute: () => Promise<void>
    cancelExecution: () => void  // Phase 4 实现
  }
}
```

**useConnectionBinding.ts**

```typescript
export function useConnectionBinding(options: {
  panelId: string
  initialConnectionId?: string
  paramsConnectionId: Ref<string>
}) {
  return {
    selectedConnection: Ref<string>
    runtimeConnId: Ref<string>
    connectionInfoText: Ref<string>
    popselectOptions: Ref<Array<{ label: string; value: string }>>
    isDuckDbConnection: Ref<boolean>
    ensureConnection: (connId: string) => Promise<boolean>
    onConnectionSelected: (connId: string) => void
  }
}
```

**useDialectSync.ts**

```typescript
export function useDialectSync(options: {
  selectedConnection: Ref<string>
}) {
  return {
    currentDialect: Ref<SqlDialect>
    getCurrentDialect: () => SqlDialect
    updateDialectHighlight: () => void
  }
}
```

### 2.6 步骤 2.5：重构 SqlEditorPanel 为编排层

重构后的 `SqlEditorPanel.vue` 仅负责：
- 组合各 composables
- 将数据分发给子组件
- 处理编辑器与结果面板的分割布局

目标行数：~80 行

---

## Phase 3：通信机制重构

> 🎯 目标：用 Pinia Store + provide/inject 替代全局 CustomEvent
> ⏱ 预估复杂度：中风险，2-3 天

### 3.1 事件迁移表

| 当前事件 | 迁移方式 | 目标 |
|----------|----------|------|
| `sql-execution-result` | **Pinia Store** | `sqlExecutionStore.results` reactive Map |
| `query-result-new` | **Pinia Store** | `sqlExecutionStore.openInNewTab()` action |
| `query-result-refresh` | **provide/inject** | 父→子回调函数 |
| `query-result-export-insert` | **provide/inject** | 子→父回调函数 |
| `query-result-updated` | **Pinia Store** | `watch(executionResults)` 替代 |
| `open-settings-panel` | **Pinia Store** | `uiStore.openSettings()` action |
| `save-sql-file` | **provide/inject** | 编辑器→布局管理器 |

### 3.2 sql-execution-store.ts 扩展

新增 action：
```typescript
function openInNewTab(result: ExecutionResult): void {
  // 通知布局管理器新建结果面板
  newTabRequests.value.set(result.panelId, result)
}

function refreshResult(panelId: string): void {
  // 触发编辑器重新执行
  refreshRequests.value.set(panelId, Date.now())
}
```

### 3.3 迁移策略

每个事件采用 **双通道过渡** 方式：
1. 先添加新的 Pinia Store / provide-inject 通信
2. 保留旧的 CustomEvent 发送
3. 验证新通道工作正常
4. 移除旧通道

### 3.4 通信架构图

```
                    ┌──────────────────────┐
                    │     Pinia Store        │
                    │  ┌─────────────────┐   │
                    │  │ sqlExecutionStore│   │
                    │  │  - results       │   │
                    │  │  - newTabRequests│   │
                    │  │  - refreshRequests│  │
                    │  └─────────────────┘   │
                    │  ┌─────────────────┐   │
                    │  │ uiStore          │   │
                    │  │  - settingsOpen  │   │
                    │  └─────────────────┘   │
                    └──────┬───────┬─────────┘
                           │       │
                 watch/read│       │watch/read
                           │       │
           ┌───────────────┴──┐ ┌──┴────────────────┐
           │ SqlEditorPanel   │ │ QueryResultPanel   │
           │ (父组件)          │←│ (子组件)           │
           │                  │ │                    │
           │ provide:         │ │ inject:            │
           │  onRefresh       │ │  refresh()         │
           │  onExportInsert  │ │  exportInsert()    │
           └──────────────────┘ └────────────────────┘
```

---

## Phase 4：体验增强

> 🎯 目标：方言高亮优化 + Abort 查询取消
> ⏱ 预估复杂度：低风险，1-2 天

### 4.1 方言高亮增量更新

**当前**：每次切换连接调用 `setMonarchTokensProvider` 完整重建 tokenizer

**优化方案**：
- 将 tokenizer 分为基础规则 + 方言 overlay
- 基础规则（注释、字符串、数字、操作符）注册一次不变
- 方言切换时仅通过 Monaco Decorations API 增量注入方言关键字样式
- 或通过 Semantic Highlighting Provider 实现

**预期收益**：切换连接时 tokenizer 重建开销从 O(n) 降至 O(1)

### 4.2 Abort 查询取消

**API 层**：`queryService.cancelQuery()` 已存在（调用 `cancel_query` Tauri 命令）

**Store 层**：在 `useSqlExecution` 中新增 `cancelExecution()`：
```typescript
const abortControllers = new Map<string, AbortController>()

async function executeWithCancel(panelId, sql, connId) {
  const controller = new AbortController()
  abortControllers.set(panelId, controller)
  try {
    return await invoke('execute_sql', { ... })
  } finally {
    abortControllers.delete(panelId)
  }
}

function cancelExecution(panelId) {
  const controller = abortControllers.get(panelId)
  controller?.abort()
  queryService.cancelQuery(runtimeConnId.value)
}
```

**UI 层**：在状态栏"执行中"指示器旁边增加 `X` 取消按钮

---

## 文件变更清单

### 新增文件

| # | 文件 | 位置 | Phase |
|---|------|------|-------|
| 1 | `sql.ts` | `src/shared/types/sql.ts` | P1 |
| 2 | `useEditorPersistence.ts` | `src/extensions/builtin/workbench/ui/composables/useEditorPersistence.ts` | P1 |
| 3 | `EditorWelcome.vue` | `src/extensions/builtin/workbench/ui/components/panels/EditorWelcome.vue` | P2 |
| 4 | `TranspileModal.vue` | `src/extensions/builtin/workbench/ui/components/panels/TranspileModal.vue` | P2 |
| 5 | `EditorToolbar.vue` | `src/extensions/builtin/workbench/ui/components/panels/EditorToolbar.vue` | P2 |
| 6 | `EditorStatusbar.vue` | `src/extensions/builtin/workbench/ui/components/panels/EditorStatusbar.vue` | P2 |
| 7 | `useMonacoEditor.ts` | `src/extensions/builtin/workbench/ui/composables/useMonacoEditor.ts` | P2 |
| 8 | `useSqlExecution.ts` | `src/extensions/builtin/workbench/ui/composables/useSqlExecution.ts` | P2 |
| 9 | `useConnectionBinding.ts` | `src/extensions/builtin/workbench/ui/composables/useConnectionBinding.ts` | P2 |
| 10 | `useDialectSync.ts` | `src/extensions/builtin/workbench/ui/composables/useDialectSync.ts` | P2 |

### 修改文件

| # | 文件 | 变更内容 | Phase |
|---|------|----------|-------|
| 1 | `SqlEditorPanel.vue` | 重构为编排层，~1600行 → ~80行 | P2 |
| 2 | `sql-editor-service.ts` | 移除 `SqlDialect` 重复定义，导入共享类型 | P1 |
| 3 | `sql-dialect-highlight.ts` | 移除 `SqlDialect` 重复定义，导入共享类型 | P1 |
| 4 | `sql-execution-store.ts` | 新增 `openInNewTab`、`refreshResult` action | P3 |
| 5 | `shared/types/index.ts` | `Connection` 接口新增 `dbType: DatabaseType` | P1 |
| 6 | `sql-dialect-highlight.ts` | 增量高亮支持 | P4 |

### 删除内容

| # | 内容 | 来源 | Phase |
|---|------|------|-------|
| 1 | 7 个 CustomEvent 监听/发送 | SqlEditorPanel.vue | P3 |
| 2 | `SqlDialect` 重复定义 | sql-editor-service.ts, sql-dialect-highlight.ts | P1 |

### 不变的内容

以下模块**不在此次优化范围内**，保持原样：
- `QueryResultPanel.vue` — 结果展示面板
- `sql-snippets.ts` — SQL 模板库
- `sql-history-service.ts` — 执行历史
- `connection-store.ts` / `runtime-connection-store.ts` — 连接管理

---

## 验收标准

### Phase 1 验收

- [ ] `SqlDialect` 全局唯一定义在 `shared/types/sql.ts`
- [ ] `Connection.dbType` 为强类型 `DatabaseType`
- [ ] SqlEditorPanel 中不再有 `(conn as any).dbType`
- [ ] `useEditorPersistence` 可正常读写草稿
- [ ] `pnpm run lint` 通过
- [ ] `pnpm run format` 通过

### Phase 2 验收

- [ ] 5 个新组件均可独立渲染
- [ ] 4 个 composables 均可独立调用
- [ ] SqlEditorPanel.vue 行数 < 100
- [ ] 所有功能与拆分前完全一致：
  - SQL 编辑（语法高亮、补全、验证、格式化）
  - SQL 执行（单/多语句、Explain、DuckDB）
  - 快捷键全部生效
  - 工具栏位置切换正常
  - 连接选择/切换正常
  - 草稿自动保存/恢复正常
- [ ] `pnpm run lint` 通过
- [ ] 无新增 TypeScript 编译错误

### Phase 3 验收

- [ ] `window` 上不再有 `sql-execution-result` / `query-result-new` / `query-result-refresh` 等事件
- [ ] 编辑器执行 SQL 后结果面板正确展示
- [ ] Execute+ 在新标签打开结果
- [ ] 结果面板刷新按钮正常工作
- [ ] 设置面板可打开
- [ ] Pinia DevTools 中可观察完整数据流

### Phase 4 验收

- [ ] 切换连接时无 Monarch tokenizer 完整重建日志
- [ ] 执行中可点击取消按钮终止查询
- [ ] 取消后编辑器状态正确恢复

---

## 风险控制

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 拆分后功能回归 | 中 | 高 | 每个 Phase 完成后执行完整功能冒烟测试 |
| composable 间循环依赖 | 低 | 高 | 保持 composable 单向依赖，编排层组装 |
| Pinia Store 与现有 EventBus 双轨冲突 | 低 | 中 | Phase 3 双通道过渡，逐事件替换 |
| 类型收紧导致大量编译错误 | 中 | 中 | 先定义类型，渐进式替换，不急于全部消除 |
| Monaco Web Worker 销毁异常 | 低 | 低 | 继承现有安全销毁逻辑（空模型替换法） |

### 回滚策略

所有修改在 Git 分支 `refactor/sql-editor-optimization` 上进行，每个 Phase 完成后提交一个 commit。如遇问题，可直接 revert 到上一 Phase 的 commit。

---

## 附录

### 相关文档

| 文档 | 路径 |
|------|------|
| SQL 编辑器当前文档 | [SQL-EDITOR.md](./SQL-EDITOR.md) |
| 前端架构文档 | [ARCHITECTURE.md](./ARCHITECTURE.md) |
| 前端组件规范 | [COMPONENTS.md](./COMPONENTS.md) |
| 前端文档索引 | [INDEX.md](./INDEX.md) |
| 项目规则 | `.trae/rules/` |

### 版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.0 | 2026-05-08 | 初始计划，待确认 |
