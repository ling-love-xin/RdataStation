use std::sync::Arc;
use std::sync::Mutex;

use duckdb::Connection;
use arrow::array::{ArrayRef, StringArray, Int64Array, Float64Array, BooleanArray, BinaryArray};
use arrow::datatypes::{Field, Schema, DataType};
use arrow::record_batch::RecordBatch;

use crate::core::driver::{Database, Transaction, DataSourceMeta};
use crate::core::error::{CoreError, DatabaseError};
use crate::core::models::{QueryResult, ArrowBatch};

/// DuckDB 数据库连接
pub struct DuckDbDatabase {
    conn: Arc<Mutex<Connection>>,
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
                std::fs::create_dir_all(parent).map_err(|e| CoreError::database(DatabaseError::Driver {
                    db_type: "duckdb".to_string(),
                    operation: "create_directory".to_string(),
                    source: e.to_string(),
                }))?;
            }
        }
        
        let conn = Connection::open(path)
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "connect".to_string(),
                source: e.to_string(),
            }))?;
        Ok(Self { 
            conn: Arc::new(Mutex::new(conn)) 
        })
    }

    pub fn from_connection(conn: Connection) -> Self {
        Self { 
            conn: Arc::new(Mutex::new(conn)) 
        }
    }
}

#[async_trait::async_trait]
impl Database for DuckDbDatabase {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        let conn = self.conn.lock().map_err(|e| CoreError::database(DatabaseError::Driver {
            db_type: "duckdb".to_string(),
            operation: "lock".to_string(),
            source: e.to_string(),
        }))?;

        let mut stmt = conn.prepare(sql)
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let columns: Vec<String> = stmt.column_names()
            .iter()
            .map(|name| name.to_string())
            .collect();

