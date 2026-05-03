# RdataStation 变更日志

> 版本：v1.0
> 最后更新：2026-05-03
> 状态：📋 持续更新

---

## 目录

- [项目变更日志](#项目变更日志)
- [文档变更日志](#文档变更日志)
- [提交规范](#提交规范)

---

## 项目变更日志

### [v2.0] - 2026-04-23

#### 新增

- **插件化架构优化**
  - DDD 分层架构（domain/infrastructure/ui）
  - 事件总线（EventBus）插件间通信机制
  - 统一 API 层
  - ExtensionContext 生命周期管理

- **前端架构**
  - 完整的插件系统
  - shared/ 共享资源中心
  - 全局类型系统

#### 变更

- **SQL 编辑器**
  - 1:n 编辑器-结果集关系重构
  - 多结果标签页支持

- **数据库导航**
  - IVM 增量视图维护
  - 三级缓存架构

---

### [v1.0] - 2026-04-20

#### 新增

- 初始插件化架构
- 内置插件：
  - connection（连接管理）
  - database（数据库导航）
  - navigator（通用导航器）
  - query（查询执行）
  - workbench（SQL 工作台）

---

## 文档变更日志

### [v2.0] - 2026-05-03

#### 新增

- `docs/README.md` - 项目文档中心总索引
- `docs/backend/TECHNICAL_OVERVIEW.md` - 技术概览

#### 变更

- `docs/backend/README.md` - 统一文档格式（版本、日期、状态）
- `docs/navigator/README.md` - 统一文档格式
- `docs/frontend/INDEX.md` - 统一文档格式
- `src-tauri/src/docs/README.md` - 统一文档格式

---

## 提交规范

项目使用 **Gitmoji + Angular** 提交规范：

### 格式

```
<emoji> <type>(<scope>): <subject>
```

### 类型

| 类型 | Emoji | 说明 |
|------|-------|------|
| feat | ✨ | 新增功能 |
| fix | 🐛 | 修复 Bug |
| docs | 📝 | 文档变更 |
| refactor | ♻️ | 代码重构 |
| perf | ⚡️ | 性能优化 |
| style | 💄 | 格式调整 |
| test | 🧪 | 测试相关 |
| build | 📦 | 构建相关 |
| chore | 🔧 | 杂项配置 |

### 示例

```bash
✨ feat(workbench): 实现 SQL 编辑器多标签页
🐛 fix(sqlite): 修复百万级数据查询超时
📝 docs: 补充前端架构文档
♻️ refactor(database): 重构导航器缓存逻辑
⚡️ perf(query): 优化查询缓存命中率
🔧 chore: 更新依赖版本
```

---

## 版本管理策略

### 分支策略

- `main` - 主分支，稳定版本
- `develop` - 开发分支
- `feature/*` - 功能分支
- `fix/*` - 修复分支

### 发布流程

1. 功能开发完成 → 合并到 `develop`
2. 测试验证 → 合并到 `main`
3. 打标签发布 → `git tag v{x.y.z}`

### 兼容性约束

- 接口遵循语义化版本（SemVer）
- 禁止破坏性变更（major 版本内）
- 10 年向前兼容目标

---

## 维护

- **最后更新**：2026-05-03
- **更新频率**：每次重要变更后同步更新
- **格式要求**：使用 Keep a Changelog 规范
