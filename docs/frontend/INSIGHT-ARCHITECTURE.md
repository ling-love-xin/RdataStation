# RdataStation 洞察体系 — 技术架构文档

> 版本：v2.0
> 创建日期：2026-05-07
> 最后更新：2026-05-07
> 状态：Phase 1 + 持久化层

---

## 一、整体数据流（含持久化）

```
用户点击列头 (AG Grid Header)
        │
        ▼
QueryResultPanel.vue
  handleGridContextMenu()
  检测 type === 'header'
        │
        │  CustomEvent: 'open-column-insight'
        │  detail: { column, tempTable }
        ▼
ColumnInsightPanel.vue
  onMounted → addEventListener
        │
        │  invoke('get_column_insight_full', { input: { temp_table, column_name } })
        ▼
Rust: result_commands.rs
  get_column_insight_full()
        │
        ▼
Rust: result_service.rs
  ResultService::get_column_insight_full()
    ├─ get_column_stats()    → ColumnStats (统计)
    ├─ get_column_sample()   → Vec<Value> (样例)
    └─ get_column_histogram() → Vec<DistributionBin> (分箱)
        │
        │  ColumnInsightFull { stats, sample, histogram }
        ▼
前端 useInsightStore
  insightStore.loadColumnInsight()
        │
        │  reactive 更新
        ▼
ColumnInsightPanel.vue 渲染
  根据 stats.stats_detail 的 kind 字段
  选择对应的渲染子组件
```

---

## 二、后端类型体系

### 2.1 ColumnInsightFull（顶层结构）

```rust
#[derive(Debug, serde::Serialize)]
pub struct ColumnInsightFull {
    pub stats: ColumnStats,
    pub sample: Vec<serde_json::Value>,
    pub histogram: Option<Vec<DistributionBin>>,
}
```

### 2.2 ColumnStats（统计信息）

```rust
#[derive(Debug, serde::Serialize)]
pub struct ColumnStats {
    pub column_name: String,
    pub data_type: String,
    pub total_count: usize,
    pub null_count: usize,
    pub null_rate: f64,
    pub unique_count: Option<usize>,
    pub stats_detail: ColumnStatsDetail,
}
```

### 2.3 ColumnStatsDetail（多态详情）

```rust
#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind")]
pub enum ColumnStatsDetail {
    Numeric(NumericStats),
    Text(TextStats),
    DateTime(DateTimeStats),
    Boolean(BooleanStats),
    Unknown,
}
```

### 2.4 NumericStats

```rust
#[derive(Debug, serde::Serialize)]
pub struct NumericStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub median: f64,
    pub p25: f64,
    pub p75: f64,
    pub sum: f64,
    pub stddev: Option<f64>,
    pub skewness: Option<f64>,
    pub kurtosis: Option<f64>,
    pub is_extreme: Vec<ExtremeValue>,
}
```

### 2.5 TextStats

```rust
#[derive(Debug, serde::Serialize)]
pub struct TextStats {
    pub min_length: usize,
    pub max_length: usize,
    pub top_values: Vec<TextFrequency>,
}

#[derive(Debug, serde::Serialize)]
pub struct TextFrequency {
    pub value: String,
    pub count: usize,
    pub ratio: f64,
}
```

### 2.6 DateTimeStats

```rust
#[derive(Debug, serde::Serialize)]
pub struct DateTimeStats {
    pub earliest: String,
    pub latest: String,
    pub span_days: i64,
    pub monthly_distribution: Vec<TextFrequency>,
}
```

### 2.7 BooleanStats

```rust
#[derive(Debug, serde::Serialize)]
pub struct BooleanStats {
    pub true_count: usize,
    pub false_count: usize,
    pub true_ratio: f64,
}
```

### 2.8 DistributionBin

```rust
#[derive(Debug, serde::Serialize)]
pub struct DistributionBin {
    pub label: String,
    pub count: usize,
    pub ratio: f64,
}
```

### 2.9 ExtremeValue

```rust
#[derive(Debug, serde::Serialize)]
pub struct ExtremeValue {
    pub value: f64,
    pub kind: String, // "outlier_high" | "outlier_low"
}
```

---

## 三、前端类型体系

### 3.1 完整的 TypeScript 类型定义

