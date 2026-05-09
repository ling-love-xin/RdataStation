use sqlx::{MySql, Pool, Row, Column};
use arrow::array::{ArrayRef, StringArray, Int64Array, Float64Array, BooleanArray, BinaryArray};
use arrow::datatypes::{Field, Schema, DataType};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

use crate::core::driver::{Database, Transaction, DataSourceMeta, ColumnDetail};
use crate::core::driver::traits::MetadataBrowser;
use crate::core::error::{CoreError, DatabaseError};
use crate::core::models::{QueryResult, ArrowBatch};

/// MySQL 数据库连接
pub struct MySqlDatabase {
    pool: Pool<MySql>,
    server_version: Option<String>,
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
        let server_version = sqlx::query_scalar::<_, String>("SELECT VERSION()")
            .fetch_one(&pool)
            .await
            .ok();
        Ok(Self { pool, server_version })
    }

    pub fn from_pool(pool: Pool<MySql>) -> Self {
        Self { pool, server_version: None }
    }
}

fn is_read_only_sql(sql: &str) -> bool {
    let sql_upper = sql.trim_start().to_uppercase();
    sql_upper.starts_with("SELECT")
        || sql_upper.starts_with("SHOW")
        || sql_upper.starts_with("DESCRIBE")
        || sql_upper.starts_with("EXPLAIN")
        || sql_upper.starts_with("SET")
}

#[async_trait::async_trait]
impl Database for MySqlDatabase {
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError> {
        let is_read_only = is_read_only_sql(sql);

        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::database(DatabaseError::query(sql, e.to_string())))?;

        let columns: Vec<String> = if let Some(first) = rows.first() {
            first.columns().iter().map(|c| c.name().to_string()).collect()
        } else {
            vec![]
        };

        build_query_result(&columns, &rows, is_read_only)
    }

    async fn query_with_params(&self, sql: &str, params: Vec<crate::core::models::Value>) -> Result<QueryResult, CoreError> {
        let is_read_only = is_read_only_sql(sql);

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

        let columns: Vec<String> = if let Some(first) = rows.first() {
            first.columns().iter().map(|c| c.name().to_string()).collect()
        } else {
            vec![]
        };

        build_query_result(&columns, &rows, is_read_only)
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
                let is_read_only = is_read_only_sql(&sql_owned);

                let rows = sqlx::query(&sql_owned)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| CoreError::database(DatabaseError::query(&sql_owned, e.to_string())))?;

                let columns: Vec<String> = if let Some(first) = rows.first() {
                    first.columns().iter().map(|c| c.name().to_string()).collect()
                } else {
                    vec![]
                };

                build_query_result(&columns, &rows, is_read_only)
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
        DataSourceMeta {
            server_version: self.server_version.clone(),
            ..DataSourceMeta::mysql()
        }
    }

    async fn list_databases(&self) -> Result<Vec<String>, CoreError> {
        self.get_databases().await.map(|nodes| {
            nodes.into_iter().map(|n| n.name).collect()
        })
    }

    async fn list_tables(&self, db: &str, _schema: Option<&str>) -> Result<Vec<crate::core::driver::SchemaObject>, CoreError> {
        let nodes = self.get_tables(db, db).await?;
        Ok(nodes.into_iter().map(|n| {
            crate::core::driver::SchemaObject {
                name: n.name,
                kind: n.kind,
                children: None,
                comment: n.comment,
            }
        }).collect())
    }

    async fn list_columns(&self, db: &str, _schema: Option<&str>, table: &str) -> Result<Vec<ColumnDetail>, CoreError> {
        let detail = self.get_table_detail(db, db, table).await?;
        Ok(detail.columns)
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
                affected_rows: if is_read_only { None } else { Some(rows.len()) },
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
            if let Err(e) = tx.rollback().await {
                eprintln!("MySQL transaction rollback error: {}", e);
            }
        }
        Ok(())
    }
}

