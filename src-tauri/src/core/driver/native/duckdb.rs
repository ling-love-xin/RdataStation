use std::sync::Arc;
use std::sync::Mutex;

use arrow::array::{ArrayRef, BinaryArray, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use duckdb::Connection;

use crate::core::driver::traits::MetadataBrowser;
use crate::core::driver::{ColumnDetail, DataSourceMeta, Database, Transaction};
use crate::core::error::{CoreError, DatabaseError};
use crate::core::models::{ArrowBatch, QueryResult};

/// DuckDB 数据库连接
pub struct DuckDbDatabase {
    conn: Arc<Mutex<Connection>>,
    server_version: Option<String>,
}

impl DuckDbDatabase {
    pub fn new(url: &str) -> Result<Self, CoreError> {
        let path = if url.starts_with("duckdb://") {
            url.trim_start_matches("duckdb://")
        } else {
            url
        };

        // 确保父目录存在
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    CoreError::database(DatabaseError::Driver {
                        db_type: "duckdb".to_string(),
                        operation: "create_directory".to_string(),
                        source: e.to_string(),
                    })
                })?;
            }
        }

        let conn = Connection::open(path).map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "connect".to_string(),
                source: e.to_string(),
            })
        })?;
        let server_version = conn
            .query_row("PRAGMA version", [], |row| row.get::<_, String>(0))
            .ok();
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            server_version,
        })
    }

    pub fn from_connection(conn: Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            server_version: None,
        }
    }
}

fn is_read_only_sql(sql: &str) -> bool {
    let sql_upper = sql.trim_start().to_uppercase();
    sql_upper.starts_with("SELECT")
        || sql_upper.starts_with("SHOW")
        || sql_upper.starts_with("DESCRIBE")
        || sql_upper.starts_with("EXPLAIN")
        || sql_upper.starts_with("PRAGMA")
}

