use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use fake::rand::rngs::StdRng;
use fake::rand::SeedableRng;
use fake::Fake;

use super::generators::generate_cell;
use crate::core::driver::native::duckdb::duckdb_rows_to_arrow;
use crate::core::duckdb::DuckDBManager;
use crate::core::models::QueryResult;
use crate::core::sql::{ColumnDefInfo, SqlEngine};
use crate::mock::error::{MockError, MockResult};
use crate::mock::history::MockHistoryStore;
use crate::mock::models::{
    ColumnDataType, ColumnDef, ColumnMappingResponse, ImportSchemaInput, MockConfig,
    MockExportFormat, MockGenerateResult, MockHistoryRecord, ScenarioTemplate,
};
use crate::mock::schema_map::ColumnMapper;
use crate::mock::templates;

/// Mock 数据引擎 —— 在 DuckDB 内存表中生成模拟数据集
///
/// # SQL 安全性
///
/// 本模块通过 [SqlEngine](crate::core::sql::SqlEngine) 构造所有 DDL/DML SQL 语句，
/// 彻底消除 `format!()` 字符串拼接：
/// - **DDL**（CREATE TABLE / DROP TABLE）：通过 `SqlEngine::build_create_table` /
///   `SqlEngine::build_drop_table` 生成
/// - **DML**（INSERT）：通过 `SqlEngine::build_insert` 生成，值由 SqlEngine 负责转义
/// - **DQL**（SELECT）：通过 `SqlEngine::build_select_all` / `SqlEngine::build_select` 生成
///
/// 仅 DuckDB 专有的 `COPY` 命令保留 `format!()` 拼接（非标准 SQL）。
pub struct MockEngine;

const TEMP_MOCK_PREFIX: &str = "temp_mock_";
const BATCH_SIZE: usize = 10_000;
const PREVIEW_ROWS: usize = 10;

impl MockEngine {
    // ==================== 数据生成 ====================

    /// 生成 Mock 数据（无进度回调）
    ///
    /// 根据 `MockConfig` 配置在 DuckDB 内存临时表中生成指定行数/列数的模拟数据。
    /// 内部调用 `generate_with_progress`，回调为空。
    pub async fn generate(config: MockConfig) -> MockResult<MockGenerateResult> {
        Self::generate_with_progress(config, |_, _| {}).await
    }

    /// 生成 Mock 数据（带进度回调）
    ///
    /// 分批次生成数据（每批 BATCH_SIZE 行），每完成一批调用 `on_progress(batch_idx, total_batches)`。
    /// 结果存储在 DuckDB 内存临时表 `temp_mock_{table_name}` 中。
    /// 返回 `MockGenerateResult` 包含预览 `QueryResult` 和耗时。
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
        for col in &config.columns {
            if col.nullable_ratio < 0.0 || col.nullable_ratio > 1.0 {
                return Err(MockError::InvalidColumn(format!(
                    "列 '{}' 的 nullable_ratio 必须介于 0.0~1.0，当前: {}",
                    col.name, col.nullable_ratio
                )));
            }
        }

        let start = Instant::now();

        let mut rng: StdRng = match config.seed {
            Some(s) => StdRng::seed_from_u64(s as u64),
            None => StdRng::seed_from_u64(rand::random()),
        };

        let safe_name = sanitize_table_name(&config.table_name);
        let table_name = format!("{}{}", TEMP_MOCK_PREFIX, safe_name);

        let db = Self::get_db()?;
        let conn = Self::get_conn(&db)?;

        let drop_sql = SqlEngine::build_drop_table(&table_name, true);
        conn.execute_batch(&drop_sql)?;

        let ddl = Self::build_create_table_ddl(&table_name, &config.columns);
        conn.execute_batch(&ddl)?;

        let total_batches = config.row_count.div_ceil(BATCH_SIZE as u32);
        let mut unique_sets: Vec<HashSet<String>> = config
            .columns
            .iter()
            .map(|c| {
                if c.unique {
                    HashSet::with_capacity(config.row_count as usize)
                } else {
                    HashSet::new()
                }
            })
            .collect();

