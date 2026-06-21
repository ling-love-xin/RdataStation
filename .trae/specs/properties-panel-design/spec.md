# 属性面板（Properties Panel）设计文档

> 版本：v2.1（含承载力分析 + 务实方案）
> 日期：2026-06-09
> 状态：设计阶段
> 原型：[properties-panel-dbeaver.html](../../prototype/properties-panel-dbeaver.html)

---

## 零、项目实际承载力分析

### 0.1 后端现状

| 方法 | MySQL | PostgreSQL | SQLite | DuckDB |
|------|:-----:|:----------:|:------:|:------:|
| `list_columns` | ✅ 已实现 | ✅ 已实现 | ❌ todo!() | ❌ todo!() |
| `list_indexes` | ⚠️ 部分 | ✅ 已实现 | ❌ todo!() | ❌ todo!() |
| `list_constraints` | ❌ 未实现 | ✅ 已实现 | ❌ todo!() | ❌ todo!() |
| `get_ddl` | ❌ 不存在 | ❌ 不存在 | ❌ 不存在 | ❌ 不存在 |
| `meta()` | ✅ | ✅ | ✅ | ✅ |

### 0.2 前端现状

```
databaseNavigatorStore.connectionCatalogs: Map<connectionId, CatalogNode[]>
  └── CatalogNode.schemas[]
       └── SchemaNode.tables[]: TableNode
            ├── name, type, rowCount?, dataLength?, indexLength?
            ├── columns: ColumnNode[]         ← 树展开时已加载 ✅
            ├── indexes?: IndexNode[]         ← 仅 PostgreSQL 已加载
            └── constraints?: ConstraintNode[] ← 仅 PostgreSQL 已加载

workbenchStore.addPanel({ type: 'properties', ... })  ← 已存在 ✅
```

### 0.3 结论：务实方案

**M1 可以立即实现，零后端改动。** 属性面板从 store 已有数据渲染，数据库驱动补齐与前端开发可并行推进。

| 数据来源 | 覆盖率 | 说明 |
|----------|:------:|------|
| Columns | 100% | 树展开时已加载，所有驱动都有 |
| Indexes | 25% | 仅 PostgreSQL，其他驱动 `todo!()` |
| Constraints | 25% | 同上 |
| DDL | 0% | 无任何驱动实现 `get_ddl` |
| 统计信息 | 100% | `rowCount`, `dataLength`, `indexLength` 已在 TableNode 中 |

**M1 策略：有数据就展示，没有就优雅降级。** 例如 Indexes Tab 数据为空时显示 "此数据库驱动尚未支持索引查询"。

### 0.4 分层推进计划

```
M1: 零后端改动，store 数据渲染
│   前端: PropertiesEditor + PropertiesPane + SubEntityPane + DdlViewer(占位)
│   后端: 无
│   交付: 表/视图双击 → 属性面板（Columns + 统计）
│
M2: 补齐驱动，激活子实体 Tab
│   后端: list_indexes/list_constraints 在 SQLite/DuckDB 实现
│   前端: 无变化（数据管道自动填充）
│   交付: 属性面板完整（Columns + Indexes + Constraints + FK + Triggers）
│
M3: DDL 生成
│   后端: Database trait 新增 get_ddl()
│   前端: DdlViewer 激活
│   交付: DDL Tab 展示完整建表语句
│
M4: 全节点覆盖
│   后端: 连接/Catalog/Schema 属性聚合
│   前端: 所有节点双击 → 属性面板
│   交付: 全功能
```

---

## 一、DBeaver 核心设计分析

DBeaver 的 Properties Editor 是一个完整的数据库对象编辑器，核心布局：

```
┌──────────────────────────────────────────────────────┐
│  [Properties]  [Data]               ← 顶部 Tab 栏    │
├──────────────────────────────────────────────────────┤
│  🗄 local_postgres / mydb / public / users           │ ← 面包屑
├──────────────────────────────────────────────────────┤
│  ┌──────────────────┬──────────────────────────────┐ │
│  │ Name  │ users    │ Type  │ BASE TABLE           │ │ ← 上 pane
│  │ Owner │ postgres │ TS    │ pg_default           │ │   对象属性
│  │ ...   │ ...      │ ...   │ ...                  │ │   (4列网格)
│  └──────────────────┴──────────────────────────────┘ │
├────────┬─────────────────────────────────────────────┤
│ Columns│ [🔄] [＋] [🗑]           [🔍 search...] [⚙] │ ← 子实体工具栏
│        │ ┌────┬──────┬────────┬────┬──────────────┐ │
│ Const..│ │ PK │ id   │ int4   │ NO │ -            │ │ ← 子实体表格
│ Indexes│ │    │ name │ varchar│ NO │ -            │ │
│ FK     │ │ .. │ ...  │ ...    │ .. │ ...          │ │
│ Trig.. │ └────┴──────┴────────┴────┴──────────────┘ │
│ DDL    │                                             │
│ Perm.. │                                             │
├────────┴─────────────────────────────────────────────┤
│  [Revert]  [Save]     (脏状态时激活)                  │
└──────────────────────────────────────────────────────┘
```

