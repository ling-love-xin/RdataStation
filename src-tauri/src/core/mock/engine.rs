use std::collections::HashSet;
use std::time::Instant;

use fake::rand::rngs::StdRng;
use fake::rand::SeedableRng;
use fake::{Fake, Faker};

use crate::core::duckdb::DuckDBManager;
use crate::core::mock::error::{MockError, MockResult};
use crate::core::mock::history::MockHistoryStore;
use crate::core::mock::models::{
    ColumnDef, ColumnDataType, ColumnMappingResponse, ImportSchemaInput, MockExportFormat,
    GeneratorConfig, Locale, MockConfig, MockGenerateResult, MockHistoryRecord, ScenarioTemplate,
};
use crate::core::mock::schema_map::ColumnMapper;
use crate::core::mock::templates;

pub struct MockEngine;

const TEMP_MOCK_PREFIX: &str = "temp_mock_";
const BATCH_SIZE: usize = 10_000;
const PREVIEW_ROWS: usize = 10;

impl MockEngine {
    // ==================== 数据生成 ====================

    pub async fn generate(config: MockConfig) -> MockResult<MockGenerateResult> {
        Self::generate_with_progress(config, |_, _| {}).await
    }

    pub async fn generate_with_progress<F>(
        config: MockConfig,
        on_progress: F,
    ) -> MockResult<MockGenerateResult>
    where
        F: Fn(usize, usize) + Send + 'static,
    {
        if config.row_count == 0 {
            return Err(MockError::InvalidRowCount(0));
        }
        if config.columns.is_empty() {
            return Err(MockError::InvalidColumn("无列定义".to_string()));
        }

        let start = Instant::now();

        let mut rng: StdRng = match config.seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::seed_from_u64(rand::random()),
        };

        let safe_name = sanitize_table_name(&config.table_name);
        let table_name = format!("{}{}", TEMP_MOCK_PREFIX, safe_name);

        let duckdb = DuckDBManager::global().get_or_create_in_memory()?;
        let conn = duckdb.lock().map_err(|e| {
            MockError::Generation(format!("DuckDB lock error: {}", e))
        })?;

        let ddl = Self::build_create_table_ddl(&table_name, &config.columns);
        conn.execute_batch(&format!("DROP TABLE IF EXISTS \"{}\"", table_name))?;
        conn.execute_batch(&ddl)?;

        let total_batches = (config.row_count + BATCH_SIZE - 1) / BATCH_SIZE;
        let mut unique_sets: Vec<HashSet<String>> = config
            .columns
            .iter()
            .map(|c| {
                if c.unique {
                    HashSet::with_capacity(config.row_count)
                } else {
                    HashSet::new()
                }
            })
            .collect();

        let quoted_cols: Vec<String> = config
            .columns
            .iter()
            .map(|c| format!("\"{}\"", c.name))
            .collect();
        let col_list = quoted_cols.join(", ");

        for batch_idx in 0..total_batches {
            let start_row = batch_idx * BATCH_SIZE;
            let count = std::cmp::min(BATCH_SIZE, config.row_count - start_row);

            let mut value_lines: Vec<String> = Vec::with_capacity(count);

            for row_idx in 0..count {
                let global_row = start_row + row_idx;
                let mut values = Vec::with_capacity(config.columns.len());

                for (col_idx, col) in config.columns.iter().enumerate() {
                    let mut attempts = 0;
                    let value = loop {
                        let val = Self::generate_cell(
                            &col.generator,
                            &mut rng,
                            global_row,
                            &config.locale,
                        );
                        if !col.unique || !unique_sets[col_idx].contains(&val) {
                            if col.unique {
                                unique_sets[col_idx].insert(val.clone());
                            }
                            break val;
                        }
                        attempts += 1;
                        if attempts > 100 {
                            return Err(MockError::Generation(format!(
                                "无法为唯一列'{}'生成不重复的值",
                                col.name
                            )));
                        }
                    };
                    values.push(value);
                }

                let escaped: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(ci, v)| {
                        let nullable_ratio = config.columns[ci].nullable_ratio;
                        if nullable_ratio > 0.0 {
                            let rand_val: f64 = (0.0..1.0).fake_with_rng(&mut rng);
                            if rand_val < nullable_ratio {
                                return "NULL".to_string();
                            }
                        }
                        format!("'{}'", v.replace('\'', "''"))
                    })
                    .collect();
                value_lines.push(format!("({})", escaped.join(", ")));
            }

