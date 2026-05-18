# 网络连接方式 迭代设计文档

> 版本：v1.1
> 更新：2026-05-19
> 状态：🚧 开发中 — v0.5.0 后端核心已完成（SSH/SSL/Proxy + test_network_config）
> 目标版本：v0.5.0 / v0.6.0 / v0.7.0

---

## 一、概述

本文档覆盖 RdataStation 数据库连接层的三种网络连接方式：

| 连接方式 | 英文 | 典型场景 |
|---------|------|---------|
| **SSH 隧道** | SSH Tunnel | 通过跳板机/堡垒机访问内网数据库 |
| **SSL/TLS 加密** | SSL/TLS | 数据库连接加密传输，证书校验 |
| **代理** | Proxy（HTTP/SOCKS）| 通过公司代理或 SOCKS5 访问外部数据库 |

三者在架构中统一建模为 `ConnectionMethod` 枚举，通过 `ConnectionConfig` → `ConnectionService` → `Connector` 三层调度。

---

## 二、当前代码架构现状

### 2.1 代码文件索引

| 文件 | 职责 | 状态 |
|------|------|------|
| [config.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/connection/config.rs) | ConnectionMethod / SshConfig / SslConfig / ProxyConfig 结构体定义 | ✅ |
| [connector.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/connection/connector.rs) | Connector trait + 5 个连接器实现 | ✅ v1.1 |
| [stream.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/connection/stream.rs) | ConnectionStream 枚举（Tcp/Tls/SshTunnel/HttpProxy/SocksProxy）| ✅ |
| [factory.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/connection/factory.rs) | ConnectionFactory 注册调度 | ✅ |
| [connection_service.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/services/connection_service.rs) | 连接服务 + URL改写(SSH隧道) | ✅ v1.1 |
| [connection_commands.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/commands/connection_commands.rs) | Tauri Commands + network_config JSON解析 | ✅ v1.1 |
| [network_store.rs](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/persistence/network_store.rs) | 网络配置持久化 CRUD | ✅ |
| `migrations/global/008_add_data_source_module.sql` | network_configs 表定义 | ✅ |
| `migrations/project_meta/010_add_data_source_module.sql` | 项目级 network_configs 表 | ✅ |

### 2.2 架构分层

```
ConnectionService (服务层) ── URL 改写（SSH 隧道 / 代理 / 协议链路）
├── apply_network_method()
│   ├── Single: SSH/SSL/Proxy → 对应隧道
│   └── Chain: 迭代 hops → process_chain() → 层层端口转发
│
ConnectionFactory (调度层)
├── DirectConnector  ← TCP 直连 ✓
├── SslConnector     ← SSL/TLS 加密 ✓ CA证书/mTLS
├── SshTunnelConnector ← SSH 隧道 ✓ russh实现
├── HttpProxyConnector ← HTTP 代理（CONNECT）✓
└── SocksProxyConnector ← SOCKS5 代理 ✓

create_database(url) → DataSourceRouter::route(config) → DynDatabase
```

### 2.3 API 接口说明

#### 2.3.1 Tauri Command 层

```typescript
// connect_database 接受 network_config_id，自动解析为 ConnectionMethod
{
  db_type: "mysql",
  url: "mysql://user@host:3306/db",
  network_config_id: "net_abc123",  // 引用网络配置
  // ... 其他字段
}
```

#### 2.3.2 网络配置 JSON 格式

```json
// SSH 隧道配置
{
  "network_type": "ssh",
  "config": {
    "host": "ssh.example.com",
    "port": 22,
    "username": "jumpuser",
    "auth_type": "password",
    "password": "...",
    "remote_host": "db.internal",
    "remote_port": 3306,
    "local_port": 0
  }
}

// SSL 配置
{
  "network_type": "ssl",
  "config": {
    "verify_server_cert": true,
    "ca_cert_path": "/path/to/ca.pem",
    "client_cert_path": "/path/to/client-cert.pem",
    "client_key_path": "/path/to/client-key.pem",
    "min_tls_version": "tls1_2"
  }
}
```

#### 2.3.3 网络配置测试命令

```typescript
// test_network_config 独立测试网络配置连通性
// Rust 返回类型: TestNetworkConfigResponse
interface TestNetworkConfigResponse {
  success: boolean
  message: string
  response_time_ms: number
  detail: string | null
}

// SSH: 建立完整隧道测试
// SSL: 验证证书文件存在
// Proxy: 测试 HTTP CONNECT / SOCKS5 代理连通性
invoke('test_network_config', { network_config_id: 'net_abc123' })
```

