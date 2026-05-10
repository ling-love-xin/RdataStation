use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::error::{CoreError, DatabaseError};
use crate::core::models::{QueryResult, Value};

/// Schema 对象类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemaObjectKind {
    Database,
    Schema,
    Table,
    View,
    Column,
    Index,
    PrimaryKey,
    ForeignKey,
    Procedure,
    Function,
}

/// Schema 对象（对象树模型）
///
/// 前端友好的统一结构，支持懒加载（children = None 表示未加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaObject {
    pub name: String,
    pub kind: SchemaObjectKind,
    pub children: Option<Vec<SchemaObject>>,
    pub comment: Option<String>,
}

/// 列详情（完整元数据）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDetail {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub default_value: Option<String>,
    pub comment: Option<String>,
}

/// 对象树节点（轻量级，用于快速树渲染）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub kind: SchemaObjectKind,
    pub icon: Option<String>,
    pub comment: Option<String>,
}

/// 对象详情（完整元数据，按需加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDetail {
    pub node: NodeInfo,
    pub columns: Vec<ColumnDetail>,
    pub index_count: Option<usize>,
    pub row_count_estimate: Option<u64>,
}

/// 元数据浏览器 trait
///
/// 提供统一的对象树导航能力，适用于所有数据库类型（关系型、NoSQL、图等）。
/// 与 Database trait 分离，支持按需实现。
#[async_trait::async_trait]
pub trait MetadataBrowser: Send + Sync {
    /// 获取顶层节点（数据库/Catalog）
    async fn get_databases(&self) -> Result<Vec<NodeInfo>, CoreError>;

    /// 获取 Schema 列表
    async fn get_schemas(&self, db: &str) -> Result<Vec<NodeInfo>, CoreError>;

    /// 获取表/视图/集合列表
    async fn get_tables(&self, db: &str, schema: &str) -> Result<Vec<NodeInfo>, CoreError>;

    /// 获取表/视图详情（含列信息）
    async fn get_table_detail(
        &self,
        db: &str,
        schema: &str,
        table: &str,
    ) -> Result<NodeDetail, CoreError>;
}

/// 数据源能力描述
///
/// 描述数据库支持的特性，用于运行时能力检测
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSourceMeta {
    /// 数据库服务器版本
    pub server_version: Option<String>,
    /// 是否支持事务
    pub supports_transaction: bool,
    /// 是否支持流式查询（大数据集分批返回）
    pub supports_streaming: bool,
    /// 是否支持 Arrow 格式（用于插件通信）
    pub supports_arrow: bool,
    /// 是否支持联邦查询（跨库查询）
    pub supports_federated: bool,
    /// 是否支持并发写入
    pub supports_concurrent_write: bool,
    /// 是否为内存数据库
    pub is_in_memory: bool,
}

impl DataSourceMeta {
    /// MySQL 元数据
    pub fn mysql() -> Self {
        Self {
            server_version: None,
            supports_transaction: true,
            supports_streaming: true,
            supports_arrow: false,
            supports_federated: false,
            supports_concurrent_write: true,
            is_in_memory: false,
        }
    }

    /// PostgreSQL 元数据
    pub fn postgres() -> Self {
        Self {
            server_version: None,
            supports_transaction: true,
            supports_streaming: true,
            supports_arrow: false,
            supports_federated: false,
            supports_concurrent_write: true,
            is_in_memory: false,
        }
    }

    /// SQLite 元数据
    pub fn sqlite() -> Self {
        Self {
            server_version: None,
            supports_transaction: true,
            supports_streaming: false,
            supports_arrow: false,
            supports_federated: false,
            supports_concurrent_write: false,
            is_in_memory: false,
        }
    }

    /// DuckDB 元数据
    pub fn duckdb() -> Self {
        Self {
            server_version: None,
            supports_transaction: true,
            supports_streaming: true,
            supports_arrow: true,
            supports_federated: true,
            supports_concurrent_write: true,
            is_in_memory: false,
        }
    }
}

/// 数据库事务
#[async_trait::async_trait]
pub trait Transaction: Send + Sync {
    /// 执行查询
    async fn query(&mut self, sql: &str) -> Result<QueryResult, CoreError>;

    /// 提交事务
    async fn commit(&mut self) -> Result<(), CoreError>;

    /// 回滚事务
    async fn rollback(&mut self) -> Result<(), CoreError>;
}

/// 数据库抽象接口
#[async_trait::async_trait]
pub trait Database: Send + Sync {
    /* ===== 核心查询能力 ===== */

    /// 执行查询
    async fn query(&self, sql: &str) -> Result<QueryResult, CoreError>;

    /// 执行参数化查询（防止 SQL 注入）
    async fn query_with_params(
        &self,
        sql: &str,
        _params: Vec<Value>,
    ) -> Result<QueryResult, CoreError> {
        // 默认实现：回退到普通查询
        // 子类应覆盖此方法以支持真正的参数化查询
        self.query(sql).await
    }