```typescript
interface ColumnInsightFull {
  stats: ColumnStats
  sample: unknown[]
  histogram: DistributionBin[] | null
}

interface ColumnStats {
  column_name: string
  data_type: string
  total_count: number
  null_count: number
  null_rate: number
  unique_count: number | null
  stats_detail: ColumnStatsDetail
}

type ColumnStatsDetail =
  | { kind: 'Numeric' } & NumericStats
  | { kind: 'Text' } & TextStats
  | { kind: 'DateTime' } & DateTimeStats
  | { kind: 'Boolean' } & BooleanStats
  | { kind: 'Unknown' }

interface NumericStats {
  min: number; max: number; avg: number; median: number
  p25: number; p75: number; sum: number
  stddev: number | null; skewness: number | null; kurtosis: number | null
  is_extreme: ExtremeValue[]
}

interface TextStats {
  min_length: number; max_length: number
  top_values: TextFrequency[]
}

interface TextFrequency {
  value: string; count: number; ratio: number
}

interface DateTimeStats {
  earliest: string; latest: string; span_days: number
  monthly_distribution: TextFrequency[]
}

interface BooleanStats {
  true_count: number; false_count: number; true_ratio: number
}

interface DistributionBin {
  label: string; count: number; ratio: number
}

interface ExtremeValue {
  value: number; kind: string
}
```

---

## 四、前端组件树

### 4.1 Dockview 面板布局

```
DockviewLayout.vue
├─ testCenter (Center Area — SQL Editor)
├─ testLeft (Left Side Bar — Database Navigator)
├─ testBottom (Bottom Panel — Query Results)
└─ rightInsight (Right Side Bar — Column Insight)   ← 新增
    └─ ColumnInsightPanel.vue
        ├─ [Header] 标题栏 ("列洞察: {列名}")
        ├─ [NCollapse] 基础统计区
        │   ├─ NumericStatsView   (kind === 'Numeric')
        │   ├─ TextStatsView      (kind === 'Text')
        │   ├─ DateTimeStatsView  (kind === 'DateTime')
        │   └─ BooleanStatsView   (kind === 'Boolean')
        ├─ [NCollapse] 分布区
        │   └─ HistogramView
        ├─ [NCollapse] 数据质量区
        │   └─ QualityHintsView
        └─ [NCollapse] 样例数据区
            └─ SampleDataView
```

### 4.2 组件职责

| 组件 | 职责 | Props | 依赖 |
|------|------|-------|------|
| ColumnInsightPanel.vue | 容器：状态管理 + 折叠布局 | insightStore | useInsightStore, NCollapse |
| NumericStatsView | 数值统计渲染 | NumericStats | NText, NTag |
| TextStatsView | 文本频率渲染 | TextStats | NButton (可点击过滤) |
| DateTimeStatsView | 日期统计渲染 | DateTimeStats | NText |
| BooleanStatsView | 布尔统计渲染 | BooleanStats | NText |
| HistogramView | 分箱直方图渲染 | DistributionBin[] | CSS bar |
| QualityHintsView | 数据质量提示 | ColumnStats + insight | NAlert |
| SampleDataView | 样例数据列表 | unknown[] | NText |

---

## 五、状态管理 (useInsightStore)

### 5.1 Store 定义

```typescript
export const useInsightStore = defineStore('insight', () => {
  // State
  const currentColumn = ref<string | null>(null)
  const currentTempTable = ref<string | null>(null)
  const insightData = ref<ColumnInsightFull | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Computed
  const statsKind = computed(() => insightData.value?.stats?.stats_detail?.kind ?? 'Unknown')
  const hasData = computed(() => insightData.value !== null)
  const nullRatePercent = computed(() =>
    insightData.value ? (insightData.value.stats.null_rate * 100).toFixed(1) : '0.0'
  )

  // Actions
  async function loadColumnInsight(tempTable: string, column: string) { ... }
  function clear() { ... }

  return { currentColumn, currentTempTable, insightData, isLoading, error,
           statsKind, hasData, nullRatePercent, loadColumnInsight, clear }
})
```

### 5.2 数据流

```
loadColumnInsight(tempTable, column)
  │
  ├─ 设置 isLoading = true, error = null
  ├─ invoke('get_column_insight_full', { input: { temp_table, column_name } })
  │   └─ 成功 → insightData = result, currentColumn = column
  │   └─ 失败 → error = e.toString()
  └─ 设置 isLoading = false
```

---

## 六、事件体系

### 6.1 事件清单

| 事件名 | 发送方 | 监听方 | detail | 用途 |
|--------|--------|--------|--------|------|
| `open-column-insight` | QueryResultPanel | ColumnInsightPanel | `{ column, tempTable }` | 打开/刷新列洞察 |
| `insight-filter-by-value` | ColumnInsightPanel | QueryResultPanel | `{ column, value }` | 按洞察值过滤结果集 |

