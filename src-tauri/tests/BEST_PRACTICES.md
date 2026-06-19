# RdataStation 数据库连接配置最佳实践

> 版本：v1.0  
> 日期：2026-06-19  
> 基于 v0.5.2 测试结果

---

## 一、总则

### 1.1 连接配置架构

```
用户输入 → DriverConnectionConfig → to_url() → Database::new(url)
                                          ↓
                              driver_properties 追加到查询参数
```

### 1.2 安全性原则

| 原则 | 说明 |
|------|------|
| 密码加密 | 所有密码/Auth 凭据使用 AES-256-GCM 加密存储（`AES:` 前缀） |
| 日志脱敏 | 日志中使用 `mask_password_in_url()` 脱敏 URL |
| 参数化查询 | 始终使用 `query_with_params()` 防止 SQL 注入 |
| 最小权限 | 数据库用户仅授予必要权限 |

---

## 二、SQLite 配置

### 2.1 连接 URL 格式

```
# 文件路径
sqlite:///absolute/path/to/database.db

# 相对路径
sqlite://relative/path/to/database.db

# 内存数据库
:memory:
```

### 2.2 推荐 driver_properties

```json
{
  "journal_mode": "WAL",
  "synchronous": "NORMAL",
  "cache_size": "-2000",
  "foreign_keys": "ON",
  "busy_timeout": "5000"
}
```

### 2.3 最佳实践

| 场景 | 建议 |
|------|------|
| 并发读 | 使用 WAL 模式，支持多读单写 |
| 性能优化 | `synchronous=NORMAL` 平衡性能与安全 |
| 大事务 | 设置 `cache_size` 为负值（KB 单位） |
| 外键约束 | 显式启用 `PRAGMA foreign_keys = ON` |

### 2.4 已知限制

- 仅支持单写者，高并发写入需排队
- 默认不启用外键约束，需显式 PRAGMA
- 文件路径不存在时创建失败，需先确保目录存在

---

## 三、DuckDB 配置

### 3.1 连接 URL 格式

```
# 文件路径
duckdb:///absolute/path/to/database.duckdb

# 内存数据库
:memory:
```

### 3.2 推荐 driver_properties

```json
{
  "threads": "4",
  "memory_limit": "1GB",
  "access_mode": "read_write"
}
```

### 3.3 最佳实践

| 场景 | 建议 |
|------|------|
| 分析查询 | 设置 `threads` 为 CPU 核心数 |
| 内存控制 | 设置 `memory_limit` 避免 OOM |
| 只读模式 | 设置 `access_mode=read_only` 保护数据 |
| 异步操作 | 使用 `spawn_blocking` 隔离阻塞操作 |

### 3.4 已知限制

- 错误查询可能阻塞 tokio 运行时线程，建议 `spawn_blocking` 隔离
- 单文件写入为串行，高并发写入场景有限制
- 不支持远程连接，仅本地文件访问

---

## 四、MySQL 配置

### 4.1 连接 URL 格式

```
# 基本格式
mysql://username:password@host:port/database

# 带参数
mysql://root:root@localhost:3306/testdb?allowPublicKeyRetrieval=TRUE&useSSL=false
```

### 4.2 推荐 driver_properties

```json
{
  "allowPublicKeyRetrieval": "TRUE",
  "useSSL": "false",
  "serverTimezone": "Asia/Shanghai",
  "characterEncoding": "utf8",
  "connect_timeout": "30",
  "socketTimeout": "60"
}
```

### 4.3 编码映射（encoding → charset）

| encoding 值 | 生成的 charset |
|-------------|---------------|
| `UTF-8` | `utf8mb4` |
| `UTF8` | `utf8mb4` |
| `GBK` | `gbk` |
| `GB2312` | `gb2312` |
| `LATIN1` | `latin1` |
| 其他 | 原值 |

### 4.4 最佳实践

| 场景 | 建议 |
|------|------|
| MySQL 8.0+ | 必须设置 `allowPublicKeyRetrieval=TRUE` 以支持 caching_sha2_password |
| 本地开发 | `useSSL=false` 简化配置 |
| 生产环境 | 启用 SSL，设置 `useSSL=true` 和证书路径 |
| 时区处理 | 设置 `serverTimezone` 避免时区偏移 |
| 字符编码 | 强烈推荐 `utf8mb4`（支持完整 Unicode） |
| 连接超时 | 设置 `connect_timeout` 避免长时间挂起 |

### 4.5 已知限制

- MySQL 8.0+ 可能限制 `information_schema` 访问，元数据浏览可能返回空
- `USE database` 语句在 prepared statement 协议中不支持，需直连目标库
- 密码特殊字符需 URL 编码

---

## 五、PostgreSQL 配置

### 5.1 连接 URL 格式

```
# 基本格式
postgres://username:password@host:port/database

# 带参数
postgres://postgres:postgresql@localhost:5432/business_db?sslmode=disable
```

### 5.2 推荐 driver_properties

```json
{
  "sslmode": "disable",
  "application_name": "RdataStation",
  "connect_timeout": "30",
  "statement_timeout": "30000",
  "idle_in_transaction_session_timeout": "60000"
}
```

### 5.3 SSL 模式说明

| sslmode | 说明 |
|---------|------|
| `disable` | 不加密（仅本地开发） |
| `require` | 加密但不验证证书 |
| `verify-ca` | 验证 CA 证书 |
| `verify-full` | 验证证书和主机名 |

### 5.4 两种驱动选择

