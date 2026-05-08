# 草稿箱 (Scratchpad) 开发进度

> 版本：v3.1
> 最后更新：2026-05-09
> 状态：✅ v3.1 排序 + 搜索行上下文 — 按名称/大小/时间排序，内容搜索结果带行号预览

---

## 开发时间线

| 阶段                                 | 状态   | 日期           |
| ------------------------------------ | ------ | -------------- |
| 阶段一：Rust 数据模型 + Store        | ✅     | 2026-05-07     |
| 阶段二：Rust Command 层              | ✅     | 2026-05-07     |
| 阶段三：命令注册                     | ✅     | 2026-05-07     |
| 阶段四：前端 TS 类型 + API           | ✅     | 2026-05-07     |
| 阶段五：前端 Composable              | ✅     | 2026-05-07     |
| 阶段六：前端 Vue 组件                | ✅     | 2026-05-07     |
| 阶段七：扩展注册 + 活动栏            | ✅     | 2026-05-07     |
| 阶段八：测试验证                     | ✅     | 2026-05-07     |
| 阶段九：文档记录                     | ✅     | 2026-05-07     |
| 阶段十：交互优化 (v1.1)              | ✅     | 2026-05-07     |
| 阶段十一：安全合规 + 性能优化 (v2.0) | ✅     | 2026-05-07     |
| **阶段十二：SQL 编辑器集成 (v2.2)**  | **✅** | **2026-05-08** |
| **阶段十三：文件元数据层 (v2.3)** | **✅** | **2026-05-08** |
| **阶段十四：Bug修复 + 代码编辑器 + 内容搜索 (v2.4)** | **✅** | **2026-05-08** |
| **阶段十五：内容搜索修复 + Config 缓存 (v2.5)**       | **✅** | **2026-05-08** |
| **阶段十六：回收站 UI + 自动保存 (v2.6)**              | **✅** | **2026-05-08** |
| **阶段十七：主题适配 + DuckDB 分析 (v2.7)**             | **✅** | **2026-05-08** |
| **阶段十八：文件监控 + 防重复Tab + 拖放导入 + 主题对齐 + 工具栏优化 (v2.8)** | **✅** | **2026-05-08** |
| **阶段十九："提升"机制 (v2.9)** | **✅** | **2026-05-08** |
| **阶段二十：UX 增强 — 键盘导航/重命名反馈/提升事件 (v3.0)** | **✅** | **2026-05-08** |
| **阶段二十一：文件排序 + 搜索行上下文 (v3.1)** | **✅** | **2026-05-09** |

---

## 新增文件清单

### Rust 后端 (4 文件)

| 文件                              | 行数 | 说明                                                                     |
| --------------------------------- | ---- | ------------------------------------------------------------------------ |
| `core/scratchpad/mod.rs`          | ~10  | 模块入口，re-export                                                      |
| `core/scratchpad/models.rs`       | ~73  | DTO 数据模型（Entry / FileMeta / AnalyzableFile / Reference / Response / SearchMatch） |
| `core/scratchpad/state.rs`        | ~48  | `Arc<Mutex<Option<Store>>>` 全局状态缓存 + `AtomicBool` watcher 状态     |
| `commands/scratchpad_commands.rs` | ~370 | 23 个 Tauri Command                                           |

### 前端 (8 文件)

| 文件                                              | 行数 | 说明                                                   |
| ------------------------------------------------- | ---- | ------------------------------------------------------ |
| `scratchpad/package.json`                         | 7    | 扩展元数据                                             |
| `scratchpad/extension.ts`                         | ~61  | 扩展注册（dockview 面板）                              |
| `scratchpad/types/index.ts`                       | ~55  | TypeScript 类型定义（含 PromoteResult / AnalyticsResourceBrief / SearchMatch） |
| `scratchpad/infrastructure/api/scratchpad-api.ts` | ~165 | Tauri invoke 封装（20 个 API）                                    |
| `scratchpad/ui/composables/use-scratchpad.ts`     | ~370 | 业务逻辑 hook（33 个导出项）                                      |
| `scratchpad/ui/components/ScratchpadPanel.vue`    | ~970 | 主面板组件（工具栏排序/搜索双模式/回收站/右键菜单/提升确认/拖放导入）      |
| `scratchpad/ui/components/ScratchpadTreeNode.vue` | ~170 | 递归树节点组件（+重命名 spinner + disabled + `entry.children` 修复）  |

