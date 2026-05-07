# API 接口文档

## 概述

本文档描述 RdataStation 后端提供的所有 Tauri 命令（API 接口）。

## 连接管理

### connect_database

创建数据库连接。

**参数**：
```typescript
interface ConnectDatabaseInput {
  db_type: string;      // 数据库类型: "mysql", "postgresql", "sqlite", "duckdb"
  url: string;          // 连接 URL
  name?: string;        // 连接名称（可选）
}
```

**返回**：
```typescript
interface ConnectDatabaseResponse {
  conn_id: string;      // 连接 ID
  name: string;         // 连接名称
  db_type: string;      // 数据库类型
  url: string;          // 连接 URL
  meta: DataSourceMeta; // 数据源元数据
}

interface DataSourceMeta {
  supports_transaction: boolean;
  supports_streaming: boolean;
  supports_arrow: boolean;
  supports_federated: boolean;
  supports_concurrent_write: boolean;
  is_in_memory: boolean;
}
```

**示例**：
```typescript
const result = await invoke('connect_database', {
  input: {
    db_type: 'postgresql',
    url: 'postgres://user:pass@localhost:5432/mydb',
    name: 'My PostgreSQL'
  }
});
// result.conn_id: "conn_abc123"
```

### get_connections

获取所有连接列表。

**参数**：无

**返回**：
```typescript
interface ConnectionInfoResponse {
  id: string;
  name: string;
  db_type: string;
  url: string;
  is_active: boolean;
  connected_at: string;  // ISO 8601 格式
}

// 返回: ConnectionInfoResponse[]
```

**示例**：
```typescript
const connections = await invoke('get_connections');
// connections: [{ id: "conn_1", name: "MyDB", ... }]
```

### switch_connection

切换当前活动连接。

**参数**：
```typescript
interface SwitchConnectionInput {
  conn_id: string;
}
```

**返回**：`void`

**示例**：
```typescript
await invoke('switch_connection', {
  input: { conn_id: 'conn_abc123' }
});
```

### close_connection

关闭指定连接。

**参数**：
```typescript
{
  conn_id: string;
}
```

**返回**：`void`

**示例**：
```typescript
await invoke('close_connection', { conn_id: 'conn_abc123' });
```

### close_all_connections

关闭所有连接。

**参数**：无

**返回**：`void`

### get_active_connection

获取当前活动连接。

**参数**：无

**返回**：
```typescript
interface ActiveConnectionResponse {
  conn_id: string;
  name: string;
  db_type: string;
}
// 或 null（如果没有活动连接）
```

### test_connection

测试数据库连接（不保存）。

**参数**：
```typescript
{
  db_type: string;
  url: string;
}
```

**返回**：
```typescript
interface TestConnectionResponse {
  success: boolean;
  message: string;
  server_version: string;
  response_time_ms: number;
}
```

## SQL 执行

### execute_sql

执行 SQL 查询。

**参数**：
```typescript
interface ExecuteSqlInput {
  conn_id?: string;     // 连接 ID（可选，使用活动连接）
  sql: string;          // SQL 语句
  timeout_ms?: number;  // 超时时间（毫秒，可选）
}
```

**返回**：
```typescript
interface ExecuteSqlResponse {
  columns: string[];           // 列名
  rows: Value[][];            // 数据行
  affected_rows?: number;     // 影响的行数（INSERT/UPDATE/DELETE）
  execution_time_ms: number;  // 执行时间
}

type Value = 
  | { type: 'null' }
  | { type: 'string'; value: string }
  | { type: 'int64'; value: number }
  | { type: 'float64'; value: number }
  | { type: 'bool'; value: boolean }
  | { type: 'bytes'; value: number[] }
  | { type: 'date'; value: string }
  | { type: 'time'; value: string }
  | { type: 'datetime'; value: string };
```

**示例**：
```typescript
const result = await invoke('execute_sql', {
  input: {
    conn_id: 'conn_abc123',
    sql: 'SELECT * FROM users WHERE id = $1',
    timeout_ms: 30000
  }
});

// result:
// {
//   columns: ["id", "name", "email"],
//   rows: [
//     [{ type: "int64", value: 1 }, { type: "string", value: "John" }, ...]
//   ],
//   execution_time_ms: 45
// }
```

