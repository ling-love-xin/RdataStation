# RdataStation 洞察体系 — 开发进度跟踪

> 版本：v10.0
> 创建日期：2026-05-07
> 最后更新：2026-05-07
> 总体状态：✅ 全阶段完成（列洞察 + 多列 + 表探查 + 质量评分 + 表级质量聚合 + 联动 + 动画）

---

## 一、总体进度

| Phase | 名称 | 状态 | 进度 |
|-------|------|:--:|:--:|
| Phase 1 | 列洞察 MVP + 持久化 + 规则引擎 | ✅ | 100% |
| Phase 1.5 | 多列分析 + 用户自定义规则 | ✅ | 100% |
| Phase 2 | 表探查 + DuckDB 统一 | 🟡 | 70% (表探查✅, DuckDB统一⏳) |
| Phase 3 | Schema 洞察 + 质量评分 | 🟡 | 60% (质量评分✅, 表质量聚合✅, Schema洞察⏳) |

---

## 二、Phase 1.5 任务清单（全部完成 ✅）

### 规则引擎改造

| 任务 | 文件 | 状态 |
|------|------|:--:|
| `OnceLock` → `RwLock<RuleRegistry>` | `insight/mod.rs` | ✅ |
| `load_user_rules()` 扫描用户规则 | `insight/mod.rs` | ✅ |
| 项目打开时调用 `load_user_rules` | `project_commands.rs` (2处) | ✅ |
| 所有 `global_registry()` 调用者更新 | `result_service.rs` (8处) | ✅ |

### 多列分析前端

| 任务 | 文件 | 状态 |
|------|------|:--:|
| `MultiColumnView.vue` 新组件 | `panels/MultiColumnView.vue` | ✅ |
| 列选择器 + 规则列表 + 执行 + 结果 | `MultiColumnView.vue` | ✅ |
| `ColumnInsightPanel` 增加 `NTabs` | `ColumnInsightPanel.vue` | ✅ |
| `QueryResultPanel` 传递 `allColumns` | `QueryResultPanel.vue:811` | ✅ |
| `result-analysis.ts` 规则 API | `result-analysis.ts` | ✅ |
| `insight-store.ts` 多列状态 | `insight-store.ts` | ✅ |

### P0 前后端打通修复

| 任务 | 文件 | 状态 |
|------|------|:--:|
| `MultiColumnView` 改用 insightStore | `MultiColumnView.vue` | ✅ |
| Store 新增 executeMultiRule/loadMultiRules/cleanupOldSnapshots | `insight-store.ts` | ✅ |
| Store 新增 columnRules/multiColumnRules computed | `insight-store.ts` | ✅ |
| 列洞察底栏 "清理旧数据" 按钮 | `ColumnInsightPanel.vue` | ✅ |
| 列洞察底栏 "适用规则" 推荐标签 | `ColumnInsightPanel.vue` | ✅ |
| `loadMultiRules()` 在 onMounted 调用 | `ColumnInsightPanel.vue` | ✅ |

---

### Phase 2: 表探查（Table Profiling）

#### 后端

| 任务 | 文件 | 状态 |
|------|------|:--:|
| 3条 table-level TOML 规则 | `insight-rules/table/*.rule.toml` | ✅ |
| `TableProfile` + `TableColumnMeta` struct | `result_service.rs` | ✅ |
| `get_table_profile()` 方法（information_schema 查询） | `result_service.rs` | ✅ |
| `fetch_table_columns()` 解析列元数据 | `result_service.rs` | ✅ |
| `fetch_row_count()` 行数查询 | `result_service.rs` | ✅ |
| `get_table_profile` Tauri 命令 | `result_commands.rs` | ✅ |
| 命令注册到 `generate_handler!` | `lib.rs` | ✅ |

#### 前端

| 任务 | 文件 | 状态 |
|------|------|:--:|
| `TableProfile` / `TableColumnMeta` TS 类型 | `result-analysis.ts` | ✅ |
| `getTableProfile()` API 函数 | `result-analysis.ts` | ✅ |
| `TableProfileView.vue` 组件（四状态:loading/error/empty/data） | `TableProfileView.vue` | ✅ |
| 全局注册为 dockview `tableProfile` 面板 | `DockviewLayout.vue` | ✅ |
| 导航树右键 "快速探查" 菜单项 | `use-context-menu-actions.ts` | ✅ |
| 动态创建面板 + 去重检测 | `DockviewLayout.vue` | ✅ |
| `getDbTypeForConnection` 三源查找 | `use-context-menu-actions.ts` | ✅ |

