# 网络连接方式 迭代设计文档

> 版本：v1.1（开发中）
> 创建：2026-05-19
> 状态：🚧 开发中 — v0.5.0 后端核心已完成（SSH/SSL）
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
ConnectionService (服务层) ── URL 改写（SSH 隧道）
├── apply_network_method()
│   ├── SSH: create_ssh_tunnel_port() → 改写 URL host:port
│   ├── SSL: 透传（sqlx 原生支持 SSL 参数）
│   └── Proxy: 暂不支持（后续通过 wrapping stream）
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

### 2.4 五种 Connector 实现完成度矩阵

| 连接器 | 结构体定义 | 工厂注册 | 运行时实现 | 服务层集成 | 状态 |
|-------|-----------|---------|-----------|-----------|------|
| DirectConnector | ✅ | ✅ | ✅ | ✅ 透传 | ✅ 完成 |
| SslConnector | ✅ | ✅ | ✅ CA+mTLS | ✅ 透传(sqlx) | ✅ 完成 |
| SshTunnelConnector | ✅ | ✅ | ✅ russh隧道 | ✅ URL改写 | ✅ 完成 |
| HttpProxyConnector | ✅ | ✅ | ✅ CONNECT | ❌ 待集成 | 🟡 待定 |
| SocksProxyConnector | ✅ | ✅ | ✅ SOCKS5 | ❌ 待集成 | 🟡 待定 |

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

HTTP Proxy 和 SOCKS5 Proxy 的 **Connector 层已完整实现**，但尚未集成到 `ConnectionService` 的服务层。

**缺失项**：`apply_network_method()` 中对 `HttpProxy`/`SocksProxy` 的处理返回 `NotSupported`。

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
│       └── Proxy: NotSupported（待集成）                                   │
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
| Proxy only | 🟡 | Connector 层已实现，service 层待集成 |
| Proxy → SSL | ❌ | 需要代理 → TLS 嵌套 |
| Proxy → SSH | ❌ | 需要代理 → SSH 隧道链 |

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

### 7.2 待完成项 🚧

| 序号 | 任务 | 优先级 | 预估 |
|------|------|--------|------|
| F1 | 前端：网络配置管理 UI（CRUD + 测试）| 🔴 P0 | 3天 |
| F2 | 前端：新建连接时选择网络配置 | 🔴 P0 | 1天 |
| F3 | Proxy 代理 service 层集成 | 🟡 P1 | 2天 |
| F4 | SSH 隧道集成测试（端到端）| 🟡 P1 | 2天 |
| F5 | 结构化持久化表（ssh_tunnel_configs 等）| 🟡 P2 | 2天 |

### 7.3 后续版本规划

```
v0.6.0（预计 7-10 天）:
  ├── SSH Agent 转发 + Host Key 校验 + 用户确认弹窗
  ├── SSH 多跳隧道链（hops）
  ├── SSL 模式智能感知（MySQL/PostgreSQL ssl_mode 自动映射）
  ├── SSL 证书过期检测
  ├── 代理方式 service 层集成（wrapping TcpStream）
  ├── no_proxy 规则匹配
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