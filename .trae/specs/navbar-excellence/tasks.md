# 导航栏卓越优化 — 任务清单

## Phase 1: 错误处理 + 代码结构（并行）

- [ ] Task 1.1: sub-loader 统一错误协议
  - [ ] use-catalog-loader.ts: 全部 catch 统一 setNodeError + rethrow
  - [ ] use-table-loader.ts: 同上
  - [ ] use-object-loader.ts: 同上
  - [ ] use-column-loader.ts: 同上
  - [ ] 验证: 每个 loader 的 catch 块遵循统一三段式

- [ ] Task 1.2: 代码结构 — loadChildren Router 模式
  - [ ] 创建 `nav-router.ts` 路由表: `Record<VirtualTreeNodeType, (node, ctx) => Promise<VirtualTreeNode[]>>`
  - [ ] 将 loadChildren switch 拆分为独立 handler 函数
  - [ ] 验证: loadChildren 行数 ≤ 40 行

- [ ] Task 1.3: 代码结构 — 清理 TODO/FIXME
  - [ ] 查找并处理 use-database-tree-loader.ts 中的 TODO
  - [ ] 清理全部无用注释

## Phase 2: 性能 + 内存 + 类型（并行）

- [ ] Task 2.1: getNavConfig 缓存
  - [ ] 在 use-database-tree-loader.ts 中添加 `const navConfigCache = new Map<string, NavigationConfig>()`
  - [ ] loadChildren 中优先从缓存读取

- [ ] Task 2.2: searchTables 索引预建
  - [ ] 在 NavigatorSearch 中使用 Map<string, VirtualTreeNode[]> 建索引
  - [ ] 搜索时 O(1) 查找而非 O(N) 遍历

- [ ] Task 2.3: 类型安全 — 消除 as 断言
  - [ ] 扫描全部 `as` 断言，用类型守卫替代
  - [ ] node.data 访问前做 discriminated union 判断

- [ ] Task 2.4: 内存管理 — AbortController
  - [ ] loadChildren 接受可选的 AbortSignal 参数
  - [ ] 所有 API 调用传递 signal
  - [ ] onUnmounted 中 abort 所有进行中的请求

## Phase 3: 可访问性 + 测试（并行）

- [ ] Task 3.1: VirtualTree ARIA
  - [ ] 添加 role="tree" 到容器
  - [ ] 每个节点 role="treeitem" + aria-expanded + aria-level + aria-selected
  - [ ] 键盘事件处理: ArrowUp/Down/Left/Right/Enter/Home/End

- [ ] Task 3.2: 单元测试
  - [ ] tree-mutation.spec.ts: 6 个测试用例
  - [ ] lazy-loader.spec.ts: 4 个测试用例
  - [ ] navigator-persistence.spec.ts: 4 个测试用例
  - [ ] 验证: vitest run --coverage

- [ ] Task 3.3: 图标 aria-label
  - [ ] 所有 Lucide 图标添加 aria-label 属性

# Task Dependencies
- Phase 1 三任务完全并行
- Phase 2 四任务完全并行，依赖 Phase 1.2
- Phase 3 三任务完全并行，可与其他 Phase 并行
- Task 1.2 (Router 模式) 是其他代码结构工作的基础