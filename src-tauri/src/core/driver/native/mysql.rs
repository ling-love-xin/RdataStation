use sqlx::{MySql, Pool, Row, Column};
use arrow::array::{ArrayRef, StringArray, Int64Array, Float64Array, BooleanArray, BinaryArray};
use arrow::datatypes::{Field, Schema, DataType};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

use crate::core::driver::{Database, Transaction, DataSourceMeta};
use crate::core::error::{CoreError, DatabaseError};
use crate::core::models::{QueryResult, ArrowBatch};

/// MySQL 数据库连接
pub struct MySqlDatabase {
    pool: Pool<MySql>,
}

impl MySqlDatabase {
    pub async fn new(url: &str) -> Result<Self, CoreError> {
        let pool = Pool::connect(url)
            .await
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "mysql".to_string(),
                operation: "connect".to_string(),
                source: e.to_string(),
            }))?;
        Ok(Self { pool })
    }

    pub fn from_pool(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Database for MySqlDatabase {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        let sql_upper = sql.trim_start().to_uppercase();
        let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("SHOW") || sql_upper.starts_with("DESCRIBE");

        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: vec![],
                batches: vec![],
                affected_rows: None,
                is_read_only: Some(is_read_only),
            });
        }

        let columns: Vec<String> = rows[0].columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();

        let batch = mysql_rows_to_arrow(&columns, &rows)?;

        Ok(QueryResult {
            columns,
            batches: vec![batch],
            affected_rows: if is_read_only { Some(rows.len()) } else { None },
            is_read_only: Some(is_read_only),
        })
    }

    async fn query_with_params(&self, sql: &str, params: Vec<crate::core::models::Value>) -> Result<QueryResult, CoreError> {
        let sql_upper = sql.trim_start().to_uppercase();
        let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("SHOW") || sql_upper.starts_with("DESCRIBE");

        let mut query_builder = sqlx::query(sql);
        
        for param in &params {
            query_builder = match param {
                crate::core::models::Value::Null => query_builder.bind(None::<String>),
                crate::core::models::Value::Bool(v) => query_builder.bind(*v),
                crate::core::models::Value::Int(v) => query_builder.bind(*v),
                crate::core::models::Value::Float(v) => query_builder.bind(*v),
                crate::core::models::Value::Text(v) => query_builder.bind(v),
                crate::core::models::Value::Bytes(v) => query_builder.bind(v.clone()),
            };
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: vec![],
                batches: vec![],
                affected_rows: None,
                is_read_only: Some(is_read_only),
            });
        }

        let columns: Vec<String> = rows[0].columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();

        let batch = mysql_rows_to_arrow(&columns, &rows)?;

        Ok(QueryResult {
            columns,
            batches: vec![batch],
            affected_rows: if is_read_only { Some(rows.len()) } else { None },
            is_read_only: Some(is_read_only),
        })
    }

    async fn query_with_cancel(
        &self,
        sql: &str,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<QueryResult, CoreError> {
        let sql_owned = sql.to_string();
        let sql_for_cancel = sql.to_string();
        let pool = self.pool.clone();

        tokio::select! {
            result = async move {
                let sql_upper = sql_owned.trim_start().to_uppercase();
                let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("SHOW") || sql_upper.starts_with("DESCRIBE");

                let rows = sqlx::query(&sql_owned)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| CoreError::database(DatabaseError::query(&sql_owned, e.to_string())))?;

                if rows.is_empty() {
                    return Ok(QueryResult {
                        columns: vec![],
                        batches: vec![],
                        affected_rows: None,
                        is_read_only: Some(is_read_only),
                    });
                }

                let columns: Vec<String> = rows[0].columns()
                    .iter()
                    .map(|c| c.name().to_string())
                    .collect();

                let batch = mysql_rows_to_arrow(&columns, &rows)?;

                Ok(QueryResult {
                    columns,
                    batches: vec![batch],
                    affected_rows: if is_read_only { Some(rows.len()) } else { None },
                    is_read_only: Some(is_read_only),
                })
            } => result,
            _ = cancel_token.cancelled() => {
                Err(CoreError::database(DatabaseError::Query {
                    sql: sql_for_cancel,
                    reason: "Query cancelled".to_string(),
                    position: None,
                }))
            }
        }
    }

    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, CoreError> {
        let tx = self.pool.begin()
            .await
            .map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "mysql".to_string(),
                operation: "begin_transaction".to_string(),
                source: e.to_string(),
            }))?;
        Ok(Box::new(MySqlTransaction::new(tx)))
    }

    fn meta(&self) -> DataSourceMeta {
        DataSourceMeta::mysql()
    }

    async fn list_databases(&self) -> Result<Vec<String>, CoreError> {
        let result = self.query("SHOW DATABASES").await?;
        let databases: Vec<String> = (0..result.total_rows())
            .filter_map(|row_idx| {
                result.batches.iter().find_map(|batch| {
                    if row_idx < batch.num_rows() {
                        if let Some(arr) = batch.column(0).as_any().downcast_ref::<StringArray>() {
                            Some(arr.value(row_idx).to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .filter(|db| !db.is_empty())
            .collect();
        Ok(databases)
    }

    async fn list_tables(&self, db: &str, _schema: Option<&str>) -> Result<Vec<crate::core::driver::SchemaObject>, CoreError> {
        let sql = format!("SHOW TABLES FROM {}", db);
        let result = self.query(&sql).await?;
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

    async fn list_columns(&self, db: &str, _schema: Option<&str>, table: &str) -> Result<Vec<crate::core::driver::SchemaObject>, CoreError> {
        let sql = format!("SHOW COLUMNS FROM {}.{}", db, table);
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
}

/// MySQL 事务
pub struct MySqlTransaction {
    tx: Option<sqlx::Transaction<'static, MySql>>,
}

impl MySqlTransaction {
    fn new(tx: sqlx::Transaction<'static, MySql>) -> Self {
        Self { tx: Some(tx) }
    }
}

#[async_trait::async_trait]
impl Transaction for MySqlTransaction {
    async fn query(&mut self, sql: &str) -> Result<QueryResult, CoreError> {
        if let Some(ref mut tx) = self.tx {
            let sql_upper = sql.trim_start().to_uppercase();
            let is_read_only = sql_upper.starts_with("SELECT") || sql_upper.starts_with("SHOW") || sql_upper.starts_with("DESCRIBE");

            let rows = sqlx::query(sql)
                .fetch_all(&mut **tx)
                .await
                .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

            if rows.is_empty() {
                return Ok(QueryResult {
                    columns: vec![],
                    batches: vec![],
                    affected_rows: None,
                    is_read_only: Some(is_read_only),
                });
            }

            let columns: Vec<String> = rows[0].columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect();

            let batch = mysql_rows_to_arrow(&columns, &rows)?;

            Ok(QueryResult {
                columns,
                batches: vec![batch],
                affected_rows: if is_read_only { Some(rows.len()) } else { None },
                is_read_only: Some(is_read_only),
            })
        } else {
            Err(CoreError::database(DatabaseError::Driver {
                db_type: "mysql".to_string(),
                operation: "query".to_string(),
                source: "Transaction already closed".to_string(),
            }))
        }
    }

    async fn commit(&mut self) -> Result<(), CoreError> {
        if let Some(tx) = self.tx.take() {
            tx.commit().await.map_err(|e| CoreError::database(DatabaseError::Driver {
                db_type: "mysql".to_string(),
                operation: "commit".to_string(),
                source: e.to_string(),
            }))?;
        }
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), CoreError> {
        if let Some(tx) = self.tx.take() {
            let _ = tx.rollback().await;
        }
        Ok(())
    }
}

/// 将 MySQL 行转换为 Arrow 批处理
fn mysql_rows_to_arrow(
    columns: &[String],
    rows: &[sqlx::mysql::MySqlRow],
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
            use sqlx::Row;
            
            if let Ok(v) = row.try_get::<Option<bool>, _>(col_idx) {
                if detected_type.is_none() {
                    detected_type = Some(DataType::Boolean);
                }
                bool_values.push(v);
            } else if let Ok(v) = row.try_get::<Option<i64>, _>(col_idx) {
                if detected_type.is_none() {
                    detected_type = Some(DataType::Int64);
                }
                int_values.push(v);
            } else if let Ok(v) = row.try_get::<Option<f64>, _>(col_idx) {
                if detected_type.is_none() {
                    detected_type = Some(DataType::Float64);
                }
                float_values.push(v);
            } else if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(col_idx) {
                if detected_type.is_none() {
                    detected_type = Some(DataType::Binary);
                }
                binary_values.push(v);
            } else if let Ok(v) = row.try_get::<Option<String>, _>(col_idx) {
                if detected_type.is_none() {
                    detected_type = Some(DataType::Utf8);
                }
                string_values.push(v);
            } else {
                if detected_type.is_none() {
                    detected_type = Some(DataType::Utf8);
                }
                string_values.push(None);
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
            db_type: "mysql".to_string(),
            operation: "arrow_conversion".to_string(),
            source: e.to_string(),
        }))
}