---

## 修改文件清单

| 文件                         | 修改内容                                                          | 影响 |
| ---------------------------- | ----------------------------------------------------------------- | ---- |
| `core/mod.rs`                | 新增 `pub mod scratchpad` + re-exports                            | 低   |
| `commands/mod.rs`            | 新增 `pub mod scratchpad_commands` + re-export                    | 低   |
| `lib.rs`                     | `generate_handler![]` 新增 23 个命令                             | 低   |
| `core/builtin-extensions.ts` | 导入注册 scratchpad 扩展                                          | 低   |
| `layout-store.ts`            | `leftActivityItems` 新增草稿箱入口 + `ACTIVEBAR_TO_PANEL_ID` 映射 | 低   |

---

## 验证结果

| 检查项                        | 历史结果                                    |
| ----------------------------- | ------------------------------------------- |
| `cargo check`                 | ✅ 编译通过，0 错误                         |
| `rustfmt`                     | ✅ 格式化通过                               |
| `cargo clippy`                | ⚠️ 环境问题（Windows 进程管理），非代码问题 |
| `pnpm typecheck` (scratchpad) | ✅ 草稿箱相关 0 错误                        |
| `pnpm lint`                   | ⚠️ 错误均为已有文件问题，与本次变更无关     |

---

## 已验证功能 ✅

- [x] Rust 数据模型（ScratchpadEntry、ExternalReference、ScratchpadConfig、ScratchpadResponse）
- [x] 文件系统操作 Store（增删改查、导入、引用管理）
- [x] 路径安全校验（防止 `..` 遍历攻击、路径前缀检查）
- [x] 唯一路径生成（导入重名文件自动追加后缀 `_1` / timestamp）
- [x] 原子写入（`.tmp` → `rename` → 失败清理）
- [x] 18 个 Tauri Command 完整实现
- [x] 前端 TypeScript 类型定义
- [x] 前端 API 层封装（11 个函数，覆盖核心 12 个命令）
- [x] Composable use-scratchpad（状态管理 + 业务逻辑 + 引用有效性检测 + 搜索过滤）
- [x] ScratchpadPanel 主面板（工具栏 + 分组 + 搜索 + 加载/错误/空状态）
- [x] ScratchpadTreeNode 递归树节点（图标映射 + 文件大小显示 + 内联重命名）
- [x] 扩展注册（dockview left 区域 + 活动栏 FileText 图标）
- [x] 外部引用分组（折叠/展开 + 失效检测 + "已失效"徽章）
- [x] 本地草稿分组（空状态提示）
- [x] 搜索过滤（文件名 + 引用别名/路径）
- [x] 新建文件对话框（自动聚焦 + Enter 确认）
- [x] 导入文件对话框（@tauri-apps/plugin-dialog 集成）
- [x] 添加外部引用对话框（别名 + 路径 + 浏览按钮）
- [x] 右键菜单（打开/重命名/折叠展开/复制路径/删除）
- [x] 外部引用右键菜单（移除引用/打开位置）
- [x] F2 内联重命名（自动聚焦全选 / Enter 确认 / Escape 取消 / Blur 提交）
- [x] Delete 键删除选中条目
- [x] Ctrl+N 快捷键新建草稿
- [x] 右键菜单溢出检测 `clampToViewport()`
- [x] `isRefInvalid` 去重到 composable
- [x] 大文件检查（前端 50MB 前置校验 + 后端 `check_file_size`）
- [x] 系统文件管理器打开（`opener` crate）

---

## v2.2 已完成 ✅

### SQL 编辑器集成（阶段十二）