### execute_transaction

在事务中执行多个 SQL。

**参数**：
```typescript
interface ExecuteTransactionInput {
  conn_id?: string;
  sqls: string[];       // SQL 语句数组
}
```

**返回**：
```typescript
interface ExecuteTransactionResponse {
  results: ExecuteSqlResponse[];
}
```

**示例**：
```typescript
const result = await invoke('execute_transaction', {
  input: {
    sqls: [
      'BEGIN',
      'UPDATE accounts SET balance = balance - 100 WHERE id = 1',
      'UPDATE accounts SET balance = balance + 100 WHERE id = 2',
      'COMMIT'
    ]
  }
});
```

## 元数据查询

### get_databases

获取数据库列表。

**参数**：
```typescript
{
  conn_id: string;
}
```

**返回**：
```typescript
interface DatabaseInfoResponse {
  name: string;
}
// 返回: DatabaseInfoResponse[]
```

**示例**：
```typescript
const databases = await invoke('get_databases', { conn_id: 'conn_abc123' });
// databases: [{ name: "postgres" }, { name: "myapp" }]
```

### get_schemas

获取 Schema 列表。

**参数**：
```typescript
{
  conn_id: string;
  database: string;
}
```

**返回**：
```typescript
interface SchemaInfoResponse {
  name: string;
}
// 返回: SchemaInfoResponse[]
```

### get_tables

获取表列表。

**参数**：
```typescript
{
  conn_id: string;
  database: string;
  schema: string;
}
```

**返回**：
```typescript
interface TableInfoResponse {
  name: string;
  type: string;  // "table" | "view"
}
// 返回: TableInfoResponse[]
```

### get_views

获取视图列表。

**参数**：
```typescript
{
  conn_id: string;
  database: string;
  schema: string;
}
```

**返回**：`TableInfoResponse[]`

### get_columns

获取表的列信息。

**参数**：
```typescript
{
  conn_id: string;
  database: string;
  schema: string;
  table: string;
}
```

**返回**：
```typescript
interface ColumnInfoResponse {
  name: string;
  data_type: string;
  nullable?: boolean;
  default_value?: string;
  is_primary_key?: boolean;
}
// 返回: ColumnInfoResponse[]
```

### list_databases

列出数据库（简化版）。

**参数**：
```typescript
{
  conn_id: string;
}
```

**返回**：`string[]`

### list_schemas

列出 Schema。

**参数**：
```typescript
{
  conn_id: string;
  database: string;
}
```

**返回**：`string[]`

### list_tables

列出表和视图。

**参数**：
```typescript
{
  conn_id: string;
  database: string;
  schema?: string;
}
```

**返回**：
```typescript
interface SchemaObjectResponse {
  name: string;
  kind: string;  // "database" | "schema" | "table" | "view" | "column"
  children?: SchemaObjectResponse[];
}
// 返回: SchemaObjectResponse[]
```

### list_columns

列出表的列。

**参数**：
```typescript
{
  conn_id: string;
  database: string;
  schema?: string;
  table: string;
}
```

**返回**：`SchemaObjectResponse[]`

## 历史记录

### get_sql_history

获取 SQL 执行历史。

**参数**：
```typescript
{
  limit?: number;  // 默认 100
}
```

**返回**：
```typescript
interface SqlHistoryResponse {
  id: string;
  sql: string;
  conn_id?: string;
  executed_at: string;  // ISO 8601
}
// 返回: SqlHistoryResponse[]
```

### search_sql_history

搜索 SQL 历史。

**参数**：
```typescript
{
  keyword: string;
  limit?: number;
}
```

**返回**：`SqlHistoryResponse[]`

### clear_sql_history

清空 SQL 历史。

**参数**：无

**返回**：`void`

### remove_sql_history

删除单条 SQL 历史。

**参数**：
```typescript
{
  id: string;
}
```

**返回**：`void`

## 最近连接

