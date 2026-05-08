# 结果集模块优化 — 开发进度文档

> 版本：v2.1
> 最后更新：2026-05-08
> 作者：RdataStation 团队
> 状态：✅ 全部完成 (49/49 核心完成) + 洞察子系统测试修复

---

## 七、DuckDB 离线扩展体系

### 架构原则

> **零 Rust feature，全 SQL LOAD**  
> Cargo.toml 只保留 `features = ["bundled"]`，所有功能扩展走 `.duckdb_extension` 文件 + SQL `LOAD`。

### 扩展清单

| 扩展             | 文件名                              | 优先级  | 说明              |
| ---------------- | ----------------------------------- | ------- | ----------------- |
| parquet          | `parquet.duckdb_extension`          | P0 启动 | Parquet 读写      |
| excel            | `excel.duckdb_extension`            | P0 启动 | Excel 读写 (xlsx) |
| json             | `json.duckdb_extension`             | P0 启动 | JSON 读写         |
| httpfs           | `httpfs.duckdb_extension`           | P1 按需 | HTTP 文件系统     |
| fts              | `fts.duckdb_extension`              | P1 按需 | 全文搜索          |
| mysql_scanner    | `mysql_scanner.duckdb_extension`    | P0 启动 | MySQL 外部表      |
| postgres_scanner | `postgres_scanner.duckdb_extension` | P0 启动 | PostgreSQL 外部表 |
| sqlite_scanner   | `sqlite_scanner.duckdb_extension`   | P0 启动 | SQLite 外部表     |

### 加载流程

```
app 启动
  ↓
DuckDBEngine::init_extensions(conn, data_dir)
  ↓
SET extension_directory = '{data_dir}/duckdb/extensions'
  ↓
P0 扩展: LOAD '{name}.duckdb_extension'  ← 遍历清单 P0 条目
  ↓
tracing::info!("已加载: parquet, excel, json, mysql_scanner, ...")

--- 运行时 ---

P1 扩展: load_extension_by_name(conn, "httpfs")  ← 用户触发功能时调用
  ↓
LOAD 'httpfs.duckdb_extension'
  ↓
tracing::info!("按需加载: httpfs")
```

### 扩展文件管理

- **存放路径**: `{app_data_dir}/duckdb/extensions/`
- **官方源**: `https://extensions.duckdb.org/v1.5.2/{platform}/{name}.duckdb_extension`
- **打包**: 随安装包分发，无需运行时下载
- **升级**: 替换 `.duckdb_extension` 文件即生效，Rust 代码无需改动

## 一、阶段总览

| 阶段                       | 分组           | 状态      | 完成度    |
| -------------------------- | -------------- | --------- | --------- |
| Phase 1: 状态管理统一      | A 组 (4项)     | ✅ 已完成 | 100%      |
| Phase 2: 前端组件拆分      | B 组 (7项)     | ✅ 已完成 | 100%      |
| Phase 3: 后端 Service 拆分 | C 组 (7项)     | ✅ 已完成 | 100%      |
| Phase 4: DuckDB 引擎优化   | D 组 (3项)     | ✅ 已完成 | 100%      |
| Phase 5: 性能优化          | E 组 (4项)     | ✅ 已完成 | 100%      |
| Phase 6: 类型安全治理      | F 组 (3项)     | ✅ 已完成 | 100%      |
| Phase 7: 规范合规清理      | G 组 (6项)     | ✅ 已完成 | 100%      |
| Phase 8: 缺失功能补齐      | H 组 (5项)     | ✅ 已完成 | 100%     |
| Phase 9: 导出 + 集成打通    | I 组 (5项)     | ✅ 已完成 | 100%      |
| Phase 10: QueryResultPanel 重构 | J 组 (5项)     | ✅ 已完成 | 100%      |
| **合计**                   | **10 组 49 项** |           | **100%**  |

---

## 二、Phase 8: H 组详细进度

