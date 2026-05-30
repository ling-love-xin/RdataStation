# 多模式编辑器面板 实施计划

> Spec: `.trae/specs/multi-mode-editor/spec.md`
> Tasks: `.trae/specs/multi-mode-editor/tasks.md`
> Checklist: `.trae/specs/multi-mode-editor/checklist.md`

---

## 一、目标总览

将当前单一 SQL 查询编辑器重构为 **VSCode 风格的面板工厂架构**，支持三种编辑器模式：

```
EditorPanelFactory.vue  (面板路由)
├─ QueryEditorPanel.vue    查询编辑器 — DBeaver/DataGrip 风格
├─ AnalysisEditorPanel.vue 分析编辑器 — DuckDB 纯分析引擎
└─ CodeEditorPanel.vue     代码编辑器 — 预留 LSP 扩展
```

---

## 二、实施步骤

### Step 1: 定义 EditorType 与编辑器类型系统

**文件**: `src/extensions/builtin/workbench/types/editor-types.ts`

新增以下类型定义：

```typescript
// 编辑器模式枚举
export type EditorType = 'query' | 'analysis' | 'code'

// 工具栏按钮配置
interface ToolbarButton {
  id: string           // 唯一标识
  group: 'execute' | 'edit' | 'mode' | 'transaction' | 'federation' | 'general'
  label: string        // 按钮文本
  icon?: string        // lucide 图标名
  shortcut?: string    // 快捷键 (如 'Ctrl+Enter')
  visible: (ctx: EditorContext) => boolean  // 可见性条件
  action: (ctx: EditorContext) => void      // 点击行为
}

// 工具栏分隔符
interface ToolbarSeparator {
  type: 'separator'
  group: string
}

type ToolbarItem = ToolbarButton | ToolbarSeparator

// 工具栏配置
interface EditorToolbarConfig {
  items: ToolbarItem[]
}

// 状态栏字段配置
interface StatusBarField {
  id: string
  label: string
  value: () => string
  visible: (type: EditorType) => boolean
}

// 编辑器面板上下文
interface EditorContext {
  editorType: EditorType
  filePath: string
  language: string
  editorView: EditorView | null
  connectionId: string | null
  isExecuting: boolean
}
```

**新增 `EditorModeResolver`**：
```typescript
// 根据文件扩展名返回默认 EditorType
function resolveEditorType(filePath: string, language: string, userPreference?: EditorType): EditorType
// 规则：.sql → query, .duckdb.sql → analysis, .rs/.ts/.py/.js/.go → code
// 用户手动切换后优先使用 userPreference
```

---

### Step 2: 拆分 EditorManager.ts（5 个独立模块）

当前 `EditorManager.ts` (~400+ 行) 包含 7 种职责，需拆分为：

| 新模块 | 文件路径 | 职责 |
|--------|----------|------|
| `FileStateStore` | `workbench/ui/stores/file-state-store.ts` | Pinia Store，管理 openFiles / activeFilePath / isDirty / file open/close |
| `EditorInstanceRegistry` | `workbench/manager/EditorInstanceRegistry.ts` | EditorView 实例注册/注销/查询/主实例判定/状态保存恢复 |
| `ResultSetManager` | `workbench/manager/ResultSetManager.ts` | 结果集 CRUD / MAX_RESULT_SETS 限制 & toast 提示 / 面板绑定 |
| `DockviewBridge` | `workbench/manager/DockviewBridge.ts` | dockviewApi 封装 / addPanel / movePanelOrGroup / getPanel / 浮动弹窗 |
| `EditorModeResolver` | `workbench/manager/EditorModeResolver.ts` | 文件扩展名 → EditorType 映射 |

**拆分策略**：
1. 先创建 5 个新模块文件，逐个抽出原 `EditorManager.ts` 的方法
2. 维持 `EditorManager` 作为兼容性 re-export 层（逐步废弃）
3. 全量搜索替换引用后，删除原文件

---

### Step 3: 修复 IPC 类型安全（前端 ↔ 后端）

**文件**: `src/extensions/builtin/query/ui/services/query.ts`

当前问题：所有 specta 调用都使用 `as unknown as` 双重强转。

**修复方案**：
1. 确认 specta `ExecuteSqlResponse` 结构体字段与前端 `ExecuteSqlResponse` 一致
2. 若不一致，修改前端 interface 以对齐 specta 类型，而非强转
3. 消除 `query.ts` 中全部 8 处 `as unknown as`
4. 为 `execute_duckdb_accelerated` 添加 `#[specta::specta]` 并在 specta 绑定中注册
5. 移除 `DuckDBAcceleratedResult` 手写 interface，改为从 specta 导入

**关键文件变更**：
```
src-tauri/src/commands/sql_commands.rs  — 添加 #[specta::specta] 到 execute_duckdb_accelerated
src-tauri/src/main.rs                   — collect_commands! 注册新命令
src/extensions/builtin/query/ui/services/query.ts — 替换 tauriInvoke 为 typed(commands.xxx)
```

---

### Step 4: 后端 — QueryResult 新增 column_types

**文件**: `src-tauri/src/core/models.rs`

```rust
impl QueryResult {
    /// 从 Arrow Schema 提取列类型名称
    pub fn column_types(&self) -> Vec<String> {
        self.batches
            .first()
            .map(|batch| {
                batch.schema()
                    .fields()
                    .iter()
                    .map(|f| {
                        // Arrow DataType → 简化的类型名
                        match f.data_type() {
                            DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64
                            | DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64
                                => "INTEGER",
                            DataType::Float16 | DataType::Float32 | DataType::Float64
                                => "FLOAT",
                            DataType::Utf8 | DataType::LargeUtf8
                                => "TEXT",
                            DataType::Boolean => "BOOLEAN",
                            DataType::Null => "NULL",
                            DataType::Binary | DataType::LargeBinary | DataType::FixedSizeBinary(_)
                                => "BYTES",
                            _ => "TEXT", // fallback
                        }.to_string()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
```

修改 `Serialize` 实现：
```rust
impl Serialize for QueryResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("QueryResult", 6)?;
        state.serialize_field("columns", &self.columns)?;
        state.serialize_field("column_types", &self.column_types())?;  // ← 新增
        state.serialize_field("rows", &self.to_rows())?;
        state.serialize_field("affected_rows", &self.affected_rows)?;
        state.serialize_field("is_read_only", &self.is_read_only)?;
        state.serialize_field("total_rows", &self.total_rows())?;
        state.end()
    }
}
```

前端同步新增：
```typescript
// src/extensions/builtin/workbench/ui/types/result.ts
interface QueryResult {
  columns: string[]
  columnTypes: string[]  // ← 新增
  rows: unknown[][]
  ...
}
```

---

### Step 5: 后端 — 分页查询命令

**文件**: `src-tauri/src/commands/sql_commands.rs`

新增命令：

```rust
#[derive(serde::Deserialize, specta::Type)]
pub struct ExecuteSqlPaginatedInput {
    pub conn_id: Option<String>,
    pub sql: String,
    pub offset: u32,
    pub limit: u32,
    pub timeout_ms: Option<u32>,
}

#[tauri::command]
#[specta::specta]
pub async fn execute_sql_paginated(
    input: ExecuteSqlPaginatedInput,
) -> Result<ExecuteSqlResponse, CoreError> {
    // 1. 执行完整查询获取 QueryResult
    // 2. 调用 result.slice(offset, limit) 截取行
    // 3. total_rows 返回原始总行数
    // 4. 序列化时仅包含截取后的行
}
```

前端新增：
```typescript
// src/extensions/builtin/query/ui/services/query.ts