        let safe_col_names: Vec<String> = config
            .columns
            .iter()
            .map(|c| {
                let safe: String = c
                    .name
                    .chars()
                    .map(|ch| {
                        if ch.is_alphanumeric() || ch == '_' {
                            ch
                        } else {
                            '_'
                        }
                    })
                    .collect::<String>()
                    .trim_matches('_')
                    .to_string();
                if safe.is_empty() {
                    return Err(MockError::Generation(format!(
                        "column name '{}' resolves to empty after sanitization",
                        c.name
                    )));
                }
                Ok(safe)
            })
            .collect::<MockResult<Vec<_>>>()?;

        for batch_idx in 0..total_batches {
            let start_row = batch_idx * BATCH_SIZE as u32;
            let count = std::cmp::min(BATCH_SIZE as u32, config.row_count - start_row);

            let mut all_values: Vec<Vec<String>> = Vec::with_capacity(count as usize);

            for row_idx in 0..count {
                let global_row = start_row + row_idx;
                let mut row_vals = Vec::with_capacity(config.columns.len());

                for (col_idx, col) in config.columns.iter().enumerate() {
                    let mut attempts = 0;
                    let value = loop {
                        let val = generate_cell(
                            &col.generator,
                            &mut rng,
                            global_row as usize,
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

                    let nullable_ratio = config.columns[col_idx].nullable_ratio;
                    if nullable_ratio > 0.0 {
                        let rand_val: f64 = (0.0..1.0).fake_with_rng(&mut rng);
                        if rand_val < nullable_ratio {
                            row_vals.push("NULL".to_string());
                            continue;
                        }
                    }
                    row_vals.push(value);
                }

                all_values.push(row_vals);
            }

            let insert_sql = SqlEngine::build_insert(&table_name, &safe_col_names, &all_values);
            conn.execute_batch(&insert_sql)?;
            on_progress(batch_idx as usize + 1, total_batches as usize);
        }

        DuckDBManager::register_temp_table(&table_name);

        let preview = Self::read_preview(&conn, &table_name, PREVIEW_ROWS)?;
        let elapsed_ms = start.elapsed().as_millis() as u64;

        let _ = MockHistoryStore::save(&config, elapsed_ms);

        Ok(MockGenerateResult {
            table_name: config.table_name.clone(),
            temp_table_name: table_name,
            row_count: config.row_count,
            preview,
            columns: config.columns.iter().map(|c| c.name.clone()).collect(),
            elapsed_ms: elapsed_ms as u32,
        })
    }

    fn get_db() -> MockResult<Arc<Mutex<duckdb::Connection>>> {
        DuckDBManager::get_or_create_in_memory()
            .map_err(|e| MockError::Generation(format!("DuckDB error: {}", e)))
    }

    fn get_conn(
        db: &Arc<Mutex<duckdb::Connection>>,
    ) -> MockResult<std::sync::MutexGuard<'_, duckdb::Connection>> {
        db.lock()
            .map_err(|e| MockError::Generation(format!("DuckDB lock error: {}", e)))
    }

    // ==================== 预览刷新 ====================

    /// 刷新临时表预览数据
    ///
    /// 从指定的 DuckDB 临时表中读取前 `limit` 行，转换为 Arrow `RecordBatch`
    /// 并封装为 `QueryResult` 返回。用于前端 ag-Grid 二次渲染。
    pub fn preview(temp_table_name: &str, limit: usize) -> MockResult<QueryResult> {
        let db = Self::get_db()?;
        let conn = Self::get_conn(&db)?;
        Self::read_preview(&conn, temp_table_name, limit)
            .map_err(|e| MockError::Preview(e.to_string()))
    }

    // ==================== 导出 ====================

