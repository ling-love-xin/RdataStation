# RdataStation 洞察体系 — 技术架构文档

> 版本：v13.0
> 创建日期：2026-05-07
> 最后更新：2026-05-08
> 状态：✅ 完整架构（DuckDB 实例统一 + 端到端联动 + 动画 + 质量评分 + 表聚合 + Schema 洞察）

---

## 一、数据流（含多列分析）

```
用户点击列头
  │  CustomEvent 'open-column-insight' { column, tempTable, allColumns }
  ▼
ColumnInsightPanel.vue
  │
  ├─ [Tab: 列洞察] → invoke('get_column_insight_full')
  │                   └─ RuleExecutor (column/*.toml)
  │
  └─ [Tab: 多列分析] → MultiColumnView.vue
       ├─ listInsightRules('multi') → 获取多列规则
       ├─ 用户选择列 + 规则
       └─ executeInsightRule({ rule_id, params, temp_table })
            └─ RuleExecutor (multi/*.toml)
```

### 表探查数据流

```
导航树 → 右键"快速探查"
  │  CustomEvent('open-table-profile') { connId, dbType, database, schema, table }
  ▼
DockviewLayout.handleTableProfile()
  │  api.addPanel({ component: 'tableProfile', params })
  ▼
TableProfileView.vue
  │  invoke('get_table_profile') → information_schema.columns + COUNT(*)
  │  → TableProfile { columns, row_count, db_type }
  └─ 点击列名 → CustomEvent('table-column-click') ← NEW
```

### 表列探查联动（端到端）← NEW

```
TableProfileView → 点击列名
  │  CustomEvent('table-column-click') { column, table, database, schema, connId }
  ▼
ColumnInsightPanel.handleTableColumnClick()
  │  insightStore.loadColumnFromTable({ connId, database, schema, table, column })
  ▼
invoke('profile_column_from_table')
  │  ResultService::profile_column_from_table()
  ├─ SqlService.execute() → SELECT * LIMIT 500 (取样)
  ├─ create_duckdb_temp_table() → DuckDB temp table
  └─ get_column_insight_full() → ColumnInsightFull (洞察)
      ▼
  洞察面板展示（NCollapse/basic/dist/quality/sample）
```

---

## 二、规则引擎模块

### 全局注册表（可变）

```rust
// insight/mod.rs
static GLOBAL_REGISTRY: OnceLock<RwLock<RuleRegistry>> = OnceLock::new();

pub fn global_registry() -> &'static RwLock<RuleRegistry> { ... }
pub fn load_user_rules(project_path: &Path) { ... }  // 项目打开时调用
```

### 内置规则 (13 条)

| 目录      | ID                                                                                                                                | 类型 |
| --------- | --------------------------------------------------------------------------------------------------------------------------------- | ---- |
| `column/` | null-check, numeric-stats, numeric-basic, text-frequency, text-length, datetime-range, datetime-monthly, boolean-ratio, histogram | 单列 |
| `multi/`  | correlation, grouped-stats, cross-tab, scatter-sample                                                                             | 多列 |

### Tauri Commands

| 命令                        | 输入                                          | 输出                                |
| --------------------------- | --------------------------------------------- | ----------------------------------- |
| `execute_insight_rule`      | `{ rule_id, params, temp_table }`             | `Value` (动态JSON)                  |
| `list_insight_rules`        | `category?`                                   | `RuleMeta[]`                        |
| `list_rules_for_column`     | `{ column_type }`                             | `RuleMeta[]`                        |
| `get_table_profile`         | `{ connId, dbType, database, schema, table }` | `TableProfile` (表探查)             |
| `profile_column_from_table` | `{ connId, database, schema, table, column }` | `ColumnInsightFull` (表列探查)      |
| `get_column_quality`        | `{ column_name, temp_table }`                 | `QualityScore` (质量评分)           |
| `batch_evaluate_columns`    | `{ conn_id, database, schema, table }`        | `TableQuality` (表级质量)           |
| `get_schema_insight`        | `{ conn_id, database, schema }`               | `SchemaInsightReport` (Schema 洞察) |

---

## 三、前端组件树