    /// 执行可取消的查询
    async fn query_with_cancel(
        &self,
        sql: &str,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<QueryResult, CoreError>;

    /// 开始事务
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, CoreError>;

    /// 获取数据源元数据
    fn meta(&self) -> DataSourceMeta;

    /// 连接健康检查（ping）
    ///
    /// 执行轻量级查询验证连接是否存活。
    /// 默认返回 Ok(())，驱动可覆盖实现真正的 ping。
    async fn ping(&self) -> Result<(), CoreError> {
        Ok(())
    }

    /// 获取连接池状态（仅池化数据库支持）
    ///
    /// 返回连接池的运行时指标。非池化数据库（如 SQLite/DuckDB 单连接）返回 None。
    async fn pool_status(&self) -> Option<PoolStatus> {
        None
    }

    /* ===== 对象树能力（Schema 浏览） ===== */

    /// 列举数据库 / catalog
    async fn list_databases(&self) -> Result<Vec<String>, CoreError> {
        Ok(vec![])
    }

    /// 列举 schema（SQLite 可返回空）
    async fn list_schemas(&self, _db: &str) -> Result<Vec<String>, CoreError> {
        Ok(vec![])
    }

    /// 列举表 / 视图
    async fn list_tables(
        &self,
        _db: &str,
        _schema: Option<&str>,
    ) -> Result<Vec<SchemaObject>, CoreError> {
        Ok(vec![])
    }

    /// 列举列
    async fn list_columns(
        &self,
        _db: &str,
        _schema: Option<&str>,
        _table: &str,
    ) -> Result<Vec<ColumnDetail>, CoreError> {
        Ok(vec![])
    }

    /// 列举存储过程
    async fn list_procedures(
        &self,
        _db: &str,
        _schema: Option<&str>,
    ) -> Result<Vec<SchemaObject>, CoreError> {
        Ok(vec![])
    }

    /// 列举函数
    async fn list_functions(
        &self,
        _db: &str,
        _schema: Option<&str>,
    ) -> Result<Vec<SchemaObject>, CoreError> {
        Ok(vec![])
    }

    /// 获取过程/函数的 DDL 源码
    ///
    /// 返回完整的 CREATE PROCEDURE/FUNCTION 语句。
    /// 不支持或不存在的 routine 返回 None。
    async fn get_routine_source(
        &self,
        _db: &str,
        _schema: Option<&str>,
        _name: &str,
        _kind: SchemaObjectKind, // Procedure 或 Function
    ) -> Result<Option<String>, CoreError> {
        Ok(None) // 默认：不支持
    }

    /* ===== 联邦查询能力 ===== */

    /// 注册外部数据库连接
    async fn register_external_database(
        &self,
        _name: &str,
        _driver: &str,
        _connection_string: &str,
    ) -> Result<(), CoreError> {
        Err(CoreError::database(DatabaseError::Driver {
            db_type: "generic".to_string(),
            operation: "register_external_database".to_string(),
            source: "Not supported".to_string(),
        }))
    }

    /// 创建外部表
    async fn create_external_table(
        &self,
        _external_db_name: &str,
        _schema_name: &str,
        _table_name: &str,
        _external_table_name: &str,
    ) -> Result<(), CoreError> {
        Err(CoreError::database(DatabaseError::Driver {
            db_type: "generic".to_string(),
            operation: "create_external_table".to_string(),
            source: "Not supported".to_string(),
        }))
    }
}

/// 动态数据库类型
pub type DynDatabase = Arc<dyn Database + Send + Sync>;

/// 数据库连接池抽象接口
///
/// 统一不同数据库驱动的连接池管理，支持：
/// - sqlx (MySQL/PostgreSQL)
/// - rusqlite (SQLite)
/// - duckdb (DuckDB)
/// - 未来: JDBC/ODBC 桥接
#[async_trait::async_trait]
pub trait DbPool: Send + Sync {
    /// 从连接池获取一个数据库连接
    ///
    /// # Returns
    ///
    /// 返回一个实现了 Database trait 的连接
    async fn acquire(&self) -> Result<Box<dyn Database + Send + Sync>, CoreError>;

    /// 关闭连接池，释放所有资源
    async fn close(&self) -> Result<(), CoreError>;

    /// 检查连接池是否已关闭
    fn is_closed(&self) -> bool;

    /// 获取连接池状态信息
    fn status(&self) -> PoolStatus;
}

/// 连接池状态
#[derive(Debug, Clone)]
pub struct PoolStatus {
    /// 连接池大小
    pub size: usize,
    /// 空闲连接数
    pub idle: usize,
    /// 活跃连接数
    pub active: usize,
    /// 等待获取连接的请求数
    pub waiting: usize,
    /// 最大连接数
    pub max_connections: usize,
    /// 最小连接数
    pub min_connections: usize,
}

impl PoolStatus {
    /// 创建未知状态（用于不支持状态查询的驱动）
    pub fn unknown() -> Self {
        Self {
            size: 0,
            idle: 0,
            active: 0,
            waiting: 0,
            max_connections: 10,
            min_connections: 2,
        }
    }
}