| 驱动 | 适用场景 | TLS 支持 |
|------|----------|----------|
| `postgres` (sqlx) | 通用场景，无需 TLS | ❌ |
| `postgres_native` (native-tls) | 需要 SSL/TLS 加密 | ✅ |

### 5.5 最佳实践

| 场景 | 建议 |
|------|------|
| 本地开发 | `sslmode=disable` |
| 生产环境 | 使用 `postgres_native` 驱动 + `sslmode=require` |
| 长时间查询 | 设置 `statement_timeout` 防止挂起 |
| 连接池 | 使用 Tauri 连接池管理，避免频繁创建/销毁 |
| 参数化查询 | 使用 `$1, $2, ...` 占位符 |

---

## 六、DriverConnectionConfig 通用规范

### 6.1 构建方式

```rust
// 方式一：url_override（推荐，前端直接传 URL）
let config = DriverConnectionConfig::new("mysql")
    .with_url_override("mysql://root:root@localhost:3306/testdb")
    .with_connect_timeout(30)
    .with_encoding("UTF-8");

// 方式二：逐字段构建（后端拼接）
let config = DriverConnectionConfig::new("postgres")
    .with_host("localhost")
    .with_port(5432)
    .with_database("business_db")
    .with_username("postgres")
    .with_password("postgresql");
```

### 6.2 URL 生成优先级

```
url_override > url_template > 硬编码匹配（legacy fallback）
```

所有路径最终都会调用 `append_query_params()` 追加 `driver_properties`、`options`、`encoding`。

### 6.3 网络配置集成

当使用 SSH 隧道或 HTTP 代理时：

```
SSH 隧道: host → 127.0.0.1, port → 转发端口
HTTP 代理: 通过代理 URL 发起连接
SSL 证书: 通过 driver_properties 传入证书路径
```

---

## 七、认证配置

### 7.1 认证类型分类

| 大类 | auth_type | 说明 |
|------|-----------|------|
| 数据库认证 | `password` | 用户名+密码 |
| | `ldap` | LDAP 认证 |
| | `kerberos` | Kerberos 认证 |
| | `os_auth` | 操作系统认证（无凭据） |
| | `trust` | 信任认证（无凭据） |
| 网络认证 | `ssh_password` | SSH 隧道密码 |
| | `proxy_password` | 代理密码 |

### 7.2 安全存储

```rust
// 存储时加密
auth_data = encrypt_auth_data(plaintext)?;  // 结果: "AES:<base64>"

// 读取时解密
plaintext = decrypt_auth_data(&auth_data)?;

// 日志中脱敏
let safe_url = mask_password_in_url(&url);
```

### 7.3 注意

- `os_auth` / `trust` 等无凭据认证方式不触发认证配置保存
- 密码/敏感字段写入前必须加密，前缀 `AES:`
- 网络认证配置支持全局和项目级两层查询

---

## 八、测试运行指南

### 8.1 文件数据库测试（无需外部服务）

```bash
cargo test --test file_database_tests -- --test-threads=1
```

### 8.2 MySQL 测试（需要 MySQL 服务）

```bash
# 准备 MySQL 服务
docker run -d --name mysql-test -p 3306:3306 -e MYSQL_ROOT_PASSWORD=root mysql:8.0

# 运行测试
cargo test --test mysql_integration_tests -- --ignored --test-threads=1
```

### 8.3 PostgreSQL 测试（需要 PostgreSQL 服务）

```bash
# 准备 PostgreSQL 服务
docker run -d --name pg-test -p 5432:5432 -e POSTGRES_PASSWORD=postgresql postgres:16

# 创建测试数据库
docker exec pg-test psql -U postgres -c "CREATE DATABASE business_db"

# 运行测试
cargo test --test postgresql_tests -- --ignored --test-threads=1
```

### 8.4 运行全部无需外部服务的测试

```bash
cargo test --test file_database_tests --test data_source_tests --test driver_registry_tests --test connection_commands_tests
```

---

## 九、常见问题排查

### 9.1 MySQL 连接失败

| 症状 | 原因 | 解决 |
|------|------|------|
| `Public Key Retrieval is not allowed` | MySQL 8.0+ caching_sha2_password | 添加 `allowPublicKeyRetrieval=TRUE` |
| `Access denied` | 密码错误或用户权限不足 | 检查用户名密码，确认 `mysql_native_password` |
| `Communications link failure` | 网络不可达 | 检查 host/port/防火墙 |

### 9.2 PostgreSQL 连接失败

| 症状 | 原因 | 解决 |
|------|------|------|
| `no pg_hba.conf entry` | 未配置远程访问 | 编辑 `pg_hba.conf` 添加信任 |
| `password authentication failed` | 密码错误 | 检查密码 |
| `SSL is required` | 服务器要求 SSL | 使用 `postgres_native` 驱动 + `sslmode=require` |

### 9.3 SQLite 连接失败

| 症状 | 原因 | 解决 |
|------|------|------|
| `unable to open database file` | 路径不存在 | 先创建父目录 |
| `database is locked` | 并发写入冲突 | 使用 WAL 模式 |
| `permission denied` | 无写入权限 | 检查文件权限 |

### 9.4 DuckDB 连接失败

| 症状 | 原因 | 解决 |
|------|------|------|
| `IO Error` 打开文件 | 路径不存在 | 先创建父目录 |
| 查询阻塞 | 错误查询卡住线程 | 使用 `spawn_blocking` 隔离 |