### 关键特征

| 特征 | 说明 |
|------|------|
| **上下分割** | 上 pane = 对象属性（可编辑 4 列网格），下 pane = 子实体 |
| **左侧垂直 Tab** | 子实体切换用左侧竖排标签（Columns / Constraints / Indexes / FK / Triggers / DDL / Permissions） |
| **顶部 Tab 栏** | Properties / Data 双 Tab |
| **可编辑** | 对象属性直接编辑，修改后 Save/Revert 按钮激活 |
| **子实体工具栏** | 每个子实体独立工具栏：刷新、新建、删除、搜索、列配置 |
| **DDL 特殊处理** | DDL 独立工具栏：Load/Save file、Open in SQL console |

---

## 二、双击行为变更

| 节点类型 | 当前行为 | 目标行为 |
|----------|----------|----------|
| 连接 | 无操作 | 打开属性编辑器 |
| Catalog/数据库 | 无操作 | 打开属性编辑器 |
| Schema | 无操作 | 打开属性编辑器 |
| 表 | 打开 SQL 编辑器 + `SELECT *` | 打开属性编辑器（默认选中 Columns） |
| 视图 | 同上 | 打开属性编辑器（默认选中 Columns） |
| 列 | 无操作 | 打开属性编辑器（默认选中该列） |
| 索引 | 无操作 | 打开属性编辑器（默认选中 Indexes） |
| 文件夹节点 | 展开/折叠 | 不变 |

右键菜单保留"查看数据"、"查看 DDL"、"编辑表"等入口。

---

## 三、后端设计

### 3.1 数据模型

```rust
/// 属性编辑器请求
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ObjectKey {
    pub connection_id: String,
    pub catalog: Option<String>,
    pub schema: Option<String>,
    pub name: String,
    pub kind: SchemaObjectKind,
}

/// 属性编辑器响应
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ObjectProperties {
    pub object_key: ObjectKey,
    pub properties: Vec<PropertyRow>,    // 上 pane 4 列网格
    pub sub_entities: Vec<SubEntity>,    // 下 pane 子实体列表
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SubEntity {
    pub id: String,               // columns | constraints | indexes | foreign_keys | triggers | ddl
    pub label: String,
    pub icon: String,
    pub count: Option<u32>,
    pub table: Option<PropertyTable>,
    pub code: Option<String>,     // DDL 专用
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PropertyRow {
    pub key: String,
    pub value: String,
    pub editable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PropertyTable {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<Vec<String>>,
}
```

### 3.2 Tauri Command

```rust
#[tauri::command]
#[specta::specta]
async fn get_object_properties(
    state: tauri::State<'_, AppState>,
    connection_id: String,
    object_key: ObjectKey,
) -> Result<ObjectProperties, String> {
    // 按 object_key.kind 分发 → 聚合 properties + sub_entities
}
```

---

## 四、前端设计

### 4.1 组件树

```
database-navigator.vue (双击)
  └── use-properties-panel.ts (composable)
       └── workbenchStore.addPanel({ type: 'properties', component: 'PropertiesEditor' })

PropertiesEditor.vue (顶层)
  ├── TopTabs: Properties | Data
  ├── BreadcrumbBar
  ├── PropertiesPane (上 pane: 4 列网格)
  │    └── PropertyGrid (可编辑字段)
  └── SubEntityPane (下 pane)
       ├── SubEntityTabs (左侧垂直 Tab)
       └── SubEntityContent (右侧)
            ├── SubEntityToolbar
            ├── DataTable (子实体表格)
            └── DdlViewer (DDL 特殊视图)
```

### 4.2 文件规划

| 文件 | 职责 |
|------|------|
| `src-tauri/src/commands/properties_commands.rs` | 新增：`get_object_properties` |
| `src-tauri/src/core/properties.rs` | 新增：数据模型 |
| `src/extensions/builtin/database/ui/composables/use-properties-panel.ts` | 新增：composable |
| `src/extensions/builtin/database/ui/components/properties/PropertiesEditor.vue` | 新增：主组件 |
| `src/extensions/builtin/database/ui/components/properties/PropertiesPane.vue` | 新增：上 pane 属性网格 |
| `src/extensions/builtin/database/ui/components/properties/SubEntityPane.vue` | 新增：下 pane 子实体 |
| `src/extensions/builtin/database/ui/components/properties/DdlViewer.vue` | 新增：DDL 查看器 |
| `src/extensions/builtin/database/ui/components/database-navigator.vue` | 修改：双击入口 |

