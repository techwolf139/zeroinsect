这是一个基于 **Rust** 语言构建高性能、轻量级 MQTT 通用聊天软件的设计方案。

Rust 语言的内存安全（Memory Safety）、无垃圾回收（No GC）以及基于 `async/await` 的高并发模型（如 Tokio），使其成为构建高吞吐、低延迟 MQTT Broker 的完美选择。

---

# 方案名称：Rust-MQTT-Chat (RMC) 架构设计

## 1. 项目概述

*   **目标**：构建一个支持单聊、群聊、离线消息、状态感知的通用即时通讯（IM）系统。
*   **核心协议**：MQTT v3.1.1 / v5.0 (推荐 v5.0，支持自定义属性)。
*   **核心语言**：Rust (Edition 2021+)。
*   **设计原则**：轻量级、低延迟、高并发、易扩展。

## 2. 系统整体架构

系统采用 **存算分离** 和 **微服务** 融合的架构。

```text
[客户端 (App/Web)] 
      | (TLS/WSS)
      v
[ 负载均衡 (Nginx/HAProxy) ]
      |
      v
[ MQTT Broker 集群 (Rust Core) ] <---> [ 认证与业务服务 (Rust API) ]
      |               |
      v               v
  [ 内建存储 ]
(会话/订阅树/缓存)    (用户数据/群组)      (消息历史/日志)
```

### 核心组件说明

1.  **MQTT Broker (Rust 实现)**:
    *   负责连接维持、协议解析、发布订阅路由、QoS 消息保障。
    *   **不处理**复杂的业务逻辑（如好友关系、群管理），只负责消息透传。
2.  **业务服务 (API Server)**:
    *   处理用户注册、登录（颁发 Token）、加好友、建群等 HTTP/gRPC 请求。
3.  **存储层**:
    *   **内建存储**: 存储 MQTT Session（会话状态）、在线状态、飞行窗口（In-flight messages）。存储用户资料、关系链。存储历史聊天记录（写多读少）。

## 3. Rust MQTT Broker 核心设计

这是本方案的灵魂所在。我们将基于 Rust 的异步生态构建 Broker。

### 3.1 技术选型
*   **异步运行时**: `Tokio` (行业标准，提供最强的 IO 调度)。
*   **协议解析**: `mqtt-bytes` (高性能零拷贝解析) 或 `ntex-mqtt`。
*   **Actor 模型**: `Actix` 或 `Tokio Tasks + Channels` (管理每个连接的状态)。
*   **内部通信**: `Flume` 或 `Tokio MPSC` (多生产者单消费者通道)。

### 3.2 模块划分
1.  **Transport Layer (传输层)**:
    *   支持 TCP (原生 MQTT) 和 WebSocket (Web端)。
    *   集成 `rustls` 或 `openssl` 处理 TLS 加密。
2.  **Session Manager (会话管理)**:
    *   使用 `DashMap` (并发 HashMap) 在内存中维护 `ClientID -> ConnectionHandle` 的映射。
    *   处理 `Clean Session` 逻辑。如果为 `false`，需持久化订阅关系到 Redis。
3.  **Topic Router (路由树)**:
    *   实现一个基于 **Radix Tree (基数树)** 的数据结构，用于快速匹配 MQTT 主题（支持通配符 `+` 和 `#`）。
    *   Rust 优势：利用 `RwLock` 实现极高的并发读性能。
4.  **Message Dispatcher (分发器)**:
    *   收到消息后，查询路由树，找到所有订阅者。
    *   **扇出 (Fan-out) 优化**: 对于大群聊，不使用循环逐个发送，而是将消息推送到共享的 Broadcast Channel，或者使用“写扩散”策略结合 Redis Pub/Sub。

### 3.3 Rust 代码结构示意 (伪代码)

```rust
use tokio::net::TcpListener;
use tokio::sync::mpsc;

struct Session {
    client_id: String,
    subscriptions: Vec<String>,
    sender: mpsc::Sender<Packet>, // 发送数据给客户端的通道
}

struct BrokerState {
    // 线程安全的会话存储
    sessions: DashMap<String, Session>,
    // 路由树
    router: RwLock<TopicTree>,
}

async fn handle_connection(socket: TcpStream, state: Arc<BrokerState>) {
    let (mut reader, mut writer) = socket.split();
    // 1. MQTT Connect Handshake (Auth)
    // 2. Loop read packets
    loop {
        let packet = read_packet(&mut reader).await.unwrap();
        match packet {
            Packet::Publish(p) => {
                // 查找订阅者并转发
                let subscribers = state.router.read().await.matches(&p.topic);
                for sub in subscribers {
                    if let Some(sess) = state.sessions.get(sub) {
                        sess.sender.send(p.clone()).await;
                    }
                }
            }
            Packet::Subscribe(s) => { /* 更新路由树 */ }
            _ => {}
        }
    }
}
```