#[async_trait::async_trait]
impl Database for DuckDbDatabase {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        let conn = self.conn.lock().map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            })
        })?;

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let columns: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|name| name.to_string())
            .collect();

        let mut rows = stmt
            .query([])
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let mut row_data: Vec<Vec<duckdb::types::Value>> = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?
        {
            let mut values: Vec<duckdb::types::Value> = Vec::with_capacity(columns.len());
            for (i, _) in columns.iter().enumerate() {
                let v = row.get::<usize, duckdb::types::Value>(i).map_err(|e| {
                    CoreError::database(DatabaseError::Driver {
                        db_type: "duckdb".to_string(),
                        operation: "row_parsing".to_string(),
                        source: e.to_string(),
                    })
                })?;
                values.push(v);
            }
            row_data.push(values);
        }

        let is_read_only = is_read_only_sql(sql);
        let row_count = row_data.len();

        let batch = if row_count > 0 {
            duckdb_rows_to_arrow(&columns, &row_data)?
        } else {
            return Ok(QueryResult {
                columns,
                batches: vec![],
                affected_rows: if is_read_only { None } else { Some(0) },
                is_read_only: Some(is_read_only),
            });
        };

        Ok(QueryResult {
            columns,
            batches: vec![batch],
            affected_rows: if is_read_only { None } else { Some(row_count) },
            is_read_only: Some(is_read_only),
        })
    }

    async fn query_with_cancel(
        &self,
        sql: &str,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<QueryResult, CoreError> {
        let conn = Arc::clone(&self.conn);
        let sql_owned = sql.to_string();
        let sql_for_error = sql.to_string();

        tokio::select! {
            result = tokio::task::spawn_blocking(move || {
                let conn = conn.lock().map_err(|e| CoreError::database(DatabaseError::Driver {
                    db_type: "duckdb".to_string(),
                    operation: "lock".to_string(),
                    source: e.to_string(),
                }))?;

                let mut stmt = conn.prepare(&sql_owned)
                    .map_err(|e| CoreError::database(DatabaseError::query(&sql_owned, e.to_string())))?;

                let columns: Vec<String> = stmt.column_names()
                    .iter()
                    .map(|name| name.to_string())
                    .collect();

                let mut rows = stmt.query([])
                    .map_err(|e| CoreError::database(DatabaseError::query(&sql_owned, e.to_string())))?;

                let mut row_data: Vec<Vec<duckdb::types::Value>> = Vec::new();
                while let Ok(Some(row)) = rows.next() {
                    let mut values: Vec<duckdb::types::Value> = Vec::with_capacity(columns.len());
                    for (i, _) in columns.iter().enumerate() {
                        let v = row.get::<usize, duckdb::types::Value>(i)
                            .map_err(|e| CoreError::database(DatabaseError::Driver {
                                db_type: "duckdb".to_string(),
                                operation: "row_parsing".to_string(),
                                source: e.to_string(),
                            }))?;
                        values.push(v);
                    }
                    row_data.push(values);
                }

                let is_read_only = is_read_only_sql(&sql_owned);
                let row_count = row_data.len();

                let batch = if row_count > 0 {
                    duckdb_rows_to_arrow(&columns, &row_data)?
                } else {
                    return Ok(QueryResult {
                        columns,
                        batches: vec![],
                        affected_rows: if is_read_only { None } else { Some(0) },
                        is_read_only: Some(is_read_only),
                    });
                };

                Ok(QueryResult {
                    columns,
                    batches: vec![batch],
                    affected_rows: if is_read_only { None } else { Some(row_count) },
                    is_read_only: Some(is_read_only),
                })
            }) => {
                result.map_err(|e| CoreError::database(DatabaseError::query(
                    &sql_for_error,
                    format!("Task panicked: {}", e),
                )))?
            }
            _ = cancel_token.cancelled() => {
                Err(CoreError::database(DatabaseError::Query {
                    sql: sql_for_error,
                    reason: "Query cancelled".to_string(),
                    position: None,
                }))
            }
        }
    }

    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, CoreError> {
        let conn = self.conn.lock().map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            })
        })?;

        conn.execute("BEGIN TRANSACTION", []).map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "begin_transaction".to_string(),
                source: e.to_string(),
            })
        })?;

        Ok(Box::new(DuckDbTransaction::new(Arc::clone(&self.conn))))
    }

    fn meta(&self) -> DataSourceMeta {
        DataSourceMeta {
            server_version: self.server_version.clone(),
            ..DataSourceMeta::duckdb()
        }
    }

    async fn ping(&self) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            })
        })?;
        conn.execute("SELECT 1", [])
            .map_err(|e| CoreError::database(DatabaseError::query("SELECT 1", e.to_string())))?;
        Ok(())
    }

    async fn list_databases(&self) -> Result<Vec<String>, CoreError> {
        Ok(vec!["main".to_string()])
    }

    async fn list_tables(
        &self,
        _db: &str,
        _schema: Option<&str>,
    ) -> Result<Vec<crate::core::driver::SchemaObject>, CoreError> {
        let nodes = self.get_tables("main", "main").await?;
        Ok(nodes
            .into_iter()
            .map(|n| crate::core::driver::SchemaObject {
                name: n.name,
                kind: n.kind,
                children: None,
                comment: n.comment,
            })
            .collect())
    }

    async fn list_columns(
        &self,
        _db: &str,
        _schema: Option<&str>,
        table: &str,
    ) -> Result<Vec<ColumnDetail>, CoreError> {
        let detail = self.get_table_detail("main", "main", table).await?;
        Ok(detail.columns)
    }

    async fn register_external_database(
        &self,
        name: &str,
        driver: &str,
        connection_string: &str,
    ) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            })
        })?;

        let sql = format!(
            "ATTACH '{}' AS {} (TYPE '{}')",
            connection_string, name, driver
        );
        conn.execute(&sql, []).map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "register_external_database".to_string(),
                source: e.to_string(),
            })
        })?;

        Ok(())
    }

    async fn create_external_table(
        &self,
        external_db_name: &str,
        schema_name: &str,
        table_name: &str,
        external_table_name: &str,
    ) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            })
        })?;

        let sql = format!(
            "CREATE EXTERNAL TABLE {}.{} AS SELECT * FROM {}.{}",
            schema_name, table_name, external_db_name, external_table_name
        );
        conn.execute(&sql, []).map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "create_external_table".to_string(),
                source: e.to_string(),
            })
        })?;

        Ok(())
    }
}