---

## 五、优化空间分析

### 5.1 交互优化

| 优化项 | 说明 | 优先级 |
|--------|------|--------|
| 属性值复制 | 点击属性值旁 📋 图标一键复制到剪贴板 | 🟡 中 |
| 子实体行双击 | 双击列/索引/约束行 → 跳转到对应对象属性 | 🟢 低 |
| 面包屑可点击 | 点击面包屑任意段 → 打开对应节点的属性 | 🟢 低 |
| 列排序 | 子实体表格支持点击列头排序 | 🟢 低 |
| 属性分组 | 上 pane 过多时（如视图 20+ 属性）分组折叠 | 🟢 低 |

### 5.2 性能优化

| 优化项 | 说明 | 优先级 |
|--------|------|--------|
| 子实体懒加载 | 首次仅加载 Columns，切换 Tab 时才加载其他子实体 | 🔴 高 |
| 属性面板缓存 | 同对象 5 分钟内不重复请求后端 | 🟡 中 |
| 虚拟滚动 | 列数 > 500 时子实体表格启用虚拟滚动 | 🟢 低 |
| 后台预加载 | 打开属性面板后，后台预加载相邻 Tab（如 Columns → 预加载 Constraints） | 🟢 低 |

### 5.3 功能扩展

| 扩展项 | 说明 | 优先级 |
|--------|------|--------|
| 连接属性 | 连接节点属性：版本、驱动、连接状态、SSL 信息 | 🔴 高 |
| Schema 属性 | Schema 节点属性：表/视图/函数计数 | 🔴 高 |
| Database 属性 | 数据库节点属性：大小、字符集、排序规则 | 🟡 中 |
| 列属性 | 列节点属性：类型、约束、外键引用 | 🟡 中 |
| 编辑持久化 | Save 按钮 → 执行 ALTER TABLE / COMMENT ON 等 DDL | 🟢 低 |
| 数据 Tab | Properties → Data 切换时执行 `SELECT * LIMIT 200` | 🟢 低 |

### 5.4 架构优化

| 优化项 | 说明 | 优先级 |
|--------|------|--------|
| 驱动 DDL 生成 | 各驱动实现 `get_ddl(object_key) -> String`，不依赖厂商特定 SQL | 🟡 中 |
| 统计信息抽象 | `get_statistics(object_key)` 统一接口，各驱动按能力实现 | 🟢 低 |
| 权限查询 | `get_permissions(object_key)` 按 `information_schema` 标准实现 | 🟢 低 |

---

## 六、开发进度指南（务实版）

### 6.1 里程碑总览

```
M1: 零后端改动 (1-2天)  →  M2: 补齐驱动 (并行)  →  M3: DDL (1天)  →  M4: 全节点 (1-2天)
   前端独立可交付            后端独立可交付            前后端联合            前后端联合
```

### 6.2 M1: 零后端改动 — store 数据渲染（1-2 天）

> **核心思路**：属性面板从 `databaseNavigatorStore.connectionCatalogs` 读取已有数据，
> 不新增任何 Rust 命令或驱动实现。有数据就展示，没有就优雅降级。

| 任务 | 产出 | 验收标准 |
|------|------|----------|
| 1.1 composable | `use-properties-panel.ts` | `open(node)` → 从 store 提取数据 → `addPanel` |
| 1.2 PropertiesEditor | `PropertiesEditor.vue` | 上下分割布局 + 面包屑渲染 |
| 1.3 PropertiesPane | `PropertiesPane.vue` | 4 列网格渲染属性（从 TableNode 字段） |
| 1.4 SubEntityPane | `SubEntityPane.vue` | 左侧 Tab + 右侧表格渲染 |
| 1.5 DdlViewer | `DdlViewer.vue` | 占位状态 "DDL 生成功能即将上线" |
| 1.6 双击改道 | `database-navigator.vue` | 表/视图双击 → 打开属性编辑器 |
| 1.7 优雅降级 | 各 Tab 空状态 | 无索引数据 → "此数据库驱动尚未支持索引查询" |

**M1 交付物**：双击任意已加载的表/视图 → 弹出属性面板，展示 Columns + 统计信息。
**文件数**：7 个前端文件，0 个后端文件，~600 行代码。

### 6.3 M2: 补齐驱动（并行，1-2 天）