### get_recent_connections

获取最近连接列表。

**参数**：无

**返回**：
```typescript
interface RecentConnectionResponse {
  name: string;
  db_type: string;
  url: string;
  last_used_at: string;  // ISO 8601
}
// 返回: RecentConnectionResponse[]
```

### remove_recent_connection

删除最近连接记录。

**参数**：
```typescript
{
  name: string;
}
```

**返回**：`void`

## 驱动管理

### get_drivers

获取所有支持的驱动列表。

**参数**：无

**返回**：
```typescript
interface DriverDescriptor {
  id: string;
  name: string;
  description: string;
  version: string;
  icon?: string;
  default_port: number;
  connection_fields: DriverField[];
  features: DriverFeatures;
}

interface DriverField {
  name: string;
  label: string;
  field_type: 'string' | 'number' | 'password' | 'boolean' | 'select';
  required: boolean;
  default_value?: string;
  options?: string[];  // 用于 select 类型
}

interface DriverFeatures {
  supports_transactions: boolean;
  supports_ssl: boolean;
  supports_ssh_tunnel: boolean;
  supports_multiple_databases: boolean;
  supports_schemas: boolean;
  supports_views: boolean;
  supports_stored_procedures: boolean;
  supports_functions: boolean;
  supports_triggers: boolean;
}

// 返回: DriverDescriptor[]
```

### get_driver_info

获取指定驱动的详细信息。

**参数**：
```typescript
{
  driver_id: string;
}
```

**返回**：`DriverDescriptor | null`

## DBI 统一数据访问 🔥

### dbi_query

通过 DBI 执行查询（支持智能路由）。

**参数**：
```typescript
interface DBIQueryInput {
  sql: string;
  conn_id?: string;
  mode?: 'native' | 'duckdb' | 'stream' | 'auto';  // 默认 'auto'
  timeout_ms?: number;
}
```

**返回**：
```typescript
interface DBIQueryResponse {
  columns: string[];
  rows: Value[][];
  affected_rows?: number;
  execution_time_ms: number;
  execution_mode: 'native' | 'duckdb' | 'stream';  // 实际使用的执行模式
  is_read_only: boolean;
}
```

**示例**：
```typescript
// 自动模式（智能推荐）
const result = await invoke('dbi_query', {
  input: {
    sql: 'SELECT u.*, o.total FROM users u JOIN orders o ON u.id = o.user_id GROUP BY u.id',
    mode: 'auto'
  }
});
// result.execution_mode: "duckdb" (复杂查询自动路由到 DuckDB)

// 强制使用原生驱动
const result2 = await invoke('dbi_query', {
  input: {
    sql: 'SELECT * FROM users WHERE id = 1',
    mode: 'native'
  }
});
```

### dbi_execute

通过 DBI 执行写操作（INSERT/UPDATE/DELETE）。

**参数**：
```typescript
interface DBIExecuteInput {
  sql: string;
  conn_id?: string;
}
```

**返回**：
```typescript
interface DBIExecuteResponse {
  affected_rows: number;
  execution_time_ms: number;
}
```

**示例**：
```typescript
const result = await invoke('dbi_execute', {
  input: {
    sql: "UPDATE users SET name = 'John' WHERE id = 1"
  }
});
// result.affected_rows: 1
```

### register_external_database

注册外部数据库到 DuckDB（用于联邦查询）。

**参数**：
```typescript
interface RegisterExternalDBInput {
  name: string;           // 数据库别名
  driver: string;         // 驱动类型: "mysql", "postgresql"
  connection_string: string;  // 连接字符串
}
```

**返回**：`void`

**示例**：
```typescript
await invoke('register_external_database', {
  input: {
    name: 'mysql_prod',
    driver: 'mysql',
    connection_string: 'mysql://user:pass@prod-host:3306/mydb'
  }
});
```

### detach_external_database

卸载外部数据库。

**参数**：
```typescript
{
  name: string;
}
```

**返回**：`void`

### load_file_source

加载文件数据源到 DuckDB（CSV/Excel/Parquet）。

