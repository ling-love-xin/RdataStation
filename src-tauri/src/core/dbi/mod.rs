/**
 * DBI (Database Interface) — 统一数据访问引擎层
 *
 * ═══════════ 架构边界 ═══════════
 * DBI 层定位：执行引擎抽象层，位于 services 之下、driver 之上。
 *
 *    commands ──► services ──► dbi ──► driver ──► native
 *                  (业务逻辑)   (引擎)   (trait)    (实现)
 *
 * ❌ 禁止：commands 直接 import dbi（commands 只访问 services）
 * ✅ 允许：services 内部调用 dbi 引擎（如 DuckDbService → DuckDBEngine）
 *
 * 职责：
 * - 多引擎路由（DriverEngine / DuckDBEngine / StreamEngine）
 * - 会话与事务管理
 * - 查询上下文传递（ExecutionContext / QueryContext）
 * - 性能统计收集
 */
pub mod context;
#[allow(clippy::module_inception)]
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