/// DuckDB 事务
pub struct DuckDbTransaction {
    conn: Arc<Mutex<Connection>>,
    committed: bool,
}

impl DuckDbTransaction {
    fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            conn,
            committed: false,
        }
    }
}

#[async_trait::async_trait]
impl Transaction for DuckDbTransaction {
    async fn query(&mut self, sql: &str) -> Result<QueryResult, CoreError> {
        let conn = self.conn.lock().map_err(|e| {
            CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            })
        })?;

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let columns: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|name| name.to_string())
            .collect();

        let mut rows = stmt
            .query([])
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let mut row_data: Vec<Vec<duckdb::types::Value>> = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let mut values: Vec<duckdb::types::Value> = Vec::with_capacity(columns.len());
            for (i, _) in columns.iter().enumerate() {
                let v = row.get::<usize, duckdb::types::Value>(i).map_err(|e| {
                    CoreError::database(DatabaseError::Driver {
                        db_type: "duckdb".to_string(),
                        operation: "row_parsing".to_string(),
                        source: e.to_string(),
                    })
                })?;
                values.push(v);
            }
            row_data.push(values);
        }

        let sql_upper = sql.trim_start().to_uppercase();
        let is_read_only = sql_upper.starts_with("SELECT")
            || sql_upper.starts_with("SHOW")
            || sql_upper.starts_with("DESCRIBE");
        let row_count = row_data.len();

        let batch = if row_count > 0 {
            duckdb_rows_to_arrow(&columns, &row_data)?
        } else {
            return Ok(QueryResult {
                columns,
                batches: vec![],
                affected_rows: if is_read_only { None } else { Some(0) },
                is_read_only: Some(is_read_only),
            });
        };

        Ok(QueryResult {
            columns,
            batches: vec![batch],
            affected_rows: if is_read_only { None } else { Some(row_count) },
            is_read_only: Some(is_read_only),
        })
    }

    async fn commit(&mut self) -> Result<(), CoreError> {
        if !self.committed {
            let conn = self.conn.lock().map_err(|e| {
                CoreError::database(DatabaseError::Driver {
                    db_type: "duckdb".to_string(),
                    operation: "lock".to_string(),
                    source: e.to_string(),
                })
            })?;

            conn.execute("COMMIT", []).map_err(|e| {
                CoreError::database(DatabaseError::Driver {
                    db_type: "duckdb".to_string(),
                    operation: "commit".to_string(),
                    source: e.to_string(),
                })
            })?;

            self.committed = true;
        }
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), CoreError> {
        if !self.committed {
            let conn = self.conn.lock().map_err(|e| {
                CoreError::database(DatabaseError::Driver {
                    db_type: "duckdb".to_string(),
                    operation: "lock".to_string(),
                    source: e.to_string(),
                })
            })?;

            if let Err(e) = conn.execute("ROLLBACK", []) {
                tracing::warn!("DuckDB transaction rollback error: {}", e);
            }
            self.committed = true;
        }
        Ok(())
    }
}