**参数**：
```typescript
interface LoadFileSourceInput {
  path: string;         // 文件绝对路径
  table_name: string;   // 临时表名
}
```

**返回**：`void`

**示例**：
```typescript
await invoke('load_file_source', {
  input: {
    path: '/path/to/data.csv',
    table_name: 'temp_csv_data'
  }
});

// 现在可以查询
const result = await invoke('dbi_query', {
  input: {
    sql: 'SELECT * FROM temp_csv_data WHERE column1 > 100',
    mode: 'duckdb'
  }
});
```

### persist_result_set

持久化结果集到 DuckDB。

**参数**：
```typescript
interface PersistResultSetInput {
  result_name: string;  // 结果集名称
  sql: string;          // 查询 SQL
}
```

**返回**：`void`

**示例**：
```typescript
await invoke('persist_result_set', {
  input: {
    result_name: 'user_orders_2024',
    sql: `
      SELECT u.*, o.total 
      FROM mysql_prod.users u 
      JOIN pg_prod.orders o ON u.id = o.user_id 
      WHERE o.created_at > '2024-01-01'
    `
  }
});

// 后续可以查询持久化的结果集
const result = await invoke('dbi_query', {
  input: {
    sql: 'SELECT * FROM user_orders_2024 WHERE total > 1000',
    mode: 'duckdb'
  }
});
```

### list_external_databases

列出已注册的外部数据库。

**参数**：无

**返回**：
```typescript
interface ExternalDatabaseInfo {
  name: string;
  driver: string;
  connection_string: string;
  read_only: boolean;
  is_attached: boolean;
}
// 返回: ExternalDatabaseInfo[]
```

### list_result_sets

列出已持久化的结果集。

**参数**：无

**返回**：
```typescript
interface ResultSetInfo {
  name: string;
  created_at: string;  // ISO 8601
  row_count: number;
  source_sql: string;
}
// 返回: ResultSetInfo[]
```

### drop_result_set

删除持久化的结果集。

**参数**：
```typescript
{
  result_name: string;
}
```

**返回**：`void`

### recommend_execution_mode

智能推荐执行模式（基于 SQL 分析）。

**参数**：
```typescript
{
  sql: string;
}
```

**返回**：
```typescript
interface RecommendModeResponse {
  mode: 'native' | 'duckdb' | 'stream';
  reason: string;  // 推荐理由
}
```

**示例**：
```typescript
const recommendation = await invoke('recommend_execution_mode', {
  input: {
    sql: 'SELECT u.*, COUNT(o.id) as order_count FROM users u LEFT JOIN orders o ON u.id = o.user_id GROUP BY u.id ORDER BY order_count DESC'
  }
});
// recommendation.mode: "duckdb"
// recommendation.reason: "Complex query with JOIN, GROUP BY, and ORDER BY - DuckDB acceleration recommended"
```

## 项目管理

### create_project

创建新项目。

**参数**：
```typescript
{
  name: string;
  path: string;
  description?: string;
}
```

**返回**：
```typescript
interface ProjectInfo {
  id: string;
  name: string;
  path: string;
  status: 'active' | 'archived';
  created_at: string;
  updated_at: string;
}
```

### open_project

打开项目。

**参数**：
```typescript
{
  path: string;
}
```

**返回**：`ProjectInfo`

### get_project_config

获取项目配置。

**参数**：
```typescript
{
  project_id: string;
}
```

**返回**：
```typescript
interface ProjectConfig {
  theme: 'light' | 'dark' | 'system';
  editor: EditorConfig;
  connections: ConnectionConfig[];
}
```

### update_project_config

更新项目配置。

**参数**：
```typescript
{
  project_id: string;
  config: Partial<ProjectConfig>;
}
```

**返回**：`void`

### get_recent_projects

获取最近项目列表。

**参数**：无

**返回**：`ProjectInfo[]`

### add_recent_project

添加项目到最近列表。

**参数**：
```typescript
{
  path: string;
}
```

**返回**：`void`

## 端口协商

### negotiate_port

协商可用端口。

**参数**：
```typescript
{
  preferred_port?: number;
  port_range?: [number, number];
}
```

