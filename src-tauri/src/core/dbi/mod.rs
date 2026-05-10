pub mod context;
/**
 * DBI (Database Interface) - 统一数据访问入口
 *
 * 作为 RdataStation 的核心数据访问层，DBI 负责：
 * - 统一查询/执行接口
 * - 会话和事务管理
 * - 查询上下文传递
 * - 多引擎路由（原生驱动 / DuckDB 加速 / 流处理）
 */
pub mod dbi;
pub mod engine;
pub mod performance;
pub mod session;

pub use context::{ExecutionContext, QueryContext};
pub use dbi::DBI;
pub use engine::{ExecutionEngine, ExecutionMode, QueryRouter, SqlFeatures};
pub use performance::{
    ModePerformanceStats, PerformanceCollector, PerformanceStats, QueryPerformanceRecord,
};
pub use session::{Session, SessionConfig, SessionMode};
