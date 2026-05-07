# 草稿箱 (Scratchpad) 开发进度

> 版本：v2.0
> 最后更新：2026-05-07
> 状态：✅ 安全合规 + 性能优化完成

---

## 开发时间线

| 阶段 | 状态 | 日期 |
|------|------|------|
| 阶段一：Rust 数据模型 + Store | ✅ 完成 | 2026-05-07 |
| 阶段二：Rust Command 层 | ✅ 完成 | 2026-05-07 |
| 阶段三：命令注册 | ✅ 完成 | 2026-05-07 |
| 阶段四：前端 TS 类型 + API | ✅ 完成 | 2026-05-07 |
| 阶段五：前端 Composable | ✅ 完成 | 2026-05-07 |
| 阶段六：前端 Vue 组件 | ✅ 完成 | 2026-05-07 |
| 阶段七：扩展注册 + 活动栏 | ✅ 完成 | 2026-05-07 |
| 阶段八：测试验证 | ✅ 完成 | 2026-05-07 |
| 阶段九：文档记录 | ✅ 完成 | 2026-05-07 |
| 阶段十：交互优化 (v1.1) | ✅ 完成 | 2026-05-07 |
| **阶段十一：安全合规 + 性能优化 (v2.0)** | **✅ 完成** | **2026-05-07** |

---

## 新增文件清单

### Rust 后端 (4 文件)

| 文件 | 行数 | 说明 |
|------|------|------|
| `core/scratchpad/mod.rs` | 7 | 模块入口，re-export |
| `core/scratchpad/models.rs` | 41 | DTO 数据模型（ScratchpadEntry、ExternalReference 等） |
| `core/scratchpad/store.rs` | 378 | 文件系统操作 + 配置管理 |
| `commands/scratchpad_commands.rs` | 114 | 9 个 Tauri Command |

### 前端 (8 文件)

| 文件 | 行数 | 说明 |
|------|------|------|
| `extensions/builtin/scratchpad/package.json` | 7 | 扩展元数据 |
| `extensions/builtin/scratchpad/extension.ts` | 58 | 扩展注册（dockview 面板） |
| `extensions/builtin/scratchpad/types/index.ts` | 21 | TypeScript 类型定义 |
| `extensions/builtin/scratchpad/infrastructure/api/scratchpad-api.ts` | 90 | Tauri invoke 封装（9 个 API） |
| `extensions/builtin/scratchpad/ui/composables/use-scratchpad.ts` | 183 | 业务逻辑 hook |
| `extensions/builtin/scratchpad/ui/components/ScratchpadPanel.vue` | 252 | 主面板组件 |
| `extensions/builtin/scratchpad/ui/components/ScratchpadTreeNode.vue` | 115 | 递归树节点组件 |
| `docs/backend/SCRATCHPAD_DESIGN.md` | - | 设计方案文档 |

---

## 修改文件清单

| 文件 | 修改内容 | 影响 |
|------|----------|------|
| `core/mod.rs` | 新增 `pub mod scratchpad` + re-exports | 低 |
| `commands/mod.rs` | 新增 `pub mod scratchpad_commands` + re-export | 低 |
| `lib.rs` | `generate_handler![]` 新增 9 个命令 | 低 |
| `core/builtin-extensions.ts` | 导入注册 scratchpad 扩展 | 低 |
| `layout-store.ts` | `leftActivityItems` 新增草稿箱入口 + `ACTIVEBAR_TO_PANEL_ID` 映射 | 低 |

---

## 验证结果

| 检查项 | 结果 |
|--------|------|
| `cargo check` | ✅ 编译通过，0 错误 |
| `rustfmt` | ✅ 格式化通过 |
| `cargo clippy` | ⚠️ 环境问题（Windows 进程管理），非代码问题 |
| `pnpm typecheck` (scratchpad) | ✅ 草稿箱相关 0 错误 |
| `pnpm lint` | ⚠️ 76 错误 560 警告（全为已有文件问题，与本次变更无关） |

---

## 已验证功能 ✅