            let insert_sql = format!(
                "INSERT INTO \"{}\" ({}) VALUES {}",
                table_name,
                col_list,
                value_lines.join(", ")
            );
            conn.execute_batch(&insert_sql)?;
            on_progress(batch_idx + 1, total_batches);
        }

        DuckDBManager::global().register_temp_table(&table_name);

        let preview = Self::read_preview(&conn, &table_name, PREVIEW_ROWS)?;
        let elapsed_ms = start.elapsed().as_millis() as u64;

        let _ = MockHistoryStore::save(&config, elapsed_ms);

        Ok(MockGenerateResult {
            table_name: config.table_name.clone(),
            temp_table_name: table_name,
            row_count: config.row_count,
            preview,
            columns: config.columns.iter().map(|c| c.name.clone()).collect(),
            elapsed_ms,
        })
    }

    // ==================== 预览刷新 ====================

    pub fn preview(
        temp_table_name: &str,
        limit: usize,
    ) -> MockResult<Vec<Vec<serde_json::Value>>> {
        let duckdb = DuckDBManager::global().get_or_create_in_memory()?;
        let conn = duckdb.lock().map_err(|e| {
            MockError::Generation(format!("DuckDB lock error: {}", e))
        })?;
        Self::read_preview(&conn, temp_table_name, limit)
    }

    // ==================== 导出 ====================

    pub fn export(
        temp_table_name: &str,
        format: &MockExportFormat,
        output_path: Option<&str>,
        table_name: Option<&str>,
    ) -> MockResult<String> {
        let duckdb = DuckDBManager::global().get_or_create_in_memory()?;
        let conn = duckdb.lock().map_err(|e| {
            MockError::Generation(format!("DuckDB lock error: {}", e))
        })?;

        match format {
            MockExportFormat::Csv | MockExportFormat::Parquet | MockExportFormat::Xlsx => {
                let ext = match format {
                    MockExportFormat::Csv => "CSV",
                    MockExportFormat::Parquet => "PARQUET",
                    MockExportFormat::Xlsx => "XLSX",
                    _ => unreachable!(),
                };
                let path = output_path.ok_or_else(|| {
                    MockError::Config(format!("{} export requires output_path", ext))
                })?;
                let sql = format!(
                    "COPY \"{}\" TO '{}' (FORMAT {}, HEADER)",
                    temp_table_name,
                    path.replace('\\', "/"),
                    ext
                );
                conn.execute_batch(&sql)?;
                Ok(format!("Exported to {}: {}", ext, path))
            }
            MockExportFormat::Table => {
                let name = table_name.unwrap_or(temp_table_name);
                let new_name = name.trim_start_matches(TEMP_MOCK_PREFIX);
                let sql = format!(
                    "CREATE TABLE \"{}\" AS SELECT * FROM \"{}\"",
                    new_name, temp_table_name
                );
                conn.execute_batch(&sql)?;
                let drop_sql = format!("DROP TABLE IF EXISTS \"{}\"", temp_table_name);
                conn.execute_batch(&drop_sql)?;
                Ok(format!("Persisted as table: {}", new_name))
            }
            MockExportFormat::SqlInsert => {
                let path = output_path.ok_or_else(|| {
                    MockError::Config("SQL INSERT export requires output_path".to_string())
                })?;
                let mut stmt =
                    conn.prepare(&format!("SELECT * FROM \"{}\"", temp_table_name))?;
                let columns: Vec<String> =
                    stmt.column_names().iter().map(|c| format!("\"{}\"", c)).collect();
                let col_list = columns.join(", ");

                let mut rows = stmt.query([])?;
                let mut insert_statements = Vec::new();

                while let Some(row) = rows.next()? {
                    let values: Vec<String> = (0..columns.len())
                        .map(|i| {
                            let val: duckdb::types::Value =
                                row.get(i).unwrap_or(duckdb::types::Value::Null);
                            value_to_sql_literal(&val)
                        })
                        .collect();
                    let target_name = table_name
                        .unwrap_or(temp_table_name)
                        .trim_start_matches(TEMP_MOCK_PREFIX);
                    insert_statements.push(format!(
                        "INSERT INTO \"{}\" ({}) VALUES ({});",
                        target_name,
                        col_list,
                        values.join(", ")
                    ));
                }
                std::fs::write(path, insert_statements.join("\n")).map_err(|e| {
                    MockError::Export {
                        format: "SQL INSERT".to_string(),
                        reason: format!("Write file failed: {}", e),
                    }
                })?;
                Ok(format!("Exported to SQL INSERT: {}", path))
            }
        }
    }

    // ==================== 列名智能映射 ====================

    pub fn map_column(column_name: &str, data_type: &str) -> MockResult<ColumnMappingResponse> {
        let dt = parse_data_type(data_type);
        Ok(ColumnMapper::infer(column_name, &dt))
    }

    pub fn map_columns_batch(
        columns: Vec<(String, String)>,
    ) -> MockResult<Vec<ColumnMappingResponse>> {
        columns
            .into_iter()
            .map(|(name, dt)| Self::map_column(&name, &dt))
            .collect()
    }

    // ==================== 场景模板 ====================

    pub fn list_templates() -> MockResult<Vec<ScenarioTemplate>> {
        Ok(templates::get_builtin_templates())
    }

    pub fn apply_template(template_id: &str) -> MockResult<ScenarioTemplate> {
        templates::get_template_by_id(template_id)
            .ok_or_else(|| MockError::Config(format!("Template not found: {}", template_id)))
    }

    // ==================== 私有方法 ====================

    fn build_create_table_ddl(table_name: &str, columns: &[ColumnDef]) -> String {
        let col_defs: Vec<String> = columns
            .iter()
            .map(|c| format!("\"{}\" {}", c.name, c.data_type.to_duckdb_type()))
            .collect();
        format!("CREATE TABLE \"{}\" ({})", table_name, col_defs.join(", "))
    }

    fn generate_cell(
        generator: &GeneratorConfig,
        rng: &mut StdRng,
        row_index: usize,
        _locale: &Locale,
    ) -> String {
        match generator {
            // ========== 数值类 ==========
            GeneratorConfig::AutoIncrement { start, step } => {
                (*start + (row_index as i64) * *step).to_string()
            }
            GeneratorConfig::RandomInt { min, max } => {
                (*min..=*max).fake_with_rng::<i64, _>(rng).to_string()
            }
            GeneratorConfig::RandomFloat {
                min,
                max,
                precision,
            } => {
                let val: f64 = (*min..*max).fake_with_rng(rng);
                format!("{:.prec$}", val, prec = *precision as usize)
            }
            GeneratorConfig::RandomDecimal { min, max, scale } => {
                let val: f64 = (*min..*max).fake_with_rng(rng);
                format!("{:.scl$}", val, scl = *scale as usize)
            }
            GeneratorConfig::Digit => {
                use fake::faker::number::en::Digit;
                Digit().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::NumberWithFormat { fmt } => {
                use fake::faker::number::en::NumberWithFormat;
                NumberWithFormat(fmt).fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Boolean { ratio } => {
                use fake::faker::boolean::en::Boolean;
                Boolean(*ratio).fake_with_rng::<bool, _>(rng).to_string()
            }

            // ========== 文本类 ==========
            GeneratorConfig::Constant { value } => value.clone(),
            GeneratorConfig::Words { min, max } => {
                use fake::faker::lorem::en::Words;
                let words: Vec<String> = Words(*min..*max).fake_with_rng(rng);
                words.join(" ")
            }
            GeneratorConfig::Sentence { min, max } => {
                use fake::faker::lorem::en::Sentence;
                Sentence(*min..*max).fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Sentences { min, max } => {
                use fake::faker::lorem::en::Sentences;
                let sentences: Vec<String> = Sentences(*min..*max).fake_with_rng(rng);
                sentences.join(" ")
            }
            GeneratorConfig::Paragraph { count } => {
                use fake::faker::lorem::en::Paragraph;
                Paragraph(*count..*count + 1).fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Paragraphs { count } => {
                use fake::faker::lorem::en::Paragraphs;
                Paragraphs(*count..(count + 1)).fake_with_rng::<Vec<String>, _>(rng).join("\n\n")
            }
            GeneratorConfig::Word => {
                use fake::faker::lorem::en::Word;
                Word().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Regex { pattern } => {
                generate_from_regex(pattern, rng)
            }
            GeneratorConfig::Template { template } => {
                generate_from_template(template, rng)
            }

            // ========== 个人信息 ==========
            GeneratorConfig::Name => {
                use fake::faker::name::zh_cn::Name;
                Name().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::NameWithTitle => {
                use fake::faker::name::en::NameWithTitle;
                NameWithTitle().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::FirstName => {
                use fake::faker::name::zh_cn::FirstName;
                FirstName().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::LastName => {
                use fake::faker::name::zh_cn::LastName;
                LastName().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Title => {
                use fake::faker::name::en::Title;
                Title().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Suffix => {
                use fake::faker::name::en::Suffix;
                Suffix().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Email => {
                use fake::faker::internet::en::FreeEmail;
                FreeEmail().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::SafeEmail => {
                use fake::faker::internet::en::SafeEmail;
                SafeEmail().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::FreeEmailProvider => {
                use fake::faker::internet::en::FreeEmailProvider;
                FreeEmailProvider().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::DomainSuffix => {
                use fake::faker::internet::en::DomainSuffix;
                DomainSuffix().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::FreeEmail => {
                use fake::faker::internet::en::FreeEmail;
                FreeEmail().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::PhoneNumber => {
                use fake::faker::phone_number::zh_cn::PhoneNumber;
                PhoneNumber().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::CellNumber => {
                use fake::faker::phone_number::zh_cn::CellNumber;
                CellNumber().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Username => {
                use fake::faker::internet::en::Username;
                Username().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Password { min, max } => {
                use fake::faker::internet::en::Password;
                Password(*min..*max).fake_with_rng::<String, _>(rng)
            }

            // ========== 地址类 ==========
            GeneratorConfig::Country => {
                use fake::faker::address::en::CountryCode;
                CountryCode().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::CountryCode => {
                use fake::faker::address::en::CountryCode;
                CountryCode().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::CountryName => {
                use fake::faker::address::en::CountryName;
                CountryName().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::City => {
                use fake::faker::address::zh_cn::CityName;
                CityName().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::CityPrefix => {
                use fake::faker::address::en::CityPrefix;
                CityPrefix().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::CitySuffix => {
                use fake::faker::address::en::CitySuffix;
                CitySuffix().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::StateName => {
                use fake::faker::address::en::StateName;
                StateName().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::StateAbbr => {
                use fake::faker::address::en::StateAbbr;
                StateAbbr().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::StreetName => {
                use fake::faker::address::zh_cn::StreetName;
                StreetName().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::StreetSuffix => {
                use fake::faker::address::en::StreetSuffix;
                StreetSuffix().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::ZipCode => {
                use fake::faker::address::en::ZipCode;
                ZipCode().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::PostCode => {
                use fake::faker::address::en::PostCode;
                PostCode().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::BuildingNumber => {
                use fake::faker::address::en::BuildingNumber;
                BuildingNumber().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::SecondaryAddress => {
                use fake::faker::address::en::SecondaryAddress;
                SecondaryAddress().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::SecondaryAddressType => {
                use fake::faker::address::en::SecondaryAddressType;
                SecondaryAddressType().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Latitude => {
                use fake::faker::address::en::Latitude;
                Latitude().fake_with_rng::<f64, _>(rng).to_string()
            }
            GeneratorConfig::Longitude => {
                use fake::faker::address::en::Longitude;
                Longitude().fake_with_rng::<f64, _>(rng).to_string()
            }
            GeneratorConfig::Geohash { precision } => {
                use fake::faker::address::en::Geohash;
                Geohash(*precision).fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::TimeZone => {
                use fake::faker::address::en::TimeZone;
                TimeZone().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::IpAddress => {
                use fake::faker::internet::en::IP;
                IP().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::IPv4 => {
                use fake::faker::internet::en::IPv4;
                IPv4().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::IPv6 => {
                use fake::faker::internet::en::IPv6;
                IPv6().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::IP => {
                use fake::faker::internet::en::IP;
                IP().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::MacAddress => {
                use fake::faker::internet::en::MACAddress;
                MACAddress().fake_with_rng::<String, _>(rng)
            }

            // ========== 日期时间 ==========
            GeneratorConfig::DateTime { min, max } | GeneratorConfig::DateTimeBetween {
                start: min,
                end: max,
            } => {
                datetime_between(min, max, rng)
            }
            GeneratorConfig::DateTimeBefore { before } => {
                let min = "2020-01-01T00:00:00Z";
                datetime_between(min, before, rng)
            }
            GeneratorConfig::DateTimeAfter { after } => {
                let max = "2030-12-31T23:59:59Z";
                datetime_between(after, max, rng)
            }
            GeneratorConfig::Date { min, max } => {
                use fake::faker::chrono::en::{Date, DateTimeBetween};
                let s = parse_date(min);
                let e = parse_date(max);
                if let (Some(start), Some(end)) = (
                    s.and_hms_opt(0, 0, 0).map(|d| d.and_utc()),
                    e.and_hms_opt(23, 59, 59).map(|d| d.and_utc()),
                ) {
                    DateTimeBetween(start, end)
                        .fake_with_rng::<chrono::DateTime<chrono::Utc>, _>(rng)
                        .format("%Y-%m-%d")
                        .to_string()
                } else {
                    Date()
                        .fake_with_rng::<chrono::NaiveDate, _>(rng)
                        .format("%Y-%m-%d")
                        .to_string()
                }
            }
            GeneratorConfig::Time => {
                use fake::faker::chrono::en::Time;
                Time()
                    .fake_with_rng::<chrono::NaiveTime, _>(rng)
                    .format("%H:%M:%S")
                    .to_string()
            }
            GeneratorConfig::Duration => {
                use fake::faker::chrono::en::Duration;
                let d: chrono::Duration = Duration().fake_with_rng(rng);
                format!("{}", d.num_seconds())
            }

            // ========== 商业类 ==========
            GeneratorConfig::CompanyName => {
                use fake::faker::company::zh_cn::CompanyName;
                CompanyName().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::CompanySuffix => {
                use fake::faker::company::en::CompanySuffix;
                CompanySuffix().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::JobTitle => {
                let titles = vec![
                    "高级工程师", "产品经理", "技术总监", "项目经理",
                    "架构师", "数据分析师", "运营经理", "市场总监",
                    "财务经理", "人力资源总监", "后端工程师", "前端工程师",
                    "测试工程师", "运维工程师", "设计师", "实习生",
                ];
                let idx = (0..titles.len()).fake_with_rng::<usize, _>(rng);
                titles[idx].to_string()
            }
            GeneratorConfig::Profession => {
                use fake::faker::company::en::Profession;
                Profession().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Industry => {
                use fake::faker::company::en::Industry;
                Industry().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Seniority => {
                use fake::faker::job::en::Seniority;
                Seniority().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Field => {
                use fake::faker::job::en::Field;
                Field().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Position => {
                use fake::faker::job::en::Position;
                Position().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Buzzword => {
                use fake::faker::company::en::Buzzword;
                Buzzword().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::BuzzwordMiddle => {
                use fake::faker::company::en::BuzzwordMiddle;
                BuzzwordMiddle().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::BuzzwordTail => {
                use fake::faker::company::en::BuzzwordTail;
                BuzzwordTail().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::CatchPhrase => {
                use fake::faker::company::en::CatchPhrase;
                CatchPhrase().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::BsVerb => {
                use fake::faker::company::en::BsVerb;
                BsVerb().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::BsAdj => {
                use fake::faker::company::en::BsAdj;
                BsAdj().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::BsNoun => {
                use fake::faker::company::en::BsNoun;
                BsNoun().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Bs => {
                use fake::faker::company::en::Bs;
                Bs().fake_with_rng::<String, _>(rng)
            }

            // ========== 金融类 ==========
            GeneratorConfig::CurrencyCode => {
                use fake::faker::currency::en::CurrencyCode;
                CurrencyCode().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::CurrencyName => {
                use fake::faker::currency::en::CurrencyName;
                CurrencyName().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::CurrencySymbol => {
                use fake::faker::currency::en::CurrencySymbol;
                CurrencySymbol().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Bic => {
                use fake::faker::finance::en::Bic;
                Bic().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Isin => {
                use fake::faker::finance::en::Isin;
                Isin().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::CreditCardNumber => {
                use fake::faker::creditcard::en::CreditCardNumber;
                CreditCardNumber().fake_with_rng::<String, _>(rng)
            }

            // ========== UUID ==========
            GeneratorConfig::UuidV1 => {
                fake::uuid::UUIDv1.fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::UuidV3 => {
                fake::uuid::UUIDv3.fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::UuidV4 => {
                fake::uuid::UUIDv4.fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::UuidV5 => {
                fake::uuid::UUIDv5.fake_with_rng::<String, _>(rng)
            }

            // ========== 网络/技术 ==========
            GeneratorConfig::Url => {
                let tlds = ["com", "org", "net", "io", "dev", "app"];
                let tld = tlds[(0..tlds.len()).fake_with_rng::<usize, _>(rng)];
                let host: String = (0..(5..12).fake_with_rng::<usize, _>(rng))
                    .map(|_| ((97..123).fake_with_rng::<u8, _>(rng)) as char)
                    .collect();
                let path: String = (0..(0..3).fake_with_rng::<usize, _>(rng))
                    .map(|_| {
                        let seg: String = (0..(3..8).fake_with_rng::<usize, _>(rng))
                            .map(|_| ((97..123).fake_with_rng::<u8, _>(rng)) as char)
                            .collect();
                        format!("/{}", seg)
                    })
                    .collect();
                format!("https://{}.{}{}", host, tld, path)
            }
            GeneratorConfig::UserAgent => {
                use fake::faker::internet::en::UserAgent;
                UserAgent().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::MimeType => {
                use fake::faker::filesystem::en::MimeType;
                MimeType().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Semver => {
                use fake::faker::filesystem::en::Semver;
                Semver().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::SemverStable => {
                use fake::faker::filesystem::en::SemverStable;
                SemverStable().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::SemverUnstable => {
                use fake::faker::filesystem::en::SemverUnstable;
                SemverUnstable().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::FilePath => {
                use fake::faker::filesystem::en::FilePath;
                FilePath().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::FileName => {
                use fake::faker::filesystem::en::FileName;
                FileName().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::FileExtension => {
                use fake::faker::filesystem::en::FileExtension;
                FileExtension().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::DirPath => {
                use fake::faker::filesystem::en::DirPath;
                DirPath().fake_with_rng::<String, _>(rng)
            }

            // ========== Picsum 图片 ==========
            GeneratorConfig::ImageUrl { width, height } => {
                use fake::faker::picsum::en::ImageCustom;
                use fake::faker::impls::picsum::ImageOptions;
                let opts = ImageOptions {
                    width: Some((*width).min(u16::MAX as u32) as u16),
                    height: Some((*height).min(u16::MAX as u32) as u16),
                    grayscale: false,
                    blur: None,
                    seed: None,
                };
                ImageCustom(opts).fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::ImageUrlWithSeed { width, height, seed } => {
                use fake::faker::picsum::en::ImageCustom;
                use fake::faker::impls::picsum::ImageOptions;
                let opts = ImageOptions {
                    width: Some((*width).min(u16::MAX as u32) as u16),
                    height: Some((*height).min(u16::MAX as u32) as u16),
                    grayscale: false,
                    blur: None,
                    seed: Some(seed.to_string()),
                };
                ImageCustom(opts).fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::ImageUrlGrayscale { width, height } => {
                use fake::faker::picsum::en::ImageCustom;
                use fake::faker::impls::picsum::ImageOptions;
                let opts = ImageOptions {
                    width: Some((*width).min(u16::MAX as u32) as u16),
                    height: Some((*height).min(u16::MAX as u32) as u16),
                    grayscale: true,
                    blur: None,
                    seed: None,
                };
                ImageCustom(opts).fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::ImageUrlBlur { width, height, blur_amount } => {
                use fake::faker::picsum::en::ImageCustom;
                use fake::faker::impls::picsum::ImageOptions;
                let opts = ImageOptions {
                    width: Some((*width).min(u16::MAX as u32) as u16),
                    height: Some((*height).min(u16::MAX as u32) as u16),
                    grayscale: false,
                    blur: Some(*blur_amount),
                    seed: None,
                };
                ImageCustom(opts).fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::ImageUrlCustom { width, height, grayscale, blur_amount, seed } => {
                use fake::faker::picsum::en::ImageCustom;
                let opts = fake::faker::impls::picsum::ImageOptions {
                    width: Some((*width).min(u16::MAX as u32) as u16),
                    height: Some((*height).min(u16::MAX as u32) as u16),
                    grayscale: *grayscale,
                    blur: *blur_amount,
                    seed: seed.map(|s| s.to_string()),
                };
                ImageCustom(opts).fake_with_rng::<String, _>(rng)
            }

            // ========== 颜色类 ==========
            GeneratorConfig::HexColor => {
                use fake::faker::color::en::HexColor;
                HexColor().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::RgbColor => {
                use fake::faker::color::en::RgbColor;
                RgbColor().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::RgbaColor => {
                use fake::faker::color::en::RgbaColor;
                RgbaColor().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::HslColor => {
                use fake::faker::color::en::HslColor;
                HslColor().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::HslaColor => {
                use fake::faker::color::en::HslaColor;
                HslaColor().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Color => {
                use fake::faker::color::en::Color;
                Color().fake_with_rng::<String, _>(rng)
            }

            // ========== Ferroid ID ==========
            GeneratorConfig::FerroidULID => {
                fake::ferroid::FerroidULID.fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::FerroidTwitterId => {
                fake::ferroid::FerroidTwitterId.fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::FerroidInstagramId => {
                fake::ferroid::FerroidInstagramId.fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::FerroidMastodonId => {
                fake::ferroid::FerroidMastodonId.fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::FerroidDiscordId => {
                fake::ferroid::FerroidDiscordId.fake_with_rng::<String, _>(rng)
            }

            // ========== 编码标准 ==========
            GeneratorConfig::Isbn => {
                use fake::faker::barcode::en::Isbn;
                Isbn().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Isbn10 => {
                use fake::faker::barcode::en::Isbn10;
                Isbn10().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::Isbn13 => {
                use fake::faker::barcode::en::Isbn13;
                Isbn13().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::RfcStatusCode => {
                use fake::faker::http::en::RfcStatusCode;
                RfcStatusCode().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::ValidStatusCode => {
                use fake::faker::http::en::ValidStatusCode;
                ValidStatusCode().fake_with_rng::<String, _>(rng)
            }

            // ========== 汽车/行政 ==========
            GeneratorConfig::LicencePlate => {
                let letters: String = (0..3)
                    .map(|_| ((65..91).fake_with_rng::<u8, _>(rng)) as char)
                    .collect();
                let digits: String = (0..4)
                    .map(|_| ((48..58).fake_with_rng::<u8, _>(rng)) as char)
                    .collect();
                format!("{}-{}", letters, digits)
            }
            GeneratorConfig::HealthInsuranceCode => {
                format!(
                    "{}{}{}",
                    (0..100).fake_with_rng::<u32, _>(rng),
                    (0..100).fake_with_rng::<u32, _>(rng),
                    (0..10000).fake_with_rng::<u32, _>(rng),
                )
            }

            // ========== Markdown（需要 Range<usize> 参数，返回 Vec<String> 或 String）==========
            GeneratorConfig::MarkdownItalicWord => {
                use fake::faker::markdown::en::ItalicWord;
                ItalicWord().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::MarkdownBoldWord => {
                use fake::faker::markdown::en::BoldWord;
                BoldWord().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::MarkdownLink => {
                use fake::faker::markdown::en::Link;
                Link().fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::MarkdownBulletPoints => {
                use fake::faker::markdown::en::BulletPoints;
                let items: Vec<String> = BulletPoints(3..6).fake_with_rng(rng);
                items.join("\n")
            }
            GeneratorConfig::MarkdownListItems => {
                use fake::faker::markdown::en::ListItems;
                let items: Vec<String> = ListItems(3..6).fake_with_rng(rng);
                items.join("\n")
            }
            GeneratorConfig::MarkdownBlockQuoteSingle => {
                use fake::faker::markdown::en::BlockQuoteSingleLine;
                BlockQuoteSingleLine(1..3).fake_with_rng::<String, _>(rng)
            }
            GeneratorConfig::MarkdownBlockQuoteMulti => {
                use fake::faker::markdown::en::BlockQuoteMultiLine;
                let lines: Vec<String> = BlockQuoteMultiLine(2..5).fake_with_rng(rng);
                lines.join("\n")
            }
            GeneratorConfig::MarkdownCode => {
                use fake::faker::markdown::en::Code;
                Code(1..3).fake_with_rng::<String, _>(rng)
            }

            // ========== 约束类 ==========
            GeneratorConfig::ForeignKey { values } => {
                let idx = (0..values.len()).fake_with_rng::<usize, _>(rng);
                values[idx].clone()
            }
            GeneratorConfig::Sequence { values, cycle } => {
                let idx = if *cycle {
                    row_index % values.len()
                } else if row_index < values.len() {
                    row_index
                } else {
                    values.len() - 1
                };
                values[idx].clone()
            }
            GeneratorConfig::Weighted { choices } => {
                let total: f64 = choices.iter().map(|(_, w)| w).sum();
                let rand_val: f64 = (0.0..total).fake_with_rng(rng);
                let mut cumulative = 0.0;
                for (val, weight) in choices {
                    cumulative += weight;
                    if rand_val < cumulative {
                        return val.clone();
                    }
                }
                choices.last().map(|(v, _)| v.clone()).unwrap_or_default()
            }
        }
    }

    fn read_preview(
        conn: &duckdb::Connection,
        table_name: &str,
        limit: usize,
    ) -> MockResult<Vec<Vec<serde_json::Value>>> {
        let sql = format!("SELECT * FROM \"{}\" LIMIT {}", table_name, limit);
        let mut stmt = conn.prepare(&sql)?;
        let column_count = stmt.column_count();

        let mut rows = stmt.query([])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            let mut row_data = Vec::with_capacity(column_count);
            for i in 0..column_count {
                let val: duckdb::types::Value = row.get(i).unwrap_or(duckdb::types::Value::Null);
                row_data.push(value_to_json(&val));
            }
            result.push(row_data);
        }
        Ok(result)
    }
}

// ==================== 辅助函数 ====================

fn sanitize_table_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_lowercase()
}

fn parse_data_type(data_type: &str) -> crate::core::mock::models::ColumnDataType {
    match data_type.to_lowercase().as_str() {
        "integer" | "int" => crate::core::mock::models::ColumnDataType::Integer,
        "bigint" => crate::core::mock::models::ColumnDataType::BigInt,
        "float" => crate::core::mock::models::ColumnDataType::Float,
        "double" => crate::core::mock::models::ColumnDataType::Double,
        "decimal" => crate::core::mock::models::ColumnDataType::Decimal {
            precision: 18,
            scale: 2,
        },
        "boolean" | "bool" => crate::core::mock::models::ColumnDataType::Boolean,
        "varchar" => crate::core::mock::models::ColumnDataType::Varchar { length: None },
        "text" => crate::core::mock::models::ColumnDataType::Text,
        "date" => crate::core::mock::models::ColumnDataType::Date,
        "datetime" => crate::core::mock::models::ColumnDataType::DateTime,
        "timestamp" => crate::core::mock::models::ColumnDataType::Timestamp,
        "uuid" => crate::core::mock::models::ColumnDataType::Uuid,
        "blob" => crate::core::mock::models::ColumnDataType::Blob,
        _ => crate::core::mock::models::ColumnDataType::Varchar { length: None },
    }
}

fn parse_date(s: &str) -> chrono::NaiveDate {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2020, 1, 1)
            .unwrap_or_default())
}

fn datetime_between(min: &str, max: &str, rng: &mut StdRng) -> String {
    use chrono::{DateTime, Utc};
    use fake::faker::chrono::en::DateTimeBetween;

    let default_start = DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
        .map(|d| d.to_utc())
        .unwrap_or_default();
    let default_end = DateTime::parse_from_rfc3339("2030-12-31T23:59:59Z")
        .map(|d| d.to_utc())
        .unwrap_or_default();

    let s = DateTime::parse_from_rfc3339(min)
        .map(|d| d.to_utc())
        .unwrap_or(default_start);
    let e = DateTime::parse_from_rfc3339(max)
        .map(|d| d.to_utc())
        .unwrap_or(default_end);

    DateTimeBetween(s, e)
        .fake_with_rng::<DateTime<Utc>, _>(rng)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn value_to_json(val: &duckdb::types::Value) -> serde_json::Value {
    match val {
        duckdb::types::Value::Null => serde_json::Value::Null,
        duckdb::types::Value::Boolean(b) => serde_json::Value::Bool(*b),
        duckdb::types::Value::TinyInt(i) => serde_json::json!(i),
        duckdb::types::Value::SmallInt(i) => serde_json::json!(i),
        duckdb::types::Value::Int(i) => serde_json::json!(i),
        duckdb::types::Value::BigInt(i) => serde_json::json!(i),
        duckdb::types::Value::HugeInt(i) => serde_json::json!(i.to_string()),
        duckdb::types::Value::Float(f) => serde_json::json!(f),
        duckdb::types::Value::Double(f) => serde_json::json!(f),
        duckdb::types::Value::Text(s) => serde_json::Value::String(s.clone()),
        _ => serde_json::Value::String(format!("{:?}", val)),
    }
}

fn value_to_sql_literal(val: &duckdb::types::Value) -> String {
    match val {
        duckdb::types::Value::Null => "NULL".to_string(),
        duckdb::types::Value::Boolean(b) => b.to_string(),
        duckdb::types::Value::TinyInt(i) => i.to_string(),
        duckdb::types::Value::SmallInt(i) => i.to_string(),
        duckdb::types::Value::Int(i) => i.to_string(),
        duckdb::types::Value::BigInt(i) => i.to_string(),
        duckdb::types::Value::Float(f) => f.to_string(),
        duckdb::types::Value::Double(f) => f.to_string(),
        duckdb::types::Value::Text(s) => format!("'{}'", s.replace('\'', "''")),
        _ => "NULL".to_string(),
    }
}

/// 从正则表达式生成随机字符串（支持常见模式）
fn generate_from_regex(pattern: &str, rng: &mut StdRng) -> String {
    let mut result = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    let len = chars.len();
    while i < len {
        match chars[i] {
            '\\' if i + 1 < len => {
                i += 1;
                match chars[i] {
                    'd' => result.push((b'0' + (0..10).fake_with_rng::<u8, _>(rng)) as char),
                    'w' => {
                        let pool = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
                        let idx = (0..pool.len()).fake_with_rng::<usize, _>(rng);
                        result.push(pool[idx] as char);
                    }
                    's' => result.push(' '),
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    c => result.push(c),
                }
            }
            '[' => {
                let mut class_chars = Vec::new();
                i += 1;
                let mut negated = false;
                if i < len && chars[i] == '^' {
                    negated = true;
                    i += 1;
                }
                while i < len && chars[i] != ']' {
                    if i + 2 < len && chars[i + 1] == '-' {
                        let start = chars[i] as u32;
                        let end = chars[i + 2] as u32;
                        if end >= start {
                            for c in (start..=end).take(256) {
                                class_chars.push(char::from_u32(c).unwrap_or(' '));
                            }
                        }
                        i += 3;
                    } else {
                        class_chars.push(chars[i]);
                        i += 1;
                    }
                }
                if i < len { i += 1; }
                if negated {
                    let full: Vec<char> =
                        (32..127).filter_map(char::from_u32).collect();
                    class_chars = full.into_iter().filter(|c| !class_chars.contains(c)).collect();
                }
                if !class_chars.is_empty() {
                    let idx = (0..class_chars.len()).fake_with_rng::<usize, _>(rng);
                    result.push(class_chars[idx]);
                }
            }
            '{' => {
                i += 1;
                let mut num_str = String::new();
                while i < len && chars[i] != '}' && chars[i] != ',' {
                    num_str.push(chars[i]);
                    i += 1;
                }
                let min: usize = num_str.parse().unwrap_or(1);
                let mut max = min;
                if i < len && chars[i] == ',' {
                    i += 1;
                    num_str.clear();
                    while i < len && chars[i] != '}' {
                        num_str.push(chars[i]);
                        i += 1;
                    }
                    max = num_str.parse().unwrap_or(min);
                }
                if i < len { i += 1; }
                // 应用量词到前一个字符
                let count = if max > min {
                    (min..=max).fake_with_rng::<usize, _>(rng)
                } else {
                    min
                };
                if let Some(last) = result.pop() {
                    for _ in 0..count { result.push(last); }
                }
            }
            '+' | '*' | '?' | '.' => {
                // 简化处理：跳过这些量词
            }
            '(' | ')' | '^' | '$' => {
                // 锚点和分组符，跳过
            }
            c => result.push(c),
        }
        i += 1;
    }
    if result.is_empty() {
        Faker.fake_with_rng::<String, _>(rng)
    } else {
        result
    }
}

/// 模板字符串替换：{{name}} → 生成值
fn generate_from_template(template: &str, rng: &mut StdRng) -> String {
    let mut result = template.to_string();

    let replacements: &[(&str, fn(&mut StdRng) -> String)] = &[
        ("name", |r| {
            use fake::faker::name::zh_cn::Name;
            Name().fake_with_rng::<String, _>(r)
        }),
        ("first_name", |r| {
            use fake::faker::name::zh_cn::FirstName;
            FirstName().fake_with_rng::<String, _>(r)
        }),
        ("last_name", |r| {
            use fake::faker::name::zh_cn::LastName;
            LastName().fake_with_rng::<String, _>(r)
        }),
        ("email", |r| {
            use fake::faker::internet::en::SafeEmail;
            SafeEmail().fake_with_rng::<String, _>(r)
        }),
        ("uuid", |r| {
            fake::uuid::UUIDv4.fake_with_rng::<String, _>(r)
        }),
        ("word", |r| {
            use fake::faker::lorem::en::Word;
            Word().fake_with_rng::<String, _>(r)
        }),
        ("sentence", |r| {
            use fake::faker::lorem::en::Sentence;
            Sentence(3..8).fake_with_rng::<String, _>(r)
        }),
        ("phone", |r| {
            use fake::faker::phone_number::zh_cn::PhoneNumber;
            PhoneNumber().fake_with_rng::<String, _>(r)
        }),
        ("date", |r| {
            use fake::faker::chrono::en::Date;
            Date().fake_with_rng::<chrono::NaiveDate, _>(r).format("%Y-%m-%d").to_string()
        }),
        ("datetime", |r| {
            fake::faker::chrono::en::DateTime().fake_with_rng::<chrono::DateTime<chrono::Utc>, _>(r).format("%Y-%m-%d %H:%M:%S").to_string()
        }),
    ];

    for (key, gen_fn) in replacements {
        let placeholder = format!("{{{{{}}}}}", key);
        if result.contains(&placeholder) {
            let value = gen_fn(rng);
            result = result.replace(&placeholder, &value);
        }
    }

    // 处理 {{int:MIN-MAX}} 格式（手动解析，避免额外依赖）
    let mut int_result = String::new();
    let template_bytes = result.as_bytes();
    let mut pos = 0;
    let prefix = b"{{int:";
    while pos < template_bytes.len() {
        if pos + prefix.len() <= template_bytes.len()
            && &template_bytes[pos..pos + prefix.len()] == prefix
        {
            let start_idx = pos + prefix.len();
            let mut end_pos = start_idx;
            while end_pos < template_bytes.len()
                && template_bytes[end_pos] != b'-'
                && template_bytes[end_pos] != b'}'
            {
                end_pos += 1;
            }
            let min_str = std::str::from_utf8(&template_bytes[start_idx..end_pos]).unwrap_or("0");
            if end_pos < template_bytes.len() && template_bytes[end_pos] == b'-' {
                end_pos += 1;
                let val_start = end_pos;
                while end_pos < template_bytes.len() && template_bytes[end_pos] != b'}' {
                    end_pos += 1;
                }
                let max_str =
                    std::str::from_utf8(&template_bytes[val_start..end_pos]).unwrap_or("100");
                if end_pos < template_bytes.len() && template_bytes[end_pos] == b'}' {
                    end_pos += 1;
                }
                let min: i64 = min_str.parse().unwrap_or(0);
                let max: i64 = max_str.parse().unwrap_or(100);
                let val: i64 = if max >= min {
                    (min..=max).fake_with_rng::<i64, _>(rng)
                } else {
                    min
                };
                int_result.push_str(&val.to_string());
            } else {
                int_result.push_str(&result[pos..end_pos]);
            }
            pos = end_pos;
        } else {
            int_result.push(result.as_bytes()[pos] as char);
            pos += 1;
        }
    }
    result = int_result;

    result
}

impl MockEngine {
    pub fn import_schema(input: &ImportSchemaInput) -> MockResult<Vec<ColumnDef>> {
    use crate::core::persistence::metadata_cache::{ConnectionType, MetadataCacheManager, MetadataCacheOps};

    let cache_conn_type = if input.connection_type == "project" {
        ConnectionType::Project
    } else {
        ConnectionType::Global
    };

    let cache_manager = MetadataCacheManager::new(
        &input.conn_id,
        cache_conn_type,
        input.project_path.as_deref(),
    )
    .map_err(|e| MockError::Config(format!("Failed to open metadata cache: {}", e)))?;

    let conn = cache_manager.open().map_err(|e| MockError::Config(format!("Failed to open cache connection: {}", e)))?;
    let ops = MetadataCacheOps::new(conn);

    let mut all_columns: Vec<ColumnDef> = Vec::new();
    let default_schema = input.schema.as_deref().unwrap_or("default");

    for table_name in &input.tables {
        match ops.list_columns(&input.database, default_schema, table_name) {
            Ok(columns) => {
                for col in columns {
                    let data_type = map_sql_type_to_column_data_type(&col.data_type);
                    let inferred = ColumnMapper::infer(&col.name, &data_type);
                    all_columns.push(ColumnDef {
                        name: col.name,
                        data_type,
                        generator: inferred.generator,
                        nullable_ratio: if col.is_nullable { 0.1 } else { 0.0 },
                        unique: col.is_unique,
                    });
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to get columns for table {}: {}", table_name, e);
            }
        }
    }

    if all_columns.is_empty() {
        return Err(MockError::Config(
            "No columns found. Ensure metadata cache has been populated.".to_string(),
        ));
    }

    Ok(all_columns)
}
}

impl MockEngine {
    pub fn save_to_scratchpad(
        temp_table_name: &str,
        format: &MockExportFormat,
        scratchpad_dir: &str,
    ) -> MockResult<String> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let base_name = temp_table_name.trim_start_matches(TEMP_MOCK_PREFIX);
        let ext = match format {
            MockExportFormat::Csv => "csv",
            MockExportFormat::Parquet => "parquet",
            MockExportFormat::Xlsx => "xlsx",
            MockExportFormat::SqlInsert => "sql",
            MockExportFormat::Table => "duckdb",
        };
        let file_name = format!("mock_{}_{}.{}", base_name, timestamp, ext);
        let output_path = format!("{}/{}", scratchpad_dir, file_name);

        Self::export(temp_table_name, format, Some(&output_path), None)
    }

    pub fn persist_as_asset(
        temp_table_name: &str,
        new_name: &str,
    ) -> MockResult<(String, i64, i32)> {
        let duckdb = DuckDBManager::global().get_or_create_in_memory()?;
        let conn = duckdb.lock().map_err(|e| {
            MockError::Generation(format!("DuckDB lock error: {}", e))
        })?;

        let sql = format!(
            "CREATE TABLE \"{}\" AS SELECT * FROM \"{}\"",
            new_name, temp_table_name
        );
        conn.execute_batch(&sql)?;

        let row_count: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM \"{}\"", new_name),
                [],
                |row| row.get(0),
            )?;

        let stmt = conn.prepare(&format!("SELECT * FROM \"{}\" LIMIT 0", new_name))?;
        let column_count = stmt.column_count() as i32;

        let drop_sql = format!("DROP TABLE IF EXISTS \"{}\"", temp_table_name);
        conn.execute_batch(&drop_sql)?;

        Ok((new_name.to_string(), row_count, column_count))
    }
}

impl MockEngine {
    pub async fn re_generate(history_id: &str) -> MockResult<MockGenerateResult> {
        let entry = MockHistoryStore::get_by_id(history_id)?
            .ok_or_else(|| MockError::Config(format!("History entry not found: {}", history_id)))?;

        let config: MockConfig = serde_json::from_str(&entry.config_json)
            .map_err(|e| MockError::Config(format!("Failed to deserialize config: {}", e)))?;

        Self::generate(config).await
    }

    pub fn get_history(limit: usize) -> MockResult<Vec<MockHistoryRecord>> {
        MockHistoryStore::list(limit)
    }

    pub fn clear_history() -> MockResult<usize> {
        MockHistoryStore::clear()
    }
}

fn map_sql_type_to_column_data_type(sql_type: &str) -> ColumnDataType {
    let lower = sql_type.to_lowercase();
    if lower.contains("int") {
        if lower.contains("big") { ColumnDataType::BigInt }
        else { ColumnDataType::Integer }
    } else if lower.contains("float") || lower.contains("real") {
        ColumnDataType::Float
    } else if lower.contains("double") {
        ColumnDataType::Double
    } else if lower.contains("decimal") || lower.contains("numeric") {
        ColumnDataType::Decimal { precision: 18, scale: 2 }
    } else if lower.contains("bool") {
        ColumnDataType::Boolean
    } else if lower.contains("date") || lower.contains("timestamp") || lower.contains("datetime") {
        ColumnDataType::DateTime
    } else if lower.contains("time") {
        ColumnDataType::DateTime
    } else if lower.contains("blob") || lower.contains("binary") {
        ColumnDataType::Blob
    } else if lower.contains("text") || lower.contains("clob") {
        ColumnDataType::Text
    } else {
        ColumnDataType::Varchar { length: None }
    }
}