| #   | 任务                                                                   | 状态 | 涉及文件                  |
| --- | ---------------------------------------------------------------------- | :--: | ------------------------- |
| 1   | `SqlEditorParams` 新增 `scratchpadRelativePath` / `scratchpadFileName` |  ✅  | `src/shared/types/sql.ts` |
| 2   | `openFileInEditor` 打通——读取文件 → 派发事件 → dockview 创建面板       |  ✅  | `ScratchpadPanel.vue`     |
| 3   | SqlEditorPanel 精简模式——`scratchpadRelativePath` 非空时关闭方言功能   |  ✅  | `SqlEditorPanel.vue`      |
| 4   | Ctrl+S 保存走 `save_scratchpad_file`                                   |  ✅  | `SqlEditorPanel.vue`      |
| 5   | 编辑器 tab 标题显示草稿文件名                                          |  ✅  | `WorkbenchView.vue`       |
| 6   | EditorToolbar `showAdvanced` prop 控制高级按钮                         |  ✅  | `EditorToolbar.vue`       |

### v2.2 实际改动文件

| 文件                      | 改动类型                                                     | 改动量 |
| ------------------------- | ------------------------------------------------------------ | :----: |
| `src/shared/types/sql.ts` | 新增 2 字段                                                  |  +2行  |
| `WorkbenchView.vue`       | 透传 params + 标题                                           | ~10行  |
| `SqlEditorPanel.vue`      | 精简模式 + Ctrl+S + invoke import                            | ~30行  |
| `EditorToolbar.vue`       | `showAdvanced` prop + v-if                                   | ~10行  |
| `ScratchpadPanel.vue`     | `openFileInEditor` 实现 + `loadFileContent`/`scratchpadPath` | ~25行  |

> **Rust 后端：零改动。**

### v2.3 已完成 ✅

### 文件元数据层（阶段十三）

| #   | 任务                                                      | 状态 | 涉及文件                      |
| --- | --------------------------------------------------------- | :--: | ----------------------------- |
| 1   | `FileMeta` 数据模型（Rust + TS）                          |  ✅  | `models.rs`, `types/index.ts` |
| 2   | `store.update_file_meta()` 方法                           |  ✅  | `store.rs`                    |
| 3   | `update_scratchpad_file_meta` Tauri Command               |  ✅  | `commands.rs`, `lib.rs`       |
| 4   | 前端 API `updateFileMeta()` + Composable `saveFileMeta()` |  ✅  | `api.ts`, `use-scratchpad.ts` |
| 5   | `openFileInEditor` 自动恢复 `lastConnectionId`            |  ✅  | `ScratchpadPanel.vue`         |
| 6   | `handleScratchpadSave` / `handleExecute` 写入 file_meta   |  ✅  | `SqlEditorPanel.vue`          |

### v2.3 实际改动文件

| 文件                              | 改动类型                                                       | 改动量 |
| --------------------------------- | -------------------------------------------------------------- | :----: |
| `core/scratchpad/models.rs`       | 新增 `FileMeta` + 更新 Config/Response                         | +12行  |
| `core/scratchpad/mod.rs`          | pub use 新增 `FileMeta`                                        |  +1行  |
| `core/scratchpad/store.rs`        | 新增 `update_file_meta()` + import `FileMeta`                  | +20行  |
| `commands/scratchpad_commands.rs` | 新增 `update_scratchpad_file_meta` 命令                        | +13行  |
| `lib.rs`                          | `generate_handler!` 注册新命令                                 |  +1行  |
| `types/index.ts`                  | 新增 `FileMeta` + 更新 `ScratchpadResponse`                    |  +7行  |
| `scratchpad-api.ts`               | 新增 `updateFileMeta()`                                        | +10行  |
| `use-scratchpad.ts`               | 新增 `saveFileMeta()` + import                                 | +14行  |
| `ScratchpadPanel.vue`             | `openFileInEditor` 读 file_meta + `response`/`computed` import |  +5行  |
| `SqlEditorPanel.vue`              | save/execute 时调用 `update_scratchpad_file_meta`              | +12行  |

### v2.4 已完成 ✅

### Bug 修复 + 代码编辑器 + 内容搜索（阶段十四）