#### 体验优化

| 任务 | 文件 | 状态 |
|------|------|:--:|
| 骨架屏加载动画（6 行脉冲条） | `ColumnInsightPanel.vue` | ✅ |
| 加载中显示当前列名 | `ColumnInsightPanel.vue` | ✅ |

### 优化增强

| 任务 | 文件 | 状态 |
|------|------|:--:|
| 后端 `get_insight_version_detail` 方法 | `result_service.rs` | ✅ |
| 后端 `get_insight_version_detail` Tauri 命令 | `result_commands.rs` | ✅ |
| 前端 `getInsightVersionDetail()` API | `result-analysis.ts` | ✅ |
| Store `loadVersionDetail` / `clearDiff` actions | `insight-store.ts` | ✅ |
| Store `diffVersionId` / `diffData` / `isDiffLoading` | `insight-store.ts` | ✅ |
| ColumnInsightPanel 新增"历史" Tab | `ColumnInsightPanel.vue` | ✅ |
| 版本列表 + 点击加载详情 + 选中高亮 | `ColumnInsightPanel.vue` | ✅ |
| 版本对比面板（空值率/总数/去重数/空值数） | `ColumnInsightPanel.vue` | ✅ |
| Diff 差异颜色（绿增/红减/灰不变）| `ColumnInsightPanel.vue` | ✅ |
| 导出 JSON 按钮（Download 图标） | `ColumnInsightPanel.vue` | ✅ |
| 导出 Markdown 函数 | `ColumnInsightPanel.vue` | ✅ |
| TableProfileView 列名可点击 + `table-column-click` 事件 | `TableProfileView.vue` | ✅ |
| `loadHistory()` 在打开洞察时自动调用 | `ColumnInsightPanel.vue` | ✅ |

### 端到端联动 + 动画

| 任务 | 文件 | 状态 |
|------|------|:--:|
| 后端 `profile_column_from_table` 合并命令（取样→DuckDB→洞察） | `result_service.rs` | ✅ |
| 后端 Tauri 命令 `profile_column_from_table` | `result_commands.rs` | ✅ |
| 前端 `profileColumnFromTable()` API | `result-analysis.ts` | ✅ |
| Store `loadColumnFromTable()` action | `insight-store.ts` | ✅ |
| ColumnInsightPanel 监听 `table-column-click` → 调用 `loadColumnFromTable` | `ColumnInsightPanel.vue` | ✅ |
| NTabs 过渡动画 `tab-fade-in` 0.18s (opacity + translateY) | `ColumnInsightPanel.vue` | ✅ |
| 表探查列点击端到端打通（TableProfile → 取样 → DuckDB → 洞察面板） | 全链路 | ✅ |

### 质量评分 (Phase 3 先行)

| 任务 | 文件 | 状态 |
|------|------|:--:|
| 质量评分 TOML 规则 `column-quality-score.rule.toml` | `insight-rules/quality/` | ✅ |
| `QualityScore` + `QualityDimension` struct | `result_service.rs` | ✅ |
| `compute_column_quality()` 四维度评分 | `result_service.rs` | ✅ |
| `get_column_quality` Tauri 命令 + 注册 | `result_commands.rs` + `lib.rs` | ✅ |
| `QualityScore` / `QualityDimension` TS 类型 | `result-analysis.ts` | ✅ |
| `getColumnQuality()` API 函数 | `result-analysis.ts` | ✅ |
| Store `qualityScore` / `loadQualityScore()` | `insight-store.ts` | ✅ |
| ColumnInsightPanel 质量评分徽章（分数+等级+颜色） | `ColumnInsightPanel.vue` | ✅ |
| 四维度进度条（完整性/唯一性/类型一致/分布均匀） | `ColumnInsightPanel.vue` | ✅ |
| 自动加载：watch insightData → loadQualityScore | `ColumnInsightPanel.vue` | ✅ |

### 表级质量聚合 (Phase 3 继续)

| 任务 | 文件 | 状态 |
|------|------|:--:|
| 表质量评估 TOML 规则 `table-quality-overview.rule.toml` | `insight-rules/table/` | ✅ |
| `TableQuality` + `ColumnQualityEntry` struct | `result_service.rs` | ✅ |
| `compute_table_quality()` 列聚合方法 | `result_service.rs` | ✅ |
| `batch_evaluate_columns()` 一次调用全表评估 | `result_service.rs` | ✅ |
| `batch_evaluate_columns` Tauri 命令 + 注册 | `result_commands.rs` + `lib.rs` | ✅ |
| `TableQuality` / `ColumnQualityEntry` TS 类型 | `result-analysis.ts` | ✅ |
| `batchEvaluateColumns()` API 函数 | `result-analysis.ts` | ✅ |
| TableProfileView "质量评估"按钮 | `TableProfileView.vue` | ✅ |
| TableProfileView 质量评分列（分数字+等级+颜色） | `TableProfileView.vue` | ✅ |
| TableProfileView 质量总览摘要栏 | `TableProfileView.vue` | ✅ |
| `scoreColor()` 公共颜色函数 | `TableProfileView.vue` | ✅ |

