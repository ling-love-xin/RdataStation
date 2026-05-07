# 草稿板模块 — 后端设计文档

> 版本：v1.1
> 最后更新：2026-05-07
> 状态：✅ P0/P1 优化完成

---

## 一、模块定位

草稿板（Scratchpad）是用户的工作文件暂存区，存放 SQL 脚本、分析用数据文件（CSV/Parquet/JSON/Excel）、笔记等。所有文件存储在项目目录下的 `.scratchpad/` 中。

**核心差异**：与分析资源管理器（Analytics Resource）不同，草稿板没有数据库元数据表，纯文件系统操作——简单、直接、本地化。

---

## 二、目录结构

```
project/
├── .scratchpad/
│   ├── my_query.sql           # 用户 SQL 文件
│   ├── sales_data.csv         # 分析用数据文件（DuckDB 可直接导入）
│   ├── notes/                 # 用户文件夹
│   │   └── schema.md
│   ├── .trash/                # 回收站（v1.1 新增）
│   │   └── old_file.sql       # 被删除的文件
│   └── .scratchpad.json       # 配置：外部引用列表
```

---

## 三、核心数据结构

### 3.1 ScratchpadEntry

```rust
pub struct ScratchpadEntry {
    pub name: String,           // 文件/文件夹名
    pub path: PathBuf,          // 绝对路径
    pub kind: ScratchpadEntryKind, // File | Folder
    pub size: u64,              // 文件大小（字节）
    pub modified_at: DateTime<Utc>,  // 最后修改时间
    pub extension: String,      // 扩展名（含点，如 ".sql"）
    pub is_external_ref: bool,  // 是否为外部引用
}
```

### 3.2 AnalyzableFile（v1.1 新增）🆕

```rust
pub struct AnalyzableFile {
    pub name: String,             // 文件名
    pub relative_path: String,    // 相对路径（相对于 .scratchpad/）
    pub file_type: String,        // 文件类型：csv/parquet/json/xlsx/sqlite/duckdb
    pub size_bytes: u64,          // 文件大小
    pub duckdb_query_hint: String, // DuckDB 推荐查询 SQL
}
```

### 3.3 ExternalReference

```rust
pub struct ExternalReference {
    pub alias: String,           // 别名
    pub path: PathBuf,           // 外部文件绝对路径
    pub created_at: DateTime<Utc>,
}
```

### 3.4 ScratchpadResponse

```rust
pub struct ScratchpadResponse {
    pub entries: Vec<ScratchpadEntry>,
    pub scratchpad_path: PathBuf,
    pub external_references: Vec<ExternalReference>,
}
```

---

## 四、API 接口

### 4.1 初始化

| Command | 说明 | 输入 | 输出 |
|---------|------|------|------|
| `init_scratchpad_store` | 打开项目时初始化 | `project_path: String` | `()` |

### 4.2 文件操作

| Command | 说明 |
|---------|------|
| `list_scratchpad_files` | 列出所有文件和文件夹 |
| `create_scratchpad_entry` | 创建文件或文件夹 |
| `delete_scratchpad_entry` | 删除（移入 `.trash/`） |
| `rename_scratchpad_entry` | 重命名 |
| `read_scratchpad_file` | 读取文件内容（文本） |
| `save_scratchpad_file` | 保存文件内容（原子写入） |
| `import_external_file` | 导入外部文件到草稿板 |
| `check_scratchpad_file_size` | 查询文件大小 |
| `open_scratchpad_in_explorer` | 在系统资源管理器中打开 |

### 4.3 外部引用

| Command | 说明 |
|---------|------|
| `add_external_reference` | 添加外部文件/目录引用 |
| `remove_external_reference` | 移除外部引用 |

### 4.4 回收站（v1.1 新增）🆕

| Command | 说明 |
|---------|------|
| `list_scratchpad_trash` | 列出回收站内容 |
| `restore_scratchpad_from_trash` | 从回收站恢复文件 |
| `empty_scratchpad_trash` | 清空回收站 |

### 4.5 分析（v1.1 新增）🆕

| Command | 说明 | 输出 |
|---------|------|------|
| `get_analyzable_files` | 获取可被 DuckDB 分析的文件列表 | `Vec<AnalyzableFile>` |

---

## 五、DuckDB 查询提示映射