| # | 任务 | 状态 | 涉及文件 |
|---|------|:--:|------|
| 1 | `rename_entry()` 迁移 file_meta 键 | ✅ | `store.rs` |
| 2 | 代码编辑器打通 `.py`/`.json`/`.txt`/`.md` | ✅ | 3 files |
| 3 | `search_file_content()` Rust 方法 | ✅ | `store.rs` |
| 4 | `search_scratchpad_content` Tauri Command | ✅ | `commands.rs`, `lib.rs` |
| 5 | 前端 API `searchFileContent()` + Composable `searchContent()` | ✅ | `api.ts`, `use-scratchpad.ts` |
| 6 | 面板搜索双模式（文件名/全文）+ Search 按钮 | ✅ | `ScratchpadPanel.vue` + i18n |

### v2.4 实际改动文件

| 文件                         | 改动类型                            | 改动量 |
| ----------------------------- | ---------------------------------- | :----: |
| `core/scratchpad/store.rs`    | 新增 `search_file_content()` + rename 迁移 | +30行  |
| `commands/scratchpad_commands.rs` | 新增 `search_scratchpad_content` 命令 | +12行  |
| `lib.rs`                      | 注册新命令                          |  +1行  |
| `types/sql.ts`                | 新增 `language` 字段                |  +1行  |
| `scratchpad-api.ts`           | 新增 `searchFileContent()`          |  +8行  |
| `use-scratchpad.ts`           | 新增 `searchContent()` + import     | +14行  |
| `zh-CN.json`                  | 新增 `searchContent`/`searchByFilename` |  +2行  |
| `ScratchpadPanel.vue`         | 搜索双模式 + 非SQL文件事件派发      | +30行  |
| `WorkbenchView.vue`           | 透传 `language` 参数                |  +3行  |
| `SqlEditorPanel.vue`          | 动态 language + 非SQL隐藏工具栏     | +12行  |

### v2.5 已完成 ✅

### 内容搜索修复 + Config 缓存（阶段十五）

| # | 任务 | 状态 | 涉及文件 |
|---|------|:--:|------|
| 1 | 修复 `filteredLocalEntries` 路径匹配（相对↔绝对） | ✅ | `ScratchpadPanel.vue` |
| 2 | `ScratchpadStore` Config 内存缓存 | ✅ | `store.rs` |

### v2.5 实际改动文件

| 文件                         | 改动类型                                  | 改动量 |
| ----------------------------- | ---------------------------------------- | :----: |
| `ScratchpadPanel.vue`         | `filteredLocalEntries` 路径对齐           |  +7行  |
| `core/scratchpad/store.rs`    | struct + `new()` + `load_config()` + `save_config()` 缓存 | +25行  |

### v2.6 已完成 ✅

### 回收站 UI + 自动保存（阶段十六）

| # | 任务 | 状态 | 涉及文件 |
|---|------|:--:|------|
| 1 | 前端 API `listTrash()` / `restoreFromTrash()` / `emptyTrash()` | ✅ | `scratchpad-api.ts` |
| 2 | Composable `trashEntries` + `loadTrashEntries()` / `restoreTrashEntry()` / `emptyTrashBin()` | ✅ | `use-scratchpad.ts` |
| 3 | 回收站折叠面板（恢复 / 清空按钮）+ i18n | ✅ | `ScratchpadPanel.vue` + `zh-CN.json` |
| 4 | `markDirty()` 触发 2s 防抖 `scheduleAutoSave()` | ✅ | `SqlEditorPanel.vue` |

### v2.6 实际改动文件

| 文件                         | 改动类型                                  | 改动量 |
| ----------------------------- | ---------------------------------------- | :----: |
| `scratchpad-api.ts`           | 新增 `listTrash()`/`restoreFromTrash()`/`emptyTrash()` | +20行  |
| `use-scratchpad.ts`           | `trashEntries` ref + 3 方法 + 4 exports  | +38行  |
| `zh-CN.json`                  | 回收站 i18n 5 个 key                      |  +5行  |
| `ScratchpadPanel.vue`         | 回收站折叠面板 + script 状态/方法          | +45行  |
| `SqlEditorPanel.vue`          | `markDirty()` 改为触发 auto-save + timer 清理 | +14行  |

