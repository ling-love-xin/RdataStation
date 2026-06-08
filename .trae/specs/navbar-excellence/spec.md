# 导航栏全面卓越优化

> 版本：v1.0
> 日期：2026-06-09
> 目标：所有维度达到满分

---

## Why
V10.8 审计发现模块在 7 个维度中仅 2 个达到 8/10，其余均在 2-7 分。需要系统性提升至全维度优秀水平。

## What Changes

### 错误处理 (6 → 10)
- 全部 sub-loader 统一错误处理模式（catch → setNodeError → rethrow）
- VirtualTree 渲染错误节点组件（回退 + 重试按钮）
- 连接断开/重连时自动恢复树状态
- 网络超时友好提示

### 性能 (7 → 10)
- getNavConfig 结果缓存（Map memorization）
- tables/views-folder 重复请求消除（V10.8 已做，确认覆盖全部路径）
- VirtualTreeNode 用 shallowRef 减少深层响应式开销
- searchTables 用 index 预建加速

### 内存管理 (7 → 10)
- 全部 composable 的 onScopeDispose 清理
- AbortController 取消正在进行的加载请求（组件卸载时）
- WeakMap 用于大型 node 数据引用

### 代码结构 (8 → 10)
- loadChildren 从 200 行 switch 拆为 Router 模式
- 提取重复的 catalog-find → schema-find 遍历为辅助函数
- 清理全部 TODO/FIXME

### 类型安全 (8 → 10)
- 消除全部 `as` 类型断言（改用类型守卫）
- node.data 使用 tagged union（按 nodeType 缩窄类型）
- keyParts 解构用 Zod-like validator

### 可访问性 (3 → 10)
- VirtualTree: role="tree" + role="treeitem" + aria-expanded + aria-level + aria-selected
- 键盘导航: ArrowUp/Down/Left/Right + Enter + Home/End
- 焦点环管理
- 图标添加 aria-label

### 测试覆盖 (2 → 10)
- tree-mutation.ts 单元测试
- lazy-loader.ts 单元测试
- navigator-persistence.ts 单元测试
- NodeKeyEncoder 单元测试
- use-catalog-loader 集成测试
- database-navigator-store 集成测试

## Impact
- Affected specs: navbar-architecture-analysis, navbar-deep-audit
- Affected code: 全模块 20 个文件

---

## ADDED Requirements

### Requirement: 统一错误处理协议
所有 sub-loader 的 catch 块 SHALL 遵循统一协议：
1. 调用 `navigatorStore.setNodeError(key, message)`
2. `console.error` 记录完整堆栈
3. 重新抛出错误（由 loadChildren catch 统一渲染错误节点）

#### Scenario: 网络断开时展开节点
- **WHEN** 用户展开 catalog 节点且数据库连接已断开
- **THEN** 树显示 `⚠ 加载失败：connection timeout` 错误节点
- **AND** 用户可点击"重试"按钮重新加载

### Requirement: getNavConfig 缓存
系统 SHALL 在首次加载后缓存 NavigationConfig，后续同 dbType 节点展开不再读文件。

### Requirement: VirtualTree ARIA 可访问性
VirtualTree 组件 SHALL 提供完整的 WAI-ARIA Tree 模式支持。

#### Scenario: 键盘导航树节点
- **WHEN** 用户按 ArrowDown
- **THEN** 焦点移至下一个可见节点
- **WHEN** 用户按 ArrowRight 在折叠节点上
- **THEN** 展开该节点
- **WHEN** 用户按 Enter 在表节点上
- **THEN** 触发选中（打开表详情）

### Requirement: 前端单元测试
关键工具函数 SHALL 具有 ≥80% 分支覆盖率。

#### Scenario: tree-mutation mutateTreeNode 测试
- **WHEN** 传入有效 path
- **THEN** 回调被调用并返回 true
- **WHEN** 传入不存在的 catalog
- **THEN** 返回 false，不抛异常