fn build_query_result(
    columns: &[String],
    rows: &[sqlx::mysql::MySqlRow],
    is_read_only: bool,
) -> Result<QueryResult, CoreError> {
    if rows.is_empty() {
        return Ok(QueryResult {
            columns: columns.to_vec(),
            batches: vec![],
            affected_rows: if is_read_only { None } else { Some(0) },
            is_read_only: Some(is_read_only),
        });
    }
    let batch = mysql_rows_to_arrow(columns, rows)?;
    Ok(QueryResult {
        columns: columns.to_vec(),
        batches: vec![batch],
        affected_rows: if is_read_only { None } else { Some(rows.len()) },
        is_read_only: Some(is_read_only),
    })
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

            if let Ok(Some(_)) = row.try_get::<Option<bool>, _>(col_idx) {
                detected_type = Some(DataType::Boolean);
                break;
            }
            if let Ok(Some(_)) = row.try_get::<Option<i64>, _>(col_idx) {
                detected_type = Some(DataType::Int64);
                break;
            }
            if let Ok(Some(_)) = row.try_get::<Option<f64>, _>(col_idx) {
                detected_type = Some(DataType::Float64);
                break;
            }
            if let Ok(Some(_)) = row.try_get::<Option<Vec<u8>>, _>(col_idx) {
                detected_type = Some(DataType::Binary);
                break;
            }
            if let Ok(Some(_)) = row.try_get::<Option<String>, _>(col_idx) {
                detected_type = Some(DataType::Utf8);
                break;
            }
        }

        let effective_type = detected_type.clone().unwrap_or(DataType::Utf8);

        for row in rows {
            use sqlx::Row;

            match effective_type {
                DataType::Boolean => {
                    bool_values.push(row.try_get::<Option<bool>, _>(col_idx).ok().flatten());
                }
                DataType::Int64 => {
                    int_values.push(row.try_get::<Option<i64>, _>(col_idx).ok().flatten());
                }
                DataType::Float64 => {
                    float_values.push(row.try_get::<Option<f64>, _>(col_idx).ok().flatten());
                }
                DataType::Binary => {
                    binary_values.push(row.try_get::<Option<Vec<u8>>, _>(col_idx).ok().flatten());
                }
                _ => {
                    string_values.push(row.try_get::<Option<String>, _>(col_idx).ok().flatten());
                }
            }
        }

        let array: ArrayRef = match effective_type {
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

#[async_trait::async_trait]
impl crate::core::driver::MetadataBrowser for MySqlDatabase {
    async fn get_databases(&self) -> Result<Vec<crate::core::driver::NodeInfo>, CoreError> {
        let result = self.query("SELECT schema_name FROM information_schema.schemata ORDER BY schema_name").await?;
        let nodes: Vec<crate::core::driver::NodeInfo> = (0..result.total_rows())
            .filter_map(|row_idx| {
                result.batches.iter().find_map(|batch| {
                    if row_idx < batch.num_rows() {
                        if let Some(arr) = batch.column(0).as_any().downcast_ref::<StringArray>() {
                            Some(crate::core::driver::NodeInfo {
                                name: arr.value(row_idx).to_string(),
                                kind: crate::core::driver::SchemaObjectKind::Database,
                                icon: Some("database".to_string()),
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

    async fn get_schemas(&self, _db: &str) -> Result<Vec<crate::core::driver::NodeInfo>, CoreError> {
        self.get_databases().await
    }

    async fn get_tables(&self, db: &str, _schema: &str) -> Result<Vec<crate::core::driver::NodeInfo>, CoreError> {
        let sql = format!("SELECT table_name, table_type FROM information_schema.tables WHERE table_schema = '{}' ORDER BY table_name", db);
        let result = self.query(&sql).await?;
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
                                icon: Some(if table_type == "VIEW" { "view".to_string() } else { "table".to_string() }),
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

    async fn get_table_detail(&self, db: &str, _schema: &str, table: &str) -> Result<crate::core::driver::NodeDetail, CoreError> {
        let sql = format!(
            "SELECT column_name, data_type, is_nullable, column_key, column_default, column_comment \
             FROM information_schema.columns \
             WHERE table_schema = '{}' AND table_name = '{}' \
             ORDER BY ordinal_position",
            db, table
        );
        let result = self.query(&sql).await?;
        let columns: Vec<crate::core::driver::ColumnDetail> = (0..result.total_rows())
            .filter_map(|row_idx| {
                result.batches.iter().find_map(|batch| {
                    if row_idx < batch.num_rows() {
                        let col_name = batch.column(0).as_any().downcast_ref::<StringArray>()?.value(row_idx);
                        let data_type = batch.column(1).as_any().downcast_ref::<StringArray>()?.value(row_idx);
                        let nullable = batch.column(2).as_any().downcast_ref::<StringArray>()?.value(row_idx) == "YES";
                        let pk = batch.column(3).as_any().downcast_ref::<StringArray>()?.value(row_idx);
                        let default = batch.column(4).as_any().downcast_ref::<StringArray>()?.value(row_idx);
                        let comment = batch.column(5).as_any().downcast_ref::<StringArray>()?.value(row_idx);
                        Some(crate::core::driver::ColumnDetail {
                            name: col_name.to_string(),
                            data_type: data_type.to_string(),
                            nullable,
                            is_primary_key: pk == "PRI",
                            is_foreign_key: pk == "MUL",
                            default_value: if default.is_empty() { None } else { Some(default.to_string()) },
                            comment: if comment.is_empty() { None } else { Some(comment.to_string()) },
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