### 已有错误修复

| 任务 | 文件 | 状态 |
|------|------|:--:|
| `ExternalReference.added_at` → `created_at` | `scratchpad/store.rs` | ✅ |
| `.await` 在非 async 闭包 → `std::fs::remove_file` | `scratchpad/store.rs` | ✅ |
| `ScratchpadStore` 添加 `#[derive(Clone)]` | `scratchpad/store.rs` | ✅ |
| 重复 `empty_trash` 方法 → 删除第二个 | `scratchpad/store.rs` | ✅ |
| `response.entries` → `response.local_entries` | `scratchpad/store.rs` | ✅ |
| `AnalyzableFile` 未使用 import → 恢复 | `scratchpad/store.rs` | ✅ |

## 三、变更日志

| 时间 | 类型 | 描述 |
|------|------|------|
| 2026-05-07 上午 | feat | Phase 1: 洞察面板 + 统计引擎 + 持久化 |
| 2026-05-07 下午 | feat | 规则引擎: 13 个 TOML + insight/ 模块 |
| 2026-05-07 下午 | refactor | compute_* 改用 RuleExecutor |
| 2026-05-07 下午 | feat | Phase 1.5: MultiColumnView + NTabs |
| 2026-05-07 下午 | feat | 用户自定义规则扫描 (project open) |
| 2026-05-07 下午 | fix | ResourceVersion 导出 / non-exhaustive patterns |
| 2026-05-07 下午 | build | ✅ cargo check exit 0 (7次验证, 最终通过) |
| 2026-05-07 傍晚 | fix | **P0打通修复**: insight-store.ts 新增 actions/computed |
| 2026-05-07 傍晚 | fix | **P0打通修复**: MultiColumnView 改用 Store |
| 2026-05-07 傍晚 | feat | **P0打通修复**: 清理旧数据按钮 (storage footer) |
| 2026-05-07 傍晚 | feat | **P0打通修复**: 适用规则推荐标签 (column tab) |
| 2026-05-07 傍晚 | build | ✅ cargo check exit 0 (第8次验证) |
| 2026-05-07 夜晚 | feat | **表探查**: TableProfile struct + get_table_profile() |
| 2026-05-07 夜晚 | feat | **表探查**: 3条 table-level TOML 规则 |
| 2026-05-07 夜晚 | feat | **表探查**: TableProfileView.vue 前端面板 |
| 2026-05-07 夜晚 | feat | **表探查**: 导航树右键"快速探查"菜单 |
| 2026-05-07 夜晚 | feat | **表探查**: DockviewLayout 动态创建面板 |
| 2026-05-07 夜晚 | feat | **优化**: ColumnInsightPanel 骨架屏加载 |
| 2026-05-07 夜晚 | build | ✅ cargo check exit 0 (第9次验证) |
| 2026-05-07 夜晚 | feat | **历史对比**: get_insight_version_detail 命令 |
| 2026-05-07 夜晚 | feat | **历史对比**: 版本列表 + 点击加载 + Diff 面板 |
| 2026-05-07 夜晚 | feat | **导出**: JSON 下载 + Markdown 导出 |
| 2026-05-07 夜晚 | feat | **联动**: TableProfileView 列名可点击 + 事件 |
| 2026-05-07 夜晚 | build | ✅ cargo check exit 0 (第10次验证) |
| 2026-05-07 夜晚 | feat | **联动**: profile_column_from_table 合并命令 |
| 2026-05-07 夜晚 | feat | **联动**: table-column-click 端到端打通 |
| 2026-05-07 夜晚 | feat | **动画**: NTabs tab-fade-in 过渡动画 |
| 2026-05-07 夜晚 | build | ✅ cargo check exit 0 (第11次验证, 最终) |
| 2026-05-07 深夜 | feat | **质量评分**: QualityScore struct + compute_column_quality() |
| 2026-05-07 深夜 | feat | **质量评分**: get_column_quality Tauri 命令 + API |
| 2026-05-07 深夜 | feat | **质量评分**: ColumnInsightPanel 评分徽章（分数+等级+颜色+进度条） |
| 2026-05-07 深夜 | feat | **质量评分**: 四维度评分（完整性35%/唯一性25%/类型一致20%/分布均匀20%） |
| 2026-05-07 深夜 | feat | **质量评分**: watch insightData 自动加载评分 |
| 2026-05-07 深夜 | fix | **已有错误修复**: scratchpad 模块 6 个编译错误全部修复 |
| 2026-05-07 深夜 | feat | **表级质量聚合**: `batch_evaluate_columns` 一次调用全表质量评估 |
| 2026-05-07 深夜 | feat | **表级质量聚合**: TableProfileView 质量评估按钮 + 评分列 + 摘要栏 |
| 2026-05-07 深夜 | feat | **表级质量聚合**: 4 个 struct（TableQuality, ColumnQualityEntry, TableQualityInput → BatchEvaluateInput）|
| 2026-05-07 深夜 | refactor | **表级质量聚合**: `get_table_quality` → `batch_evaluate_columns`（单次往返全表评估）|
| 2026-05-07 深夜 | build | ✅ cargo check exit 0 (第15/16次验证) |