| 编号 | 任务                | 状态 | 说明                                                                                                                                                                                                                                                                                                                                 |
| ---- | ------------------- | ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| H1   | 单元格编辑持久化    | ✅   | Rust `save_cell_update` Tauri 命令 + 前端 API + store action (UPDATE 写回)                                                                                                                                                                                                                                                           |
| H2   | 导出格式扩展        | ✅   | SQL Dump (INSERT INTO) 导出已实现；Excel/Parquet 后续安装依赖包                                                                                                                                                                                                                                                                      |
| H3   | 列配置持久化        | ✅   | `useGridConfig.saveColumnState/restoreColumnState/clearColumnState` — localStorage                                                                                                                                                                                                                                                   |
| H4   | 过滤预设 (Preset)   | ✅   | `useFilterPresets` composable + [FilterPresetSelector.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/result-panel/FilterPresetSelector.vue) UI 组件                                                                                                               |
| H5   | 多结果集对比 (Diff) | ✅   | [useResultDiff.ts](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/composables/useResultDiff.ts) 引擎 + [ResultDiffViewer.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/result-panel/ResultDiffViewer.vue) 可视化组件 |

---

## 三、H1 单元格编辑实现细节

### 数据流

```
用户编辑单元格 (AG Grid)
  ↓ cellValueChanged 事件
resultStore.saveCellUpdate(tabId, col, newVal, rowIdx)
  ↓ 构建 rowIdentity (所有未修改列的 {col: val} 映射)
  ↓ 调用 Tauri invoke('save_cell_update', {...})
Rust: result_commands::save_cell_update
  ↓ value_to_sql() 格式化值
  ↓ SqlService.execute(UPDATE `table` SET `col`=new WHERE cond1 AND cond2..)
  ↓ 返回 { success, affected_rows, message }
前端: 成功 → 本地 objectRows 即时更新 → dirtyRows 标记
      失败 → 单元格回滚（值恢复）
```

### 涉及文件

| 层    | 文件                          | 方式                                                            |
| ----- | ----------------------------- | --------------------------------------------------------------- |
| 类型  | `types/result.ts`             | 新增 `CellUpdateInput/Result`、`tableName` 字段                 |
| Rust  | `commands/result_commands.rs` | 新增 `save_cell_update` Tauri 命令 + `value_to_sql` 辅助        |
| 注册  | `lib.rs`                      | `.invoke_handler` 添加 `save_cell_update`                       |
| API   | `services/result-analysis.ts` | 新增 `saveCellUpdate()` 函数                                    |
| Store | `stores/result-store.ts`      | 新增 `tableName`/`extractTableName()`/`saveCellUpdate()` action |

### 表名提取策略

```typescript
function extractTableName(sql: string, fallbackColumn: string): string {
  // 1. 匹配 FROM table 模式
  fromMatch = sql.match(/FROM `(\w+)` /i)
  // 2. 回退：匹配 JOIN table 模式
  joinMatch = sql.match(/JOIN `(\w+)` /i)
  // 3. 最终回退：_result_<column>
}
```

---

## 四、H4 过滤预设 API

```typescript
const { presets, addPreset, removePreset, updatePreset, getPreset, getPresetsByMode } =
  useFilterPresets()

// 新建
addPreset('活跃用户', 'quick', "status = 'active'")
// 按模式筛选
const sqlPresets = getPresetsByMode('sql')
// 删除
removePreset(presetId)
```

---

## 五、H5 多结果集对比引擎实现细节

### 数据流

```
用户选择 Tab A + Tab B + 键列
  ↓ useResultDiff(tabA, tabB, keyColumns)
列差异: Set 运算 → ColumnDiff[] (inBoth / onlyInA / onlyInB)
行差异: 键列 Hash → Map<key, Row> → RowDiff[] (unchanged/added/removed/modified)
  ↓ ComputedRef<DiffResult | null>
ResultDiffViewer.vue 渲染
  ├── 摘要区: NTag 统计（列数/共同/独有 + 行变更数）
  ├── 列差异区: 列名 + 归属标签
  └── 行差异区: 表格（状态/Key/A侧/B侧）+ 颜色编码
```

