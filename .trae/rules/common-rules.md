# 一、架构红线（Architecture Gates）

❌ 禁止：循环依赖

```markdown
❌ core 依赖 adapters
❌ adapters 依赖具体 datasource 实现
✅ adapters 只能依赖 core
```

❌ 禁止：层级越界

```markdown
❌ services 直接调用 datasource/\* (绕过 driver)
❌ connection/factory 写死具体数据库类型
✅ factory 只返回 Box
```

<br />

# 二、Trait 与 接口约束（基于 traits.rs）

❌ 禁止：修改宪法

```markdown
❌ 禁止修改 driver/traits.rs 中的 trait 定义
❌ 禁止给 Database trait 增加与数据访问无关的方法
```

✅ 必须：实现完整性

```markdown
✅ datasource/mysql.rs 必须 impl Database + DbPool
✅ 所有 datasource 必须正确处理 meta() 方法
```

# 三、数据契约检查（IPC 生死线）

❌ 致命违规：非 Arrow 传输

```rust
// ❌ 禁止在以下位置出现
async fn query(\&self, sql: \&str) -> Result\<Vec<Row>, CoreError>; // 违规
async fn query(\&self, sql: \&str) -> Result\<QueryResult, CoreError>; // 需检查 QueryResult 内部
```

✅ 合规标准

```rust
// ✅ 必须确保路径畅通
Database::query()
↓
QueryResult
└─ batches: Vec<RecordBatch>  ✅
↓
Tauri Command
↓
JSON
```

❌ 禁止：在 IPC 层做转换

```markdown
❌ Tauri Command 中手动将 Row 转为 JSON
✅ Tauri Command 只负责调用 service 并返回结果
```

四、连接池与资源管理
✅ 必须：Pool 下沉

```markdown
✅ datasource/pool.rs 只定义 DbPool trait
✅ 具体 Pool (SqlxMySqlPool) 必须在 datasource/{db}/pool.rs
❌ 禁止存在顶层 connection/pool.rs 文件
```

✅ 必须：Acquire 返回 Database 实例

```rust
// ✅ 正确
async fn acquire(\&self) -> Result\<Box<dyn Database>, CoreError>;

// ❌ 错误
async fn acquire(\&self) -> Result\<Pool<MySql>, CoreError>;
```

# 五、错误处理与 Rust 规范

❌ 禁止：unwrap / expect

```rust
// ❌ 禁止在生产代码中
let x = foo().unwrap();

// ✅ 必须
let x = foo()?;
```

✅ 必须：使用 CoreError

```rust
// ✅ 必须
use crate::core::error::CoreError;

// ❌ 禁止
use std::io::Error;
```

# 六、前端交互约束

✅ 必须：SchemaObject 懒加载

```rust
// ✅ 正确
SchemaObject {
children: None, // 表示未加载
}

// ❌ 错误
SchemaObject {
children: Some(vec!\[]), // 误判为空
}
```

# 七、检查清单（Checklist）

在提交任何代码前，请自检以下问题：

- [ ] DuckDB 是否被写死为唯一执行引擎？（必须是可插拔的）
- [ ] QueryResult 内部是否包含 RecordBatch？
- [ ] services 层是否只调用 connection / driver，不直接碰 datasource？
- [ ] Rust 代码中是否存在 unwrap()？
- [ ] Pool 是否只负责连接，不负责 SQL 执行？

# 八、项目约束

- 每次提交代码前，必须先自检以上问题，确保代码符合项目规范，并且汇报修改内容。
- 本Skill适用于Trae CN AI编辑器，启用后AI将全程遵循上述规范，生成可直接编译、贴合项目需求的代码，无需重复说明项目规则。
- 技术栈升级需遵循既定策略，升级前需测试兼容性，确保不破坏核心功能与接口兼容性，升级后更新本Skill对应版本信息。
- 所有新增功能、修复均需符合项目架构与长期约束，确保项目轻量、高效、可扩展，支撑10年生命周期。