**返回**：
```typescript
{
  port: number;
  is_preferred: boolean;
}
```

### is_port_available

检查端口是否可用。

**参数**：
```typescript
{
  port: number;
}
```

**返回**：`boolean`

### get_common_db_ports

获取常用数据库端口。

**参数**：无

**返回**：
```typescript
{
  mysql: 3306;
  postgresql: 5432;
  mongodb: 27017;
  redis: 6379;
  // ...
}
```

## 洞察引擎 🔍

洞察系统提供 12 个 Tauri 命令，覆盖列洞察、多列分析、规则引擎、持久化和存储管理。

### get_column_insight_full

计算并返回完整的列洞察报告。

**参数**：
```typescript
{
  tempTable: string;
  column: string;
}
```

**返回**：`ColumnInsightFull` (包含 stats / histogram / sample)

### save_column_insight_snapshot

将当前列洞察保存为持久化版本快照。

**参数**：
```typescript
{
  temp_table: string;
  column_name: string;
}
```

**返回**：`string` (version_id)

### get_column_insight_history

查询某列的所有洞察版本历史。

**参数**：
```typescript
{
  column_name: string;
}
```

**返回**：`InsightVersionEntry[]`

### get_insight_storage_stats

获取洞察存储统计（快照数、列数、体积）。

**参数**：无

**返回**：`InsightStorageStats`

### cleanup_insight_snapshots

清理超过指定天数的旧洞察快照。

**参数**：
```typescript
{
  older_than_days: number;  // 默认 90
}
```

**返回**：`CleanupResult { removed_count, freed_bytes }`

### get_insight_version_detail

获取指定版本的完整洞察数据。

**参数**：
```typescript
{
  version_id: string;
}
```

**返回**：`ColumnInsightFull | null`

### get_table_profile

获取表的元数据探查（列名/类型/可空/主键/行数）。

**参数**：
```typescript
{
  conn_id: string;
  db_type: string;
  database: string;
  schema: string;
  table: string;
}
```

**返回**：`TableProfile { table_name, db_type, columns: TableColumnMeta[], row_count, schema_name }`

### profile_column_from_table

从表探查结果中点击列名，触发端到端列洞察（无需预先建立 DuckDB temp 表）。

> 后端自动：`SqlService 取样 → DuckDB temp 表 → 列洞察全量分析` 一步完成。

**参数**：
```typescript
{
  conn_id: string;
  database: string;
  schema: string;
  table: string;
  column_name: string;
}
```

**返回**：`ColumnInsightFull { table_name, column_name, column_type, stats_detail, histogram, sample_values }`

### get_column_quality

计算列数据质量评分（0-100），基于完整性、唯一性、类型一致性、分布均匀性四维度加权。

**参数**：
```typescript
{
  column_name: string;
  temp_table: string;
}
```

**返回**：`QualityScore { column_name, overall_score, level, dimensions: [{ name, score, weight, detail }], summary }`

**等级划分**：
| 分数 | 等级 | 颜色 |
|:--:|:--:|:--:|
| ≥85 | 优秀 | 绿色 |
| ≥70 | 良好 | 蓝色 |
| ≥50 | 一般 | 黄色 |
| ≥30 | 较差 | 橙色 |
| <30 | 差 | 红色 |

### batch_evaluate_columns

一次调用完成全表所有列的质量评估（取样 → DuckDB 临时表 → 逐列洞察 → 聚合评分）。

> 后端自动：SELECT LIMIT 500 → JSON 解析 → DuckDB temp 表 → 全列 insight → 聚合 TableQuality

**参数**：
```typescript
{
  conn_id: string;
  database: string;
  schema: string;
  table: string;
}
```

**返回**：`TableQuality { table_name, overall_score, level, column_scores: [{ column_name, quality_score, level, null_rate }], summary, scored_count, total_columns }`

**使用场景**：TableProfileView "质量评估" 按钮，导航树右键表探查后一键评估

### execute_insight_rule

执行一条声明式规则（支持列洞察和多列分析）。