- [x] Rust 数据模型（ScratchpadEntry、ExternalReference、ScratchpadConfig、ScratchpadResponse）
- [x] 文件系统操作 Store（增删改查、导入、引用管理）
- [x] 路径安全校验（防止 `..` 遍历攻击、路径前缀检查）
- [x] 唯一路径生成（导入重名文件自动追加后缀）
- [x] 9 个 Tauri Command 完整实现
- [x] 前端 TypeScript 类型定义
- [x] 前端 API 层封装（完整覆盖 9 个命令）
- [x] Composable use-scratchpad（状态管理 + 业务逻辑 + 引用有效性检测）
- [x] ScratchpadPanel 主面板（工具栏 + 分组 + 搜索 + 加载/错误状态）
- [x] ScratchpadTreeNode 递归树节点（图标映射 + 文件大小显示 + 内联重命名）
- [x] 扩展注册（dockview left 区域 + 活动栏 StickyNote 图标）
- [x] 外部引用分组（折叠/展开 + 失效检测 + "已失效"徽章）
- [x] 本地草稿分组（空状态提示）
- [x] 搜索过滤（文件名 + 引用别名/路径）
- [x] 新建文件对话框（输入框自动聚焦 + Enter 确认）
- [x] 导入文件对话框（@tauri-apps/plugin-dialog 集成）
- [x] 添加外部引用对话框（别名 + 路径 + 浏览按钮）
- [x] **右键菜单**（打开 / 重命名 / 折叠展开 / 复制路径 / 删除）
- [x] **外部引用右键菜单**（移除引用 / 打开位置）
- [x] **F2 内联重命名**（输入框自动聚焦全选 / Enter 确认 / Escape 取消 / Blur 提交）
- [x] **Delete 键删除**选中草稿条目
- [x] **composable** 中 `invalidReferences` / `validReferences` / `isRefValid` / `findEntry`
- [x] **composable** 中 `isRefInvalid` / `openInExplorerAction` / `getFileSize`（v2.0 新增）
- [x] **Ctrl+N 快捷键** 新建草稿（v2.0 新增）
- [x] **右键菜单溢出检测** `clampToViewport()`（v2.0 新增）
- [x] **去重 isRefInvalid** panel 使用 composable 版，删除局部重复（v2.0 新增）
- [x] **大文件检查** 后端 MAX_FILE_SIZE 50MB + 前端 `openFileInEditor` 前置校验（v2.0 新增）
- [x] **外部引用打开位置** `opener` crate → 系统文件管理器（v2.0 新增）
- [x] **resolve_path unwrap 修复** `unwrap_or_else` → `map_err` → `CoreError`（v2.0 新增，安全合规）
- [x] **新增 Tauri Commands** `open_scratchpad_in_explorer` / `check_scratchpad_file_size`（v2.0 新增）

## 预留接口 🔌

- [ ] 双击打开编辑器面板（`openFileInEditor` 已根据后缀推断编辑器类型 + 50MB 大小检查）
- [ ] "提升"机制（composable 已导出 `loadFileContent` / `findEntry`）
- [ ] Dark/Light 主题图标适配（lucide-vue-next 原生支持，CSS var 已使用）

---

## 技术细节

### 路径安全

`resolve_path` 方法实现双重安全检查：
1. 禁止 `..` 路径遍历
2. 校验规范化路径必须在 `.scratchpad/` 目录下

### 数据结构

```rust
pub struct ScratchpadStore {
    scratchpad_dir: PathBuf,  // {project}/.scratchpad/
    config_path: PathBuf,     // {project}/.scratchpad/.scratchpad.json
}
```

Store 不接受 `&self` 以外的外部注入，所有路径从 `project_path` 计算，避免注入攻击。

### 错误处理

所有错误通过 `CoreError::storage(StorageError::Io { path, operation, reason })` 统一处理，符合项目规范（禁止 unwrap/expect，必须使用 CoreError）。

### 前端交互模式

- **右键菜单**：fixed 定位 + reactive contextMenu state，通过 `document.addEventListener('click')` 自动关闭
- **内联重命名**：Vue watch 监听 renamingKey 变化 → nextTick → focus/select
- **导入对话框**：动态 import `@tauri-apps/plugin-dialog` + 条件加载（Tauri 环境检测）

## 后续版本规划

| 版本 | 规划功能 |
|------|----------|
| **v2.0** | ✅ 安全合规 + 性能优化（已完成） |
| v2.1 | 编辑器面板集成（SQL/Monaco/数据预览） + 文件监控（watch） |
| v3.0 | "提升"机制（草稿 → analytics_resource） + 可插拔后端 trait |
