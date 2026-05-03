use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use crate::core::driver::traits::{Database, Transaction, DataSourceMeta, SchemaObject};
use crate::core::error::{CoreError, CommonError};
use crate::core::models::QueryResult;

pub struct WasmDriver {
    meta: DataSourceMeta,
}

impl WasmDriver {
    pub fn new(_url: &str) -> Result<Self, CoreError> {
        Ok(Self {
            meta: DataSourceMeta {
                supports_transaction: false,
                supports_streaming: false,
                supports_arrow: true,
                supports_federated: false,
                supports_concurrent_write: false,
                is_in_memory: false,
            },
        })
    }
}

#[async_trait]
impl Database for WasmDriver {
    async fn query(&self, _sql: &str) -> Result<QueryResult, CoreError> {
        Err(CoreError::common(CommonError::NotSupported("WASM driver query not implemented yet".to_string())))
    }

    async fn query_with_cancel(
        &self,
        _sql: &str,
        _cancel_token: CancellationToken,
    ) -> Result<QueryResult, CoreError> {
        Err(CoreError::common(CommonError::NotSupported("WASM driver query_with_cancel not implemented yet".to_string())))
    }

    fn meta(&self) -> DataSourceMeta {
        self.meta.clone()
    }

    async fn list_databases(&self) -> Result<Vec<String>, CoreError> {
        Ok(vec![])
    }

    async fn list_schemas(&self, _db: &str) -> Result<Vec<String>, CoreError> {
        Ok(vec![])
    }

    async fn list_tables(&self, _db: &str, _schema: Option<&str>) -> Result<Vec<SchemaObject>, CoreError> {
        Ok(vec![])
    }

    async fn list_columns(&self, _db: &str, _schema: Option<&str>, _table: &str) -> Result<Vec<SchemaObject>, CoreError> {
        Ok(vec![])
    }

    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, CoreError> {
        Err(CoreError::common(CommonError::NotSupported("WASM driver transactions not implemented yet".to_string())))
    }
}