### v2.7 已完成 ✅

### 主题适配 + DuckDB 分析（阶段十七）

| # | 任务 | 状态 | 涉及文件 |
|---|------|:--:|------|
| 1 | CSS 变量对齐 `global.css` 主题系统（10+ 处） | ✅ | `ScratchpadPanel.vue` |
| 2 | `AnalyzableFile` TS 接口 + `getAnalyzableFiles()` API | ✅ | `types/index.ts`, `scratchpad-api.ts` |
| 3 | `loadAnalyzableFiles()` composable 方法 | ✅ | `use-scratchpad.ts` |
| 4 | 右键菜单 "用 DuckDB 分析"（csv/parquet/json/xlsx） | ✅ | `ScratchpadPanel.vue` + i18n |

### v2.7 实际改动文件

| 文件                         | 改动类型                                  | 改动量 |
| ----------------------------- | ---------------------------------------- | :----: |
| `ScratchpadPanel.vue`         | CSS 变量全面对齐 + DuckDB 分析入口          | +30行  |
| `types/index.ts`              | 新增 `AnalyzableFile` 接口                 |  +7行  |
| `scratchpad-api.ts`           | 新增 `getAnalyzableFiles()`（17 个 IPC）    |  +6行  |
| `use-scratchpad.ts`           | `analyzableFiles` ref + `loadAnalyzableFiles()`（29 个导出） | +15行  |
| `zh-CN.json`                  | 新增 `analyzeWithDuckDB`                   |  +1行  |

### v2.8 已完成 ✅

### 文件监控 + 防重复Tab + 拖放导入 + 主题变量对齐（阶段十八）

| # | 任务 | 状态 | 涉及文件 |
|---|------|:--:|------|
| 1 | 防重复 Tab — 同一草稿文件不重复打开 editor tab | ✅ | `WorkbenchView.vue` |
| 2 | 拖放导入 — 从文件资源管理器拖放文件到草稿箱面板 | ✅ | `ScratchpadPanel.vue` |
| 3 | 文件监控 — `notify` crate + Tauri event 推送 | ✅ | `Cargo.toml`, `state.rs`, `commands.rs`, `lib.rs` |
| 4 | 前端事件监听 — `listen('scratchpad-changed')` 自动刷新 | ✅ | `api.ts`, `use-scratchpad.ts`, `ScratchpadPanel.vue` |
| 5 | CSS 变量对齐 — `ScratchpadTreeNode.vue` 迁移至全局主题变量 | ✅ | `ScratchpadTreeNode.vue` |

### v2.8 实际改动文件

| 文件 | 改动类型 | 改动量 |
| ---- | -------- | :----: |
| `Cargo.toml` | 新增 `notify = "6"` 依赖 | +2行 |
| `core/scratchpad/state.rs` | 新增 `AtomicBool` watcher 状态 + `is_watching`/`set_watching` 方法 | +15行 |
| `commands/scratchpad_commands.rs` | 新增 `watch_scratchpad` / `unwatch_scratchpad` (22 cmd) | +65行 |
| `lib.rs` | 注册 `watch_scratchpad` / `unwatch_scratchpad` | +2行 |
| `infrastructure/api/scratchpad-api.ts` | 新增 `watchScratchpad()` / `unwatchScratchpad()` (19 API) | +8行 |
| `ui/composables/use-scratchpad.ts` | 新增 `startWatching()` / `stopWatching()` (31 exports) | +16行 |
| `ui/components/ScratchpadPanel.vue` | 拖放导入 dragover/dragleave/drop + Tauri event listener + onMounted/Unmounted | +20行 |
| `ui/components/ScratchpadTreeNode.vue` | CSS 变量迁移至 `--color-*` 主题系统 | ~4行 |

### 文件监控技术细节