### 6.2 事件时序

```
QueryResultPanel                    ColumnInsightPanel
     │                                     │
     │── open-column-insight ──────────────▶│ onMounted 时注册监听
     │   { column: "region",                │
     │     tempTable: "rs_abc123" }         │
     │                                     │── loadColumnInsight()
     │                                     │── 渲染统计信息
     │                                     │
     │◀── insight-filter-by-value ─────── │ 用户点击频率分布中的值
     │   { column: "region",                │
     │     value: "华东" }                   │
     │                                     │
     │── AG Grid quickFilter ──            │
```

---

## 七、DuckDB SQL 查询模板

### 7.1 数值列统计

```sql
SELECT
    COUNT(*) AS total,
    COUNT("{col}") AS non_null,
    MIN("{col}")::DOUBLE AS min,
    MAX("{col}")::DOUBLE AS max,
    AVG("{col}")::DOUBLE AS avg,
    MEDIAN("{col}")::DOUBLE AS median,
    PERCENTILE_DISC(0.25) WITHIN GROUP (ORDER BY "{col}")::DOUBLE AS p25,
    PERCENTILE_DISC(0.75) WITHIN GROUP (ORDER BY "{col}")::DOUBLE AS p75,
    SUM("{col}")::DOUBLE AS sum,
    STDDEV_SAMP("{col}")::DOUBLE AS stddev,
    SKEWNESS("{col}")::DOUBLE AS skewness,
    KURTOSIS("{col}")::DOUBLE AS kurtosis
FROM "{table}"
```

### 7.2 文本列频率

```sql
SELECT
    "{col}"::VARCHAR AS value,
    COUNT(*) AS count,
    COUNT(*) * 1.0 / SUM(COUNT(*)) OVER() AS ratio
FROM "{table}"
WHERE "{col}" IS NOT NULL
GROUP BY 1
ORDER BY 2 DESC
LIMIT 10
```

### 7.3 日期列统计

```sql
SELECT
    MIN("{col}")::VARCHAR AS earliest,
    MAX("{col}")::VARCHAR AS latest,
    DATEDIFF('day', MIN("{col}"), MAX("{col}")) AS span_days
FROM "{table}"
WHERE "{col}" IS NOT NULL
```

### 7.4 日期列月度分布

```sql
SELECT
    STRFTIME("{col}", '%Y-%m') AS month,
    COUNT(*) AS count,
    COUNT(*) * 1.0 / SUM(COUNT(*)) OVER() AS ratio
FROM "{table}"
WHERE "{col}" IS NOT NULL
GROUP BY 1
ORDER BY 1
```

### 7.5 布尔列统计

```sql
SELECT
    COUNT(*) FILTER (WHERE "{col}") AS true_count,
    COUNT(*) FILTER (WHERE NOT "{col}") AS false_count
FROM "{table}"
```

### 7.6 分箱直方图 (10 箱)

```sql
WITH bounds AS (
    SELECT MIN("{col}")::DOUBLE AS lo, MAX("{col}")::DOUBLE AS hi
    FROM "{table}"
    WHERE "{col}" IS NOT NULL
)
SELECT
    CASE
        WHEN r = 10 THEN '> ' || CAST(lo + 9.0 * (hi - lo) / 10.0 AS VARCHAR)
        ELSE CAST(lo + (r - 1) * (hi - lo) / 10.0 AS VARCHAR)
             || ' - '
             || CAST(lo + r * (hi - lo) / 10.0 AS VARCHAR)
    END AS label,
    COUNT(*) AS count,
    COUNT(*) * 1.0 / SUM(COUNT(*)) OVER() AS ratio
FROM "{table}", bounds, (SELECT UNNEST(GENERATE_SERIES(1, 10)) AS r) bins
WHERE "{col}" IS NOT NULL
  AND "{col}" >= lo + (r - 1) * (hi - lo) / 10.0
  AND ("{col}" < lo + r * (hi - lo) / 10.0 OR r = 10)
GROUP BY r, lo, hi
ORDER BY r
```

### 7.7 样例数据

```sql
SELECT "{col}" FROM "{table}" LIMIT 5
```

---

## 八、前端目录结构

```
src/
├── extensions/
│   └── builtin/
│       └── workbench/
│           └── ui/
│               ├── components/
│               │   ├── DockviewLayout.vue          ← 修改：注册右侧面板
│               │   └── panels/
│               │       ├── ColumnInsightPanel.vue    ← 新增：洞察面板容器
│               │       └── QueryResultPanel.vue      ← 修改：完善列头事件
│               ├── services/
│               │   └── result-analysis.ts            ← 修改：扩展类型 + API
│               └── stores/
│                   └── insight-store.ts              ← 新增：洞察状态管理
```