/// 将 DuckDB 行转换为 Arrow 批处理
pub fn duckdb_rows_to_arrow(
    columns: &[String],
    rows: &[Vec<duckdb::types::Value>],
) -> Result<ArrowBatch, CoreError> {
    let num_rows = rows.len();
    let num_cols = columns.len();

    let mut arrays: Vec<ArrayRef> = Vec::with_capacity(num_cols);

    for col_idx in 0..num_cols {
        let mut string_values: Vec<Option<String>> = Vec::with_capacity(num_rows);
        let mut int_values: Vec<Option<i64>> = Vec::with_capacity(num_rows);
        let mut float_values: Vec<Option<f64>> = Vec::with_capacity(num_rows);
        let mut bool_values: Vec<Option<bool>> = Vec::with_capacity(num_rows);
        let mut binary_values: Vec<Option<Vec<u8>>> = Vec::with_capacity(num_rows);

        let mut detected_type: Option<DataType> = None;

        for row in rows {
            if let Some(value) = row.get(col_idx) {
                match value {
                    duckdb::types::Value::Null => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Utf8);
                        }
                        string_values.push(None);
                        int_values.push(None);
                        float_values.push(None);
                        bool_values.push(None);
                        binary_values.push(None);
                    }
                    duckdb::types::Value::Boolean(b) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Boolean);
                        }
                        bool_values.push(Some(*b));
                    }
                    duckdb::types::Value::TinyInt(i) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Int64);
                        }
                        int_values.push(Some(*i as i64));
                    }
                    duckdb::types::Value::SmallInt(i) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Int64);
                        }
                        int_values.push(Some(*i as i64));
                    }
                    duckdb::types::Value::Int(i) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Int64);
                        }
                        int_values.push(Some(*i as i64));
                    }
                    duckdb::types::Value::BigInt(i) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Int64);
                        }
                        int_values.push(Some(*i));
                    }
                    duckdb::types::Value::UTinyInt(i) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Int64);
                        }
                        int_values.push(Some(*i as i64));
                    }
                    duckdb::types::Value::USmallInt(i) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Int64);
                        }
                        int_values.push(Some(*i as i64));
                    }
                    duckdb::types::Value::UInt(i) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Int64);
                        }
                        int_values.push(Some(*i as i64));
                    }
                    duckdb::types::Value::UBigInt(i) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Int64);
                        }
                        int_values.push(Some(*i as i64));
                    }
                    duckdb::types::Value::HugeInt(i) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Int64);
                        }
                        int_values.push(Some(*i as i64));
                    }
                    duckdb::types::Value::Float(f) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Float64);
                        }
                        float_values.push(Some(*f as f64));
                    }
                    duckdb::types::Value::Double(f) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Float64);
                        }
                        float_values.push(Some(*f));
                    }
                    duckdb::types::Value::Text(s) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Utf8);
                        }
                        string_values.push(Some(s.clone()));
                    }
                    duckdb::types::Value::Blob(b) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Binary);
                        }
                        binary_values.push(Some(b.clone()));
                    }
                    _ => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Utf8);
                        }
                        string_values.push(None);
                    }
                }
            }
        }

        let effective_type = match detected_type {
            Some(t) => t,
            None => DataType::Utf8,
        };

        let array: ArrayRef = match effective_type {
            DataType::Boolean => Arc::new(BooleanArray::from(bool_values)),
            DataType::Int64 => Arc::new(Int64Array::from(int_values)),
            DataType::Float64 => Arc::new(Float64Array::from(float_values)),
            DataType::Binary => {
                let refs: Vec<Option<&[u8]>> = binary_values
                    .iter()
                    .map(|opt| opt.as_ref().map(|v| v.as_slice()))
                    .collect();
                Arc::new(BinaryArray::from(refs))
            }
            _ => Arc::new(StringArray::from(string_values)),
        };

        arrays.push(array);
    }

    let fields: Vec<Field> = columns
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let data_type = arrays[i].data_type().clone();
            Field::new(name, data_type, true)
        })
        .collect();

    let schema = Arc::new(Schema::new(fields));

    RecordBatch::try_new(schema, arrays).map_err(|e| {
        CoreError::database(DatabaseError::Driver {
            db_type: "duckdb".to_string(),
            operation: "arrow_conversion".to_string(),
            source: e.to_string(),
        })
    })
}