### 2.4 五种 Connector 实现完成度矩阵

| 连接器 | 结构体定义 | 工厂注册 | 运行时实现 | 服务层集成 | 状态 |
|-------|-----------|---------|-----------|-----------|------|
| DirectConnector | ✅ | ✅ | ✅ | ✅ 透传 | ✅ 完成 |
| SslConnector | ✅ | ✅ | ✅ CA+mTLS | ✅ 透传(sqlx) | ✅ 完成 |
| SshTunnelConnector | ✅ | ✅ | ✅ russh隧道 | ✅ URL改写 | ✅ 完成 |
| HttpProxyConnector | ✅ | ✅ | ✅ CONNECT | ✅ URL改写 | ✅ 完成 |
| SocksProxyConnector | ✅ | ✅ | ✅ SOCKS5 | ✅ URL改写 | ✅ 完成 |

---

## 三、SSH 隧道（SSH Tunnel）✅ v0.5.0 已实现

### 3.1 场景地图

```
场景 A：单跳 SSH 隧道 ✅ 已实现
  本机 ──SSH──► 跳板机 ──TCP──► 数据库:3306
  用户先 SSH 到跳板机，再通过端口转发连数据库

场景 B：多跳 SSH 级联（堡垒机链）🚧 待 v0.6.0
  本机 ──SSH──► 堡垒机A ──SSH──► 堡垒机B ──TCP──► 数据库:5432

场景 C：SSH 隧道 + 端口映射 ✅ 已实现
  localhost:13306 ──SSH──► 跳板机 ──► 内网DB:3306
  前端通过 localhost:tunnel_port 访问，底层走 SSH 隧道

场景 D：SSH Agent 转发 🚧 待 v0.6.0

场景 E：本地端口自动分配 ✅ 已实现
  localhost:0 → 系统自动分配空闲端口 → SSH → DB

场景 F：SSH 连接超时与自动重连 🚧 待 v0.7.0
```

### 3.2 当前结构体设计

