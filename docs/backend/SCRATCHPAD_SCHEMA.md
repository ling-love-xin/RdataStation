# 草稿箱模块 — 全栈数据模型与接口文档

> 版本：v1.8
> 最后更新：2026-05-08
> 状态：✅ v1.8 全栈打通 — 17 API / 29 composable / 20 commands

---

## 一、模块定位

草稿箱（Scratchpad）是用户随项目携带的临时文件暂存区，存放 SQL 脚本、分析用数据文件（CSV/Parquet/JSON/Excel）、Python 脚本、笔记等。所有文件存储在项目目录下的 `.scratchpad/` 中。

**核心差异**：与分析资源管理器（Analytics Resource）不同，草稿箱没有数据库元数据表，纯文件系统操作——简单、直接、本地化。

---

## 二、目录结构

### 2.1 物理存储

```
project/
├── .scratchpad/
│   ├── my_query.sql           # 用户 SQL 文件
│   ├── sales_data.csv         # 分析用数据文件（DuckDB 可直接导入）
│   ├── notes/                 # 用户文件夹（最多 4 层嵌套）
│   │   └── schema.md
│   ├── .trash/                # 回收站
│   │   └── old_file.sql       # 被删除的文件（软删除）
│   └── .scratchpad.json       # 草稿箱配置（外部引用列表）
```

### 2.2 后端代码（Rust）

```
src-tauri/src/
├── core/scratchpad/
│   ├── mod.rs          # re-export（~10 行）
│   ├── models.rs       # DTO：Entry / AnalyzableFile / Reference / Response（~49 行）
│   ├── state.rs        # Arc<Mutex<Option<Store>>> 全局缓存（~33 行）
│   └── store.rs        # 文件系统操作（~749 行）
├── commands/
│   └── scratchpad_commands.rs  # 18 个 Tauri Command（~166 行）
└── lib.rs              # 状态注册 + generate_handler! 注册
```

### 2.3 前端代码（Vue 3 + TypeScript）

```
src/extensions/builtin/scratchpad/
├── package.json                        # 扩展元数据
├── extension.ts                        # 扩展入口（注册 dockview 面板）
├── types/
│   └── index.ts                        # TypeScript 类型定义
├── infrastructure/
│   └── api/
│       └── scratchpad-api.ts           # 11 个 Tauri invoke 封装函数
└── ui/
    ├── composables/
    │   └── use-scratchpad.ts           # 业务逻辑 hook（21 个导出项）
    └── components/
        ├── ScratchpadPanel.vue         # 主面板（工具栏 + 分组 + 搜索 + 新建/导入/引用模态框 + 右键菜单）
        └── ScratchpadTreeNode.vue      # 递归树节点（图标映射 + 内联重命名）
```

---

## 三、核心数据结构

### 3.1 Rust 侧（后端）

#### ScratchpadEntry

```rust
pub struct ScratchpadEntry {
    pub name: String,              // 文件/文件夹名
    pub path: PathBuf,             // 绝对路径
    pub kind: ScratchpadEntryKind, // File | Folder
    pub size: u64,                 // 文件大小（字节）
    pub modified_at: DateTime<Utc>,// 最后修改时间
    pub extension: String,         // 扩展名（含点，如 ".sql"）
    pub is_external_ref: bool,     // 是否为外部引用
}
```

#### AnalyzableFile

```rust
pub struct AnalyzableFile {
    pub name: String,              // 文件名
    pub relative_path: String,     // 相对路径（相对于 .scratchpad/）
    pub file_type: String,         // csv/parquet/json/xlsx/sqlite/duckdb
    pub size_bytes: u64,           // 文件大小
    pub duckdb_query_hint: String, // DuckDB 推荐查询 SQL
}
```

#### ExternalReference

```rust
pub struct ExternalReference {
    pub alias: String,             // 别名
    pub path: PathBuf,             // 外部文件/目录绝对路径
    pub created_at: DateTime<Utc>,
}
```

#### ScratchpadResponse

```rust
pub struct ScratchpadResponse {
    pub local_entries: Vec<ScratchpadEntry>,
    pub external_references: Vec<ExternalReference>,
    pub scratchpad_path: PathBuf,
    pub file_meta: HashMap<String, FileMeta>,
}
```

#### FileMeta

```rust
pub struct FileMeta {
    pub last_connection_id: Option<String>,
    pub last_executed_at: Option<DateTime<Utc>>,
}
```

### 3.2 TypeScript 侧（前端）

```typescript
// src/extensions/builtin/scratchpad/types/index.ts
export type ScratchpadEntryKind = 'file' | 'folder'

export interface ScratchpadEntry {
  name: string
  path: string
  kind: ScratchpadEntryKind
  size: number
  modified_at: string
  extension: string
  is_external_ref: boolean
}

export interface ExternalReference {
  alias: string
  path: string
  created_at: string
}

export interface ScratchpadResponse {
  local_entries: ScratchpadEntry[]
  external_references: ExternalReference[]
  scratchpad_path: string
  file_meta: Record<string, FileMeta>
}

export interface FileMeta {
  last_connection_id?: string
  last_executed_at?: string
}
```