**Rust 侧 (notify crate)**：
- `watch_scratchpad` 命令：创建 `RecommendedWatcher`，监控 `.scratchpad/` 递归目录
- `unwatch_scratchpad` 命令：设置 `AtomicBool` 停止标志
- 防抖：500ms 内重复变更仅触发一次 `scratchpad-changed` event
- 幂等：重复调用 `watch_scratchpad` 自动跳过（`is_watching()` 检测）

**前端侧 (Tauri event)**：
- `onMounted`: 调用 `startWatching()` → `invoke('watch_scratchpad')`，注册 `listen('scratchpad-changed', loadFiles)`
- `onUnmounted`: 注销 event listener → `stopWatching()` → `invoke('unwatch_scratchpad')`

**拖放导入技术细节**：
- `dragover.prevent` + `dragleave` 控制 `.dragover-active` CSS class（虚线边框覆盖层）
- `drop.prevent` 读取 `event.dataTransfer.files[].path`（Tauri 提供的本地文件路径）
- 直接调用 `importFile(path)` 导入

---

## 技术细节

### 路径安全

`resolve_path_impl` 三重防御：

1. 禁止 `..` 路径遍历
2. canonicalize 后检查前缀是否在 `.scratchpad/` 下
3. 支持 `must_exist=false` 模式（用于创建文件前的路径校验）

### 数据结构

```rust
pub struct ScratchpadStore {
    scratchpad_dir: PathBuf,  // {project}/.scratchpad/
    config_path: PathBuf,     // {project}/.scratchpad/.scratchpad.json
}
```

Store 不接受外部注入，所有路径从 `project_path` 计算，避免注入攻击。

### 错误处理

全部通过 `CoreError::storage(StorageError::Io { path, operation, reason })` 统一处理，禁止 unwrap/expect。

### 前端交互模式

- **右键菜单**：fixed 定位 + reactive contextMenu state + document click 自动关闭
- **内联重命名**：Vue watch 监听 renamingKey → nextTick → focus/select
- **导入对话框**：动态 import `@tauri-apps/plugin-dialog` + Tauri 环境检测
- **搜索过滤**：computed 属性实时响应 searchQuery 变化

### v2.8 已完成 ✅

### 文件监控 + 防重复Tab + 拖放导入 + 主题对齐 + 工具栏优化（阶段十八）

| # | 任务 | 状态 | 涉及文件 |
|---|------|:--:|------|
| 1 | 防重复 Tab | ✅ | `WorkbenchView.vue` |
| 2 | 拖放导入 | ✅ | `ScratchpadPanel.vue` |
| 3 | 文件监控 `notify` crate | ✅ | `Cargo.toml`, `state.rs`, `commands.rs`, `lib.rs` |
| 4 | 前端 Tauri event listener | ✅ | `api.ts`, `use-scratchpad.ts`, `ScratchpadPanel.vue` |
| 5 | CSS 主题变量对齐 | ✅ | `ScratchpadTreeNode.vue`, `ScratchpadPanel.vue` |
| 6 | 工具栏人性化优化 | ✅ | `ScratchpadPanel.vue`, `zh-CN.json` |

### v2.8 变更文件

| 文件 | 改动 | 行数 |
|------|------|:--:|
| `Cargo.toml` | 新增 `notify = "6"` | +2 |
| `state.rs` | +`AtomicBool` watcher + `is_watching`/`set_watching` | +15 |
| `commands.rs` | +`watch_scratchpad` / `unwatch_scratchpad` | +65 |
| `api.ts` | +`watchScratchpad()` / `unwatchScratchpad()` | +8 |
| `use-scratchpad.ts` | +`startWatching()` / `stopWatching()` | +16 |
| `ScratchpadPanel.vue` | 拖放 dragover/drop + toolbar 重构 + Tauri event listener | +30 |
| `ScratchpadTreeNode.vue` | CSS 变量 `--color-bg-*` / `--color-text-*` 对齐 | ~4 |
| `zh-CN.json` | +6 i18n 键 | +6 |

### v2.9 已完成 ✅

### "提升"机制 — 草稿 → 分析资源（阶段十九）

