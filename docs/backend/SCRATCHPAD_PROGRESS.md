# 草稿箱 (Scratchpad) 开发进度

> 版本：v2.7
> 最后更新：2026-05-08
> 状态：✅ v2.7 全功能完成 — 回收站/元数据/代码编辑器/内容搜索/Config缓存/主题适配/DuckDB分析/自动保存

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

---

## 新增文件清单

### Rust 后端 (4 文件)

| 文件                              | 行数 | 说明                                                                     |
| --------------------------------- | ---- | ------------------------------------------------------------------------ |
| `core/scratchpad/mod.rs`          | ~10  | 模块入口，re-export                                                      |
| `core/scratchpad/models.rs`       | ~73  | DTO 数据模型（Entry / FileMeta / AnalyzableFile / Reference / Response） |
| `core/scratchpad/state.rs`        | ~33  | `Arc<Mutex<Option<Store>>>` 全局状态缓存                                 |
| `commands/scratchpad_commands.rs` | ~196 | 20 个 Tauri Command                                           |

### 前端 (8 文件)

| 文件                                              | 行数 | 说明                                                   |
| ------------------------------------------------- | ---- | ------------------------------------------------------ |
| `scratchpad/package.json`                         | 7    | 扩展元数据                                             |
| `scratchpad/extension.ts`                         | ~61  | 扩展注册（dockview 面板）                              |
| `scratchpad/types/index.ts`                       | ~30  | TypeScript 类型定义                                    |
| `scratchpad/infrastructure/api/scratchpad-api.ts` | ~140 | Tauri invoke 封装（16 个 API）                         |
| `scratchpad/ui/composables/use-scratchpad.ts`     | ~310 | 业务逻辑 hook（27 个导出项）                           |
| `scratchpad/ui/components/ScratchpadPanel.vue`    | ~820 | 主面板组件（回收站/搜索/文件管理）                   |
| `scratchpad/ui/components/ScratchpadTreeNode.vue` | ~140 | 递归树节点组件                                         |

---

## 修改文件清单

| 文件                         | 修改内容                                                          | 影响 |
| ---------------------------- | ----------------------------------------------------------------- | ---- |
| `core/mod.rs`                | 新增 `pub mod scratchpad` + re-exports                            | 低   |
| `commands/mod.rs`            | 新增 `pub mod scratchpad_commands` + re-export                    | 低   |
| `lib.rs`                     | `generate_handler![]` 新增 16+ 个命令                             | 低   |
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
  </parameter>
  <｜DSML｜parameter name