#[async_trait::async_trait]
impl crate::core::driver::MetadataBrowser for DuckDbDatabase {
    async fn get_databases(&self) -> Result<Vec<crate::core::driver::NodeInfo>, CoreError> {
        Ok(vec![crate::core::driver::NodeInfo {
            name: "main".to_string(),
            kind: crate::core::driver::SchemaObjectKind::Database,
            icon: Some("database".to_string()),
            comment: None,
        }])
    }

    async fn get_schemas(
        &self,
        _db: &str,
    ) -> Result<Vec<crate::core::driver::NodeInfo>, CoreError> {
        Ok(vec![crate::core::driver::NodeInfo {
            name: "main".to_string(),
            kind: crate::core::driver::SchemaObjectKind::Schema,
            icon: Some("schema".to_string()),
            comment: None,
        }])
    }

    async fn get_tables(
        &self,
        _db: &str,
        _schema: &str,
    ) -> Result<Vec<crate::core::driver::NodeInfo>, CoreError> {
        let result = self.query("SELECT table_name, table_type FROM information_schema.tables WHERE table_schema = 'main' ORDER BY table_name").await?;
        let nodes: Vec<crate::core::driver::NodeInfo> = (0..result.total_rows())
            .filter_map(|row_idx| {
                result.batches.iter().find_map(|batch| {
                    if row_idx < batch.num_rows() {
                        if let (Some(name_arr), Some(type_arr)) = (
                            batch.column(0).as_any().downcast_ref::<StringArray>(),
                            batch.column(1).as_any().downcast_ref::<StringArray>(),
                        ) {
                            let table_type = type_arr.value(row_idx);
                            let kind = if table_type == "VIEW" {
                                crate::core::driver::SchemaObjectKind::View
                            } else {
                                crate::core::driver::SchemaObjectKind::Table
                            };
                            Some(crate::core::driver::NodeInfo {
                                name: name_arr.value(row_idx).to_string(),
                                kind,
                                icon: Some(if table_type == "VIEW" {
                                    "view".to_string()
                                } else {
                                    "table".to_string()
                                }),
                                comment: None,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .collect();
        Ok(nodes)
    }

    async fn get_table_detail(
        &self,
        _db: &str,
        _schema: &str,
        table: &str,
    ) -> Result<crate::core::driver::NodeDetail, CoreError> {
        let safe_table = table.replace('\'', "''");
        let sql = format!("SELECT column_name, data_type, is_nullable, column_default FROM information_schema.columns WHERE table_schema = 'main' AND table_name = '{}' ORDER BY ordinal_position", safe_table);
        let result = self.query(&sql).await?;
        let columns: Vec<crate::core::driver::ColumnDetail> = (0..result.total_rows())
            .filter_map(|row_idx| {
                result.batches.iter().find_map(|batch| {
                    if row_idx < batch.num_rows() {
                        let col_name = batch
                            .column(0)
                            .as_any()
                            .downcast_ref::<StringArray>()?
                            .value(row_idx);
                        let data_type = batch
                            .column(1)
                            .as_any()
                            .downcast_ref::<StringArray>()?
                            .value(row_idx);
                        let nullable = batch
                            .column(2)
                            .as_any()
                            .downcast_ref::<StringArray>()?
                            .value(row_idx)
                            == "YES";
                        let default = batch
                            .column(3)
                            .as_any()
                            .downcast_ref::<StringArray>()?
                            .value(row_idx);
                        Some(crate::core::driver::ColumnDetail {
                            name: col_name.to_string(),
                            data_type: data_type.to_string(),
                            nullable,
                            is_primary_key: false,
                            is_foreign_key: false,
                            default_value: if default.is_empty() {
                                None
                            } else {
                                Some(default.to_string())
                            },
                            comment: None,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(crate::core::driver::NodeDetail {
            node: crate::core::driver::NodeInfo {
                name: table.to_string(),
                kind: crate::core::driver::SchemaObjectKind::Table,
                icon: Some("table".to_string()),
                comment: None,
            },
            columns,
            index_count: None,
            row_count_estimate: None,
        })
    }
}