### 涉及文件

| 文件                                                                                                                                                                   | 说明            |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------- |
| [useResultDiff.ts](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/composables/useResultDiff.ts)                            | Diff 算法引擎   |
| [ResultDiffViewer.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/result-panel/ResultDiffViewer.vue) | Diff 可视化组件 |

### 颜色编码

| 行状态    | 背景色              | 含义           |
| --------- | ------------------- | -------------- |
| unchanged | 透明                | 行完全相同     |
| added     | rgba(16,185,129,8%) | 仅存在于 B 侧  |
| removed   | rgba(239,68,68,8%)  | 仅存在于 A 侧  |
| modified  | rgba(245,158,11,8%) | 键相同但值不同 |

---

## 六、DuckDB 连接池可配置化

### 新增 Tauri 命令

| 命令                   | 说明                                  |
| ---------------------- | ------------------------------------- |
| `get_duckdb_pool_info` | 获取当前池大小 / 偏好大小 / 限制范围  |
| `set_duckdb_pool_size` | 设置偏好大小（范围 1-32），可选重启池 |
| `restart_duckdb_pool`  | 以偏好大小重建连接池（清空临时表）    |

### 涉及文件

| 层   | 文件                                                                                                                   | 变更                                                                         |
| ---- | ---------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Rust | [duckdb.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/duckdb.rs)                       | OnceLock → RwLock + preferred_pool_size + set/restart 方法                   |
| 命令 | [result_commands.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/commands/result_commands.rs) | 3 个新 Tauri 命令                                                            |
| 注册 | [lib.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/lib.rs)                                  | 注册 `get_duckdb_pool_info` / `set_duckdb_pool_size` / `restart_duckdb_pool` |

---

## 七、新增文件统计（本轮 v1.5）

| 目录                                         | 本轮新增 | 累计新增 | 本轮修改 | 累计修改 |
| -------------------------------------------- | -------- | -------- | -------- | -------- |
| `ui/composables/useResultDiff.ts`            | 0        | 1        | 0        | 0        |
| `ui/components/.../ResultDiffViewer.vue`     | 1        | 5        | 0        | 0        |
| `ui/components/.../FilterPresetSelector.vue` | 1        | 6        | 0        | 0        |
| `src-tauri/src/core/duckdb.rs`               | 0        | 0        | 1        | 2        |
| `src-tauri/src/commands/result_commands.rs`  | 0        | 0        | 1        | 2        |
| `src-tauri/src/lib.rs`                       | 0        | 0        | 1        | 2        |
| **总计**                                     | **2**    | **20**   | **3**    | **18**   |

---

## 九、Phase 9: I 组（导出 + 集成打通）

| 编号 | 任务                              | 状态 | 涉及文件                                           |
| ---- | --------------------------------- | ---- | -------------------------------------------------- |
| I1   | Save 按钮: dirtyCells → saveCellUpdate 写回 DB | ✅   | [QueryResultPanel.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue) — handleSave/handleCancel/buildRowIdentity |
| I2   | FilterPresetSelector 集成到工具栏 | ✅   | [QueryResultPanel.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue) — getCurrentExpression/applyPreset/saveFilterPreset |
| I3   | DuckDB COPY TO: Parquet/Excel 导出 | ✅   | [duckdb_service.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/services/duckdb_service.rs) — ExportFormat 枚举 + export_temp_table / [result_commands.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/commands/result_commands.rs) — export_result_to_file 命令 + 前端 exportMenuOptions 扩展 |
| I4   | ResultDiffViewer 集成 (NModal)    | ✅   | [QueryResultPanel.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue) — GitCompare 按钮 + NModal 对话框 |
| I5   | Cancel 脏行回滚 + objectRows 即时更新 | ✅   | [QueryResultPanel.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue) — handleSave 成功后写 objectRows / handleCancel 回滚 oldValue |