| 扩展名 | DuckDB 查询 |
|--------|------------|
| `.csv` | `SELECT * FROM read_csv_auto('filename.csv');` |
| `.tsv` | `SELECT * FROM read_csv_auto('filename.tsv', delim='\\t');` |
| `.parquet` | `SELECT * FROM read_parquet('filename.parquet');` |
| `.json` / `.ndjson` | `SELECT * FROM read_json_auto('filename.json');` |
| `.xlsx` / `.xls` | `SELECT * FROM st_read('filename.xlsx');` |
| `.sqlite` / `.db` | `ATTACH 'filename.db' AS sqlite_db (TYPE sqlite);` |
| `.duckdb` | `ATTACH 'filename.duckdb' AS duckdb_db;` |

---

## 六、v1.1 变更记录

### P0 — 安全与数据完整性

| 变更 | 文件 | 说明 |
|------|------|------|
| ✅ resolve_path 支持非存在路径 | `store.rs` | 新增 `resolve_path_impl(must_exist)` 分支；`must_exist=false` 用父路径检查代替 canonicalize |
| ✅ 移除 read_file 大小限制 | `store.rs` | 删除 `MAX_FILE_SIZE` 常量和大小检查逻辑，大文件由 DuckDB 分析 |
| ✅ save_file 原子写入 | `store.rs` | 先写 `.tmp` 临时文件，再 rename 覆盖；rename 失败时清理 tmp |

### P1 — 设计增强

| 变更 | 文件 | 说明 |
|------|------|------|
| ✅ ScratchpadState 缓存 | `state.rs` (新) | `Arc<Mutex<Option<ScratchpadStore>>>` 缓存实例，避免每次命令重建 |
| ✅ 回收站 (.trash/) | `store.rs` | `delete_entry` 改为 rename 到 `.trash/`；新增 `list_trash`/`restore_from_trash`/`empty_trash` |
| ✅ DuckDB 分析元数据 | `models.rs` (新) | `AnalyzableFile` 结构体 + `duckdb_query_hint` 映射表 + `get_analyzable_files` API |
| ✅ 命令层重构 | `scratchpad_commands.rs` | 全部命令改用 `ScratchpadState`；新增 4 个命令 |

---

## 七、原子写入流程

```
save_file(path, content)
  │
  ├── resolve_path(path) → file_path      // 路径安全检查
  │
  ├── 创建 temp_path = file_path.tmp
  │
  ├── fs::write(temp_path, content)       // 写入临时文件
  │
  ├── fs::rename(temp_path → file_path)   // 原子替换
  │   │
  │   └── 失败时 if temp_path exists:
  │         fs::remove_file(temp_path)    // 清理临时文件
  │
  └── Ok(())
```

---

## 八、回收站工作流

```
delete_entry(path)
  │
  ├── resolve_path(path) → target_path    // 路径校验
  │
  ├── 确保 .trash/ 存在                    // 首次自动创建
  │
  ├── unique_path(.trash/name) → trash_target  // 避免重名
  │
  ├── fs::rename(target → trash_target)   // 移入回收站
  │
  └── Ok(())

restore_from_trash(name)
  │
  ├── unique_path(.scratchpad/name) → target  // 恢复路径
  │
  ├── fs::rename(.trash/name → target)    // 移回
  │
  └── Ok(entry)
```

---

## 九、文件结构

```
src-tauri/src/
├── core/scratchpad/
│   ├── mod.rs          # re-export
│   ├── models.rs       # ScratchpadEntry / AnalyzableFile / ExternalReference / ScratchpadResponse
│   ├── state.rs        # ScratchpadState (v1.1 新增)
│   └── store.rs        # ScratchpadStore 文件系统操作
├── commands/
│   └── scratchpad_commands.rs  # Tauri 命令层
└── lib.rs              # 状态注册 + 命令注册
```

---

## 十、DuckDB 分析工作流（产品方向）

```
用户拖入 sales_2024.csv (2GB) → 草稿板保存
                              → get_analyzable_files() 返回：
                                { name: "sales_2024.csv", 
                                  duckdb_query_hint: "SELECT * FROM read_csv_auto('sales_2024.csv');" }
                              → 前端"在 DuckDB 中打开"按钮
                              → 后端执行 hint SQL → 返回前 1000 行 + schema
                              → 用户在 SQL 编辑器中自由分析
```