| # | 任务 | 状态 | 涉及文件 |
|---|------|:--:|------|
| 1 | Rust `promote_scratchpad_to_resource` 命令 | ✅ | `commands.rs` |
| 2 | 后缀→resource_type 映射 | ✅ | `commands.rs` |
| 3 | 跨模块访问 `AnalyticsResourceState` | ✅ | `commands.rs`, `lib.rs` |
| 4 | `PromoteResult` / `AnalyticsResourceBrief` TS 类型 | ✅ | `types/index.ts` |
| 5 | `promoteScratchpadToResource()` API | ✅ | `api.ts` |
| 6 | `promoteToResource()` composable | ✅ | `use-scratchpad.ts` |
| 7 | 右键菜单 "提升为分析资源" + `GitBranch` 图标 | ✅ | `ScratchpadPanel.vue` |
| 8 | 确认对话框：两个按钮（保留/删除原稿） | ✅ | `ScratchpadPanel.vue` |
| 9 | i18n 键 + lib.rs 注册 | ✅ | `zh-CN.json`, `lib.rs` |

### v2.9 变更文件

| 文件 | 改动 | 行数 |
|------|------|:--:|
| `commands/scratchpad_commands.rs` | +`promote_scratchpad_to_resource` + `extension_to_resource_type` + `PromoteResult` | +90 |
| `lib.rs` | 注册 `promote_scratchpad_to_resource` | +1 |
| `scratchpad-api.ts` | +`promoteScratchpadToResource()` | +10 |
| `types/index.ts` | +`PromoteResult` / `AnalyticsResourceBrief` | +12 |
| `use-scratchpad.ts` | +`promoteToResource()` + import | +18 |
| `ScratchpadPanel.vue` | +`GitBranch` icon + context menu item + NModal confirm dialog + `handlePromoteConfirm` | +25 |
| `zh-CN.json` | +`promoteToResource` / `promoteToResourceConfirm` | +2 |

### "提升"机制数据流

```
草稿文件右键 "提升为分析资源" (GitBranch)
  → showPromoteConfirm = true
  → NModal 确认对话框
  → 用户选择:
      [提升并保留草稿] → promoteToResource(path, removeAfter=false)
      [提升并删除草稿] → promoteToResource(path, removeAfter=true)
  → API: invoke('promote_scratchpad_to_resource', { relativePath, removeAfter })
  → Rust: read_file + check_file_size + extension_to_resource_type
  → Rust: CreateResourceRequest → AnalyticsResourceStore::create_resource
  → Rust: if removeAfter → ScratchpadStore::delete_entry (软删除到 .trash/)
  → 返回 PromoteResult { resource, removed }
  → 前端: removeAfter 时自动 loadFiles() 刷新面板
```

### v3.0 已完成 ✅

### UX 增强 — 键盘导航/重命名反馈/提升事件（阶段二十）

| # | 任务 | 状态 | 涉及文件 |
|---|------|:--:|------|
| 1 | ↑↓ 键盘导航 + scrollIntoView 自动滚动 | ✅ | `ScratchpadPanel.vue` |
| 2 | Enter 打开选中文件 | ✅ | `ScratchpadPanel.vue` |
| 3 | 重命名 empty 防提交 | ✅ | `ScratchpadTreeNode.vue` |
| 4 | 重命名 loading spinner + disabled | ✅ | `ScratchpadTreeNode.vue` |
| 5 | promote 后 emit `analytics-resource-changed` | ✅ | `scratchpad_commands.rs` |
| 6 | TreeNode `children` 修复为 `entry.children \|\| []` | ✅ | `ScratchpadTreeNode.vue` |

### v3.0 变更文件

| 文件 | 改动 | 行数 |
|------|------|:--:|
| `ScratchpadPanel.vue` | +`handleKeydown`: ArrowUp/Down/Enter 分支 + `scrollToSelected` | +30 |
| `ScratchpadTreeNode.vue` | +`renamingSaving` + `.rename-wrapper` + `.rename-spinner` + disabled input | +30 |
| `scratchpad_commands.rs` | +`app: AppHandle` param + `app.emit("analytics-resource-changed")` | +3 |

### 键盘导航快捷键表