        let mut rows = stmt.query([])
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let mut row_data: Vec<Vec<duckdb::types::Value>> = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let values: Vec<duckdb::types::Value> = columns.iter().enumerate()
                .map(|(i, _)| {
                    row.get::<usize, duckdb::types::Value>(i).unwrap_or(duckdb::types::Value::Null)
                })
                .collect();
            row_data.push(values);
        }

        let sql_upper = sql.trim_start().to_uppercase();
        let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("SHOW") || sql_upper.starts_with("DESCRIBE");
        let row_count = row_data.len();

        let batch = if row_count > 0 {
            duckdb_rows_to_arrow(&columns, &row_data)?
        } else {
            return Ok(QueryResult {
                columns,
                batches: vec![],
                affected_rows: if is_read_only { Some(0) } else { None },
                is_read_only: Some(is_read_only),
            });
        };

        Ok(QueryResult {
            columns,
            batches: vec![batch],
            affected_rows: if is_read_only { Some(row_count) } else { None },
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
                    let values: Vec<duckdb::types::Value> = columns.iter().enumerate()
                        .map(|(i, _)| {
                            row.get::<usize, duckdb::types::Value>(i).unwrap_or(duckdb::types::Value::Null)
                        })
                        .collect();
                    row_data.push(values);
                }

                let sql_upper = sql_owned.trim_start().to_uppercase();
                let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("SHOW") || sql_upper.starts_with("DESCRIBE");
                let row_count = row_data.len();

                let batch = if row_count > 0 {
                    duckdb_rows_to_arrow(&columns, &row_data)?
                } else {
                    return Ok(QueryResult {
                        columns,
                        batches: vec![],
                        affected_rows: if is_read_only { Some(0) } else { None },
                        is_read_only: Some(is_read_only),
                    });
                };

                Ok(QueryResult {
                    columns,
                    batches: vec![batch],
                    affected_rows: if is_read_only { Some(row_count) } else { None },
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
        let conn = self.conn.lock().map_err(|e| CoreError::database(DatabaseError::Driver {
            db_type: "duckdb".to_string(),
            operation: "lock".to_string(),
            source: e.to_string(),
        }))?;

        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "begin_transaction".to_string(),
                source: e.to_string(),
            }))?;

        Ok(Box::new(DuckDbTransaction::new(Arc::clone(&self.conn))))
    }

    fn meta(&self) -> DataSourceMeta {
        DataSourceMeta::duckdb()
    }

    async fn list_tables(&self, _db: &str, _schema: Option<&str>) -> Result<Vec<crate::core::driver::SchemaObject>, CoreError> {
        let sql = "SHOW TABLES";
        let result = self.query(sql).await?;
        let tables: Vec<crate::core::driver::SchemaObject> = (0..result.total_rows())
            .filter_map(|row_idx| {
                result.batches.iter().find_map(|batch| {
                    if row_idx < batch.num_rows() {
                        if let Some(arr) = batch.column(0).as_any().downcast_ref::<StringArray>() {
                            Some(crate::core::driver::SchemaObject {
                                name: arr.value(row_idx).to_string(),
                                kind: crate::core::driver::SchemaObjectKind::Table,
                                children: None,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .filter(|obj| !obj.name.is_empty())
            .collect();
        Ok(tables)
    }

    async fn list_columns(&self, _db: &str, _schema: Option<&str>, table: &str) -> Result<Vec<crate::core::driver::SchemaObject>, CoreError> {
        let sql = format!("DESCRIBE {}", table);
        let result = self.query(&sql).await?;
        let columns: Vec<crate::core::driver::SchemaObject> = (0..result.total_rows())
            .filter_map(|row_idx| {
                result.batches.iter().find_map(|batch| {
                    if row_idx < batch.num_rows() {
                        if let Some(arr) = batch.column(0).as_any().downcast_ref::<StringArray>() {
                            Some(crate::core::driver::SchemaObject {
                                name: arr.value(row_idx).to_string(),
                                kind: crate::core::driver::SchemaObjectKind::Column,
                                children: None,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .filter(|obj| !obj.name.is_empty())
            .collect();
        Ok(columns)
    }

    async fn register_external_database(
        &self,
        name: &str,
        driver: &str,
        connection_string: &str
    ) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|e| CoreError::database(DatabaseError::Driver {
            db_type: "duckdb".to_string(),
            operation: "lock".to_string(),
            source: e.to_string(),
        }))?;

        let sql = format!("ATTACH '{}' AS {} (TYPE '{}')", connection_string, name, driver);
        conn.execute(&sql, [])
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "register_external_database".to_string(),
                source: e.to_string(),
            }))?;

        Ok(())
    }

    async fn create_external_table(
        &self,
        external_db_name: &str,
        schema_name: &str,
        table_name: &str,
        external_table_name: &str
    ) -> Result<(), CoreError> {
        let conn = self.conn.lock().map_err(|e| CoreError::database(DatabaseError::Driver {
            db_type: "duckdb".to_string(),
            operation: "lock".to_string(),
            source: e.to_string(),
        }))?;

        let sql = format!(
            "CREATE EXTERNAL TABLE {}.{} AS SELECT * FROM {}.{}",
            schema_name, table_name, external_db_name, external_table_name
        );
        conn.execute(&sql, [])
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "create_external_table".to_string(),
                source: e.to_string(),
            }))?;

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
        let conn = self.conn.lock().map_err(|e| CoreError::database(DatabaseError::Driver {
            db_type: "duckdb".to_string(),
            operation: "lock".to_string(),
            source: e.to_string(),
        }))?;

        let mut stmt = conn.prepare(sql)
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let columns: Vec<String> = stmt.column_names()
            .iter()
            .map(|name| name.to_string())
            .collect();

        let mut rows = stmt.query([])
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let mut row_data: Vec<Vec<duckdb::types::Value>> = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let values: Vec<duckdb::types::Value> = columns.iter().enumerate()
                .map(|(i, _)| {
                    row.get::<usize, duckdb::types::Value>(i).unwrap_or(duckdb::types::Value::Null)
                })
                .collect();
            row_data.push(values);
        }

        let sql_upper = sql.trim_start().to_uppercase();
        let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("SHOW") || sql_upper.starts_with("DESCRIBE");
        let row_count = row_data.len();

        let batch = if row_count > 0 {
            duckdb_rows_to_arrow(&columns, &row_data)?
        } else {
            return Ok(QueryResult {
                columns,
                batches: vec![],
                affected_rows: if is_read_only { Some(0) } else { None },
                is_read_only: Some(is_read_only),
            });
        };

        Ok(QueryResult {
            columns,
            batches: vec![batch],
            affected_rows: if is_read_only { Some(row_count) } else { None },
            is_read_only: Some(is_read_only),
        })
    }

    async fn commit(&mut self) -> Result<(), CoreError> {
        if !self.committed {
            let conn = self.conn.lock().map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            }))?;

            conn.execute("COMMIT", [])
                .map_err(|e| CoreError::database(DatabaseError::Driver {
                    db_type: "duckdb".to_string(),
                    operation: "commit".to_string(),
                    source: e.to_string(),
                }))?;

            self.committed = true;
        }
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), CoreError> {
        if !self.committed {
            let conn = self.conn.lock().map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "duckdb".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            }))?;

            let _ = conn.execute("ROLLBACK", []);
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

        let array: ArrayRef = match detected_type.unwrap_or(DataType::Utf8) {
            DataType::Boolean => {
                Arc::new(BooleanArray::from(bool_values))
            }
            DataType::Int64 => {
                Arc::new(Int64Array::from(int_values))
            }
            DataType::Float64 => {
                Arc::new(Float64Array::from(float_values))
            }
            DataType::Binary => {
                let refs: Vec<Option<&[u8]>> = binary_values.iter().map(|opt| opt.as_ref().map(|v| v.as_slice())).collect();
                Arc::new(BinaryArray::from(refs))
            }
            _ => {
                Arc::new(StringArray::from(string_values))
            }
        };

        arrays.push(array);
    }

    let fields: Vec<Field> = columns.iter()
        .enumerate()
        .map(|(i, name)| {
            let data_type = arrays[i].data_type().clone();
            Field::new(name, data_type, true)
        })
        .collect();

    let schema = Arc::new(Schema::new(fields));

    RecordBatch::try_new(schema, arrays)
        .map_err(|e| CoreError::database(DatabaseError::Driver {
            db_type: "duckdb".to_string(),
            operation: "arrow_conversion".to_string(),
            source: e.to_string(),
        }))
}