---

## 九、API 契约

### 9.1 Tauri Command

```
命令名: get_column_insight_full

输入:
{
  "input": {
    "temp_table": "rs_abc123",
    "column_name": "amount"
  }
}

输出 (成功):
{
  "stats": {
    "column_name": "amount",
    "data_type": "DOUBLE",
    "total_count": 45678,
    "null_count": 959,
    "null_rate": 0.021,
    "unique_count": 38201,
    "stats_detail": {
      "kind": "Numeric",
      "min": 0.01,
      "max": 9999.99,
      "avg": 245.32,
      "median": 198.00,
      "p25": 50.0,
      "p75": 450.0,
      "sum": 11200000.0,
      "stddev": 187.45,
      "skewness": 1.82,
      "kurtosis": 2.5,
      "is_extreme": [
        { "value": 9999.99, "kind": "outlier_high" }
      ]
    }
  },
  "sample": [245.30, 189.00, 1234.56, 876.50, 9999.99],
  "histogram": [
    { "label": "0 - 1000", "count": 20632, "ratio": 0.452 },
    { "label": "1000 - 2000", "count": 17401, "ratio": 0.381 },
    ...
  ]
}

输出 (失败):
"Failed to load insight: DuckDB table not found"
```

---

## 十、持久化架构

### 10.1 双库分工

```
项目 (.RSMETA/)
├── project.db (SQLite)          ← 元数据：执行记录、版本追踪
│   └── insight_snapshots        表
│       ├── id, entity_type, entity_name
│       ├── entity_source (conn_id, db, schema, table)
│       ├── snapshot_id → FK 到 DuckDB 表
│       ├── row_count, elapsed_ms
│       ├── created_at
│       └── version_id, parent_version_id, checksum
│
└── analytics.duckdb (DuckDB)    ← 分析数据：洞察快照 JSON 存储
    ├── insight_column_snapshots  表
    │   ├── snapshot_id PRIMARY KEY
    │   ├── column_name, data_type
    │   ├── stats_json TEXT       ← ColumnInsightFull 序列化
    │   └── created_at
    │
    ├── insight_table_reports     表 (Phase 2)
    │   ├── report_id, table_name
    │   ├── report_json TEXT       ← TableProfile 序列化
    │   └── created_at
    │
    └── insight_schema_reports    表 (Phase 3)
        ├── report_id, schema_name
        ├── report_json TEXT       ← SchemaReport 序列化
        └── created_at
```

### 10.2 版本化策略

- DuckDB 中的每条洞察记录包含 `version_id`, `parent_version_id`, `checksum`
- 同列再次分析时，创建新版本，`parent_version_id` 指向上一个版本
- 前端可展示"历史洞察版本"并做 diff 对比
- 符合项目已有的 `Versioned<T>` 设计模式

### 10.3 迁移系统集成

- `project_meta/008_insight_snapshots.sql` — SQLite 建表
- `project_analysis/002_insight_storage.sql` — DuckDB 建表
- `ProjectDatabaseManager::open()` 自动运行迁移（无需手动触发）

### 10.4 DuckDB 实例统一

**Phase 1 状态**：`result_service.rs` 使用独立的 in-memory DuckDB 实例（`get_or_create_duckdb()`）

**Phase 1 优化（本次）**：增加可选的持久化写入路径
- `get_column_insight_full()` 仍用 in-memory DuckDB 做实时计算（毫秒级）
- 新增 `save_column_insight_snapshot()` 将结果异步写入 `analytics.duckdb`
- 新增 `get_column_insight_history()` 从 `analytics.duckdb` 读取历史快照

**Phase 2（后续）**：结果集临时表直接创建在 `analytics.duckdb` 中，彻底统一实例

### 10.5 持久化时间线

| 操作 | 存储位置 | 触发时机 |
|------|---------|---------|
| 列洞察实时计算 | in-memory DuckDB 临时表 | 点击列头（立即） |
| 列洞察快照保存 | `analytics.duckdb` + `project.db` | 用户点"保存洞察"或自动保存 |
| 列洞察历史查询 | `analytics.duckdb` | 打开洞察面板 → "历史"标签 |
| 表探查报告 | `analytics.duckdb` + `project.db` | Phase 2，点击"快速探查" |
| Schema 洞察报告 | `analytics.duckdb` + `project.db` | Phase 3，点击"Schema 洞察" |