---

## 十、Phase 10: J 组（QueryResultPanel 重构）

| 编号 | 任务                                | 状态 | 涉及文件 |
| ---- | ----------------------------------- | ---- | -------- |
| J1   | ResultGridView 增强：全 props/events + AgGridVue 封装 | ✅ | [ResultGridView.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/result-panel/ResultGridView.vue) — 完整 grid wrapper |
| J2   | 替换内联 `<AgGridVue>` (75行) → `<ResultGridView>` | ✅ | [QueryResultPanel.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue) — 约 -80 行模板 |
| J3   | 替换内联 Text View → `<ResultTextView>` | ✅ | [QueryResultPanel.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue) — 约 -25 行模板 + -9 行 computed |
| J4   | 替换内联 Record View → `<ResultRecordView>` + 导航栏 | ✅ | [QueryResultPanel.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue) — 约 -32 行模板 + -30 行 CSS + prev/next 导航按钮 |
| J5   | 清理冗余：移除 AgGridVue import、gridThemeClass、textViewContent、formatCellValue、未用 CSS | ✅ | [QueryResultPanel.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue) — 约 -45 行 script + -55 行 CSS |

### 重构效果

| 指标             | 重构前  | 重构后  | 变化     |
| ---------------- | ------- | ------- | -------- |
| 模板行数         | ~270 行 | ~270 行 | 持平     |
| Script 行数      | ~955 行 | ~898 行 | **-57 行** |
| CSS 行数         | ~400 行 | ~340 行 | **-60 行** |
| 内联 AG Grid     | 75 行   | 0 行    | ✅ 消除  |
| 内联 Text View   | 25 行   | 0 行    | ✅ 消除  |
| 内联 Record View | 32 行   | 0 行    | ✅ 消除  |
| 冗余 computed    | 2 个    | 0 个    | ✅ 消除  |
| 冗余 import      | 1 个    | 0 个    | ✅ 消除  |
| 冗余 CSS         | 5 块    | 0 块    | ✅ 消除  |

### 涉及文件

| 文件 | 说明 |
| ---- | ---- |
| [ResultGridView.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/result-panel/ResultGridView.vue) | 完整 AG Grid 封装组件 |
| [QueryResultPanel.vue](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src/extensions/builtin/workbench/ui/components/panels/QueryResultPanel.vue) | 主面板 — 子组件替换 + 清理 |

### 副产物：Rust state.rs 编译修复

| 文件 | 说明 |
| ---- | ---- |
| [state.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/adapters/tauri/state.rs) | `WarmingTask.progress` 改为 `Mutex<WarmingProgressState>` 解决临时值 + CloneToUninit 编译错误 |
| [cache_warming_commands.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/commands/cache_warming_commands.rs) | `get_warming_progress` 适配 `Mutex` 读取 |

---

## 十一、统计看板

### 完成度

| 状态      | 数量 | 占比   |
| --------- | ---- | ------ |
| ✅ 已完成 | 49   | 100%   |
| ⏳ 待开始 | 0    | 0%     |

### 待办清单

🎉 全部完成，无待办项。

---

## 十二、Phase 11: 洞察子系统测试修复