| 任务 | 产出 | 验收标准 |
|------|------|----------|
| 2.1 SQLite list_columns | `sqlite.rs` | `PRAGMA table_info` → ColumnNode[] |
| 2.2 SQLite list_indexes | `sqlite.rs` | `PRAGMA index_list` + `PRAGMA index_info` → IndexNode[] |
| 2.3 DuckDB list_columns | `duckdb.rs` | `DESCRIBE` 或 `information_schema` → ColumnNode[] |
| 2.4 DuckDB list_indexes | `duckdb.rs` | `PRAGMA show_tables` 扩展 → IndexNode[] |
| 2.5 MySQL list_constraints | `mysql.rs` | `information_schema.TABLE_CONSTRAINTS` → ConstraintNode[] |

**M2 交付物**：所有驱动 Indexes + Constraints Tab 激活。前端无需改动。

### 6.4 M3: DDL 生成（1 天）

| 任务 | 产出 | 验收标准 |
|------|------|----------|
| 3.1 Database trait | `traits.rs` 新增 `get_ddl()` | 签名：`async fn get_ddl(&self, object_key: ObjectKey) -> Result<String>` |
| 3.2 PostgreSQL DDL | `pg_get_tabledef` / `pg_get_viewdef` | 返回完整建表/建视图语句 |
| 3.3 MySQL DDL | `SHOW CREATE TABLE` / `SHOW CREATE VIEW` | 同上 |
| 3.4 SQLite DDL | `SELECT sql FROM sqlite_master` | 同上 |
| 3.5 DuckDB DDL | `SELECT sql FROM duckdb_tables()` | 同上 |
| 3.6 DdlViewer 激活 | 从占位 → 真实渲染 | SQL 语法高亮 + Copy 按钮 |

**M3 交付物**：DDL Tab 展示完整建表语句。

### 6.5 M4: 全节点覆盖（1-2 天）

| 任务 | 产出 | 验收标准 |
|------|------|----------|
| 4.1 连接属性 | kind=Connection → 版本/驱动/SSL/状态 | 双击连接节点 |
| 4.2 Catalog 属性 | kind=Catalog → 大小/字符集/排序规则 | 双击数据库节点 |
| 4.3 Schema 属性 | kind=Schema → 表/视图/函数计数 | 双击 Schema 节点 |
| 4.4 列/索引属性 | 双击列/索引 → 跳转到表属性并选中对应 Tab | 双击列 → 高亮该列 |
| 4.5 右键菜单 | 右键 → 查看数据 → 切 Data Tab | 回归测试 |

**M4 交付物**：全节点双击 → 属性面板。

### 6.6 后续迭代

| 迭代 | 内容 |
|------|------|
| V2.1 | 编辑持久化（Save → ALTER TABLE / COMMENT ON） |
| V2.2 | Data Tab 集成（属性面板内嵌 SQL 编辑器 + 数据表格） |
| V2.3 | 属性面板缓存 + 虚拟滚动（大表优化） |
| V2.4 | 面包屑可点击导航 |

---

## 七、文件清单

### M1（零后端改动）

| 文件 | 类型 | 行数估算 |
|------|------|----------|
| `use-properties-panel.ts` | TS composable | ~80 |
| `PropertiesEditor.vue` | Vue 组件 | ~150 |
| `PropertiesPane.vue` | Vue 组件 | ~80 |
| `SubEntityPane.vue` | Vue 组件 | ~120 |
| `DdlViewer.vue` | Vue 组件 | ~40 |
| `database-navigator.vue` | 修改 | ~10 行 diff |

**M1 总计：6 个前端文件，0 个后端文件，~480 行代码。**

### M2-M4（后端补齐）

| 文件 | 类型 | 行数估算 |
|------|------|----------|
| `core/properties.rs` | Rust 数据模型 | ~80 |
| `commands/properties_commands.rs` | Rust 命令 | ~200 |
| `driver/native/sqlite.rs` | 修改 | ~60 行 diff |
| `driver/native/duckdb.rs` | 修改 | ~60 行 diff |
| `driver/native/mysql.rs` | 修改 | ~40 行 diff |
| `driver/traits.rs` | 修改 | ~10 行 diff |

---

## 八、风险与边界

| 风险 | 缓解 |
|------|------|
| DDL 获取在不同数据库差异大 | 每个驱动独立实现 `get_ddl()`，默认返回 "不支持" |
| 行数统计大表慢 | 异步加载 + 骨架屏 + 超时 3s 回退 |
| 属性面板与 SQL 编辑器 Tab 竞争 | 属性面板使用独立 `panelId` 命名空间 |
| 用户双击打开数据（肌肉记忆） | 右键菜单保留"查看数据"，Data Tab 一键切换 |

---

## 九、版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.0 | 2026-06-09 | 初版（折叠区块设计） |
| v2.0 | 2026-06-09 | 重构为 DBeaver 风格（上下分割 + 左侧 Tab），移除 ER Diagram，新增开发进度指南 |