## 4. MQTT Topic 设计 (通信协议)

设计清晰的 Topic 是实现聊天功能的关键。

| 功能 | Topic 模式 | 权限 | Payload (JSON/Protobuf) | 说明 |
| :--- | :--- | :--- | :--- | :--- |
| **私聊** | `chat/u/{target_id}` | 仅目标用户Sub | `{msg_id, content, type, time}` | 发送给某人 |
| **群聊** | `chat/g/{group_id}` | 群成员Sub | `{sender_id, content, ...}` | 发送到群组 |
| **通知** | `sys/u/{user_id}` | 仅用户Sub | `{type: "friend_req", ...}` | 系统通知(加好友等) |
| **状态** | `status/u/{user_id}` | 好友Sub | `online` / `offline` | 此时利用 Retain 消息 |
| **上行** | `inbox/server` | 仅Pub | `{cmd: "ack", ...}` | 客户端发送信令给后端 |

## 5. 关键功能实现细节

### 5.1 认证与鉴权 (Authentication)
*   MQTT `CONNECT` 报文包含 `Username` 和 `Password`。
*   **Rust 实现**: Broker 收到 Connect 包后，通过 HTTP Client (如 `reqwest`) 异步调用业务后台的 API 验证 Token。
*   为了性能，Broker 内部维护一个短时的 LRU Cache，避免频繁请求 Auth Server。

### 5.2 离线消息处理 (Offline Messages)
*   **场景**: 用户 B 不在线，用户 A 给 B 发消息。
*   **实现**:
    1.  Broker 检测到 B 无活跃连接。
    2.  如果 QoS=1，Broker 将消息序列化并存入 Redis List 或 Cassandra (Key: `offline:user_id`)。
    3.  B 上线 (Connect) 且 `CleanSession=false`。
    4.  Broker 触发 Hook，从存储中拉取离线消息并推送给 B。

### 5.3 消息可靠性 (QoS 设计)
*   **聊天消息**: 使用 **QoS 1** (At least once)。确保消息到达 Broker，Broker 确保投递给接收方。客户端需处理去重。
*   **状态/正在输入**: 使用 **QoS 0**。丢了也无所谓，追求低延迟。
*   **消息存储**: 所有的 QoS 1 消息在分发前，异步写入消息历史数据库（用于漫游）。

## 6. 性能优化 (Rust 特性应用)

1.  **Zero-Copy (零拷贝)**:
    *   解析 MQTT 协议时，尽量使用 `Bytes` crate，通过切片（Slicing）引用网络缓冲区的数据，而不是复制字符串。
2.  **Backpressure (背压控制)**:
    *   利用 Tokio 的 Channel 容量限制。如果客户端网络卡顿，发送队列满了，Broker 应暂停从该客户端读取数据或丢弃 QoS 0 消息，防止内存撑爆。
3.  **IO 多路复用 (Epoll/Kqueue)**:
    *   Tokio 底层自动处理，单线程可轻松挂载数万个空闲连接。
4.  **连接保活**:
    *   使用高效的时间轮 (Timer Wheel) 算法管理 MQTT KeepAlive 心跳，比单纯的 `sleep` 更加节省资源。


持久化采用 **嵌入式 Key-Value (KV) 存储引擎**。

这种方案将数据以文件的形式直接存储在磁盘上，**无需安装任何外部数据库软件**（，数据库引擎直接编译进 Rust 二进制文件中。

# 方案：Rust 嵌入式 KV 存储 (NoSQL)

## 1. 技术选型

我们采用 **LSM-Tree** 或 **B-Tree** 结构的嵌入式存储引擎。

*   **推荐引擎**: **RocksDB** (通过 `rust-rocksdb`) 或 **Redb** / **Sled** (纯 Rust 实现)。
    *   **RocksDB**: Facebook 开发，工业级标准，读写性能极强（适合高吞吐聊天）。
    *   **Redb**: 纯 Rust 实现，ACID 事务支持，零拷贝，安全性极高（适合 Rust 项目）。
*   **序列化**: `Serde` + `Bincode` (二进制) 或 `Serde JSON`。

**本方案以 `RocksDB` 为例**，因为它在处理海量写入（聊天消息日志）时性能最好。

## 2. 架构图 (极简版)

```text
[ 客户端 ]
    |
    v
[ MQTT Broker (Rust Binary) ]
    |
    +-- [ 内存: DashMap (Session/PubSub) ]
    |
    +-- [ 存储引擎: RocksDB / Redb ] <--- (编译在 APP 内部)
            |
            +-- ./data/users.db   (用户数据)
            +-- ./data/messages.db (离线消息 & 历史)
```