| 按键 | 行为 | 说明 |
|------|------|------|
| `↑` / `↓` | 移动选中项 | 在 `filteredLocalEntries` 内循环边界 |
| `Enter` | 打开选中文件 | = `dblclick` 行为 |
| `F2` | 开始重命名 | 保持 |
| `Delete` | 删除选中项 | 软删除到 `.trash/` |
| `Ctrl+N` | 新建文件 | 弹出文件名输入框 |

### v3.1 已完成 ✅

### 文件排序 + 搜索行上下文（阶段二十一）

| # | 任务 | 状态 | 涉及文件 |
|---|------|:--:|------|
| 1 | Rust `ScratchpadEntry` 精简（移除 `extension`/`is_external_ref`，`modified_at` 改为 `Option<String>` ISO 8601） | ✅ | `models.rs`, `store.rs` |
| 2 | `SearchMatch` 结构体 + `search_file_content` 返回 `Vec<SearchMatch>` | ✅ | `models.rs`, `store.rs`, `mod.rs`, `commands.rs` |
| 3 | 前端 `SearchMatch` 类型 + API/composable 返回类型更新 | ✅ | `types/index.ts`, `api.ts`, `use-scratchpad.ts` |
| 4 | 排序下拉菜单（按名称/大小/修改时间，升序/降序切换） | ✅ | `ScratchpadPanel.vue` |
| 5 | 搜索行上下文展示（文件→匹配计数→行号+行内容，最多 5 行） | ✅ | `ScratchpadPanel.vue` |
| 6 | i18n 扩展（7 个新 key） | ✅ | `zh-CN.json` |

### v3.1 变更文件

| 文件 | 改动 | 行数 |
|------|------|:--:|
| `core/scratchpad/models.rs` | 移除 `extension`/`is_external_ref` + `modified_at` 改类型 + 新增 `SearchMatch` | ~12 |
| `core/scratchpad/store.rs` | `scan_dir`/`search_file_content`/`create_entry`/`restore_entry`/`rename_entry`/`import_file`/`get_analyzable_files` 适配 | ~30 |
| `core/scratchpad/mod.rs` | pub use 新增 `SearchMatch` | +1 |
| `commands/scratchpad_commands.rs` | `search_scratchpad_content` 返回类型改为 `Vec<SearchMatch>` | +2 |
| `types/index.ts` | 新增 `modified_at` / `SearchMatch` + 移除 `extension`/`is_external_ref` | +6 |
| `scratchpad-api.ts` | `searchFileContent` 返回 `Promise<SearchMatch[]>` | +1 |
| `use-scratchpad.ts` | `searchContent` 返回 `Promise<SearchMatch[]>` | +1 |
| `ScratchpadPanel.vue` | 排序工具栏按钮 + `contentResults` Map + 搜索结果面板 + CSS | +120 |
| `zh-CN.json` | 7 个新 i18n 键 | +7 |

### 排序交互细节

- 默认按**名称升序**排列（首次加载）
- 工具栏右侧三个按钮：按名称 / 按大小 / 按修改时间
- 点击当前排序字段 → 切换升序/降序，图标实时反映：
  - 非当前字段 → `ArrowUpDown`
  - 当前字段升序 → `ArrowUp`
  - 当前字段降序 → `ArrowDown`
- 排序在 `filteredLocalEntries` computed 中通过 `[...entries].sort()` 实现
- 排序与搜索过滤可同时生效（先过滤再排序）

### 内容搜索行上下文细节

- `search_file_content` 返回 `Vec<SearchMatch>`（替代旧的 `Vec<String>` 文件路径）
- 前端 `watch(searchQuery)` 调用 `searchContent()` → 按 `m.file` 分组到 `Map<string, SearchMatch[]>`
- 有结果时，搜索面板替换树视图：
  - 文件图标 + 文件名 + 匹配计数 badge（圆角灰色背景）
  - 每文件最多显示前 5 行匹配：行号（等宽右对齐）+ 行内容（等宽截断）
  - 超过 5 行显示 "...还有 N 处匹配未显示"
- 无结果时，树视图正常显示