> **Rust↔TS 映射**：`serde(rename_all = "snake_case")` 保证 JSON key 一致；`PathBuf` 序列化为 `String`；`DateTime<Utc>` 序列化为 ISO 8601 字符串。

---

## 四、API 接口全表

### 4.1 Tauri Command → 前端 API 映射

| #   | Tauri Command                   | 前端封装函数                                   | 类别     | 状态 |
| --- | ------------------------------- | ---------------------------------------------- | -------- | :--: |
| 1   | `init_scratchpad_store`         | _(扩展激活时自动调用)_                         | 初始化   |  ✅  |
| 2   | `list_scratchpad_files`         | `listScratchpadFiles()`                        | 文件列表 |  ✅  |
| 3   | `create_scratchpad_entry`       | `createScratchpadEntry(name, isFolder)`        | CRUD     |  ✅  |
| 4   | `delete_scratchpad_entry`       | `deleteScratchpadEntry(relativePath)`          | CRUD     |  ✅  |
| 5   | `rename_scratchpad_entry`       | `renameScratchpadEntry(relativePath, newName)` | CRUD     |  ✅  |
| 6   | `read_scratchpad_file`          | `readScratchpadFile(relativePath)`             | 读写     |  ✅  |
| 7   | `save_scratchpad_file`          | `saveScratchpadFile(relativePath, content)`    | 读写     |  ✅  |
| 8   | `import_external_file`          | `importExternalFile(sourcePath)`               | 导入     |  ✅  |
| 9   | `add_external_reference`        | `addExternalReference(alias, path)`            | 引用     |  ✅  |
| 10  | `remove_external_reference`     | `removeExternalReference(alias)`               | 引用     |  ✅  |
| 11  | `open_scratchpad_in_explorer`   | `openInExplorer(path)`                         | 工具     |  ✅  |
| 12  | `check_scratchpad_file_size`    | `checkFileSize(relativePath)`                  | 工具     |  ✅  |
| 13  | `get_analyzable_files`          | `getAnalyzableFiles()`                           | 分析     |  ✅  |
| 14  | `update_scratchpad_file_meta`   | `updateFileMeta(relativePath, connectionId?)`   | 元数据   |  ✅  |
| 15  | `search_scratchpad_content`     | `searchFileContent(query)`                      | 搜索     |  ✅  |
| 16  | `list_scratchpad_trash`         | `listTrash()`                                    | 回收站   |  ✅  |
| 17  | `restore_scratchpad_from_trash` | `restoreFromTrash(trashName)`                   | 回收站   |  ✅  |
| 18  | `empty_scratchpad_trash`        | `emptyTrash()`                                  | 回收站   |  ✅  |

> **状态说明**：18/18 main 命令全部封装（不含 1 个 size_check + 1 个 analysis）。回收站、元数据、内容搜索全栈打通。

### 4.2 前端 Composable（useScratchpad）完整导出

```typescript
export function useScratchpad() {
  return {
    // ── 响应式状态 ──
    response, // Ref<ScratchpadResponse | null>
    isLoading, // Ref<boolean>
    error, // Ref<string | null>
    searchQuery, // Ref<string>
    localEntries, // ComputedRef<ScratchpadEntry[]>   (搜索过滤)
    externalReferences, // ComputedRef<ExternalReference[]> (搜索过滤)
    scratchpadPath, // ComputedRef<string>
    invalidReferences, // ComputedRef<ExternalReference[]>
    validReferences, // ComputedRef<ExternalReference[]>

    // ── CRUD ──
    loadFiles, // () => Promise<void>
    createEntry, // (name, isFolder) => Promise<ScratchpadEntry | null>
    deleteEntry, // (relativePath) => Promise<boolean>
    renameEntry, // (relativePath, newName) => Promise<ScratchpadEntry | null>
    loadFileContent, // (relativePath) => Promise<string | null>
    saveFile, // (relativePath, content) => Promise<boolean>

    // ── 导入 & 引用 ──
    importFile, // (sourcePath) => Promise<ScratchpadEntry | null>
    addReference, // (alias, path) => Promise<ExternalReference | null>
    removeReference, // (alias) => Promise<boolean>

    // ── 工具 ──
    isRefValid, // (ref) => boolean
    isRefInvalid, // (ref) => boolean
    findEntry, // (entryPath) => ScratchpadEntry | undefined
    openInExplorerAction, // (path) => Promise<boolean>
    getFileSize, // (relativePath) => Promise<number | null>
    clearError, // () => void
    saveFileMeta, // (relativePath, connectionId?) => Promise<boolean>
    searchContent, // (query) => Promise<string[]>
    trashEntries, // Ref<ScratchpadEntry[]>
    loadTrashEntries, // () => Promise<void>
    restoreTrashEntry, // (trashName) => Promise<boolean>
    emptyTrashBin, // () => Promise<boolean>
    analyzableFiles, // Ref<AnalyzableFile[]>
    loadAnalyzableFiles, // () => Promise<void>
  }
}
```