| 编号 | 任务 | 状态 | 涉及文件 |
| ---- | ---- | ---- | -------- |
| K1   | `test_execute_single` 修复：移除 `result.quality`/`result.data` → `result["value"]` | ✅ | [rule_executor.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/insight/rule_executor.rs#L509) |
| K2   | `test_execute_list` 修复：移除 `result.quality`/`result.data` → `result.as_array()` | ✅ | [rule_executor.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/insight/rule_executor.rs#L547) |
| K3   | `connection_commands.rs` 未使用变量 `connection_info` → `_connection_info` | ✅ | [connection_commands.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/commands/connection_commands.rs#L581) |

### 问题根因

`RuleExecutor::execute()` 返回 `Result<serde_json::Value, CoreError>`，但 `test_execute_single` 和 `test_execute_list` 错误地访问了 `ExecutionResult` 的字段（`.quality`、`.data`）。这两个字段仅存在于 `execute_qualified()` 的返回值 `ExecutionResult` 上。

### 修复后状态

| 检查项 | 结果 |
| ------ | ---- |
| `cargo check` | ✅ exit 0（4 个预存 warning） |
| `test_execute_single` | ✅ `result["value"]` 直接索引 |
| `test_execute_list` | ✅ `result.as_array()` 直接转换 |
| `test_execute_qualified` (3 个) | ✅ 未受影响，已正确使用 `execute_qualified()` |

---

## 版本历史

| 版本 | 日期       | 说明                                                                                                                                                                              |
| ---- | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| v2.1 | 2026-05-08 | 洞察子系统测试修复 — rule_executor 2 测试 `.quality`/`.data` → 正确 Value 访问 + connection_commands unused variable |
| v2.0 | 2026-05-08 | 🎉 **全部完成** — Phase 10: QueryResultPanel 重构（子组件替换 + 冗余清理 -117行）+ Rust state.rs warming 编译修复；49/49 = 100% |
| v1.11 | 2026-05-08 | 洞察子系统 DuckDB TTL — `cleanup_expired_tables()` 30分钟过期清理 + `register_temp_table()` 自动触发 + 2 TTL 测试；`TEMP_TABLE_PREFIX` 死代码消除 + `unused_mut` 修复 |
| v1.10 | 2026-05-08 | 洞察子系统规则内务优化 — rule_executor panic!消除+代码去重+10单测+占位符安全, rule_registry tracing日志, schema_analyzer Arrow解析器去重, 前端P0/P1修复（any/!非空断言/空catch/i18n/isOpen） |
| v1.9 | 2026-05-08 | I 组：Save → saveCellUpdate 真写DB + Cancel 脏行回滚 + FilterPresetSelector 集成 + DuckDB Parquet/Excel 导出 (COPY TO) + ResultDiffViewer NModal 集成 + exportMenuOptions 扩展 i18n |
| v1.8 | 2026-05-08 | DuckDB 扩展架构重构：离线 .duckdb_extension + SQL LOAD 替代 Cargo feature flags；P0/P1 分级加载；Cargo.toml 去 parquet/excel/json features；arrow-array/arrow-buffer 噪音依赖删除 |
| v1.7 | 2026-05-08 | 洞察子系统深度代码质量优化 |
| v1.5 | 2026-05-08 | H5(多结果集对比引擎+组件) + G4(Rust unwrap清理完成) + 连接池可配置化 + 过滤预设UI面板 + duckdb 1.1.1→1.10502.0（含 parquet），为 Parquet/Excel/CSV 原生导出铺路；进度 92.3%       |
| v1.4 | 2026-05-08 | H组实施 — H1(单元格编辑持久化) + H2(SQL Dump) + H4(过滤预设管理器)；进度 82.1%                                                                                                    |
| v1.3 | 2026-05-08 | Phase 5-6 完成 — E 组(性能优化) + F 组(类型安全) + G 组(规范清理) + H3(列状态持久化)                                                                                              |
| v1.2 | 2026-05-08 | Phase 3-4 完成 — C 组(Rust Service 拆分 7 文件) + D 组(DuckDB 连接池/沙箱/LRU)                                                                                                    |
| v1.1 | 2026-05-08 | Phase 1 完成 — A 组(状态管理统一)                                                                                                                                                 |
| v1.0 | 2026-05-08 | 初始版本，39 项任务待开始                                                                                                                                                         |