```
ColumnInsightPanel.vue
└─ NTabs { default: 'column' }
    ├─ NTabPane name="column" tab="列洞察"
    │   ├─ [Empty/Loading(骨架屏)/Error/Data] 四状态
    │   ├─ HeaderActions: Download(导出JSON) + FileText(导出Markdown) + Save(保存快照)
    │   ├─ QualityScore → 评分徽章(分数+等级+颜色) + 四维度进度条
    │   ├─ NCollapse (basic/dist/quality/sample)
    │   ├─ StorageFooter → 版本数 + "清理旧数据"按钮 + columnRules 推荐标签
    │   └─ uses: insightStore.qualityScore (自动加载)
    │
    ├─ NTabPane name="multi" tab="多列分析"
    │   └─ MultiColumnView.vue → 使用 insightStore (不再直接调用 API)
    │       ├─ NSelect multiple → 列选择
    │       ├─ NSelect → 规则选择 (insightStore.multiColumnRules)
    │       ├─ NButton "执行分析" → insightStore.executeMultiRule()
    │       └─ NDataTable / KV 对 → 结果渲染 (insightStore.multiResult)
    │
    └─ NTabPane name="history" tab="历史"
        ├─ [Empty/List] 两状态
        ├─ 版本列表 → entry.version_id + checksum + 日期
        ├─ onClick → insightStore.loadVersionDetail(version_id) → 加载 diffData
        ├─ DiffPanel → 旧值 → 新值（绿增/红减/灰不变）
        └─ 取消对比 → clearDiff()

TableProfileView.vue (dockview 'tableProfile')
├─ props: { connId, dbType, database, schema, table }
├─ [Loading / Error / Data] 三状态
├─ 导航树右键 → CustomEvent('open-table-profile') → DockviewLayout 动态创建
├─ NDataTable → 列元数据表 (列名/类型/可空/PK)
├─ NTag → dbType + RowCount
└─ 去重: 同表点击聚焦已有面板
```

### MultiColumnView 渲染

| 规则 result_type | 渲染                  |
| :--------------: | --------------------- |
|     `single`     | KV 行 (`密钥: 值` 对) |
|      `list`      | `NDataTable` 虚拟表格 |

---

## 四、用户自定义规则

```
{project}/.RSMETA/insight-rules/my-rule.rule.toml
```

加载时机：`open_project_by_path` → `insight::load_user_rules(&path_buf)`

冲突策略：同名 `rule_id` — 后加载的（用户）覆盖先加载的（内置）

---

## 五、文件清单

### 新增 (25)

```
src-tauri/insight-rules/column/ (9)
src-tauri/insight-rules/multi/ (4)
src-tauri/insight-rules/table/ (4) ← Phase 2 + 表质量
src-tauri/insight-rules/quality/ (1) ← 质量评分
src-tauri/src/core/duckdb.rs ← DuckDB 全局单例
src-tauri/src/core/insight/mod.rs
src-tauri/src/core/insight/rule_registry.rs
src-tauri/src/core/insight/rule_executor.rs
src-tauri/src/core/insight/rule_types.rs
src-tauri/src/core/insight/schema_analyzer.rs ← Schema 洞察
src/.../components/panels/MultiColumnView.vue
src/.../components/panels/TableProfileView.vue
src/.../components/panels/SchemaInsightPanel.vue ← Schema 洞察
```

### 修改 (18 → 多轮迭代)

```
src-tauri/Cargo.toml (添加 toml)
src-tauri/src/core/mod.rs (添加 insight + duckdb + re-export)
src-tauri/src/core/services/result_service.rs (compute_* + 规则API + TableProfile + profile_column_from_table + DuckDBManager委托)
src-tauri/src/commands/result_commands.rs (5个 insight 命令 + TableProfile + 版本详情 + 表列探查)
src-tauri/src/commands/project_commands.rs (load_user_rules)
src-tauri/src/lib.rs (注册 insight 命令)
src-tauri/src/core/persistence/mod.rs (ResourceVersion 导出修复)
src-tauri/src/core/dbi/engine/duckdb_engine.rs (DuckDB 实例统一: tokio Mutex → std Mutex)
src/.../components/panels/ColumnInsightPanel.vue (NTabs + MultiColumnView + P0打通 + 骨架屏 + 历史/对比/导出 + 过渡动画 + table-column-click)
src/.../components/panels/QueryResultPanel.vue (allColumns in event)
src/.../services/result-analysis.ts (RuleMeta + TableProfile + API 全部)
src/.../stores/insight-store.ts (多列 state + P0打通 + 版本对比 + 表列探查)
src/.../components/panels/MultiColumnView.vue (P0打通: 改用 Store)
src/.../components/DockviewLayout.vue (注册 tableProfile + 动态创建面板)
src/.../composables/use-context-menu-actions.ts (快速探查菜单 + getDbTypeForConnection)
```