[config.rs:60-82](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/connection/config.rs#L60-L82)：

```rust
pub struct SshConfig {
    pub host: String,          // SSH 服务器主机
    pub port: u16,             // SSH 服务器端口（默认 22）
    pub username: String,      // SSH 用户名
    pub auth: SshAuth,         // 认证方式
    pub remote_host: String,   // 目标 DB 主机
    pub remote_port: u16,      // 目标 DB 端口
    pub local_port: u16,       // 本地绑定端口（0=自动）
    pub timeout_secs: u64,     // 超时
}
```

### 3.3 运行时实现（v1.1 已完成）

[connector.rs:286-429](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/connection/connector.rs#L286-L429) `establish_ssh_tunnel`：

```
实际执行流（基于 russh 0.49.2）：
  1. russh::client::connect → SSH 协议握手
  2. authenticate_password / authenticate_publickey → 用户认证
  3. TcpListener::bind(local_bind) → 绑定本地端口
  4. tokio::spawn → 后台任务 accept 本地连接
  5. channel_open_direct_tcpip → 通过 SSH 打开到远程 DB 的端口转发
  6. channel.into_stream() → ChannelStream (AsyncRead + AsyncWrite)
  7. tokio::io::split → 双向拷贝（local ↔ channel）
  8. TcpStream::connect(local_addr) → 返回本地 TcpStream
```

**关键技术点**：

1. **认证方式**：
   - `Password` → `session.authenticate_password(username, password).await`
   - `PrivateKey` → `russh::keys::load_secret_key()` + `PrivateKeyWithHashAlg::new(key, None)` + `session.authenticate_publickey(username, key)`
   - `Agent` → 暂未实现，返回 `NotSupported` 错误

2. **Channel 双向拷贝**：
   ```rust
   let mut channel_stream = channel.into_stream();
   let (mut local_read, mut local_write) = tokio::io::split(local_stream);
   let (mut channel_read, mut channel_write) = tokio::io::split(&mut channel_stream);
   tokio::join!(
       tokio::io::copy(&mut local_read, &mut channel_write),
       tokio::io::copy(&mut channel_read, &mut local_write),
   );
   ```

3. **Session 生命周期管理**：使用 `Arc<tokio::sync::Mutex<Handle>>` 包装，auth 阶段 lock→authenticate→drop lock，端口转发阶段 clone Arc 到 spawn task。

### 3.4 服务层集成：URL 改写

[connection_service.rs:401-480](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/services/connection_service.rs#L401-L480)

```
apply_network_method() SSH 处理流程：
  1. create_ssh_tunnel_port(ssh_config) → (tunnel_stream, local_port)
  2. drop(tunnel_stream) → 释放本地连接，后台 spawn task 继续维护隧道
  3. rewrite_url_host_port(url, "127.0.0.1", local_port)
     → mysql://user:pass@10.0.0.5:3306/db → mysql://user:pass@127.0.0.1:54321/db
  4. create_database(effective_url) → sqlx 连接到 localhost:tunnel_port
```

### 3.5 下一步优化

| 优化项 | 版本 | 说明 |
|--------|------|------|
| SSH Agent 转发 | v0.6.0 | `russh_keys::agent` 模块接入 |
| Host Key 校验 | v0.6.0 | 检查 `known_hosts`，弹出确认对话框 |
| 多跳隧道链 | v0.6.0 | hops 链 + 逐跳 SSH 连接 |
| 自动重连 | v0.7.0 | 网络闪断后自动重建隧道 |

---

## 四、SSL/TLS 加密 ✅ v0.5.0 已实现

### 4.1 场景地图

```
场景 A：单向 SSL（服务器证书校验）✅ 已实现
场景 B：双向 SSL/mTLS（客户端证书认证）✅ 已实现
场景 C：SSL with CA Certificate ✅ 已实现
场景 D：SSL 模式选择 🚧 待 v0.6.0
场景 E：自签名证书开发环境 ✅ 已实现
场景 F：SSL 证书过期检测 🚧 待 v0.6.0
```

### 4.2 当前结构体设计

```rust
pub struct SslConfig {
    pub verify_server_cert: bool,            // 是否验证服务器证书
    pub ca_cert_path: Option<String>,        // CA 证书路径
    pub client_cert_path: Option<String>,    // 客户端证书路径（mTLS）
    pub client_key_path: Option<String>,     // 客户端私钥路径（mTLS）
    pub min_tls_version: TlsVersion,         // 最低 TLS 版本（Tls1_0 ~ Tls1_3）
}
```

### 4.3 运行时实现（v1.1 已完成）

[connector.rs:119-230](file:///e:/myapps/tauirapps/RdataStation/rdata-station/src-tauri/src/core/driver/connection/connector.rs#L119-L230) `establish_tls`：

```
实际执行流（基于 native-tls + tokio-native-tls）：
  1. TlsConnector::builder()
       .danger_accept_invalid_certs(!verify_server_cert)  ← 自签名证书开关
       .min_protocol_version(map_tls_version(min))        ← TLS 版本约束
       .max_protocol_version(Tlsv12)                      ← 默认上限

  2. CA 证书（可选）
     std::fs::read(ca_cert_path) → native_tls::Certificate::from_pem() → builder.add_root_certificate()

  3. 客户端证书 mTLS（可选）
     std::fs::read(cert_path + key_path) → Identity::from_pkcs8()
     失败时 fallback PKCS#12: Identity::from_pkcs12(cert, "")

  4. tokio_connector.connect(domain, tcp_stream).await
```

### 4.4 服务层集成

SSL 在 service 层**不做特殊处理**（透传 URL），因为 sqlx 原生支持 SSL 参数：
- MySQL: `mysql://host/db?ssl-mode=VERIFY_CA&ssl-ca=/path/ca.pem`
- PostgreSQL: `postgres://host/db?sslmode=verify-ca&sslrootcert=/path/ca.pem`

当前 SslConfig 用于 connector 层的 TLS 流加密，未来 v0.6.0 可统一两套 SSL 机制。

### 4.5 下一步优化

| 优化项 | 版本 | 说明 |
|--------|------|------|
| SSL 模式自动映射 | v0.6.0 | MySQL 5 级 / PostgreSQL 6 级 → SslMode 枚举 |
| 证书过期检测 | v0.6.0 | peer_certificate() → 过期前告警 |
| 密码套件配置 | v0.7.0 | 高级安全需求 |

---

## 五、代理（Proxy）

### 5.1 场景地图

```
场景 A：HTTP 代理（公司内网出口）✅ Connector层实现
场景 B：SOCKS5 代理（无认证）✅ Connector层实现
场景 C：SOCKS5 代理（用户名/密码认证）✅ Connector层实现
场景 D：代理旁路规则（no_proxy）🚧 未实现
场景 E：代理链（多个代理串联）🚧 未实现
场景 F：不同数据库走不同代理 🚧 未实现
```

### 5.2 当前状态

HTTP Proxy 和 SOCKS5 Proxy 的 **Connector 层已完整实现**，并通过**本地端口转发 + URL 改写**模式集成到 `ConnectionService` 服务层。

**已实现**：
- `apply_network_method()` 中对 `HttpProxy`/`SocksProxy` 调用 `create_proxy_tunnel_port()` → `rewrite_url_host_port()`
- URL 解析自动识别目标数据库主机端口，默认 MySQL:3306 / PostgreSQL:5432
- HTTP CONNECT 代理 Basic Auth 认证、SOCKS5 代理用户名密码认证
- `test_network_config` Tauri Command 独立测试代理连通性

### 5.3 下一步优化

| 优化项 | 版本 | 说明 |
|--------|------|------|
| 代理方式服务层集成 | v0.5.1 | wrapping TcpStream 注入 sqlx |
| no_proxy 规则匹配 | v0.6.0 | CIDR + 域名 glob 匹配 |
| 代理链支持 | v0.6.0 | upstream_proxy 嵌套 |
| NTLM 认证 | v0.7.0 | 企业环境常见需求 |

---

## 六、统一连接配置流程（ConnectionConfig Pipeline）

### 6.1 前端 → 后端 连接建立全链路（v1.1 已实现）

```
┌─ 前端 ───────────────────────────────────────────────────────────────────┐
│  ConnectDatabaseInput {                                                   │
│    db_type: "mysql",                                                      │
│    url: "mysql://user@host:3306/db",                                      │
│    connection_type: "project",                                            │
│    network_config_id: "net_abc123",   ← 引用预配置的网络配置              │
│  }                                                                        │
└──────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─ connect_database Command ───────────────────────────────────────────────┐
│  1. 校验 driver / environment / auth 三步链                                │
│  2. parse_network_method(network_config_id):                              │
│     - 查询 global/project DB 的 network_configs 表                        │
│     - parse_config_json(network_type, config):                            │
│       "ssh"  → serde_json → SshConfig → ConnectionMethod::Ssh            │
│       "ssl"  → serde_json → SslConfig → ConnectionMethod::Ssl            │
│       "proxy"→ serde_json → ProxyConfig→ ConnectionMethod::HttpProxy/Socks│
│  3. service.connect_with_type(..., network_method)                       │
└──────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─ ConnectionService ──────────────────────────────────────────────────────┐
│  connect_with_type():                                                     │
│    1. 参数校验（url 非空）                                                │
│    2. apply_network_method(url, &network_method, &conn_id):               │
│       ├── SSH: create_ssh_tunnel_port() → rewrite URL → localhost:port   │
│       ├── SSL: 透传 URL（sqlx 原生 SSL 参数）                             │
│       └── Proxy: create_proxy_tunnel_port() → rewrite URL → localhost:port                                   │
│    3. create_database(effective_url) → DataSourceRouter::route()         │
│    4. 保存到 ConnectionManager + SQLite（safe_url 使用原始 URL）          │
│    5. 保存到最近连接记录                                                   │
└──────────────────────────────────────────────────────────────────────────┘
```

### 6.2 连接方式组合矩阵

| 组合 | 是否支持 | 说明 |
|------|---------|------|
| Direct | ✅ | 最常见 |
| SSL only | ✅ | 直接 TLS（CA证书 + mTLS）|
| SSH only | ✅ | russh 隧道 + URL 改写 |
| Proxy only | ✅ | URL改写 + 端口转发 |
| Proxy → SSH | ✅ | Proxy 隧道作为 SSH 握手通道 |
| SSH → SSH | ✅ | 多跳堡垒机链 |
| Proxy → SSH → DB | ✅ | 公司代理 → 跳板机 → 内网 DB |
| SSH → SSL → DB | ✅ | sqlx 在 SSH 隧道内处理 SSL |
| Proxy → SSL → DB | ❌ | 需要代理 → TLS 嵌套（v0.6.0）|

### 6.3 协议链路配置 JSON 格式

```json
// Proxy → SSH → DB 链式配置
{
  "network_type": "chain",
  "config": [
    {
      "type": "http_proxy",
      "host": "corp-proxy.example.com",
      "port": 8080,
      "auth": { "username": "user", "password": "pass" }
    },
    {
      "type": "ssh",
      "host": "bastion.example.com",
      "port": 22,
      "username": "jumpuser",
      "auth_type": "password",
      "password": "...",
      "remote_host": "db.internal",
      "remote_port": 3306
    }
  ]
}
```

### 6.4 协议链路处理流程

```
process_chain(url="mysql://db.internal:3306/db", hops=[Proxy→SSH]):

Hop 0 (HttpProxy): target = SSH hop 的 host:port = "bastion.example.com:22"
  create_proxy_tunnel_port(proxy, "bastion.example.com", 22)
  → localhost:13301 → proxy → bastion:22
  tunnel_port = Some(13301)

Hop 1 (Ssh): connect_override = ("127.0.0.1", 13301)
  create_ssh_tunnel_port(ssh, connect_override)
  → 通过 localhost:13301 (经 proxy) SSH 握手到 bastion
  → channel_open_direct_tcpip("db.internal", 3306)
  → localhost:13302 → SSH → db.internal:3306
  tunnel_port = Some(13302)

Final: rewrite_url(url, "127.0.0.1", 13302)
  → "mysql://127.0.0.1:13302/db"
  sqlx 连接 localhost:13302 → proxy → SSH → DB ✅
```

**核心原理：每跳创建一个本地端口转发，本地端口作为下一跳的 TCP 入口，层层嵌套。**

---

## 七、v0.5.0 开发计划

### 7.1 已完成项 ✅

| 序号 | 任务 | 状态 |
|------|------|------|
| P0 | SSH 隧道：russh 握手 + 端口转发 + 双向拷贝 | ✅ |
| P0 | SSH 隧道：Password / PrivateKey 认证 | ✅ |
| P1 | SSL：CA 证书 + 客户端证书 mTLS (PKCS8/PKCS12) | ✅ |
| P1 | SSL：TLS 版本映射 (TlsVersion → native_tls::Protocol) | ✅ |
| P2 | ConnectionService: apply_network_method() URL 改写 | ✅ |
| P2 | ConnectionCommands: parse_network_method() JSON 解析 | ✅ |
| P3 | cargo check + clippy -D warnings 全部通过 | ✅ |
| P3 | Proxy 代理 service 层集成（本地端口转发 + URL 改写）| ✅ |
| P3 | test_network_config Tauri Command（SSH/SSL/Proxy 连通性测试）| ✅ |
| P2 | SSH Host Key 指纹日志记录 | ✅ |
| P1 | 协议链路（Chain）：Proxy→SSH / SSH→SSH / Proxy→SSH→DB | ✅ |

### 7.2 待完成项 🚧

| 序号 | 任务 | 优先级 | 预估 |
|------|------|--------|------|
| F1 | 前端：网络配置管理 UI（CRUD + 测试）| 🔴 P0 | 3天 |
| F2 | 前端：新建连接时选择网络配置 | 🔴 P0 | 1天 |
| F3 | ~~Proxy 代理 service 层集成~~ | ~~🟡 P1~~ | ✅ v0.5.0 |
| F4 | SSH 隧道集成测试（端到端）| 🟡 P1 | 2天 |
| F5 | 结构化持久化表（ssh_tunnel_configs 等）| 🟡 P2 | 2天 |

### 7.3 后续版本规划

```
v0.6.0（预计 7-10 天）:
  ├── SSH Agent 转发 + Host Key 校验 + 用户确认弹窗
  ├── SSH 多跳隧道链（已实现基础链式，v0.6.0 加 hops 配置可视化）
  ├── SSL 模式智能感知（MySQL/PostgreSQL ssl_mode 自动映射）
  ├── SSL 证书过期检测
  ├── no_proxy 规则匹配
  ├── 代理 → SSL 嵌套（Proxy → TLS）
  └── 连接方式性能对比监控

v0.7.0（预计 5-7 天）:
  ├── SSH 隧道自动重连
  ├── NTLM 代理认证
  ├── TLS 1.3 密码套件配置
  └── 连接方式性能对比（Direct/SSL/SSH/Proxy 延迟监控）
```

---

## 八、前端 UI 设计要点

详见前端设计文档：`docs/frontend/NETWORK-CONFIG-UI-DESIGN.md`

### 核心页面

1. **网络配置管理页** — NetworkConfigManager.vue
   - 列表视图：所有网络配置（SSH/SSL/Proxy 分类 Tab）
   - 新建/编辑表单：根据 network_type 动态渲染表单字段
   - 测试连接按钮：验证 SSH/SSL/Proxy 配置

2. **新建连接对话框改造** — NewConnectionDialog.vue
   - 增加"网络配置"下拉选择框（可选）
   - 关联 network_config_id 字段

### 关键技术点

- dockview-vue 布局基座
- naive-ui 组件（NTabs, NSelect, NInput, NForm 等）
- lucide-vue-next 图标

---

## 九、附录

### A. 依赖项

| 依赖 | 用途 | 版本 | 状态 |
|------|------|------|------|
| `russh` | SSH 协议实现 | 0.49.2 | ✅ 已使用 |
| `russh-keys` | SSH 密钥管理 | 0.49.2 | ✅ 已使用 |
| `native-tls` | TLS 实现 | 0.2.13 | ✅ 已使用 |
| `tokio-native-tls` | 异步 TLS 包装 | 0.3.1 | ✅ 已使用 |
| `tokio-socks` | SOCKS5 代理 | 0.5.2 | ✅ 已使用 |
| `base64` | HTTP Proxy Basic Auth 编码 | 0.22.1 | ✅ 已使用 |

### B. 版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.1 | 2026-05-19 | P0 SSH隧道 + P1 SSL证书 + service/cmd 集成完成，cargo check/clippy 通过 |
| v1.0 | 2026-05-19 | 初始规划文档，场景地图 + 结构体设计 + 三期路线图 |

---

## 十、动态协议链设计（v0.6.0 核心特性）

### 10.1 设计目标

将当前固定的单一 `ConnectionMethod` 扩展为**动态有序协议链**，支持用户在数据源连接时自由组合 SSH 隧道、代理、SSL/TLS 加密，按序执行。

### 10.2 三种协议的本质差异

这是理解协议链设计的基础——三者做的事情完全不同：

```
协议     | 本质              | 产生新网络节点？ | 延迟代价        | 在链中的位置
─────────┼───────────────────┼─────────────────┼────────────────┼─────────────
SSH      | 隧道创建器         | ✅ 是            | 高 (5-30ms/跳)  | 任意位置
Proxy    | 流量中继器         | ✅ 是            | 中 (3-15ms/跳)  | 任意位置
SSL/TLS  | 流加密包装器       | ❌ 否            | 极低 (仅握手)   | 必须最后
```

**关键认知**：
- SSL/TLS 拿一个已建立好的 TCP 流做 TLS 握手，只能在链的最末端。不可能在已加密的 TLS 流上再建 SSH（SSH 需要 TCP 三次握手，TLS 流已是一个加密字节流）。
- SSH 和 Proxy 每增加一个，就在数据路径上插入一个真实的网络节点，增加一次 TCP 往返延迟。
- 每经过一个 SSH/Proxy 跳，网络的"立足点"就变了，新立足点可能有自己的网络约束，因此**交替穿插是完全合理的**。

### 10.3 协议链结构

```
正确的协议链结构：

  [N 个 SSH/Proxy 网络跳] → [可选 1 层 TLS 加密] → [DB]

  N = 网络跳数（SSH + Proxy 个数），TLS 不占跳数但占用末尾位置
```

### 10.4 全场景矩阵（N = SSH/Proxy 网络跳数）

#### 0 跳 — 频次 ~85%

| # | 链路 | 频次 | 场景 |
|---|------|------|------|
| 1 | 直连 → DB | 80% | 本地开发、内网直连 |
| 2 | TLS → DB | 5% | 云数据库强制 TLS（RDS/Cloud SQL），无跳板 |

#### 1 跳 — 频次 ~13%

| # | 链路 | 频次 | 场景 |
|---|------|------|------|
| 3 | SSH → DB | 8% | 单堡垒机访问内网库 |
| 4 | Proxy → DB | 2% | 公司代理出口访问外部 DB |
| 5 | SSH → TLS → DB | 2.5% | 堡垒机 + 云数据库 TLS |
| 6 | Proxy → TLS → DB | 0.5% | 代理出口 + TLS |

#### 2 跳 — 频次 ~11%（含交替穿插）

| # | 链路 | 频次 | 场景 |
|---|------|------|------|
| 7 | **Proxy → SSH → DB** | 3% | ★ 公司代理 → 堡垒机 → 内网 DB，最常用多跳 |
| 8 | **SSH → Proxy → DB** | 2% | ★ 跳板机入内网 → 内网代理访问受限 DB |
| 9 | SSH₁ → SSH₂ → DB | 1.5% | 双跳板：DMZ + 生产网段 |
| 10 | Proxy₁ → Proxy₂ → DB | 0.3% | 代理链：公司代理 → 境外代理 |
| 11 | Proxy → SSH → TLS → DB | 2.5% | 代理 + 堡垒机 + TLS 云数据库 |
| 12 | SSH → Proxy → TLS → DB | 0.8% | VPN跳板 + 内网代理 + TLS |
| 13 | SSH₁ → SSH₂ → TLS → DB | 0.5% | 双跳板 + TLS |

#### 3 跳 — 频次 ~3%（交替穿插为主）

| # | 链路 | 频次 | 场景 |
|---|------|------|------|
| 14 | **SSH → Proxy → SSH → DB** | 1% | ★ VPN跳板 → 内网代理审计 → 生产跳板 |
| 15 | Proxy → SSH₁ → SSH₂ → DB | 1.2% | 代理 + 双跳板 |
| 16 | Proxy₁ → Proxy₂ → SSH → DB | 0.4% | 双代理 + 跳板 |
| 17 | SSH₁ → SSH₂ → Proxy → DB | 0.2% | 双跳板入隔离区 + 代理出站 |
| 18 | +TLS 变体 | 0.3% | 以上变体末尾加 TLS |

**SSH → Proxy → SSH 典型场景**：开发者 → VPN跳板机(入内网) → 内网HTTP代理(审计) → 生产跳板机(隔离) → DB。这是中大型企业的标配架构。

#### 4 跳 — 频次 ~0.2%

| # | 链路 | 场景 |
|---|------|------|
| 19 | Proxy → SSH → Proxy → SSH → DB | 双交替：多区域 + 多安全域。跨国金融机构合规架构 |
| 20 | SSH → Proxy → SSH → Proxy → DB | 双交替反向 |
| 21 | Proxy₁ → Proxy₂ → SSH₁ → SSH₂ → DB | 双代理 + 双跳板 |

#### 5 跳 — 频次 < 0.01%

仅理论组合，实际网络中不存在需要跨越 5 个不同安全域的场景。

### 10.5 频次漏斗图

```
N=0  ████████████████████████████████████████  85%  直连/纯TLS
N=1  ██████                                    13%  单跳
N=2  █████                                     11%  代理+SSH(含交替)
N=3  ██                                         3%  双跳板+代理等(含交替)
N=4  ▏                                        0.2%  双交替
N=5  ·                                       <0.01%
────────────────────────────────────────────────────────
      3跳覆盖 99%    4跳覆盖 99.8%    5跳覆盖全部
```

### 10.6 嵌套上限设计

| 上限 | 覆盖 | 决定 |
|------|------|------|
| **代码硬上限** | 4 个 SSH/Proxy 网络跳 + 1 层 TLS + DB = **最多 6 层** | 覆盖 99.8%，超过时拒绝并提示 |
| **UI 黄色警告** | 3 跳（含）以上 | 显示延迟风险警告："当前协议链包含 N 个网络跳，建连延迟预期 ~XXms" |
| **V5 原型默认** | 1 跳 SSH + 1 层 TLS + 1 跳 Proxy = 2 跳 + TLS | 每种协议至少保留 1 个实例，开关控制是否生效 |

**不推荐无限套娃的原因**：每增加一个 SSH/Proxy 跳，建连时间增加 30-80ms，每个查询增加节点 RTT 延迟，故障点 +1，调试复杂度翻倍。

### 10.7 每跳延迟基准（参考值）

| 跳类型 | TCP 握手 | 协议握手 | 总延迟增量 |
|--------|----------|----------|-----------|
| 本地/内网 SSH | ~2ms | ~15ms | ~17ms |
| 跨区域 SSH | ~30ms | ~20ms | ~50ms |
| HTTP Proxy (CONNECT) | ~5ms | ~5ms | ~10ms |
| SOCKS5 Proxy | ~5ms | ~8ms | ~13ms |
| TLS 握手 | N/A | ~15ms | ~15ms |

```
最大延迟预估（4 跳 + TLS）：
内网场景：17ms×4 + 15ms ≈ 83ms  ← 可接受
跨区域场景：50ms×4 + 30ms ≈ 230ms ← 有感知但仍可用
```

### 10.8 协议链 JSON 数据模型（规划）

```json
{
  "chain": [
    { "type": "ssh", "profile_id": "ssh-prod-bastion" },
    { "type": "proxy", "profile_id": "proxy-corp-socks5" },
    { "type": "ssh", "profile_id": "ssh-internal-bastion" },
    { "type": "ssl", "profile_id": "ssl-rds-default" }
  ]
}
```

### 10.9 v0.6.0 后端实现要点

1. `ConnectionMethod` → `Vec<Hop>` 数据结构升级
2. `HopExecutor` 链式引擎：按序执行每个 hop，SSH/Proxy 产出地址改写或流包装，SSL 在最终流上包装
3. 硬上限：`const MAX_HOP_CHAIN: usize = 4`（SSH/Proxy 跳数）
4. 隧道句柄数组管理：`Vec<TunnelHandle>` 在连接生命周期中保持所有 SSH 隧道 alive
5. 每跳超时累计：前 N 跳超时之和 + 最终跳超时 = 总超时

---

## 十一、大厂网页跳板机集成可行性分析

### 11.1 什么是"网页跳板机"

大厂常见的跳板机/堡垒机不再是传统 SSH 22 端口的服务器，而是**基于 Web 的零信任接入网关**：

| 产品 | 厂商 | 接入方式 |
|------|------|----------|
| AWS Session Manager | AWS | `aws ssm start-session` → WebSocket → EC2 |
| Cloud IAP (Identity-Aware Proxy) | GCP | `gcloud compute ssh --tunnel-through-iap` → HTTP 代理 |
| Azure Bastion | Azure | HTML5 WebSocket，浏览器内终端 |
| JumpServer | 开源 | Web Terminal + SSH 代理网关 |
| Teleport | 开源/商业 | `tsh` CLI → WebSocket/HTTP2 → 目标节点 |
| 阿里云堡垒机 | 阿里云 | Web 运维门户 + SSH/RDP 代理 |

**共同特征**：
- 不暴露标准 SSH 22 端口
- 通过 WebSocket / HTTP2 隧道传输
- 使用 Token/Cookie/OAuth 而非 SSH Key 认证
- 通常有 CLI 工具可以创建**本地代理端口**

### 11.2 桌面客户端的集成方案

#### 方案 A：CLI 钩子（推荐，通用性最好）

```
用户在 RdataStation 中配置"前置命令"：

1. 常规 Tab 填连接信息（如 Aurora host:port）
2. 网络 Tab 选择"外部隧道"类型
3. 填写前置命令：
   $ aws ssm start-session \
       --target i-1234567890abcdef0 \
       --document-name AWS-StartPortForwardingSessionToRemoteHost \
       --parameters '{"host":["mydb.cluster-xxx.rds.amazonaws.com"],"portNumber":["3306"],"localPortNumber":["13306"]}'

4. 点击"连接" → RdataStation 执行前置命令（后台进程）
5. 等待 localhost:13306 就绪 → sqlx 连接 127.0.0.1:13306

同样适用于：
- gcloud compute ssh --tunnel-through-iap --ssh-flag="-L 15432:db:5432"
- tsh proxy db --tunnel --port=15432  (Teleport)
```

**优点**：零侵入，不依赖任何 SDK，只需要 `child_process.spawn` + 端口就绪检测。
**实现**：在 `ConnectionService` 增加 `pre_connect_commands: Vec<String>` 字段。

#### 方案 B：SDK 集成（深度集成，按厂商逐个实现）

| 厂商 | Rust SDK | 可行性 |
|------|----------|--------|
| AWS SSM | `aws-sdk-ssm` (官方) | ✅ 可集成，通过 `StartSession` API + WebSocket |
| GCP IAP | 无官方 Rust SDK | 🟡 需通过 REST/WebSocket 自实现 |
| Teleport | `tsh` CLI 或 gRPC API | 🟡 CLI 包装更实用 |

**缺点**：维护成本高，每个厂商需要单独的 Connector 实现。

#### 方案 C：用户手动启动隧道（当前可行方案）

```
用户手动在终端执行：
$ aws ssm start-session ... --localPortNumber 13306

然后在 RdataStation 中连接 127.0.0.1:13306
（或者在常规 Tab 中填 localhost:13306，网络 Tab 设为"直连"）
```

这是当前就支持的，零开发成本。

### 11.3 推荐路线

| 阶段 | 方案 | 版本 |
|------|------|------|
| 当前 | 方案 C：用户手动启动隧道 | ✅ v0.5.0 已可用 |
| 短期 | 方案 A：`pre_connect_commands` 前置命令钩子 | v0.7.0 |
| 中期 | 方案 A 增强：端口就绪检测 + 进程生命周期管理 | v0.8.0 |
| 长期 | 方案 B：AWS SSM SDK 深度集成 | v1.0+ |

**结论**：网页跳板机完全可以集成。最务实的路径是先支持"前置 shell 命令"，利用各厂商已有的 CLI 工具做端口转发，RdataStation 只需管理进程生命周期 + 检测端口就绪。这个方案通用性强、零依赖、覆盖所有厂商。