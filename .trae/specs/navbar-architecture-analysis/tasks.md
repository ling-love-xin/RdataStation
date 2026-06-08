# 导航栏深度分析 — 改进任务

> 基于 spec.md 第十节发现的已知问题。

- [ ] Task 1: 修复 schema 节点下 loadTables 重复调用
  - 文件: `use-database-tree-loader.ts` 第 872-875 行
  - 影响: 展开 schema 节点时对后端发送 2 次相同的 loadTables 请求
  - 方案: 删除重复的 `navigatorStore.loadTables(...)` 调用，保留 1 次

- [ ] Task 2: 跨连接的树展开状态持久化
  - 当前: navigator-persistence 仅按 connId 维度保存
  - 目标: 记录最近一次展开的完整路径，重启 APP 后自动恢复
  - 需要设计新的数据结构

- [ ] Task 3: searchObjects 性能优化
  - 当前: 每次搜索完整遍历 connectionCatalogs
  - 候选方案:
    - 前端建倒排索引（name → path 映射）
    - 后端提供全文搜索 Tauri 命令
    - 添加 300ms debounce 防抖

# Task Dependencies
- Task 1 独立，可直接修复
- Task 2 依赖 navigator-persistence.ts 重构
- Task 3 可独立实施