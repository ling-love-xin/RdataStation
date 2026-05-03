use std::sync::Arc;
use std::sync::Mutex;

use rusqlite::Connection;
use arrow::array::{ArrayRef, StringArray, Int64Array, Float64Array, BooleanArray, BinaryArray};
use arrow::datatypes::{Field, Schema, DataType};
use arrow::record_batch::RecordBatch;

use crate::core::driver::{Database, Transaction, DataSourceMeta};
use crate::core::error::{CoreError, DatabaseError};
use crate::core::models::{QueryResult, Value, ArrowBatch};

/// SQLite 数据库连接
pub struct SqliteDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteDatabase {
    pub fn new(url: &str) -> Result<Self, CoreError> {
        let path = if url.starts_with("sqlite://") {
            url.trim_start_matches("sqlite://")
        } else {
            url
        };
        
        // 确保父目录存在
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| CoreError::database(DatabaseError::Driver {
                    db_type: "sqlite".to_string(),
                    operation: "create_directory".to_string(),
                    source: e.to_string(),
                }))?;
            }
        }
        
        let conn = Connection::open(path)
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "sqlite".to_string(),
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
impl Database for SqliteDatabase {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        self.query_with_params(sql, vec![]).await
    }

    async fn query_with_params(&self, sql: &str, params: Vec<Value>) -> Result<QueryResult, CoreError> {
        let conn = self.conn.lock().map_err(|e| CoreError::database(DatabaseError::Driver {
            db_type: "sqlite".to_string(),
            operation: "lock".to_string(),
            source: e.to_string(),
        }))?;

        let mut stmt = conn.prepare(sql)
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let columns: Vec<String> = stmt.column_names()
            .iter()
            .map(|name| name.to_string())
            .collect();

        let params_slice: Vec<rusqlite::types::Value> = params.iter()
            .map(|v| match v {
                Value::Null => rusqlite::types::Value::Null,
                Value::Bool(b) => rusqlite::types::Value::Integer(*b as i64),
                Value::Int(i) => rusqlite::types::Value::Integer(*i),
                Value::Float(f) => rusqlite::types::Value::Real(*f),
                Value::Text(s) => rusqlite::types::Value::Text(s.clone()),
                Value::Bytes(b) => rusqlite::types::Value::Blob(b.clone()),
            })
            .collect();

        let mut rows = match params_slice.len() {
            0 => stmt.query([]),
            1 => stmt.query([&params_slice[0]]),
            2 => stmt.query([&params_slice[0], &params_slice[1]]),
            3 => stmt.query([&params_slice[0], &params_slice[1], &params_slice[2]]),
            4 => stmt.query([&params_slice[0], &params_slice[1], &params_slice[2], &params_slice[3]]),
            5 => stmt.query([&params_slice[0], &params_slice[1], &params_slice[2], &params_slice[3], &params_slice[4]]),
            _ => {
                let params_refs: Vec<&dyn rusqlite::ToSql> = params_slice.iter()
                    .take(16)
                    .map(|v| v as &dyn rusqlite::ToSql)
                    .collect();
                stmt.query(rusqlite::params_from_iter(params_refs))
            }
        }
        .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let mut row_data: Vec<Vec<rusqlite::types::Value>> = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let values: Vec<rusqlite::types::Value> = columns.iter().enumerate()
                .map(|(i, _)| {
                    row.get::<usize, rusqlite::types::Value>(i).unwrap_or(rusqlite::types::Value::Null)
                })
                .collect();
            row_data.push(values);
        }

        let sql_upper = sql.trim_start().to_uppercase();
        let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("PRAGMA");
        let row_count = row_data.len();

        let batch = if row_count > 0 {
            sqlite_rows_to_arrow(&columns, &row_data)?
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
                    db_type: "sqlite".to_string(),
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

                let mut row_data: Vec<Vec<rusqlite::types::Value>> = Vec::new();
                while let Ok(Some(row)) = rows.next() {
                    let values: Vec<rusqlite::types::Value> = columns.iter().enumerate()
                        .map(|(i, _)| {
                            row.get::<usize, rusqlite::types::Value>(i).unwrap_or(rusqlite::types::Value::Null)
                        })
                        .collect();
                    row_data.push(values);
                }

                let sql_upper = sql_owned.trim_start().to_uppercase();
                let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("PRAGMA");
                let row_count = row_data.len();

                let batch = if row_count > 0 {
                    sqlite_rows_to_arrow(&columns, &row_data)?
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
            db_type: "sqlite".to_string(),
            operation: "lock".to_string(),
            source: e.to_string(),
        }))?;

        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "sqlite".to_string(),
                operation: "begin_transaction".to_string(),
                source: e.to_string(),
            }))?;

        Ok(Box::new(SqliteTransaction::new(Arc::clone(&self.conn))))
    }

    fn meta(&self) -> DataSourceMeta {
        DataSourceMeta::sqlite()
    }

    async fn list_tables(&self, _db: &str, _schema: Option<&str>) -> Result<Vec<crate::core::driver::SchemaObject>, CoreError> {
        let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'";
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
        let sql = format!("PRAGMA table_info({})", table);
        let result = self.query(&sql).await?;
        let columns: Vec<crate::core::driver::SchemaObject> = (0..result.total_rows())
            .filter_map(|row_idx| {
                result.batches.iter().find_map(|batch| {
                    if row_idx < batch.num_rows() {
                        if let Some(arr) = batch.column(1).as_any().downcast_ref::<StringArray>() {
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
}

/// SQLite 事务
pub struct SqliteTransaction {
    conn: Arc<Mutex<Connection>>,
    committed: bool,
}

impl SqliteTransaction {
    fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            conn,
            committed: false,
        }
    }
}

#[async_trait::async_trait]
impl Transaction for SqliteTransaction {
    async fn query(&mut self, sql: &str) -> Result<QueryResult, CoreError> {
        let conn = self.conn.lock().map_err(|e| CoreError::database(DatabaseError::Driver {
            db_type: "sqlite".to_string(),
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

        let mut row_data: Vec<Vec<rusqlite::types::Value>> = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let values: Vec<rusqlite::types::Value> = columns.iter().enumerate()
                .map(|(i, _)| {
                    row.get::<usize, rusqlite::types::Value>(i).unwrap_or(rusqlite::types::Value::Null)
                })
                .collect();
            row_data.push(values);
        }

        let sql_upper = sql.trim_start().to_uppercase();
        let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("PRAGMA");
        let row_count = row_data.len();

        let batch = if row_count > 0 {
            sqlite_rows_to_arrow(&columns, &row_data)?
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
                db_type: "sqlite".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            }))?;

            conn.execute("COMMIT", [])
                .map_err(|e| CoreError::database(DatabaseError::Driver {
                    db_type: "sqlite".to_string(),
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
                db_type: "sqlite".to_string(),
                operation: "lock".to_string(),
                source: e.to_string(),
            }))?;

            let _ = conn.execute("ROLLBACK", []);
            self.committed = true;
        }
        Ok(())
    }
}

/// 将 SQLite 行转换为 Arrow 批处理
fn sqlite_rows_to_arrow(
    columns: &[String],
    rows: &[Vec<rusqlite::types::Value>],
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
            // 每一行必须向所有 vector 插入，否则各列长度不一致
            if let Some(value) = row.get(col_idx) {
                match value {
                    rusqlite::types::Value::Null => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Utf8);
                        }
                        string_values.push(None);
                        int_values.push(None);
                        float_values.push(None);
                        bool_values.push(None);
                        binary_values.push(None);
                    }
                    rusqlite::types::Value::Integer(i) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Int64);
                        }
                        string_values.push(None);
                        int_values.push(Some(*i));
                        float_values.push(None);
                        bool_values.push(None);
                        binary_values.push(None);
                    }
                    rusqlite::types::Value::Real(f) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Float64);
                        }
                        string_values.push(None);
                        int_values.push(None);
                        float_values.push(Some(*f));
                        bool_values.push(None);
                        binary_values.push(None);
                    }
                    rusqlite::types::Value::Text(s) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Utf8);
                        }
                        string_values.push(Some(s.clone()));
                        int_values.push(None);
                        float_values.push(None);
                        bool_values.push(None);
                        binary_values.push(None);
                    }
                    rusqlite::types::Value::Blob(b) => {
                        if detected_type.is_none() {
                            detected_type = Some(DataType::Binary);
                        }
                        string_values.push(None);
                        int_values.push(None);
                        float_values.push(None);
                        bool_values.push(None);
                        binary_values.push(Some(b.clone()));
                    }
                }
            } else {
                // row.get() 返回 None（列不存在于该行）
                string_values.push(None);
                int_values.push(None);
                float_values.push(None);
                bool_values.push(None);
                binary_values.push(None);
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
            db_type: "sqlite".to_string(),
            operation: "arrow_conversion".to_string(),
            source: e.to_string(),
        }))
}