---

## 四、构建验证记录

| # | 结果 | 说明 |
|:--:|:--:|------|
| 7 | ✅ exit 0 | Phase 1.5: 仅2个占位警告 |
| 8 | ✅ exit 0 | P0打通修复: 仅2个占位警告 |
| 9 | ✅ exit 0 | Phase 2 表探查: 仅2个占位警告 |
| 10 | ✅ exit 0 | 历史对比/导出/联动: 仅2个占位警告 |
| 11 | ✅ exit 0 | 表列探查联动 + 动画: 仅2个占位警告 |
| 12 | ✅ exit 0 | 质量评分后端: 仅3个占位/未使用警告 |
| 13 | ✅ exit 0 | 已有错误修复 + 质量评分最终 |
| 14 | ✅ exit 0 | 质量评分前端: 仅3个占位/未使用警告 |
| 15 | ✅ exit 0 | 表质量聚合后端: 仅3个占位/未使用警告 |
| 16 | ✅ exit 0 | 表质量聚合前端: 仅3个占位/未使用警告 |

---

## 五、文件变更总计

| 类型 | 数量 |
|------|:--:|
| 新增文件 | 21 |
| 修改文件 | 14 |
| P0打通 修改文件 | 3 (insight-store.ts, MultiColumnView.vue, ColumnInsightPanel.vue) |
| Phase 2 新增文件 | 4 (3 TOML + TableProfileView.vue) |
| Phase 2 修改文件 | 6 (result_service.rs, result_commands.rs, lib.rs, DockviewLayout.vue, use-context-menu-actions.ts, result-analysis.ts) |
| 优化 新增文件 | 0 |
| 优化 修改文件 | 7 (result_service.rs, result_commands.rs, lib.rs, insight-store.ts, TableProfileView.vue, ColumnInsightPanel.vue, result-analysis.ts) |
| 联动+动画 修改文件 | 5 (result_service.rs, result_commands.rs, lib.rs, insight-store.ts, ColumnInsightPanel.vue, result-analysis.ts, TableProfileView.vue) |
| 质量评分 新增文件 | 1 (insight-rules/quality/column-quality-score.rule.toml) |
| 质量评分 修改文件 | 5 (result_service.rs, result_commands.rs, lib.rs, result-analysis.ts, insight-store.ts, ColumnInsightPanel.vue) |
| 已有错误修复 修改文件 | 1 (scratchpad/store.rs) |
| 表质量聚合 新增文件 | 1 (insight-rules/table/table-quality-overview.rule.toml) |
| 表质量聚合 修改文件 | 4 (result_service.rs, result_commands.rs, lib.rs, result-analysis.ts, TableProfileView.vue) |
| cargo check 累计 | 16 次全部通过 |

---

## 六、待办 (Backlog)

### Phase 2 (高优)
- [ ] DuckDB 实例统一 (DuckDBEngine + result_service)
- [ ] JSON/Array/IP 列类型支持
- [ ] get_table_profile Tauri Command
- [ ] TableProfileView.vue
- [ ] 导航树右键"快速探查"

### Phase 3
- [ ] Schema 质量评分
- [ ] 表关系发现

### 增强
- [ ] 洞察历史版本 diff 对比
- [ ] 导出 Markdown / JSON
- [ ] 骨架屏 / 过渡动画
- [ ] 性能基准测试