---

## 五、全栈数据流

```
┌── Frontend (Vue 3 + TS) ────────────────────────────────────┐
│  ScratchpadPanel.vue ─ ─ use ─ ─ ▶ useScratchpad()          │
│       │                             │ invoke()               │
│       ▼                             ▼                        │
│  ScratchpadTreeNode.vue    scratchpad-api.ts                 │
│  (递归树 + 内联重命名)      (11 个 IPC 封装)                  │
├──────────────────────────────────────┼───────────────────────┤
│                            Tauri IPC Bridge                   │
├──────────────────────────────────────┼───────────────────────┤
│                                      ▼                        │
│  scratchpad_commands.rs (18 cmd) ─ ─ ▶ ScratchpadState        │
│                                      └─ ─ ▶ ScratchpadStore   │
│                                           ├─ resolve_path()  │
│                                           ├─ scan_dir()      │
│                                           ├─ save_file()     │
│                                           └─ delete_entry()  │
│                                                ↓              │
│                                  {project}/.scratchpad/       │
└──────────────────────────────────────────────────────────────┘
```

---

## 六、路径安全 & 原子写入 & 回收站

| 机制             | 实现                                                                                                                          |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| **路径遍历防护** | `resolve_path_impl` 拒绝 `..` + canonicalize 前缀校验 + `validate_name` 拒绝 `/` `\` `..`                                     |
| **原子写入**     | `save_file`: `fs::write(tmp_path, content)` → `fs::rename(tmp_path → file_path)` → 失败清理 tmp                               |
| **回收站**       | `delete_entry`: `rename(target → .trash/unique_name)`；恢复时 `rename` 回 `.scratchpad/`；`empty_trash` 直接 `remove_dir_all` |

---

## 七、编辑器类型映射（openFileInEditor）

| 后缀                     | 目标编辑器                                            |  状态   |
| ------------------------ | ----------------------------------------------------- | :-----: |
| `.sql`                   | **sql-editor**（核心场景——需打通连接管理 + 执行引擎） | ⏳ v2.1 |
| `.py`                    | code-editor（Monaco 通用）                            |   ⏳    |
| `.csv`                   | data-preview（DuckDB 导入场景）                       |   ⏳    |
| `.json` / `.txt` / `.md` | code-editor                                           |   ⏳    |

> **⚠️ 当前**：`openFileInEditor()` 骨架已就绪（含 50MB 大小预检），但实际编辑器面板创建逻辑尚未打通——这是 v2.1 阶段核心任务。

---

## 八、文件变更记录（自 v1.1 起）

| 变更                    | 文件                        | 说明                                                                          |
| ----------------------- | --------------------------- | ----------------------------------------------------------------------------- |
| ✅ ScratchpadState 缓存 | `state.rs` (新)             | `Arc<Mutex<Option<ScratchpadStore>>>` 避免每次命令重建 Store                  |
| ✅ 回收站 (.trash/)     | `store.rs`                  | `delete_entry` 改为 rename 到 `.trash/`；新增 list/restore/empty              |
| ✅ DuckDB 分析元数据    | `models.rs`                 | `AnalyzableFile` 结构体 + `duckdb_query_hint` 映射表 + `get_analyzable_files` |
| ✅ 命令层重构           | `commands`                  | 全部改用 `ScratchpadState`；新增 4 命令                                       |
| ✅ 安全合规修复         | `store.rs`                  | `resolve_path` 中 unwrap 替换为 CoreError                                     |
| ✅ 原子写入             | `store.rs`                  | 先写 `.tmp` 临时文件，再 rename；失败清理 tmp                                 |
| ✅ 前端对话框           | `ScratchpadPanel.vue`       | 新建/导入/引用的模态框                                                        |
| ✅ 右键菜单             | `ScratchpadPanel.vue`       | 打开/重命名/删除/复制路径/展开折叠 + 移除引用/打开位置                        |
| ✅ F2 内联重命名        | `ScratchpadTreeNode.vue`    | Enter/Escape/Blur 确认取消                                                    |
| ✅ 大文件检查           | Panel + Store               | 前端 50MB 前置校验 + 后端 `check_file_size`                                   |
| ✅ 系统管理器打开       | `store.rs` + `opener` crate | 外部引用右键"打开位置"                                                        |
| ✅ 右键菜单溢出检测     | `ScratchpadPanel.vue`       | `clampToViewport()` 防越界                                                    |

</parameter>
</｜DSML｜inv
