# Mock 数据生成器 — 架构设计与开发计划

> 版本：v2.3
> 日期：2026-05-09
> 状态：Phase 0-7 后端全部完成 ✅ | 前端：主面板 + Store ✅ | 前端子功能开发中 ⏳
> 基于：现有代码库调研 + 产品设计文档 v2.0 + fake crate v5.1.0 实际 API 验证

---

## 目录

1. [架构概览](#1-架构概览)
2. [Rust 后端模块设计](#2-rust-后端模块设计)
3. [数据模型定义](#3-数据模型定义)
4. [Tauri Command 接口](#4-tauri-command-接口)
5. [前端组件设计](#5-前端组件设计)
6. [智能列名映射机制](#6-智能列名映射机制)
7. [场景模板设计](#7-场景模板设计)
8. [分阶段开发计划](#8-分阶段开发计划)
9. [技术依赖清单](#9-技术依赖清单)
10. [不明确事项 & 待确认决策](#10-不明确事项--待确认决策)

---

## 1. 架构概览

### 1.1 整体分层

```
┌─────────────────────────────────────────────────────────────┐
│                    前端 (Vue 3 + TS)                         │
│  ┌──────────┐  ┌───────────┐  ┌────────────┐  ┌──────────┐ │
│  │MockPanel │  │ImportDlg  │  │TemplateDlg │  │ AdvConfig│ │
│  │ (主面板)  │  │(导入结构) │  │(场景模板)  │  │ (高级配置)│ │
│  └────┬─────┘  └─────┬─────┘  └─────┬──────┘  └────┬─────┘ │
│       │              │              │              │        │
│  ┌────┴──────────────┴──────────────┴──────────────┴─────┐  │
│  │              composables/useMockGenerator.ts           │  │
│  │              (业务逻辑 Hook)                            │  │
│  └────────────────────────┬───────────────────────────────┘  │
│                           │ tauri.invoke()                   │
├───────────────────────────┼─────────────────────────────────┤
│                     Tauri IPC                                │
├───────────────────────────┼─────────────────────────────────┤
│                     Rust 后端                                │
│  ┌────────────────────────┴───────────────────────────────┐  │
│  │           commands/mock_commands.rs                     │  │
│  │   mock_generate | mock_import_schema | mock_export     │  │
│  │   mock_get_templates | mock_get_history | ...          │  │
│  └────────────────────────┬───────────────────────────────┘  │
│                           │                                   │
│  ┌────────────────────────┴───────────────────────────────┐  │
│  │           core/mock/  (新增模块)                         │  │
│  │  ┌────────────┐ ┌──────────────┐ ┌──────────────────┐  │  │
│  │  │ engine.rs  │ │ schema_map.rs│ │templates.rs     │  │  │
│  │  │ 数据生成   │ │ 列名映射     │ │ 场景模板        │  │  │
│  │  │ 引擎       │ │ 规则引擎     │ │ 管理            │  │  │
│  │  └─────┬──────┘ └──────┬───────┘ └────────┬─────────┘  │  │
│  │        │               │                  │            │  │
│  │  ┌─────┴───────────────┴──────────────────┴─────────┐  │  │
│  │  │              models.rs / error.rs                 │  │  │
│  │  │              数据模型 & 错误定义                   │  │  │
│  │  └──────────────────────────────────────────────────┘  │  │
│  └─────────────────────────────────────────────────────────┘  │
│                           │                                   │
│  ┌────────────────────────┴───────────────────────────────┐  │
│  │              复用现有基建                                │  │
│  │  ┌───────────────┐  ┌──────────────┐  ┌─────────────┐ │  │
│  │  │DuckDbService  │  │ScratchpadStore│ │MetadataCache│ │  │
│  │  │(临时表+DuckDB)│  │(草稿箱文件)  │  │(表结构缓存) │  │
│  │  └───────────────┘  └──────────────┘  └─────────────┘ │  │
│  └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 核心依赖关系

```
core/mock/engine.rs
  ├── fake (crate)              ← 数据生成引擎
  ├── rand                        ← 随机种子
  ├── DuckDB Appender API         ← 批量写入
  └── arrow                       ← RecordBatch 构建

core/mock/schema_map.rs
  ├── 列名 → 生成器映射表 (硬编码规则)
  └── 置信度评分算法

core/mock/templates.rs
  ├── 内置场景模板 (JSON 配置)
  └── 模板 DSL 解析

commands/mock_commands.rs
  ├── core/mock/*                 ← 调用 mock 模块
  ├── core/services/duckdb_service ← 复用 DuckDB 服务
  ├── core/scratchpad/*           ← 复用草稿箱存储
  └── core/cache/metadata_cache   ← 复用元数据缓存
```

---

## 2. Rust 后端模块设计

### 2.1 新增目录结构（✅ 已实现）

```
src-tauri/src/
├── core/
│   └── mock/                        ← 新增模块
│       ├── mod.rs                   ← ✅ 模块入口，pub use re-export
│       ├── models.rs                ← ✅ 数据模型（~60个 GeneratorConfig 变体）
│       ├── error.rs                 ← ✅ MockError 定义
│       ├── engine.rs                ← ✅ 核心生成引擎（单文件 ~800行）
│       ├── schema_map.rs            ← ✅ 列名→生成器智能映射（~60条规则）
│       └── templates.rs             ← ✅ Phase 5（场景模板管理）
│       └── history.rs               ← ✅ Phase 7（生成历史 DuckDB 存储）
│
├── commands/
│   └── mock_commands.rs             ← ✅ 新增 Tauri 命令（6个命令）
│
└── lib.rs                            ← ✅ 已更新：注册 mock_commands
```

> **设计变更**：原计划创建 `generators/` 子模块（7个子文件），实际简化合并到 `engine.rs` 单文件实现。原因：fake v5.1 的 Dummy trait 统一了 API 模式，无需按类别分文件。`templates.rs` 推迟到 Phase 5。

### 2.2 Fake v5.1.0 API 路径对照表（⚠️ 关键）
> 以下是在开发过程中实际验证的 fake v5.1.0 API 路径。多个生成器在 v5.1 中模块路径发生变化：

| 生成器 | 预期路径（v2.9） | **实际路径（v5.1）** | 说明 |
|--------|------------------|---------------------|------|
| Seniority | `company::en::Seniority` | **`job::en::Seniority`** | 🔄 移动到 job 模块 |
| Field | `company::en::Field` | **`job::en::Field`** | 🔄 移动到 job 模块 |
| Position | `company::en::Position` | **`job::en::Position`** | 🔄 移动到 job 模块 |
| UUIDv1/v3/v4/v5 | `uuid::en::UUIDv1` | **`fake::uuid::UUIDv1`** | 🔄 crate 根级 re-export |
| FerroidULID 等 | `ferroid::en::FerroidULID` | **`fake::ferroid::FerroidULID`** | 🔄 crate 根级 re-export |
| Semver | `semver::en::Semver` | **`filesystem::en::Semver`** | 🔄 合并到 filesystem 模块 |
| LicencePlate | `automotive::en::LicencePlate` | **`automotive::fr_fr::LicencePlate`** | ⚠️ 仅限 FR_FR/IT_IT/NL_NL 区域，EN 无实现 |
| HealthInsuranceCode | `administrative::en::HealthInsuranceCode` | **`administrative::fr_fr::HealthInsuranceCode`** | ⚠️ 仅限 FR_FR 区域 |
| Url | `url::en::Url` | **❌ 不存在** | 🔴 fake v5.1 无独立 Url 生成器，改为自定义实现 |
| Date | `time::raw::Date(start, end)` | **`chrono::en::Date()`** (无参) | ⚠️ API 变更：不再接受日期范围参数 |
| DateTimeBetween | `time::raw::DateTimeBetween(start, end)` | **`chrono::en::DateTimeBetween(start, end)`** | ⚠️ 输入类型改为 `chrono::DateTime<Utc>` |
| Time | `time::en::Time()` | **`chrono::en::Time()`** | 🔄 使用 chrono 模块 |
| Duration | `time::en::Duration()` | **`chrono::en::Duration()`** | 🔄 使用 chrono 模块 |
| Code (Markdown) | 返回 `Vec<String>` | **返回 `String`** | ⚠️ 返回类型变更 |
| ValidStatusCode | 返回 `u16` | **返回 `String`** | ⚠️ Dummy<String> 而非 Dummy<u16> |

> **关键教训**：fake 4.x → 5.x 是 breaking change，`time` feature 的生成器（time crate 类型）与 `chrono` feature 的生成器（chrono crate 类型）是两套并行的实现。项目统一使用 `chrono` feature。

> **🎉 覆盖里程碑（v2.1）**：fake v5.1.0 共 **21 模块 106 个生成器**，RdataStation GeneratorConfig 已 **100% 覆盖**：
> - `models.rs`：106 个变体（含 15 个 v2.1 新增：Word, BuzzwordMiddle, BuzzwordTail, FreeEmailProvider, DomainSuffix, FreeEmail, IPv4, IPv6, IP, SecondaryAddressType, ImageUrl, ImageUrlWithSeed, ImageUrlGrayscale, ImageUrlBlur, ImageUrlCustom）
> - `engine.rs`：106 条 `generate_cell` 匹配分支
> - `schema_map.rs`：~80 条智能映射规则
> - `mock-api.ts`：`GeneratorType` 联合类型 100% 对齐后端枚举

### 2.3 engine.rs — 核心生成引擎（✅ 已实现）

**实现要点：**
- Dummy trait API：`Foo.fake_with_rng::<String, _>(&mut rng)` 统一模式
- RNG：使用 `rand::random::<u64>()`（项目 rand 0.8）生成种子，`fake::rand::StdRng`（rand 0.10）作为生成器 RNG
- DuckDB 写入：使用批量 `INSERT INTO` 语句（每 10000 行一批），而非 Appender API（兼容性更好）
- sanitize：自定义 `sanitize_table_name()` 函数过滤特殊字符，替代 slug crate
- 日期生成：使用 `chrono::en::DateTimeBetween` + `chrono::NaiveDate`（而非 `time` crate 类型）
- 预览读取：`ORDER BY rowid LIMIT 10`，直接从 DuckDB 查询
- 导出：CSV/Parquet/XLSX 通过 DuckDB `COPY TO`；SQL INSERT 手动拼接；Table 模式通过 CREATE TABLE AS SELECT

```rust
// engine.rs 核心流程（实际实现）
use fake::{Fake, Faker, Dummy};
use fake::rand::SeedableRng;

impl MockEngine {
    pub fn generate(config: &MockConfig) -> Result<MockGenerateResult, MockError> {
        // 1. 初始化 RNG（项目 rand 0.8 → fake::rand::StdRng）
        let mut rng: StdRng = match config.seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::seed_from_u64(rand::random()),
        };
        
        // 2. 创建 DuckDB 表
        // 3. 批量 INSERT（batch_size = 10_000）
        // 4. 读取预览前10行
        // 5. 返回 MockGenerateResult
    }
    
    fn generate_cell(gen: &GeneratorConfig, rng: &mut StdRng, _locale: &Locale) -> String {
        // 匹配 ~60 个 GeneratorConfig 变体，每个调用对应 fake v5.1 API
    }
}
```

### 2.4 schema_map.rs — 智能列名映射（✅ 已实现 ~80 条规则，完整覆盖 106 种 GeneratorConfig 变体）

```rust
// 设计概要

/// Mock 生成配置
pub struct MockConfig {
    pub table_name: String,
    pub row_count: usize,
    pub seed: Option<u64>,
    pub locale: Locale,
    pub columns: Vec<ColumnDef>,
}

/// 单列定义
pub struct ColumnDef {
    pub name: String,
    pub data_type: ColumnDataType,
    pub generator: GeneratorConfig,
    pub nullable_ratio: f64,       // NULL 比例 0.0-1.0
    pub unique: bool,               // 是否唯一
}

/// 生成器配置（枚举，暴露 fake-rs v5.1 全部能力）
pub enum GeneratorConfig {
    // ============ 数值类（增强：NumberWithFormat, Digit, Boolean）============
    AutoIncrement { start: i64, step: i64 },
    RandomInt { min: i64, max: i64 },
    RandomFloat { min: f64, max: f64, precision: u8 },
    RandomDecimal { min: f64, max: f64, scale: u8 },
    Digit,                                          // 🆕 v4: 随机 0-9 数字
    NumberWithFormat { fmt: String },               // 🆕 v4: 模式 "#^###-####"（#=0-9, ^=1-9）
    Boolean { ratio: u8 },                          // 🆕 v4: 按比例 true/false (0-100)
    
    // ============ 字符串/文本类 ============
    Constant { value: String },
    Words { min: usize, max: usize },
    Sentence { min: usize, max: usize },            // 🆕 v5.1: 句子
    Sentences { min: usize, max: usize },           // 🆕 v5.1: 多句
    Paragraph { count: usize },                     // 🆕 v5.1: 段落（替代 Lorem）
    Paragraphs { count: usize },                    // 🆕 v5.1: 多段落
    Regex { pattern: String },
    Template { template: String },                  // "USER-{id:06d}"
    
    // ============ 🆕 Markdown 生成器（v4.4+）============
    MarkdownItalicWord,                             // *italic*
    MarkdownBoldWord,                               // **bold**
    MarkdownLink,                                   // [text](url)
    MarkdownBulletPoints,                           // - item
    MarkdownListItems,                              // 1. item
    MarkdownBlockQuoteSingle,                       // > quote
    MarkdownBlockQuoteMulti,                        // > multi-line
    MarkdownCode,                                   // `code`
    
    // ============ 个人信息类 ============
    Name,
    NameWithTitle,                                  // 🆕: 带头衔姓名 Dr. 张三
    FirstName,
    LastName,
    Title,                                          // 🆕: 头衔 Mr./Mrs./Dr.
    Suffix,                                         // 🆕: 后缀 Jr./Sr./III
    Email,
    SafeEmail,
    PhoneNumber,
    CellNumber,                                     // 🆕 v5.1: 手机号（比 PhoneNumber 更激进）
    Username,
    Password { min: usize, max: usize },            // 🆕 v5.1: 可配置长度
    
    // ============ 地址类 ============
    Country,
    CountryCode,                                    // 🆕 v5.1: ISO 国家代码 CN/US
    CountryName,                                    // 🆕 v5.1: 国家全名
    City,
    CityPrefix,                                     // 🆕 v5.1: 城市前缀（如 "New"）
    CitySuffix,                                     // 🆕 v5.1: 城市后缀（如 "burgh"）
    StateName,                                      // 🆕 v5.1: 州/省名
    StateAbbr,                                      // 🆕 v5.1: 州/省缩写
    StreetName,                                     // 🆕 v5.1: 街道名
    StreetSuffix,                                   // 🆕 v5.1: 街道后缀 St./Ave.
    ZipCode,
    PostCode,                                       // 🆕 v5.1: 邮政编码（国际化）
    BuildingNumber,                                 // 🆕 v5.1: 楼号
    SecondaryAddress,                               // 🆕 v5.1: 二级地址 Apt 3B
    Latitude,                                       // 🆕 v5.1: 纬度
    Longitude,                                      // 🆕 v5.1: 经度
    Geohash { precision: u8 },                      // 🆕 v5.1: GeoHash 编码
    TimeZone,                                       // 🆕 v5.1: 时区
    IpAddress,
    MacAddress,                                     // 🆕 v5.1: MAC 地址
    
    // ============ 日期时间类（增强：Between/Before/After/Duration）============
    DateTime { min: String, max: String },
    DateTimeBefore { before: String },              // 🆕 v4: 某时间之前
    DateTimeAfter { after: String },                // 🆕 v4: 某时间之后
    DateTimeBetween { start: String, end: String }, // 🆕 v4: 时间区间（更语义化）
    Date { min: String, max: String },
    Time,
    Duration,                                       // 🆕 v4: 随机时间间隔
    
    // ============ 商业类 ============
    CompanyName,
    CompanySuffix,                                  // 🆕 v5.1: 公司后缀 Inc./LLC
    JobTitle,
    Profession,                                     // 🆕 v5.1: 职业
    Industry,                                       // 🆕 v5.1: 行业
    Seniority,                                      // 🆕 v5.1: 资历级别
    Field,                                          // 🆕 v5.1: 领域
    Position,                                       // 🆕 v5.1: 职位
    Buzzword,                                       // 🆕 v5.1: 商业热词
    CatchPhrase,                                    // 🆕 v5.1: 商业口号
    BsVerb, BsAdj, BsNoun, Bs,                     // 🆕 v5.1: 商业套话组合
    
    // ============ 金融类 🆕 v4+ ============
    CurrencyCode,
    CurrencyName,
    CurrencySymbol,
    Bic,                                            // 🆕 v4: 银行 SWIFT/BIC 代码
    Isin,                                           // 🆕 v4: 国际证券识别码
    CreditCardNumber,
    
    // ============ 网络/技术类（增强：多种 UUID）============
    UuidV1,                                         // 🆕 v4: 基于时间 UUID
    UuidV3,                                         // 🆕 v4: 基于名称 MD5 UUID
    UuidV4,                                         // v5.1 替代原 Uuid
    UuidV5,                                         // 🆕 v4: 基于名称 SHA-1 UUID
    Url,
    UserAgent,
    MimeType,
    Semver,                                         // 🆕 v5.1: 语义版本号
    SemverStable,                                   // 🆕 v5.1: 稳定版号
    SemverUnstable,                                 // 🆕 v5.1: 不稳定版号
    FilePath,                                       // 🆕 v5.1: 文件路径
    FileName,                                       // 🆕 v5.1: 文件名
    FileExtension,                                  // 🆕 v5.1: 扩展名
    DirPath,                                        // 🆕 v5.1: 目录路径
    
    // ============ 🆕 颜色类 v4+ ============
    HexColor,                                       // #ff5733
    RgbColor,                                       // rgb(255, 87, 51)
    RgbaColor,                                      // rgba(255, 87, 51, 0.8)
    HslColor,                                       // hsl(12, 100%, 60%)
    HslaColor,                                      // hsla(12, 100%, 60%, 0.8)
    Color,                                          // 随机颜色
    
    // ============ 🆕 Ferroid ID 类 v4.4+ ============
    FerroidULID,                                    // 26-char 排序 ID
    FerroidTwitterId,                               // Twitter/X Snowflake ID
    FerroidInstagramId,                             // Instagram ID
    FerroidMastodonId,                              // Mastodon ID
    FerroidDiscordId,                               // Discord Snowflake ID
    
    // ============ 🆕 条形码与标准编码 v4+ ============
    Isbn,                                           // ISBN 号
    Isbn10,                                         // ISBN-10
    Isbn13,                                         // ISBN-13
    RfcStatusCode,                                  // HTTP 状态码类别
    ValidStatusCode,                                // 有效 HTTP 状态码
    
    // ============ 🆕 汽车与行政 v5.1 ============
    LicencePlate,                                   // 车牌号
    HealthInsuranceCode,                            // 医保编号
    
    // ============ 约束类 ============
    ForeignKey { values: Vec<String> },
    Sequence { values: Vec<String>, cycle: bool },
    Weighted { choices: Vec<(String, f64)> },
    Either { left: Box<GeneratorConfig>, right: Box<GeneratorConfig> }, // 🆕 v4: 组合两个生成器
}

/// 数据生成结果
pub struct MockGenerateResult {
    pub table_name: String,
    pub temp_table_name: String,    // DuckDB 中实际表名
    pub row_count: usize,
    pub preview: Vec<Vec<serde_json::Value>>,  // 前10行预览
    pub columns: Vec<String>,
    pub elapsed_ms: u64,
}
```

**关键实现逻辑（fake v5.1 API）：**

```rust
// engine.rs 核心流程（适配 fake v5.1 + rand 0.10）
use fake::{Fake, Faker, Dummy};
use fake::rand::rngs::StdRng;
use fake::rand::SeedableRng;
use fake::faker::name::zh_cn::Name as ZhCnName;
use fake::faker::internet::zh_cn::SafeEmail as ZhCnSafeEmail;
use fake::locales::ZH_CN;

impl MockEngine {
    /// 1. 验证配置
    /// 2. 使用种子初始化 fake::rand::StdRng (rand 0.10)
    /// 3. 逐行生成数据（fake v5.1 的 locale-aware 生成器）
    /// 4. 通过 DuckDB Appender API 批量写入
    /// 5. 返回前10行预览
    pub async fn generate(config: MockConfig) -> Result<MockGenerateResult, MockError> {
        // Step 1: 初始化 RNG（使用 fake::rand 0.10，不是项目 rand 0.8）
        let mut rng: StdRng = match config.seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_rng(fake::rand::thread_rng()),
        };
        
        // Step 2: 构建 Arrow Schema
        let schema = Self::build_arrow_schema(&config.columns);
        let table_name = format!("temp_mock_{}", slug::slugify(&config.table_name));
        
        // Step 3: 分批生成 + DuckDB Appender 批量写入
        let duckdb = DuckDbService::get_or_create_duckdb()?;
        let conn = duckdb.lock().map_err(...)?;
        
        // 先创建表
        let ddl = Self::build_create_table_ddl(&table_name, &config.columns);
        conn.execute_batch(&ddl)?;
        
        // 使用 Appender API 批量写入
        let mut appender = conn.appender(&table_name)?;
        let batch_size = 10_000;
        let total_batches = (config.row_count + batch_size - 1) / batch_size;
        
        for batch_idx in 0..total_batches {
            let start = batch_idx * batch_size;
            let count = std::cmp::min(batch_size, config.row_count - start);
            let rows = Self::generate_batch(&config, &mut rng, start, count);
            let record_batch = Self::rows_to_arrow_batch(&schema, &rows)?;
            appender.append_batch(record_batch)?;
            // 每批次后刷新，控制内存
        }
        appender.flush()?;
        
        // Step 4: 读取前10行预览
        let preview = Self::read_preview_table(&conn, &table_name, 10)?;
        
        // Step 5: 注册为临时表
        DuckDBManager::global().register_temp_table(&table_name);
        
        Ok(MockGenerateResult {
            table_name: config.table_name.clone(),
            temp_table_name: table_name,
            row_count: config.row_count,
            preview,
            columns: config.columns.iter().map(|c| c.name.clone()).collect(),
            elapsed_ms,
        })
    }
    
    /// 根据生成器类型，使用 fake v5.1 API 生成单批次数据
    fn generate_row(generator: &GeneratorConfig, rng: &mut StdRng, locale: &Locale) -> String {
        match generator {
            // --- 数值类 ---
            GeneratorConfig::AutoIncrement { start, step } => { ... },
            GeneratorConfig::RandomInt { min, max } => {
                (*min..=*max).fake_with_rng::<i64, _>(rng).to_string()
            },
            GeneratorConfig::NumberWithFormat { fmt } => {
                // fake v5.1: NumberWithFormat
                let nf = fake::faker::number::raw::NumberWithFormat(EN, fmt.as_str());
                nf.fake_with_rng::<String, _>(rng)
            },
            GeneratorConfig::Boolean { ratio } => {
                // fake v5.1: Boolean(ratio)
                let bool_gen = fake::faker::boolean::raw::Boolean(EN, *ratio);
                bool_gen.fake_with_rng::<bool, _>(rng).to_string()
            },
            
            // --- 姓名类（locale-aware）---
            GeneratorConfig::Name => {
                // fake v5.1: 根据 locale 选择对应的 Name 生成器
                locale_name::<EN>().fake_with_rng::<String, _>(rng)
            },
            
            // --- 日期类（fake v5.1 增强 API）---
            GeneratorConfig::DateTimeBetween { start, end } => {
                let s = DateTime::parse_from_rfc3339(start)?;
                let e = DateTime::parse_from_rfc3339(end)?;
                // fake v5.1: DateTimeBetween 直接返回 DateTime<Utc>
                fake::faker::time::raw::DateTimeBetween(EN, s.into(), e.into())
                    .fake_with_rng::<chrono::DateTime<chrono::Utc>, _>(rng)
                    .to_rfc3339()
            },
            GeneratorConfig::DateTimeBefore { before } => {
                let b = DateTime::parse_from_rfc3339(before)?;
                fake::faker::time::raw::DateTimeBefore(EN, b.into())
                    .fake_with_rng::<chrono::DateTime<chrono::Utc>, _>(rng)
                    .to_rfc3339()
            },
            
            // --- 🆕 颜色类 ---
            GeneratorConfig::HexColor => {
                fake::faker::color::raw::HexColor(EN).fake_with_rng::<String, _>(rng)
            },
            
            // --- 🆕 Ferroid ID ---
            GeneratorConfig::FerroidULID => {
                fake::faker::ferroid::raw::FerroidULID(EN).fake_with_rng::<String, _>(rng)
            },
            
            // --- 🆕 金融 ---
            GeneratorConfig::Bic => {
                fake::faker::finance::raw::Bic(EN).fake_with_rng::<String, _>(rng)
            },
            GeneratorConfig::Isin => {
                fake::faker::finance::raw::Isin(EN).fake_with_rng::<String, _>(rng)
            },
            
            // ... 其他生成器类似
        }
    }
}
```

### 2.3 schema_map.rs — 智能列名映射

```rust
/// 列名映射规则（三级置信度）
pub struct ColumnMappingRule {
    pub patterns: Vec<String>,        // 匹配模式列表
    pub generator: GeneratorConfig,    // 推荐生成器
    pub confidence: ConfidenceLevel,
    pub sample_value: String,         // 示例值
}

pub enum ConfidenceLevel {
    High,    // 🟢 精确匹配
    Medium,  // 🟡 模糊匹配
    Low,     // ⚪ 兜底
}

impl ColumnMapper {
    /// 根据列名+数据类型推断最佳生成器
    pub fn infer(column_name: &str, data_type: &ColumnDataType) -> ColumnMappingRule {
        let name_lower = column_name.to_lowercase();
        
        // 🟢 精确匹配表（优先级最高）
        let exact_rules = [
            ("id", GeneratorConfig::AutoIncrement { start: 1, step: 1 }),
            ("email", GeneratorConfig::SafeEmail),
            ("name", GeneratorConfig::Name),
            ("username", GeneratorConfig::Username),
            ("password", GeneratorConfig::Password),
            ("phone", GeneratorConfig::PhoneNumber),
            ("address", GeneratorConfig::StreetAddress),
            ("city", GeneratorConfig::City),
            ("country", GeneratorConfig::Country),
            ("zipcode", GeneratorConfig::ZipCode),
            ("uuid", GeneratorConfig::Uuid),
            ("url", GeneratorConfig::Url),
            ("company", GeneratorConfig::CompanyName),
            ("job", GeneratorConfig::JobTitle),
            ("ip", GeneratorConfig::IpAddress),
            ("credit_card", GeneratorConfig::CreditCardNumber),
            ("created_at", GeneratorConfig::DateTime { min: ..., max: ... }),
            ("updated_at", GeneratorConfig::DateTime { min: ..., max: ... }),
            ("amount", GeneratorConfig::RandomDecimal { min: 0.01, max: 99999.99, scale: 2 }),
            ("price", GeneratorConfig::RandomDecimal { min: 0.01, max: 9999.99, scale: 2 }),
            ("quantity", GeneratorConfig::RandomInt { min: 1, max: 1000 }),
            ("status", GeneratorConfig::ForeignKey { values: vec!["active","pending","cancelled"] }),
        ];
        
        // 🟡 模糊匹配规则
        // 基于子字符串包含关系
        // ...
        
        // ⚪ 兜底：根据数据类型推断
        // INTEGER → RandomInt
        // VARCHAR → Words(1..3)
        // DECIMAL → RandomDecimal
        
        ColumnMappingRule { ... }
    }
}
```

### 2.4 templates.rs — 场景模板

```rust
/// 场景模板定义
pub struct ScenarioTemplate {
    pub id: String,
    pub name: String,              // "电商订单"
    pub description: String,
    pub category: TemplateCategory,
    pub tables: Vec<TemplateTable>,
}

pub enum TemplateCategory {
    ECommerce,
    SocialMedia,
    Finance,
    Healthcare,
    Education,
    IoT,
    Custom,
}

pub struct TemplateTable {
    pub name: String,
    pub row_count: usize,
    pub columns: Vec<ColumnDef>,
    pub relations: Vec<TableRelation>,  // 表间关联
}

pub struct TableRelation {
    pub source_column: String,
    pub target_table: String,
    pub target_column: String,
    pub relation_type: RelationType,  // OneToOne, OneToMany
}
```

### 7.3 首批内置模板（6个，利用 v5.1 新功能）

| 模板 | 表数量 | 说明 | v5.1 新生成器利用 |
|------|--------|------|-----------------|
| `ecommerce` | 3（users, products, orders） | 电商订单系统 | DateTimeBetween, CellNumber, CountryCode |
| `social_media` | 3（users, posts, comments） | 社交平台 | FerroidTwitterId, HexColor, MarkdownBlockQuoteMulti |
| `finance` | 3（accounts, transactions, categories） | 金融账务 | Bic, Isin, CreditCardNumber, Decimal |
| `company` | 2（employees, departments） | 企业组织架构 | NameWithTitle, Profession, Industry, LicencePlate |
| `iot_device` 🆕 | 2（devices, readings） | IoT 设备数据 | MacAddress, Geohash, Latitude, Longitude, Semver |
| `content_cms` 🆕 | 2（articles, media） | 内容管理系统 | MarkdownBoldWord, MarkdownLink, FilePath, MimeType, UuidV4 |

### 7.4 新增模板示例：IoT 设备（利用 v5.1 地理 + 网络 + 版本号能力）

```json
{
  "id": "iot_device",
  "name": "IoT 设备监控",
  "description": "包含设备注册和传感器读数两表",
  "category": "iot",
  "locale": "zh_cn",
  "tables": [
    {
      "name": "devices",
      "rowCount": 500,
      "columns": [
        { "name": "id", "dataType": "bigint", "generator": { "type": "auto_increment" } },
        { "name": "device_uid", "dataType": "varchar", "generator": { "type": "uuid_v4" } },
        { "name": "name", "dataType": "varchar", "generator": { "type": "words", "params": { "min": 2, "max": 3 } } },
        { "name": "mac_address", "dataType": "varchar", "generator": { "type": "mac_address" } },
        { "name": "firmware_version", "dataType": "varchar", "generator": { "type": "semver_stable" } },
        { "name": "latitude", "dataType": "double", "generator": { "type": "latitude" } },
        { "name": "longitude", "dataType": "double", "generator": { "type": "longitude" } },
        { "name": "geohash", "dataType": "varchar", "generator": { "type": "geohash", "params": { "precision": 7 } } },
        { "name": "status", "dataType": "varchar", "generator": { "type": "foreign_key", "params": { "values": ["online","offline","maintenance","error"] } } },
        { "name": "registered_at", "dataType": "datetime", "generator": { "type": "datetime_between", "params": { "start": "2023-01-01T00:00:00Z", "end": "2025-12-31T23:59:59Z" } } }
      ]
    },
    {
      "name": "sensor_readings",
      "rowCount": 50000,
      "columns": [
        { "name": "id", "dataType": "bigint", "generator": { "type": "auto_increment" } },
        { "name": "device_id", "dataType": "bigint", "generator": { "type": "random_int", "params": { "min": 1, "max": 500 } } },
        { "name": "temperature", "dataType": "float", "generator": { "type": "random_float", "params": { "min": -20.0, "max": 60.0, "precision": 2 } } },
        { "name": "humidity", "dataType": "float", "generator": { "type": "random_float", "params": { "min": 0.0, "max": 100.0, "precision": 1 } } },
        { "name": "battery_level", "dataType": "float", "generator": { "type": "random_float", "params": { "min": 0.0, "max": 100.0, "precision": 1 } } },
        { "name": "recorded_at", "dataType": "datetime", "generator": { "type": "datetime_between", "params": { "start": "2025-05-01T00:00:00Z", "end": "2025-05-08T23:59:59Z" } } }
      ]
    }
  ]
}
```

### 7.5 新增模板示例：内容管理系统 CMS

```json
{
  "id": "content_cms",
  "name": "内容管理系统",
  "description": "包含文章和媒体资源两表",
  "category": "cms",
  "locale": "zh_cn",
  "tables": [
    {
      "name": "articles",
      "rowCount": 5000,
      "columns": [
        { "name": "id", "dataType": "bigint", "generator": { "type": "auto_increment" } },
        { "name": "title", "dataType": "varchar", "generator": { "type": "sentence", "params": { "min": 3, "max": 8 } } },
        { "name": "slug", "dataType": "varchar", "generator": { "type": "uuid_v4" } },
        { "name": "author", "dataType": "varchar", "generator": { "type": "name" } },
        { "name": "body_md", "dataType": "text", "generator": { "type": "paragraphs", "params": { "count": 3 } } },
        { "name": "excerpt", "dataType": "varchar", "generator": { "type": "sentences", "params": { "min": 1, "max": 2 } } },
        { "name": "featured_color", "dataType": "varchar", "generator": { "type": "hex_color" } },
        { "name": "status", "dataType": "varchar", "generator": { "type": "foreign_key", "params": { "values": ["draft","review","published","archived"] } } },
        { "name": "is_featured", "dataType": "boolean", "generator": { "type": "boolean", "params": { "ratio": 15 } } },
        { "name": "published_at", "dataType": "datetime", "generator": { "type": "datetime_between", "params": { "start": "2024-01-01T00:00:00Z", "end": "2025-05-08T23:59:59Z" } } }
      ]
    },
    {
      "name": "media",
      "rowCount": 1000,
      "columns": [
        { "name": "id", "dataType": "bigint", "generator": { "type": "auto_increment" } },
        { "name": "file_name", "dataType": "varchar", "generator": { "type": "file_name" } },
        { "name": "file_path", "dataType": "varchar", "generator": { "type": "file_path" } },
        { "name": "mime_type", "dataType": "varchar", "generator": { "type": "mime_type" } },
        { "name": "file_size", "dataType": "bigint", "generator": { "type": "random_int", "params": { "min": 1024, "max": 104857600 } } },
        { "name": "uploaded_at", "dataType": "datetime", "generator": { "type": "datetime_between", "params": { "start": "2024-06-01T00:00:00Z", "end": "2025-05-08T23:59:59Z" } } }
      ]
    }
  ]
}
```

> 所有模板文件内置在 `src-tauri/resources/templates/`，编译时通过 `include_dir!` 宏嵌入二进制；用户自定义模板可放在 `.scratchpad/mock/templates/` 目录下，运行时动态加载。

---

## 3. 数据模型定义

### 3.1 Rust 端核心模型

```rust
// core/mock/models.rs

/// 列数据类型（映射到 DuckDB 类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnDataType {
    Integer,
    BigInt,
    Float,
    Double,
    Decimal { precision: u8, scale: u8 },
    Boolean,
    Varchar { length: Option<usize> },
    Text,
    Date,
    DateTime,
    Timestamp,
    Uuid,
    Blob,
}

/// 生成历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockGenerateHistory {
    pub id: String,
    pub table_name: String,
    pub row_count: usize,
    pub seed: Option<u64>,
    pub columns: Vec<ColumnDef>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub generated_as: GeneratedAs,  // 临时表 / 文件 / 永久表
    pub status: HistoryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeneratedAs {
    TempTable { name: String },
    File { path: String, format: String },
    PermanentTable { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryStatus {
    Success,
    Failed { reason: String },
    TempCleaned,  // 临时表已清理
}

/// 语言/地区（fake v5.1 支持 13 种）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Locale {
    ZhCn,  // 🇨🇳 Simplified Chinese (zh_cn)
    En,    // 🇺🇸 English (en)
    JaJp,  // 🇯🇵 Japanese (ja_jp)
    ZhTw,  // 🇹🇼 Traditional Chinese (zh_tw)
    FrFr,  // 🇫🇷 French (fr_fr)
    DeDe,  // 🇩🇪 German (de_de)
    ItIt,  // 🇮🇹 Italian (it_it)
    PtBr,  // 🇧🇷 Portuguese/Brazil (pt_br)
    PtPt,  // 🇵🇹 Portuguese/Portugal (pt_pt)
    NlNl,  // 🇳🇱 Dutch (nl_nl)        🆕 v4
    ArSa,  // 🇸🇦 Arabic (ar_sa)       🆕 v5.1
    TrTr,  // 🇹🇷 Turkish (tr_tr)      🆕 v5.0
    FaIr,  // 🇮🇷 Persian/Iran (fa_ir) 🆕 v5.0
}

/// MockError — 遵循 CoreError 模式
#[derive(Debug, thiserror::Error)]
pub enum MockError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Generation failed: {0}")]
    Generation(String),
    
    #[error("DuckDB error: {0}")]
    DuckDB(#[from] crate::core::error::CoreError),
    
    #[error("Export failed: format={0}, reason={1}")]
    Export(String, String),
    
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
}
```

### 3.2 TypeScript 端类型定义

```typescript
// src/types/mock.ts

/** 列数据类型 */
export type ColumnDataType =
  | 'integer'
  | 'bigint'
  | 'float'
  | 'double'
  | 'decimal'
  | 'boolean'
  | 'varchar'
  | 'text'
  | 'date'
  | 'datetime'
  | 'timestamp'
  | 'uuid'
  | 'blob'

/** 生成器类型（枚举，映射 fake-rs v5.1 全部 ~60 种能力） */
export type GeneratorType =
  // 数值类（增强）
  | 'auto_increment' | 'random_int' | 'random_float' | 'random_decimal'
  | 'digit' | 'number_with_format' | 'boolean'              // 🆕 v4
  // 文本类
  | 'constant' | 'words' | 'sentence' | 'sentences'         // 🆕 v5.1
  | 'paragraph' | 'paragraphs' | 'regex' | 'template'       // ✅ 更合理
  // Markdown 🆕 v4.4+
  | 'md_italic' | 'md_bold' | 'md_link' | 'md_bullet'
  | 'md_list' | 'md_blockquote' | 'md_code'
  // 个人信息
  | 'name' | 'name_with_title' | 'first_name' | 'last_name' // 🆕 v5.1
  | 'title' | 'suffix' | 'email' | 'safe_email' | 'username'
  | 'password' | 'phone_number' | 'cell_number'             // 🆕 v5.1
  // 地址（大幅增强）
  | 'country' | 'country_code' | 'country_name' | 'city'    // 🆕
  | 'city_prefix' | 'city_suffix' | 'state_name' | 'state_abbr'
  | 'street_name' | 'street_suffix' | 'zip_code' | 'post_code'
  | 'building_number' | 'secondary_address'
  | 'latitude' | 'longitude' | 'geohash' | 'timezone'       // 🆕
  | 'ip_address' | 'mac_address'                             // 🆕
  // 日期时间（增强）
  | 'datetime' | 'datetime_before' | 'datetime_after'        // 🆕
  | 'datetime_between' | 'date' | 'time' | 'duration'        // 🆕
  // 商业
  | 'company_name' | 'company_suffix' | 'job_title'          // 🆕
  | 'profession' | 'industry' | 'seniority' | 'field'        // 🆕
  | 'position' | 'buzzword' | 'catch_phrase'                 // 🆕
  // 金融 🆕 v4+
  | 'currency_code' | 'currency_name' | 'currency_symbol'
  | 'bic' | 'isin' | 'credit_card_number'
  // 网络/技术
  | 'uuid_v1' | 'uuid_v3' | 'uuid_v4' | 'uuid_v5'           // 🆕 多版本
  | 'url' | 'user_agent' | 'mime_type'
  | 'semver' | 'semver_stable' | 'semver_unstable'           // 🆕
  | 'file_path' | 'file_name' | 'file_extension' | 'dir_path' // 🆕
  // 颜色 🆕 v4+
  | 'hex_color' | 'rgb_color' | 'rgba_color'
  | 'hsl_color' | 'hsla_color' | 'color'
  // Ferroid ID 🆕 v4.4+
  | 'ferroid_ulid' | 'ferroid_twitter_id' | 'ferroid_instagram_id'
  | 'ferroid_mastodon_id' | 'ferroid_discord_id'
  // 编码 🆕 v4+
  | 'isbn' | 'isbn10' | 'isbn13' | 'rfc_status' | 'valid_status'
  // 汽车 & 行政 🆕 v5.1
  | 'licence_plate' | 'health_insurance'
  // 约束
  | 'foreign_key' | 'sequence' | 'weighted' | 'either'       // 🆕 v4

/** 单列定义 */
interface ColumnDef {
  name: string
  dataType: ColumnDataType
  generator: GeneratorConfig
  nullableRatio: number      // 0.0 ~ 1.0
  unique: boolean
}

/** 生成器配置 */
interface GeneratorConfig {
  type: GeneratorType
  params?: Record<string, unknown>
  sampleValue?: string        // 示例值
  confidenceLevel: 'high' | 'medium' | 'low'  // 🟢🟡⚪
}

/** Mock生成配置（前端→后端） */
interface MockGenerateInput {
  tableName: string
  rowCount: number
  seed: number | null
  locale: Locale
  columns: ColumnDef[]
}

/** Mock生成结果（后端→前端） */
interface MockGenerateResult {
  tableName: string
  tempTableName: string
  rowCount: number
  preview: Array<Record<string, unknown>>  // 前10行
  columns: string[]
  elapsedMs: number
}

/** 导出格式 */
type ExportFormat = 'csv' | 'parquet' | 'xlsx' | 'table' | 'sql_insert'

/** 导出请求 */
interface MockExportInput {
  tempTableName: string
  format: ExportFormat
  outputPath?: string        // CSV/Parquet/Excel 文件路径，TABLE 不需要
}

/** 场景模板 */
interface ScenarioTemplate {
  id: string
  name: string
  description: string
  category: string
  tables: TemplateTable[]
}

interface TemplateTable {
  name: string
  rowCount: number
  columns: ColumnDef[]
}

/** 从数据库导入请求 */
interface ImportSchemaInput {
  connId: string
  database: string
  schema: string | null
  tables: string[]           // 多选表名
}

/** 生成历史记录 */
interface MockHistoryRecord {
  id: string
  tableName: string
  rowCount: number
  seed: number | null
  columns: ColumnDef[]
  generatedAt: string         // ISO 8601
  generatedAs: {
    type: 'temp_table' | 'file' | 'permanent'
    name: string
  }
  status: 'success' | 'failed' | 'temp_cleaned'
}
```

---

## 4. Tauri Command 接口

### 4.1 已实现命令一览（✅ 后端全部完成）

| 命令 | 用途 | 输入 | 输出 | 状态 |
|------|------|------|------|------|
| `mock_generate` | 生成Mock数据 | `MockConfig` | `MockGenerateResult` | ✅ |
| `mock_preview` | 刷新预览（不重新生成） | `temp_table_name` | `preview rows` | ✅ |
| `mock_export` | 导出为指定格式 | `MockExportInput` | `export result` | ✅ |
| `mock_map_column` | 智能映射单个列名 | `column_name, data_type` | `ColumnMappingResponse` | ✅ |
| `mock_map_columns_batch` | 批量智能映射 | `Vec<ColumnDef>` | `Vec<ColumnMappingResponse>` | ✅ |
| `mock_list_templates` | 获取场景模板列表 | `() ` | `Vec<ScenarioTemplate>` | ✅ |
| `mock_import_schema` | 从 MetadataCache 导入表结构 + 推断生成器 | `ImportSchemaInput` | `Vec<ColumnDef>` | ✅ |
| `mock_apply_template` | 应用场景模板 | `template_id` | `ScenarioTemplate` | ✅ |
| `mock_save_to_scratchpad` | 一键保存到草稿箱（按格式路由） | `MockSaveToScratchpadInput` | `saved_path` | ✅ |
| `mock_persist_as_asset` | 保存Table到分析资源管理器 | `MockPersistAssetInput` | `MockPersistAssetResult` | ✅ |

### 4.2 历史 & 重生成命令（✅ Phase 7）

| 命令 | 用途 | 输入 | 输出 | 状态 |
|------|------|------|------|------|
| `mock_get_history` | 获取生成历史 | `limit: usize` | `Vec<MockHistoryRecord>` | ✅ |
| `mock_clear_history` | 清除生成历史 | `()` | `usize` (清除条数) | ✅ |
| `mock_re_generate` | 基于历史记录重新生成 | `history_id: String` | `MockGenerateResult` | ✅ |

> **实现要点**：
> - 存储在 DuckDB `_system.mock_history` 表中（内存数据库，跟随会话生命周期）
> - 每次 `mock_generate` 成功后自动保存历史记录
> - 表自动创建 (`CREATE TABLE IF NOT EXISTS`)，上限 200 条（LRU 淘汰）
> - `mock_re_generate` 从 `config_json` 反序列化 `MockConfig` 并重新调用 `generate()`

### 4.3 命令实现示例

```rust
// commands/mock_commands.rs

use crate::core::mock::{MockEngine, MockConfig, MockError, ColumnMapper};
use crate::core::services::duckdb_service::DuckDbService;

/// 生成 Mock 数据
#[tauri::command]
pub async fn mock_generate(
    config: MockConfig,
) -> Result<MockGenerateResult, String> {
    MockEngine::generate(config)
        .await
        .map_err(|e| e.to_string())
}

/// 导出 Mock 数据
#[tauri::command]
pub async fn mock_export(
    input: MockExportInput,
) -> Result<String, String> {
    match input.format {
        ExportFormat::Csv | ExportFormat::Parquet | ExportFormat::Xlsx => {
            // 复用现有 export_temp_table 模式
            let format = match input.format {
                ExportFormat::Csv => duckdb_service::ExportFormat::Csv,
                ExportFormat::Parquet => duckdb_service::ExportFormat::Parquet,
                ExportFormat::Xlsx => duckdb_service::ExportFormat::Xlsx,
                _ => unreachable!(),
            };
            DuckDbService::export_temp_table(&input.temp_table_name, &input.output_path, format)
                .map_err(|e| e.to_string())
        }
        ExportFormat::Table => {
            // 持久化：RENAME temp_mock_xxx → mock_xxx
            MockEngine::persist_table(&input.temp_table_name)
                .await
                .map_err(|e| e.to_string())
        }
        ExportFormat::SqlInsert => {
            // 生成 INSERT INTO 语句
            MockEngine::export_as_insert_sql(&input.temp_table_name, &input.output_path)
                .await
                .map_err(|e| e.to_string())
        }
    }
}

/// 从元数据缓存导入表结构
#[tauri::command]
pub async fn mock_import_schema(
    input: ImportSchemaInput,
) -> Result<Vec<ColumnDef>, String> {
    MockEngine::import_schema(&input.conn_id, &input.database, input.schema.as_deref(), &input.tables)
        .await
        .map_err(|e| e.to_string())
}

/// 智能列名映射
#[tauri::command]
pub async fn mock_map_column(
    column_name: String,
    data_type: String,
) -> Result<ColumnMappingResponse, String> {
    let rule = ColumnMapper::infer(&column_name, &ColumnDataType::from_str(&data_type));
    Ok(ColumnMappingResponse {
        generator: rule.generator,
        confidence: rule.confidence.to_string(),
        sample_value: rule.sample_value,
    })
}
```

### 4.3 lib.rs 注册更新

在 `src-tauri/src/commands/mod.rs` 中新增：
```rust
pub mod mock_commands;
pub use mock_commands::*;
```

在 `src-tauri/src/lib.rs` 的 `generate_handler!` 中新增所有 mock 命令。

---

## 5. 前端组件设计

### 5.1 组件树

```
src/extensions/builtin/workbench/ui/components/panels/
└── mock/
    ├── MockGeneratorPanel.vue          ← 主面板（dockview tab）
    ├── MockFieldTable.vue              ← 字段定义表格（AG Grid）
    ├── MockPreviewTable.vue            ← 数据预览表格
    ├── MockAdvancedDrawer.vue          ← 高级配置抽屉（⚙，从右侧滑出）
    ├── MockImportSchemaDialog.vue      ← 从数据库导入结构弹窗
    ├── MockTemplateSelectDialog.vue    ← 场景模板选择弹窗
    ├── MockHistoryTab.vue              ← 生成历史 Tab
    ├── MockSaveAsDropdown.vue          ← "另存为"下拉菜单
    └── MockGeneratorToolbar.vue        ← 工具栏入口按钮

src/composables/
└── useMockGenerator.ts                 ← 业务逻辑 Hook

src/stores/modules/
└── useMockStore.ts                     ← Pinia 状态管理

src/api/
└── mock-api.ts                         ← Tauri invoke 封装

src/types/
└── mock.ts                             ← TypeScript 类型定义
```

### 5.2 MockGeneratorPanel.vue — 主面板布局

```vue
<template>
  <div class="mock-generator-panel">
    <!-- 表元信息区 -->
    <div class="mock-meta">
      <n-input v-model:value="config.tableName" :placeholder="$t('mock.tableName')" />
      <n-input-number v-model:value="config.rowCount" :min="1" />
      <n-input-number v-model:value="config.seed" :allow-null="true" :placeholder="$t('mock.seed')" />
      <n-select v-model:value="config.locale" :options="localeOptions" />
    </div>

    <!-- 字段定义表格 -->
    <MockFieldTable
      :columns="config.columns"
      @add="addColumn"
      @remove="removeColumn"
      @map="handleAutoMap"
      @open-advanced="openAdvancedDrawer"
    />

    <!-- 数据预览 -->
    <MockPreviewTable
      :data="previewData"
      :loading="isGenerating"
      @refresh="refreshPreview"
    />

    <!-- 生成历史 Tab -->
    <n-tabs>
      <n-tab-pane :tab="$t('mock.configTab')" />
      <n-tab-pane :tab="$t('mock.historyTab')">
        <MockHistoryTab
          :history="history"
          @re-generate="handleReGenerate"
        />
      </n-tab-pane>
    </n-tabs>

    <!-- 底部操作栏 -->
    <div class="mock-footer">
      <n-button @click="cancel">{{ $t('common.cancel') }}</n-button>
      <n-button @click="saveToScratchpad">{{ $t('mock.save') }}</n-button>
      <MockSaveAsDropdown @select="handleSaveAs" />
      <n-button type="primary" @click="generate">{{ $t('mock.generate', { count: config.rowCount }) }}</n-button>
    </div>
  </div>
</template>
```

### 5.3 useMockGenerator.ts — 业务逻辑 Hook

```typescript
// composables/useMockGenerator.ts
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { MockConfig, MockGenerateResult, ColumnDef, MockHistoryRecord } from '@/types/mock'

export function useMockGenerator() {
  const config = ref<MockConfig>(createDefaultConfig())
  const previewData = ref<Array<Record<string, unknown>>>([])
  const isGenerating = ref(false)
  const history = ref<MockHistoryRecord[]>([])
  const currentTempTable = ref<string | null>(null)
  const elapsedMs = ref(0)

  async function generate() {
    isGenerating.value = true
    try {
      const result = await invoke<MockGenerateResult>('mock_generate', {
        config: config.value,
      })
      previewData.value = result.preview
      currentTempTable.value = result.tempTableName
      elapsedMs.value = result.elapsedMs
      await loadHistory()
    } finally {
      isGenerating.value = false
    }
  }

  async function saveToScratchpad() {
    if (!currentTempTable.value) return
    await invoke('mock_save_to_scratchpad', {
      tempTableName: currentTempTable.value,
      tableName: config.value.tableName,
    })
    // Toast: "已保存到草稿箱"
  }

  async function handleSaveAs(format: ExportFormat) {
    if (!currentTempTable.value) return
    if (format === 'table') {
      await invoke('mock_export', {
        tempTableName: currentTempTable.value,
        format: 'table',
      })
      // 切换视图到分析资源管理器
    } else {
      const filePath = await pickSavePath(format)
      await invoke('mock_export', {
        tempTableName: currentTempTable.value,
        format,
        outputPath: filePath,
      })
    }
  }

  async function importSchemaFromCache(connId: string, db: string, schema: string | null, tables: string[]) {
    const columns = await invoke<ColumnDef[]>('mock_import_schema', {
      connId, database: db, schema, tables,
    })
    config.value.columns = columns
  }

  async function autoMapColumns() {
    for (const col of config.value.columns) {
      const mapped = await invoke<ColumnMappingResponse>('mock_map_column', {
        columnName: col.name,
        dataType: col.dataType,
      })
      col.generator = mapped.generator
      col._confidence = mapped.confidence
      col._sampleValue = mapped.sampleValue
    }
  }

  // ... 更多方法

  return {
    config,
    previewData,
    isGenerating,
    history,
    elapsedMs,
    generate,
    saveToScratchpad,
    handleSaveAs,
    importSchemaFromCache,
    autoMapColumns,
    loadTemplates,
    applyTemplate,
    loadHistory,
    cancel,
  }
}
```

### 5.4 面板注册（extension.ts 扩展）

```typescript
// src/extensions/builtin/workbench/extension.ts 中新增

import MockGeneratorPanel from './ui/components/panels/mock/MockGeneratorPanel.vue'

// activate() 中新增
const mockPanelDisposable = context.window.registerViewProvider('mockGenerator', {
  component: MockGeneratorPanel,
  title: 'Mock 数据生成器',
  location: 'center',      // dockview 中心区域
  icon: 'Dices',           // lucide 图标
  order: 20,
})

disposables.push(mockPanelDisposable)
```

### 5.5 入口触发（工具栏 + 草稿箱右键）

```typescript
// 工具栏按钮
context.commands.registerCommand('mock.openPanel', () => {
  context.window.openView('mockGenerator')
})

// 草稿箱右键菜单 → "新建Mock数据"
context.commands.registerCommand('mock.openFromScratchpad', () => {
  context.window.openView('mockGenerator', {
    source: 'scratchpad',
  })
})
```

---

## 6. 智能列名映射机制

### 6.1 映射规则优先级

```
1. 精确匹配（🟢 High）     → 列名完全等于规则关键词
                             email → SafeEmail, id → AutoIncrement

2. 子串匹配（🟢 High）     → 列名包含关键词 + 后缀
                             user_id → AutoIncrement, created_at → DateTime

3. 模糊匹配（🟡 Medium）   → 列名包含部分关键词
                             info → Words(3..5), data → RandomText

4. 类型推断（⚪ Low）       → 仅根据数据类型推断
                             VARCHAR → Words(1..3), INTEGER → RandomInt(0..1000)
```

### 6.2 高置信度规则表（首批 ~45 条规则，利用 fake v5.1 新功能）

| 列名模式 | 数据类型 | 生成器 | 示例值 | v5.1 新增? |
|---------|---------|--------|--------|-----------|
| `id`, `*_id`, `*_key` | INTEGER | AutoIncrement | 1, 2, 3... | -- |
| `uuid`, `guid` | VARCHAR | **UuidV4** | 550e8400-... | ✅ 多版本可选 |
| `uuid_v1` | VARCHAR | **UuidV1** | 基于时间的 UUID | 🆕 |
| `ulid`, `sortable_id` | VARCHAR | **FerroidULID** | 01ARZ3NDEK... | 🆕 |
| `twitter_id`, `x_id` | BIGINT | **FerroidTwitterId** | 18273645... | 🆕 |
| `discord_id` | BIGINT | **FerroidDiscordId** | 1234567890... | 🆕 |
| `email`, `*_email` | VARCHAR | SafeEmail | zhangsan@mail.com | -- |
| `name`, `*_name` | VARCHAR | Name | 张三 | -- |
| `full_name` | VARCHAR | **NameWithTitle** | Dr. 张三 | 🆕 |
| `title`, `honorific` | VARCHAR | **Title** | Mr. | 🆕 |
| `username`, `login` | VARCHAR | Username | john_doe | -- |
| `password`, `pwd`, `pass` | VARCHAR | Password {min:8, max:20} | ******** | ✅ 可配长度 |
| `phone`, `mobile`, `tel` | VARCHAR | PhoneNumber | 138-xxxx-xxxx | -- |
| `cell`, `cellphone` | VARCHAR | **CellNumber** | 139xxxxxxxx | 🆕 |
| `address`, `addr` | VARCHAR | StreetName + BuildingNumber | 北京市朝阳区... | ✅ 更精细 |
| `street` | VARCHAR | **StreetName** | 长安街 | 🆕 |
| `city` | VARCHAR | City | 北京市 | -- |
| `country`, `nation` | VARCHAR | **CountryName** | 中华人民共和国 | 🆕 |
| `country_code` | VARCHAR(2) | **CountryCode** | CN | 🆕 |
| `state`, `province` | VARCHAR | **StateName** | 广东省 | 🆕 |
| `postcode` | VARCHAR | **PostCode** | 100000 | 🆕 |
| `zipcode`, `zip` | VARCHAR | ZipCode | 100000 | -- |
| `latitude`, `lat` | FLOAT | **Latitude** | 39.9042 | 🆕 |
| `longitude`, `lng`, `lon` | FLOAT | **Longitude** | 116.4074 | 🆕 |
| `geohash` | VARCHAR | **Geohash** {precision:6} | wx4g0b | 🆕 |
| `timezone` | VARCHAR | **TimeZone** | Asia/Shanghai | 🆕 |
| `mac`, `mac_address` | VARCHAR | **MacAddress** | 00:1A:2B:3C:4D:5E | 🆕 |
| `url`, `link`, `href` | VARCHAR | Url | https://... | -- |
| `ip`, `ip_address` | VARCHAR | IpAddress | 192.168.1.1 | -- |
| `user_agent`, `ua` | VARCHAR | UserAgent | Mozilla/5.0... | -- |
| `mime`, `mime_type` | VARCHAR | MimeType | application/json | -- |
| `semver`, `version` | VARCHAR | **Semver** | 1.2.3 | 🆕 |
| `file_path` | VARCHAR | **FilePath** | /usr/local/bin/app | 🆕 |
| `file_name` | VARCHAR | **FileName** | document.pdf | 🆕 |
| `extension` | VARCHAR | **FileExtension** | .pdf | 🆕 |
| `color`, `hex` | VARCHAR | **HexColor** | #ff5733 | 🆕 |
| `rgb` | VARCHAR | **RgbColor** | rgb(255,87,51) | 🆕 |
| `company`, `corp` | VARCHAR | CompanyName | XX科技有限公司 | -- |
| `job`, `job_title` | VARCHAR | JobTitle | 高级工程师 | ✅ |
| `profession` | VARCHAR | **Profession** | 软件工程师 | 🆕 |
| `industry` | VARCHAR | **Industry** | 信息技术 | 🆕 |
| `bic`, `swift` | VARCHAR | **Bic** | BKCHCNBJ | 🆕 |
| `isin` | VARCHAR | **Isin** | US0378331005 | 🆕 |
| `isbn` | VARCHAR | **Isbn** | 978-3-16-148410-0 | 🆕 |
| `credit_card`, `card_no` | VARCHAR | CreditCardNumber | 4539-xxxx-xxxx-xxxx | -- |
| `amount`, `price`, `fee`, `cost` | DECIMAL | RandomDecimal(0.01..99999.99) | 1234.56 | -- |
| `quantity`, `qty`, `count` | INTEGER | RandomInt(1..1000) | 42 | -- |
| `is_active`, `enabled`, `flag` | BOOLEAN | **Boolean** {ratio:50} | true/false | 🆕 |
| `ratio`, `percentage` | FLOAT | RandomFloat(0..100, 2) | 67.89 | ✅ |
| `status`, `state` | VARCHAR | ForeignKey(["active","pending",...]) | active | -- |
| `type`, `category`, `kind` | VARCHAR | ForeignKey(枚举值) | annual | ✅ |
| `created_at`, `create_time` | DATETIME | **DateTimeBetween** | 2024-01-15 08:30:00 | ✅ 更语义化 |
| `updated_at`, `update_time` | DATETIME | **DateTimeBefore** | 2024-01-15 12:45:00 | 🆕 |
| `birth_date`, `dob` | DATE | Date(1950-01-01..2005-12-31) | 1990-05-20 | ✅ |
| `description`, `desc`, `body` | TEXT | Paragraph {count:1} | Lorem ipsum... | ✅ 更合理 |
| `note`, `remark`, `memo` | VARCHAR | Words(3..8) | 这是一条备注信息 | -- |
| `first_name` | VARCHAR | FirstName | 张 | -- |
| `last_name` | VARCHAR | LastName | 三 | -- |
| `age` | INTEGER | RandomInt(18..80) | 35 | -- |
| `gender`, `sex` | VARCHAR | ForeignKey(["男","女"]) | 男 | -- |
| `plate`, `license_plate` | VARCHAR | **LicencePlate** | 京A12345 | 🆕 |
| `insurance`, `health_id` | VARCHAR | **HealthInsuranceCode** | 123456789012 | 🆕 |
| `markdown`, `md_content` | TEXT | **MarkdownBlockQuoteMulti** | > quote... | 🆕 |
| `http_status`, `status_code` | INTEGER | **ValidStatusCode** | 200, 404, 500 | 🆕 |

---

## 7. 场景模板设计

### 7.1 模板格式（JSON）

```json
{
  "id": "ecommerce",
  "name": "电商订单",
  "description": "包含用户、商品、订单三张关联表",
  "category": "e_commerce",
  "locale": "zh_cn",
  "tables": [
    {
      "name": "users",
      "rowCount": 1000,
      "columns": [
        { "name": "id", "dataType": "bigint", "generator": { "type": "auto_increment" } },
        { "name": "name", "dataType": "varchar", "generator": { "type": "name" } },
        { "name": "email", "dataType": "varchar", "generator": { "type": "safe_email" } },
        { "name": "phone", "dataType": "varchar", "generator": { "type": "phone_number" } },
        { "name": "address", "dataType": "text", "generator": { "type": "street_address" } },
        { "name": "created_at", "dataType": "datetime", "generator": { "type": "datetime", "params": { "min": "2023-01-01", "max": "2024-12-31" } } }
      ]
    },
    {
      "name": "products",
      "rowCount": 200,
      "columns": [
        { "name": "id", "dataType": "bigint", "generator": { "type": "auto_increment" } },
        { "name": "name", "dataType": "varchar", "generator": { "type": "words", "params": { "min": 2, "max": 4 } } },
        { "name": "price", "dataType": "decimal", "generator": { "type": "random_decimal", "params": { "min": 9.99, "max": 9999.99, "scale": 2 } } },
        { "name": "stock", "dataType": "integer", "generator": { "type": "random_int", "params": { "min": 0, "max": 10000 } } }
      ]
    },
    {
      "name": "orders",
      "rowCount": 10000,
      "columns": [
        { "name": "id", "dataType": "bigint", "generator": { "type": "auto_increment" } },
        { "name": "user_id", "dataType": "bigint", "generator": { "type": "random_int", "params": { "min": 1, "max": 1000 } } },
        { "name": "product_id", "dataType": "bigint", "generator": { "type": "random_int", "params": { "min": 1, "max": 200 } } },
        { "name": "amount", "dataType": "decimal", "generator": { "type": "random_decimal", "params": { "min": 9.99, "max": 99999.99, "scale": 2 } } },
        { "name": "quantity", "dataType": "integer", "generator": { "type": "random_int", "params": { "min": 1, "max": 100 } } },
        { "name": "status", "dataType": "varchar", "generator": { "type": "foreign_key", "params": { "values": ["pending","paid","shipped","delivered","cancelled"] } } },
        { "name": "created_at", "dataType": "datetime", "generator": { "type": "datetime", "params": { "min": "2024-01-01", "max": "2025-05-31" } } }
      ]
    }
  ]
}
```

### 7.2 模板加载机制

- 编译时：`include_dir!` 宏将 `resources/templates/` 嵌入二进制
- 运行时：通过 `TemplateManager::list()` 读取
- 扩展：支持用户自定义模板存储在项目 `.scratchpad/mock/templates/` 下

---

## 8. 分阶段开发计划

### 开发进度总览

```
Phase 0: 技术准备          ✅ 已完成 (2026-05-08)
Phase 1: 核心引擎 MVP       ✅ 已完成 (2026-05-08)  
Phase 2: Tauri Command      ✅ 已完成 (2026-05-08)
Phase 3: 前端类型 & API     ✅ 已完成 (2026-05-08)
Phase 4: 数据库导入结构     ✅ 已完成（后端）
Phase 5: 场景模板           ✅ 已完成
Phase 6: 持久化 & 资源管理器 ✅ 已完成
Phase 7: 生成历史 & 边界场景   ✅ 已完成
Phase 8: 前端主面板 & Store      ✅ 已完成
Phase 9: 前端子功能             ⏳ 待开发
```

### Phase 0：技术准备 ✅

| 任务 | 说明 | 状态 |
|------|------|------|
| 0.1 添加 `fake` crate 依赖 | `Cargo.toml` 添加 `fake = { version = "5", features = [14个] }` | ✅ |
| 0.2 Cargo check 通过 | 修复 50+ 编译错误，适配 fake v5.1 API 变更 | ✅ |
| 0.3 TypeScript 类型定义 | 创建 `src/shared/api/mock-api.ts`，定义前端类型 + API | ✅ |
| 0.4 DuckDB 集成基础 | `engine.rs` 中的批量 INSERT + 预览 + 导出 | ✅ |

### Phase 1：核心引擎 MVP ✅

| 任务 | 说明 |
|------|------|
| 0.1 添加 `fake` crate 依赖 | `Cargo.toml` 添加 `fake = { version = "2.9", features = ["derive", "chrono", "uuid", "http"] }` |
| 0.2 添加 CSV/Parquet 导出依赖 | 如果需要独立于 DuckDB 的导出（`csv` crate, `parquet` crate, `calamine` for Excel） |
| 0.3 TypeScript 类型定义 | 创建 `src/types/mock.ts`，定义所有前端类型接口 |
| 0.4 DuckDB `temp_mock_` 前缀支持 | 扩展现有 `DuckDbService`，支持 `temp_mock_` 前缀和批量写入 |

### Phase 1：核心引擎 MVP ✅

| 任务 | 优先级 | 产出 | 状态 |
|------|--------|------|------|
| 1.1 `core/mock/models.rs` | 🔴 高 | 数据模型定义（106 GeneratorConfig 变体、13 种 Locale、13 种 ColumnDataType） | ✅ |
| 1.2 `core/mock/error.rs` | 🔴 高 | MockError 错误类型（含 From<CoreError/DuckDB/PoisonError>） | ✅ |
| 1.3 `core/mock/engine.rs` | 🔴 高 | 核心生成引擎（~800行，批量 INSERT + 预览 + 5种导出模式） | ✅ |
| 1.4 `core/mock/schema_map.rs` | 🟡 中 | 列名智能映射（~80条规则，三级置信度，覆盖 106 种 GeneratorConfig） | ✅ |
| 1.5 `core/mock/mod.rs` | 🔴 高 | 模块入口，pub use re-export | ✅ |

### Phase 2：Tauri Command ✅

| 任务 | 优先级 | 产出 | 状态 |
|------|--------|------|------|
| 2.1 `commands/mock_commands.rs` | 🔴 高 | 13 个命令：mock_generate, mock_preview, mock_export, mock_map_column, mock_map_columns_batch, mock_list_templates, mock_import_schema, mock_apply_template, mock_save_to_scratchpad, mock_persist_as_asset, mock_get_history, mock_clear_history, mock_re_generate | ✅ |
| 2.2 `commands/mod.rs` + `lib.rs` 注册 | 🔴 高 | 命令注册到 `generate_handler!` | ✅ |

### Phase 3：前端类型 & API ✅

| 任务 | 优先级 | 产出 | 状态 |
|------|--------|------|------|
| 3.1 `shared/api/mock-api.ts` | 🔴 高 | TypeScript 类型定义 + `mockApi` invoke 封装（GeneratorType 100% 对齐 106 后端变体） | ✅ |

### Phase 4：从数据库导入结构 ✅

| 任务 | 优先级 | 产出 | 状态 |
|------|--------|------|------|
| 4.1 `mock_import_schema` 命令 | 🟡 中 | 从 MetadataCache 读取表/列结构 + ColumnMapper 推断生成器 | ✅ |
| 4.2 `MockImportSchemaDialog.vue` | 🟡 中 | 数据源→Schema→表选择弹窗 | ⏳ 前端 |
| 4.3 结构自动填充 + 智能映射 | 🟡 中 | 导入后自动触发 `mock_map_columns_batch` | ⏳ 前端 |

### Phase 5：场景模板 ✅

| 任务 | 优先级 | 产出 | 状态 |
|------|--------|------|------|
| 5.1 4个内置模板 | 🟢 低 | `templates.rs` 内置 4 个场景（电商/HR/博客/金融） | ✅ |
| 5.2 `mock_apply_template` 命令 | 🟢 低 | 应用模板 | ✅ |
| 5.3 `MockTemplateSelectDialog.vue` | 🟢 低 | 模板选择弹窗 | ⏳ 前端 |
| 5.4 工具栏/右键入口 | 🟢 低 | "新建Mock表"、"从数据库导入结构"、"从场景模板创建" | ⏳ 前端 |

### Phase 6：持久化 & 资源管理器集成 ✅

> **核心规则：保存目标由导出格式决定**

| 导出格式 | 保存目标 | 后端命令 | 生命周期 |
|----------|----------|----------|----------|
| `Table` | **分析资源管理器** | `mock_persist_as_asset` → DuckDB 持久表 + 返回元数据供前端注册 | 项目级，跨会话 |
| `CSV` | **草稿箱** | `mock_save_to_scratchpad` → `.scratchpad/mock/xxx.csv` | 会话级 |
| `Parquet` | **草稿箱** | `mock_save_to_scratchpad` → `.scratchpad/mock/xxx.parquet` | 会话级 |
| `XLSX` | **草稿箱** | `mock_save_to_scratchpad` → `.scratchpad/mock/xxx.xlsx` | 会话级 |
| `SQL Insert` | **草稿箱** | `mock_save_to_scratchpad` → `.scratchpad/mock/xxx.sql` | 会话级 |

| 任务 | 优先级 | 产出 | 状态 |
|------|--------|------|------|
| 6.1 `mock_save_to_scratchpad` 命令 | 🟡 中 | 一键保存到 `.scratchpad/mock/`，自动生成含时间戳的文件名 | ✅ |
| 6.2 `mock_persist_as_asset` 命令 | 🟡 中 | 从 TEMP TABLE 创建持久 DuckDB 表，返回 `(tableName, rowCount, colCount)` | ✅ |
| 6.3 草稿箱 `Mock数据` 分组 | 🟡 中 | 草稿箱资源树中显示 `mock/` 分组节点 | ⏳ 前端 |
| 6.4 分析资源管理器注册 | 🟢 低 | 前端调用 `create_analytics_resource` 将持久表注册为资产 | ⏳ 前端 |
| 6.5 自动清理 | 🟢 低 | `temp_mock_` 表生命周期管理 | ⏳ |

### Phase 7：生成历史 & 边界场景 ✅

| 任务 | 优先级 | 产出 | 状态 |
|------|--------|------|------|
| 7.1 `core/mock/history.rs` | 🔴 高 | DuckDB `_system.mock_history` 表（自动建表、保存、查询、清理、LRU 200条上限） | ✅ |
| 7.2 `mock_get_history` 命令 | 🟡 中 | 查询生成历史记录列表（支持 limit 参数） | ✅ |
| 7.3 `mock_clear_history` 命令 | 🟡 中 | 清空所有生成历史 | ✅ |
| 7.4 `mock_re_generate` 命令 | 🟡 中 | 根据历史记录 ID 反序列化 `config_json` → `MockConfig`，重新调用 `generate()` | ✅ |
| 7.5 自动保存历史 | 🟡 中 | `engine.rs` generate() 成功后自动调用 `MockHistoryStore::save()` | ✅ |
| 7.6 `nullable_ratio` 支持 | 🟡 中 | INSERT 时随机按 nullable_ratio 概率将值设为 NULL | ✅ |
| 7.7 前端 API | 🟡 中 | `mockApi.getHistory()`, `mockApi.clearHistory()`, `mockApi.reGenerate()` | ✅ |
| 7.8 `MockHistoryTab.vue` | 🟢 低 | 历史记录 Tab | ⏳ 前端 |
| 7.9 错误处理与加载状态 | 🟢 低 | 全局错误提示 + Loading | ⏳ 前端 |

**总预估：16-24 天（约 3-5 周）**

---

## 9. 技术依赖清单

### 9.1 新增 Rust Crate（需添加到 Cargo.toml）

```toml
[dependencies]
# ⚡ 数据生成引擎（v5.1.0, 2026-03-16 发布）
# 注：fake v5 内部使用 rand 0.10（fake::rand 重导出），与项目 rand 0.8 不冲突
fake = { version = "5", features = [
    "derive",         # ✅ #[derive(Dummy)] 派生宏
    "chrono",         # ✅ 日期时间：DateTime/Date/Time/Duration/DateTimeBetween
    "uuid",           # ✅ UUID v1/v3/v4/v5
    "http",           # ✅ HTTP 状态码：RfcStatusCode/ValidStatusCode
    "ferroid",        # 🆕 Ferroid ID：ULID/TwitterId/InstagramId/MastodonId/DiscordId
    "ulid",           # 🆕 ULID 基础类型（ferroid 依赖）
    "semver",         # 🆕 语义版本号：Semver/SemverStable/SemverUnstable
    "random_color",   # 🆕 颜色生成：Hex/RGB/RGBA/HSL/HSLA
    "geo",            # 🆕 地理坐标：Latitude/Longitude/Geohash
    "url",            # 🆕 URL 类型（Url 生成器依赖）
    "serde_json",     # 🆕 JSON 序列化（预览数据通过 IPC 传递）
    "bigdecimal",     # ⚡ BigDecimal 精确小数（金融场景）
    "rust_decimal",   # ⚡ rust_decimal 精确小数（Decimal 列类型）
] }
# either 是 default feature，无需显式声明

# -- 以下依赖已存在于 Cargo.toml --
# duckdb = "1.10502.0"     ✅ 已有（Appender API 批量写入）
# arrow = "58.1.0"          ✅ 已有（RecordBatch 构建）
# rand = "0.8"              ✅ 已有（项目级 RNG，独立于 fake::rand 0.10）
# chrono = "0.4.41"         ✅ 已有（DateTime 类型）
# uuid = "1.16.0"           ✅ 已有（fake 的 uuid feature 兼容）
# serde = "1.0.219"         ✅ 已有
# serde_json = "1.0.135"    ✅ 已有
# thiserror = "1.0.69"      ✅ 已有
```

### Feature → 生成器对照表（13 个 feature → 60+ 生成器）

| Feature | 影响的生成器 (常用列举) | 缺失后果 |
|---------|----------------------|---------|
| `derive` | `#[derive(Dummy)]` 宏，简化结构体 | struct Foo {name, email} 一键生成 |
| `chrono` | `DateTime`, `DateTimeBetween`, `DateTimeBefore`, `DateTimeAfter`, `Date`, `Time`, `Duration` | 所有日期时间列无 fake 能力 |
| `uuid` | `UuidV1`, `UuidV3`, `UuidV4`, `UuidV5` | UUID 列降级为随机字符串 |
| `http` | `RfcStatusCode`, `ValidStatusCode`, `UserAgent` | HTTP 列无 fake 能力 |
| `ferroid` | `FerroidULID`, `FerroidTwitterId`, `FerroidInstagramId`, `FerroidMastodonId`, `FerroidDiscordId` | ID 列全部失效 |
| `ulid` | `FerroidULID` (内部依赖) | ULID 列无 fake 能力 |
| `semver` | `Semver`, `SemverStable`, `SemverUnstable` | 版本号列降级为随机字符串 |
| `random_color` | `HexColor`, `RgbColor`, `RgbaColor`, `HslColor`, `HslaColor`, `Color` | 颜色列全部失效 |
| `geo` | `Latitude`, `Longitude`, `Geohash` | 地理坐标列降级为随机 float |
| `url` | `Url` | URL 列降级为随机字符串 |
| `serde_json` | 数据序列化传递（IPC 懒复制到前端预览） | JSON 预览传递类型不安全 |
| `bigdecimal` | `BigDecimal`, `PositiveBigDecimal`, `NegativeBigDecimal`, `NoBigDecimalPoints` | 金融精度丢失 |
| `rust_decimal` | `Decimal`, `PositiveDecimal`, `NegativeDecimal`, `NoDecimalPoints` | 小数精度丢失 |

> ⚠️ **rand 版本共存说明**：
> - 项目直接依赖 `rand = "0.8"`（用于数据库键值等业务 RNG）
> - `fake v5.1` 内部使用 `rand 0.10`，通过 `fake::rand` 模块重导出
> - Mock 引擎应使用 `fake::rand::rngs::StdRng` 和 `fake::rand::SeedableRng`
> - 两个 rand major 版本不冲突（Cargo 将它们视为不同 crate）
> - 未来如果 `rdata-station` 其余模块也升级到 `rand 0.10`，可统一为单版本

**MSRV 变化**：`fake v5.1` 要求 Rust ≥ 1.63，项目当前远高于此，无影响。

### 9.2 前端无新增依赖

- naïve-ui：✅ 已有（NButton, NInput, NSelect, NDrawer, NTabs, NModal 等）
- AG Grid：✅ 已有（预览表格、字段编辑表格）
- dockview-vue：✅ 已有（面板注册与管理）
- lucide-vue-next：✅ 已有（图标 Dices / Database / FileDown 等）

### 9.3 前端实现 ✅

#### 9.3.1 Pinia Store — `useMockStore`

| 文件 | 路径 |
|------|------|
| Store | `src/stores/useMockStore.ts` |

**状态字段**：
- `tableName`, `rowCount`, `seed`, `locale` — 生成配置
- `columns: ColumnDef[]` — 列定义列表，含 `generator.type` / `dataType`
- `generatedTableName` — 生成后的临时表名
- `previewData` — 预览数据 `Array<unknown[]>`
- `previewLoading` / `generateLoading` — 加载状态

**操作**：
- `addColumn()` / `removeColumn(idx)` / `updateColumn(idx, patch)` / `setColumnType(idx, type)`
- `generate()` → `mockApi.generate(config)` → 自动缓存预览
- `doExport(format)` / `saveToScratchpad(format)` / `persistAsAsset(name)`
- `loadHistory()` / `clearHistory()` / `reGenerate(historyId)`

#### 9.3.2 主面板 — `MockPanel.vue`

| 文件 | 路径 |
|------|------|
| 面板 | `src/extensions/builtin/workbench/ui/components/panels/MockPanel.vue` |

**面板布局**：

```
┌─────────────────────────────────┐
│  Mock 数据生成器          🕐 历史│
├─────────────────────────────────┤
│  表名 [  ] 行数 [100] 种子[  ] │
│  地区 [ZH_CN ▼]                 │
├─────────────────────────────────┤
│  列配置                  + 添加列│
│  ┌────────┬──────┬────────┬──┐ │
│  │id      │INTEGER│auto_inc│🗑│ │
│  │username│VARCHAR│username│🗑│ │
│  │email   │VARCHAR│email   │🗑│ │
│  └────────┴──────┴────────┴──┘ │
├─────────────────────────────────┤
│  ▶ 生成 100 行   已生成 100 行  │
├─────────────────────────────────┤
│  预览 (前 50 行)   📥 导出/保存 │
│  ┌────┬────────┬──────┬────────│
│  │ id │username│email │created │
│  │  1 │alice   │a@x.. │2024..  │
│  └────┴────────┴──────┴────────│
├─────────────────────────────────┤
│  生成历史                 🗑 清空│
│  mock_users 100行  完成了 05/09│
└─────────────────────────────────┘
```

**功能覆盖**：
- ✅ 39 种常用生成器可选（通过 `NSelect` 搜索）
- ✅ 列管理（添加/删除/重命名/类型切换）
- ✅ 一键生成 + 自动预览
- ✅ 4 种导出格式（CSV/Parquet/XLSX/SQL INSERT）
- ✅ 保存到草稿箱（4 种格式）
- ✅ 持久化为分析资产
- ✅ 生成历史列表 + 重新生成
- ✅ 加载状态 + 错误提示

#### 9.3.3 面板注册

| 文件 | 变更 |
|------|------|
| `src/extensions/builtin/workbench/extension.ts` | 注册 `mockPanel` → 右侧面板（order=1，icon=Database） |

---

## 10. 不明确事项 & 待确认决策

| # | 事项 | 推荐方案 | 影响 |
|---|------|---------|------|
| 1 | **从数据库导入的元数据来源** | 优先使用**内存 MetadataCache**（`core::cache::metadata_cache`），缓存未命中时触发 `refresh_metadata_cache` 后再读。持久化 SQLite 作为兜底 | 如果在 Phase 4 发现缓存机制不完善，需要先完善缓存刷新 |
| 2 | **`temp_mock_` vs `rs_` 前缀** | Mock 专用 `temp_mock_` 前缀，使用独立的 `DuckDbService::create_mock_temp_table` 方法；清理逻辑已有 `cleanup_temp_table` 基础 | 需要扩展 `DuckDBManager` 支持按前缀批量清理 |
| 3 | **生成历史存储** | 项目级持久化，存储在项目 DuckDB 的 `_system.mock_history` 表中（跟随项目生命周期）。不放在内存中避免丢失 | Phase 7 实现时需要设计 SQLite schema |
| 4 | **场景模板格式** | JSON 配置文件，编译时 `include_dir!` 嵌入；支持用户自定义模板放在 `.scratchpad/mock/templates/` | 模板格式标准化，JSON Schema 校验 |
| 5 | **持久化操作实现** | `CREATE TABLE mock_xxx AS SELECT * FROM temp_mock_xxx; DROP TABLE temp_mock_xxx;`（保留原始数据，避免 RENAME 的元数据残留） | 与 DuckDB 现有 API 兼容 |
| 6 | **fake v5.1 生成器完整列表** | 已确认 **60+ 生成器**：Markdown(8种)、Ferroid ID(5种)、UUID(4种)、颜色(6种)、日期(6种)、金融(3种)、地址(15种)、文件系统(4种)、行政/汽车(2种)、Either组合器。首批实现全部 `GeneratorConfig` 枚举，但前端 UI 按分类折叠展示 | `GeneratorConfig` 枚举 60+ 变体，前端用分类 Tab 组织 |
| 7 | **高级配置抽屉的 fake v5.1 能力暴露程度** | 主力暴露 3 层：🔵 基础（列名/类型/生成器切换）、🟡 常用参数（日期范围、数值范围、Nullable比例、布尔比例）、🔴 高级（正则模式、NumberWithFormat、Geohash精度、Password长度、Either组合） | 三层渐进式 UI，简单模式只暴露基础层 |
| 8 | **\[已确认] fake v5.1 + rand 0.10 兼容性** | `fake::rand` (v0.10) 与项目 `rand 0.8` 不冲突。Mock 引擎统一使用 `fake::rand::StdRng`。按 crate 版本隔离，不需要修改项目其余部分 | Cargo.toml 只需加 `fake = "5"`，无需改现有 rand 依赖 |

---

## 附录

### A. 已调研的现有基建（可复用）

| 模块 | 路径 | 复用方式 |
|------|------|---------|
| DuckDbService | `core/services/duckdb_service.rs` | 复用 `create_temp_table_internal`、`export_temp_table` 模式 |
| DuckDBManager | `core/duckdb.rs` | 复用全局连接，新增 `register_mock_table` |
| ScratchpadStore | `core/scratchpad/store.rs` | 直接调用现有 CRUD（`create_entry`、`save_file`、`read_file`） |
| ScratchpadState | `core/scratchpad/state.rs` | 复用 state 管理模式 |
| MetadataCache | `core/cache/metadata_cache.rs` | 复用 `get_tables`、`get_columns` API |
| AnalyticsResource | `commands/analytics_resource_commands.rs` | 持久化表在分析资源管理器中展示 |
| extension.ts | `extensions/builtin/workbench/extension.ts` | 复用 `registerViewProvider` 模式 |

### B. 关键代码路径（Rust — 已实现文件）

```
新增 ✅:
  src-tauri/src/core/mock/mod.rs                ← 模块入口
  src-tauri/src/core/mock/models.rs             ← 数据模型（60+ 枚举变体）
  src-tauri/src/core/mock/error.rs              ← MockError（6 种错误变体）
  src-tauri/src/core/mock/engine.rs             ← 核心引擎（~850行）
  src-tauri/src/core/mock/schema_map.rs         ← 列名映射（~60条规则）
  src-tauri/src/commands/mock_commands.rs       ← Tauri 命令（6个）
  src-tauri/resources/templates/*.json          ← ⏳ 内置场景模板（Phase 5）

修改 ✅:
  src-tauri/Cargo.toml                          ← 添加 fake v5.1 + 14 features
  src-tauri/src/core/mod.rs                     ← 添加 pub mod mock
  src-tauri/src/commands/mod.rs                 ← 添加 pub mod mock_commands + pub use
  src-tauri/src/lib.rs                          ← generate_handler! 注册 + allow()
```

### C. 关键代码路径（前端 — 已实现文件）

```
新增 ✅:
  src/shared/api/mock-api.ts                    ← ✅ TS 类型定义 + mockApi invoke 封装
  src/stores/useMockStore.ts                    ← ✅ Pinia Store（Phase 8）
  src/extensions/builtin/workbench/extension.ts ← ✅ 面板注册 mockPanel（Phase 8）
  src/extensions/builtin/workbench/ui/components/panels/MockPanel.vue  ← ✅ 主面板 UI（Phase 8）

待开发 ⏳:
  src/composables/useMockGenerator.ts           ← ⏳ 业务 Hook（Phase 9）
  src/extensions/builtin/workbench/ui/components/panels/mock/*.vue ← ⏳ 子组件（Phase 9）
  src/types/mock.ts                             ← ⏳ 独立 TS 类型文件（Phase 9）
```

### D. 实际 Cargo.toml fake 依赖配置

```toml
# src-tauri/Cargo.toml（实际配置）
fake = { version = "5", features = [
    "derive",         # #[derive(Dummy)] 派生宏
    "chrono",         # 日期时间生成器（项目统一使用 chrono 模块）
    "uuid",           # UUIDv1/v3/v4/v5（fake::uuid::UUIDv4）
    "http",           # HTTP 状态码（ValidStatusCode）
    "ferroid",        # Ferroid ID（fake::ferroid::FerroidULID 等）
    "ulid",           # ULID 基础类型（ferroid 内部依赖）
    "semver",         # 语义版本号（filesystem::en::Semver）
    "random_color",   # 颜色生成器
    "geo",            # 地理坐标（Latitude/Longitude/Geohash）
    "url",            # URL 类型（实际无 Url 生成器，保留用于类型支持）
    "serde_json",     # JSON 序列化
    "bigdecimal",     # BigDecimal 精确小数
    "rust_decimal",   # rust_decimal 精确小数
    "time",           # 时间类型支持（chrono 生成器内部依赖）
] }
```