    /// 导出临时表数据到文件
    ///
    /// 支持 CSV / Parquet / Xlsx / SQL INSERT 四种格式。
    /// 使用 DuckDB `COPY` 命令直接导出，利用原生 I/O 优化。
    /// Xlsx 格式通过 JSON 中转（DuckDB 不原生支持 xlsx）。
    pub fn export(
        temp_table_name: &str,
        format: &MockExportFormat,
        output_path: Option<&str>,
        table_name: Option<&str>,
    ) -> MockResult<String> {
        let db = Self::get_db()?;
        let conn = Self::get_conn(&db)?;

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
                let create_sql = SqlEngine::build_create_table_as_select(
                    new_name,
                    &SqlEngine::build_select_all(temp_table_name, None),
                );
                conn.execute_batch(&create_sql)?;
                let drop_sql = SqlEngine::build_drop_table(temp_table_name, true);
                conn.execute_batch(&drop_sql)?;
                Ok(format!("Persisted as table: {}", new_name))
            }
            MockExportFormat::SqlInsert => {
                let path = output_path.ok_or_else(|| {
                    MockError::Config("SQL INSERT export requires output_path".to_string())
                })?;
                let select_sql = SqlEngine::build_select_all(temp_table_name, None);
                let mut stmt = conn.prepare(&select_sql)?;
                let columns: Vec<String> = stmt
                    .column_names()
                    .iter()
                    .map(|c| format!("\"{}\"", c))
                    .collect();
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

    /// 单列智能映射
    ///
    /// 根据列名和数据类型的语义分析，自动推荐最合适的 `GeneratorConfig`。
    /// 返回 `ColumnMappingResponse` 含置信度和示例值。
    pub fn map_column(column_name: &str, data_type: &str) -> MockResult<ColumnMappingResponse> {
        let dt = parse_data_type(data_type);
        Ok(ColumnMapper::infer(column_name, &dt))
    }

    /// 批量列智能映射
    ///
    /// 对多列表名+数据类型对进行批量推断，返回 `Vec<ColumnMappingResponse>`。
    pub fn map_columns_batch(
        columns: Vec<(String, String)>,
    ) -> MockResult<Vec<ColumnMappingResponse>> {
        columns
            .into_iter()
            .map(|(name, dt)| Self::map_column(&name, &dt))
            .collect()
    }

    // ==================== 场景模板 ====================

    /// 列出所有内置场景模板
    ///
    /// 返回 6 个预定义模板：电商/HR/博客/金融/社交媒体/企业通讯录。
    /// 每个模板包含表结构、列定义、推荐 GeneratorConfig。
    pub fn list_templates() -> MockResult<Vec<ScenarioTemplate>> {
        Ok(templates::get_builtin_templates())
    }

    /// 按 ID 查找场景模板详情
    ///
    /// 返回完整的 `ScenarioTemplate`，包含该场景下所有表的列定义和推荐配置。
    pub fn apply_template(template_id: &str) -> MockResult<ScenarioTemplate> {
        templates::get_template_by_id(template_id)
            .ok_or_else(|| MockError::TemplateNotFound(template_id.to_string()))
    }

    // ==================== 私有方法 ====================

    fn build_create_table_ddl(table_name: &str, columns: &[ColumnDef]) -> String {
        let col_infos: Vec<ColumnDefInfo> = columns
            .iter()
            .map(|c| ColumnDefInfo {
                name: c.name.clone(),
                data_type: c.data_type.to_duckdb_type(),
                unique: c.unique,
                nullable: c.nullable_ratio > 0.0,
            })
            .collect();

        SqlEngine::build_create_table(table_name, &col_infos, false)
    }

    fn read_preview(
        conn: &duckdb::Connection,
        table_name: &str,
        limit: usize,
    ) -> MockResult<QueryResult> {
        let sql = SqlEngine::build_select_all(table_name, Some(limit as i64));
        let mut stmt = conn.prepare(&sql)?;

        let row_data: Vec<Vec<duckdb::types::Value>>;
        {
            let mut rows = stmt.query([])?;
            let mut data: Vec<Vec<duckdb::types::Value>> = Vec::new();
            while let Some(row) = rows.next()? {
                let mut cols: Vec<duckdb::types::Value> = Vec::new();
                for i in 0.. {
                    match row.get::<usize, duckdb::types::Value>(i) {
                        Ok(v) => cols.push(v),
                        Err(_) => break,
                    }
                }
                data.push(cols);
            }
            row_data = data;
        }

        let column_count = if let Some(first) = row_data.first() {
            first.len()
        } else {
            stmt.column_count()
        };

        let columns: Vec<String> = if column_count > 0 {
            (0..column_count)
                .map(|i| stmt.column_name(i).map_or("unknown", |v| v).to_string())
                .collect()
        } else {
            Vec::new()
        };

        let arrow_batch = duckdb_rows_to_arrow(&columns, &row_data)?;
        let _total = arrow_batch.num_rows();

        Ok(QueryResult {
            columns,
            batches: vec![arrow_batch],
            ..Default::default()
        })
    }
}

// ==================== 辅助函数 ====================

fn sanitize_table_name(name: &str) -> String {
    let safe = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_lowercase();
    if safe.is_empty() {
        format!(
            "auto_table_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        )
    } else {
        safe
    }
}

fn parse_data_type(data_type: &str) -> crate::mock::models::ColumnDataType {
    match data_type.to_lowercase().as_str() {
        "integer" | "int" => crate::mock::models::ColumnDataType::Integer,
        "bigint" => crate::mock::models::ColumnDataType::BigInt,
        "float" => crate::mock::models::ColumnDataType::Float,
        "double" => crate::mock::models::ColumnDataType::Double,
        "decimal" => crate::mock::models::ColumnDataType::Decimal {
            precision: 18,
            scale: 2,
        },
        "boolean" | "bool" => crate::mock::models::ColumnDataType::Boolean,
        "varchar" => crate::mock::models::ColumnDataType::Varchar { length: None },
        "text" => crate::mock::models::ColumnDataType::Text,
        "date" => crate::mock::models::ColumnDataType::Date,
        "datetime" => crate::mock::models::ColumnDataType::DateTime,
        "timestamp" => crate::mock::models::ColumnDataType::Timestamp,
        "uuid" => crate::mock::models::ColumnDataType::Uuid,
        "blob" => crate::mock::models::ColumnDataType::Blob,
        _ => crate::mock::models::ColumnDataType::Varchar { length: None },
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

impl MockEngine {
    /// 从真实数据库导入表结构
    ///
    /// 读取指定数据库连接中目标表的列信息（名称/类型/注释），
    /// 通过 `ColumnMapper` 自动推断每列的 `GeneratorConfig`。
    /// 返回 `Vec<ColumnDef>` 可直接用于 Mock 生成配置。
    pub fn import_schema(input: &ImportSchemaInput) -> MockResult<Vec<ColumnDef>> {
        use crate::core::persistence::metadata_cache::{
            ConnectionType, MetadataCacheManager, MetadataCacheOps,
        };

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

        let conn = cache_manager
            .open()
            .map_err(|e| MockError::Config(format!("Failed to open cache connection: {}", e)))?;
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
                    tracing::warn!(
                        "Warning: Failed to get columns for table {}: {}",
                        table_name,
                        e
                    );
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
    /// 将临时表保存到用户草稿本
    ///
    /// 创建 DuckDB 持久化表（或追加到同名表），从临时表复制全部数据。
    /// 返回新表名或更新后的行数。
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

    /// 将临时表持久化为项目资产
    ///
    /// 以新名称将临时表转为 DuckDB 持久化双挂载表（内存+文件），
    /// 返回 `(表名, 行数, 列数)`。
    pub fn persist_as_asset(
        temp_table_name: &str,
        new_name: &str,
    ) -> MockResult<(String, i64, i32)> {
        let db = Self::get_db()?;
        let conn = Self::get_conn(&db)?;

        let safe_name = sanitize_table_name(new_name);

        let create_sql = SqlEngine::build_create_table_as_select(
            &safe_name,
            &SqlEngine::build_select_all(temp_table_name, None),
        );
        conn.execute_batch(&create_sql)?;

        let count_sql = SqlEngine::build_select(&safe_name, &["COUNT(*)"], None);
        let row_count: i64 = conn.query_row(&count_sql, [], |row| row.get(0))?;

        let column_count: i32;
        {
            let desc_sql = SqlEngine::build_select_all(&safe_name, Some(0));
            let mut stmt_desc = conn.prepare(&desc_sql)?;
            let _rows = stmt_desc.query([])?;
            column_count = stmt_desc.column_count() as i32;
        }

        let drop_sql = SqlEngine::build_drop_table(temp_table_name, true);
        conn.execute_batch(&drop_sql)?;

        Ok((safe_name, row_count, column_count))
    }
}

impl MockEngine {
    /// 基于历史记录重新生成
    ///
    /// 反序列化历史条目中的 `MockConfig`，使用相同的种子和配置重新执行生成。
    /// 注意：无种子时会得到与历史不同的随机结果。
    pub async fn re_generate(history_id: &str) -> MockResult<MockGenerateResult> {
        let entry = MockHistoryStore::get_by_id(history_id)?
            .ok_or_else(|| MockError::Config(format!("History entry not found: {}", history_id)))?;

        let config: MockConfig = serde_json::from_str(&entry.config_json)
            .map_err(|e| MockError::Config(format!("Failed to deserialize config: {}", e)))?;

        Self::generate(config).await
    }

    /// 获取生成历史记录
    ///
    /// 按时间倒序返回最近 `limit` 条（上限 500）生成记录，
    /// 含配置 JSON 和耗时。用于前端历史面板展示和二次生成。
    pub fn get_history(limit: usize) -> MockResult<Vec<MockHistoryRecord>> {
        MockHistoryStore::list(limit)
    }

    /// 清空生成历史
    ///
    /// 删除 DuckDB 内存历史表中全部记录，返回已清除的条数。
    pub fn clear_history() -> MockResult<usize> {
        MockHistoryStore::clear()
    }
}

fn map_sql_type_to_column_data_type(sql_type: &str) -> ColumnDataType {
    let lower = sql_type.to_lowercase();
    if lower.contains("int") {
        if lower.contains("big") {
            ColumnDataType::BigInt
        } else {
            ColumnDataType::Integer
        }
    } else if lower.contains("float") || lower.contains("real") {
        ColumnDataType::Float
    } else if lower.contains("double") {
        ColumnDataType::Double
    } else if lower.contains("decimal") || lower.contains("numeric") {
        ColumnDataType::Decimal {
            precision: 18,
            scale: 2,
        }
    } else if lower.contains("bool") {
        ColumnDataType::Boolean
    } else if lower.contains("date")
        || lower.contains("timestamp")
        || lower.contains("datetime")
        || lower.contains("time")
    {
        ColumnDataType::DateTime
    } else if lower.contains("blob") || lower.contains("binary") {
        ColumnDataType::Blob
    } else if lower.contains("text") || lower.contains("clob") {
        ColumnDataType::Text
    } else {
        ColumnDataType::Varchar { length: None }
    }
}

/// Infer ColumnDataType from a SQL type string (lowercase, with optional size).
/// e.g. "int" -> Integer, "varchar(255)" -> Varchar { length: Some(255) }
#[allow(dead_code)]
fn infer_datatype_for_column(sql_type: &str) -> ColumnDataType {
    let lower = sql_type.trim().to_lowercase();

    if lower == "int"
        || lower == "integer"
        || lower == "int4"
        || lower == "int8"
        || lower == "bigint"
        || lower == "smallint"
        || lower == "tinyint"
        || lower == "serial"
        || lower == "bigserial"
    {
        ColumnDataType::Integer
    } else if lower.starts_with("varchar") || lower.starts_with("char") {
        let length = lower
            .trim_start_matches(|c: char| c.is_alphabetic())
            .trim_matches(|c: char| c == '(' || c == ')')
            .parse::<u32>()
            .ok()
            .map(|v| v as usize);
        ColumnDataType::Varchar {
            length: length.map(|v| v as u32),
        }
    } else if lower.starts_with("decimal") || lower.starts_with("numeric") {
        let inner = lower
            .trim_start_matches(|c: char| c.is_alphabetic())
            .trim_matches(|c: char| c == '(' || c == ')');
        let parts: Vec<&str> = inner.split(',').collect();
        let precision = parts
            .first()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(10);
        let scale = parts
            .get(1)
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        ColumnDataType::Decimal { precision, scale }
    } else if lower == "float"
        || lower == "real"
        || lower == "float4"
        || lower == "double"
        || lower == "float8"
        || lower == "double precision"
    {
        ColumnDataType::Float
    } else if lower == "bool" || lower == "boolean" {
        ColumnDataType::Boolean
    } else if lower == "date" {
        ColumnDataType::Date
    } else if lower.starts_with("timestamp") || lower.starts_with("datetime") {
        ColumnDataType::DateTime
    } else if lower == "time" {
        ColumnDataType::Timestamp
    } else if lower == "blob" || lower == "bytea" || lower == "binary" {
        ColumnDataType::Blob
    } else if lower.contains("text") || lower.contains("clob") {
        ColumnDataType::Text
    } else {
        ColumnDataType::Varchar { length: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::CoreError;
    use crate::mock::generators::generate_cell;
    use crate::mock::models::GeneratorConfig;
    use crate::mock::models::Locale;

    #[test]
    fn test_sanitize_table_name_alphanumeric() {
        assert_eq!(sanitize_table_name("hello_world"), "hello_world");
    }

    #[test]
    fn test_sanitize_table_name_with_spaces() {
        assert_eq!(sanitize_table_name("my table"), "my_table");
    }

    #[test]
    fn test_sanitize_table_name_special_chars() {
        assert_eq!(sanitize_table_name("user-data@2024"), "user_data_2024");
    }

    #[test]
    fn test_build_create_table_ddl_single_column() {
        let cols = vec![ColumnDef {
            name: "id".to_string(),
            data_type: ColumnDataType::Integer,
            generator: GeneratorConfig::AutoIncrement { start: 1, step: 1 },
            nullable_ratio: 0.0,
            unique: true,
        }];
        let ddl = MockEngine::build_create_table_ddl("users", &cols);
        assert_eq!(ddl, "CREATE TABLE \"users\" (id INT UNIQUE NOT NULL)");
    }

    #[test]
    fn test_build_create_table_ddl_multi_column() {
        let cols = vec![
            ColumnDef {
                name: "id".to_string(),
                data_type: ColumnDataType::Integer,
                generator: GeneratorConfig::AutoIncrement { start: 1, step: 1 },
                nullable_ratio: 0.0,
                unique: true,
            },
            ColumnDef {
                name: "name".to_string(),
                data_type: ColumnDataType::Varchar { length: Some(100) },
                generator: GeneratorConfig::Name,
                nullable_ratio: 0.0,
                unique: false,
            },
        ];
        let ddl = MockEngine::build_create_table_ddl("users", &cols);
        assert_eq!(
            ddl,
            "CREATE TABLE \"users\" (id INT UNIQUE NOT NULL, name VARCHAR(100) NOT NULL)"
        );
    }

    #[test]
    fn test_generate_cell_auto_increment() {
        let mut rng = StdRng::seed_from_u64(42);
        let gen = GeneratorConfig::AutoIncrement {
            start: 100,
            step: 5,
        };
        let val = generate_cell(&gen, &mut rng, 3, &Locale::ZhCn);
        assert_eq!(val, "115");
    }

    #[test]
    fn test_generate_cell_random_int_range() -> Result<(), CoreError> {
        let mut rng = StdRng::seed_from_u64(42);
        let gen = GeneratorConfig::RandomInt { min: 10, max: 20 };
        let val: i64 = generate_cell(&gen, &mut rng, 0, &Locale::ZhCn)
            .parse()
            .map_err(|e| CoreError::from(format!("parse int error: {}", e)))?;
        assert!((10..=20).contains(&val));
        Ok(())
    }

    #[test]
    fn test_generate_cell_constant() {
        let mut rng = StdRng::seed_from_u64(42);
        let gen = GeneratorConfig::Constant {
            value: "hello".to_string(),
        };
        let val = generate_cell(&gen, &mut rng, 0, &Locale::ZhCn);
        assert_eq!(val, "hello");
    }

    #[test]
    fn test_generate_cell_boolean() {
        let mut rng = StdRng::seed_from_u64(42);
        let gen = GeneratorConfig::Boolean { ratio: 100 };
        let val = generate_cell(&gen, &mut rng, 0, &Locale::ZhCn);
        assert_eq!(val, "true");
    }

    #[test]
    fn test_infer_datatype_for_column_int() {
        let dt = infer_datatype_for_column("int");
        assert!(matches!(dt, ColumnDataType::Integer));
    }

    #[test]
    fn test_infer_datatype_for_column_varchar_size() {
        let dt = infer_datatype_for_column("varchar(255)");
        assert!(matches!(dt, ColumnDataType::Varchar { .. }));
    }

    #[test]
    fn test_infer_datatype_for_column_decimal() {
        let dt = infer_datatype_for_column("decimal(10,2)");
        assert!(matches!(dt, ColumnDataType::Decimal { .. }));
    }
}