**参数**：
```typescript
{
  rule_id: string;           // 规则 ID (如 "correlation")
  params: Record<string, string>;  // SQL 模板参数
  temp_table: string;
}
```

**返回**：动态 `Value` (JSON)

### list_insight_rules

列出所有可用的洞察规则。

**参数**：`category?: string` (可选过滤 "column" / "multi")

**返回**：`RuleMeta[]`

### list_rules_for_column

根据列类型过滤适用规则。

**参数**：
```typescript
{
  column_type: string;  // "Numeric" | "Text" | "DateTime" | "Boolean"
}
```

**返回**：`RuleMeta[]`

**示例**：
```typescript
// 执行多列相关性分析
const result = await invoke('execute_insight_rule', {
  input: {
    rule_id: 'correlation',
    params: { table: 'tmp_abc', col1: 'age', col2: 'salary' },
    temp_table: 'tmp_abc'
  }
});
// result: { correlation: 0.87, p_value: 0.001 }

// 清理 90 天前的旧快照
const cleaned = await invoke('cleanup_insight_snapshots', {
  input: { older_than_days: 90 }
});
// cleaned: { removed_count: 15, freed_bytes: 204800 }
```

## 错误处理

### 错误响应格式

```typescript
// 成功响应
{
  // 返回数据
}

// 错误响应
{
  error: string;  // 错误消息
}
```

### 错误代码

| 错误 | 说明 |
|------|------|
| `Connection not found` | 连接不存在 |
| `Connection timeout` | 连接超时 |
| `Authentication failed` | 认证失败 |
| `Database not found` | 数据库不存在 |
| `Query syntax error` | SQL 语法错误 |
| `Constraint violation` | 约束冲突 |
| `Pool exhausted` | 连接池耗尽 |

### 前端错误处理示例

```typescript
import { invoke } from '@tauri-apps/api/core';

try {
  const result = await invoke('execute_sql', {
    input: { sql: 'SELECT * FROM users' }
  });
  console.log(result);
} catch (error) {
  // 错误处理
  if (error.includes('Connection not found')) {
    // 提示用户连接已断开
    showReconnectDialog();
  } else if (error.includes('syntax error')) {
    // 高亮 SQL 错误位置
    highlightSqlError(error);
  } else {
    // 通用错误提示
    showErrorNotification(error);
  }
}
```

## TypeScript 类型定义

完整的 TypeScript 类型定义：

```typescript
// types/api.ts

export type DatabaseType = 'mysql' | 'postgresql' | 'sqlite' | 'duckdb' | 'mongodb';

export interface ConnectionConfig {
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
  ssl?: boolean;
  ssh?: SshConfig;
}

export interface SshConfig {
  host: string;
  port: number;
  username: string;
  private_key?: string;
  password?: string;
}

export type Value =
  | { type: 'null' }
  | { type: 'string'; value: string }
  | { type: 'int64'; value: number }
  | { type: 'float64'; value: number }
  | { type: 'bool'; value: boolean }
  | { type: 'bytes'; value: Uint8Array }
  | { type: 'date'; value: string }
  | { type: 'time'; value: string }
  | { type: 'datetime'; value: string };

export interface QueryResult {
  columns: string[];
  rows: Value[][];
  affected_rows?: number;
  execution_time_ms: number;
}

// ... 其他类型定义
```

## API 调用工具函数

```typescript
// utils/api.ts

import { invoke } from '@tauri-apps/api/core';

export async function executeSql(
  sql: string,
  connId?: string
): Promise<QueryResult> {
  const response = await invoke<ExecuteSqlResponse>('execute_sql', {
    input: { sql, conn_id: connId }
  });
  
  return {
    columns: response.columns,
    rows: response.rows,
    affected_rows: response.affected_rows,
    execution_time_ms: response.execution_time_ms
  };
}

export async function getTables(
  connId: string,
  database: string,
  schema: string
): Promise<string[]> {
  const response = await invoke<TableInfoResponse[]>('get_tables', {
    input: { conn_id: connId, database, schema }
  });
  
  return response.map(t => t.name);
}

// ... 其他工具函数
```