## 3. Key-Value 数据模型设计

由于没有 SQL 的表结构，我们需要通过 **Key 的前缀设计 (Key Design)** 来模拟表和索引。

### 3.1 用户存储 (Users)
存储用户的基础信息。

*   **Key 格式**: `user:{user_id}`
*   **Value**: JSON/Bincode 序列化后的 User 结构体。

```json
// Value 示例
{
  "username": "alice",
  "password_hash": "argon2$...",
  "created_at": 1678888888
}
```

### 3.2 群组存储 (Groups)
存储群组成员列表。

*   **Key 格式**: `group:{group_id}`
*   **Value**: 包含成员 ID 列表的 JSON。

```json
// Value 示例
{
  "owner": "user_id_A",
  "members": ["user_id_A", "user_id_B", "user_id_C"]
}
```

### 3.3 离线消息队列 (Offline Inbox)
这是最关键的设计。利用 KV 存储的 **扫描 (Scan / Iterator)** 功能。

*   **Key 格式**: `inbox:{target_user_id}:{timestamp_ns}:{msg_id}`
    *   *说明*: 将 `target_user_id` 放在最前面，可以让同一个用户的离线消息在磁盘上物理相邻。
*   **Value**: 消息体的二进制数据 (Payload)。

**操作逻辑**:
1.  **存储**: 直接 `put(key, value)`。
2.  **读取**: 使用 `prefix_iterator("inbox:{user_id}:")` 获取该用户所有未读消息。
3.  **删除**: 确认接收后，`delete(key)`。

### 3.4 历史记录 (Timeline / History)
用于漫游消息。

*   **Key 格式**: `timeline:{chat_id}:{timestamp_ns}`
    *   *chat_id*: 可以是 `group_id` 或 `p2p_session_id`。
*   **Value**: 完整消息结构体。

## 4. Rust 代码实现 (核心逻辑)

### 4.1 引入依赖
```toml
[dependencies]
rocksdb = "0.21"  # 或者使用 redb = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
```

### 4.2 存储引擎封装 (Repository Pattern)

我们将 RocksDB 封装为一个 `Store` 结构体，对外提供业务接口。

```rust
use rocksdb::{DB, Options, IteratorMode};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// 定义存储路径
const DB_PATH: &str = "./rmc_storage";

#[derive(Clone)]
pub struct KvStore {
    db: Arc<DB>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserProfile {
    pub username: String,
    pub password_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub id: String,
    pub from: String,
    pub content: String,
    pub timestamp: i64,
}

impl KvStore {
    // 初始化数据库
    pub fn new() -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        // 优化写入性能
        opts.set_keep_log_file_num(10); 
        
        let db = DB::open(&opts, DB_PATH).expect("Failed to open RocksDB");
        Self { db: Arc::new(db) }
    }

    // --- 用户相关 ---
    
    pub fn save_user(&self, user_id: &str, profile: &UserProfile) -> anyhow::Result<()> {
        let key = format!("user:{}", user_id);
        let val = serde_json::to_vec(profile)?; // 序列化为字节
        self.db.put(key, val)?;
        Ok(())
    }

    pub fn get_user(&self, user_id: &str) -> anyhow::Result<Option<UserProfile>> {
        let key = format!("user:{}", user_id);
        match self.db.get(key)? {
            Some(val) => {
                let profile: UserProfile = serde_json::from_slice(&val)?;
                Ok(Some(profile))
            }
            None => Ok(None),
        }
    }

    // --- 离线消息相关 (核心) ---

    // 存入离线箱
    pub fn save_offline_msg(&self, target_id: &str, msg: &ChatMessage) -> anyhow::Result<()> {
        // Key 设计：inbox:{user_id}:{timestamp}
        let key = format!("inbox:{}:{}", target_id, msg.timestamp);
        let val = serde_json::to_vec(msg)?;
        self.db.put(key, val)?;
        Ok(())
    }

    // 拉取并清空离线消息
    pub fn pop_offline_msgs(&self, target_id: &str) -> anyhow::Result<Vec<ChatMessage>> {
        let prefix = format!("inbox:{}:", target_id);
        let mut messages = Vec::new();
        
        // 扫描所有以 prefix 开头的 Key
        let iter = self.db.prefix_iterator(prefix.as_bytes());
        
        for item in iter {
            let (key, val) = item?;
            // 再次校验前缀（RocksDB iterator 有时会滑过界）
            if !String::from_utf8_lossy(&key).starts_with(&prefix) {
                break;
            }
            
            let msg: ChatMessage = serde_json::from_slice(&val)?;
            messages.push(msg);
            
            // 读完即删（或者等待 ACK 后再删，视 QoS 需求而定）
            self.db.delete(key)?; 
        }
        
        Ok(messages)
    }
}
```

