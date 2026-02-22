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
[ Redis Cluster ]  [ Postgres/MySQL ]  [ InfluxDB/ClickHouse ]
(会话/订阅树/缓存)    (用户数据/群组)      (消息历史/日志)
```

### 核心组件说明

1.  **MQTT Broker (Rust 实现)**:
    *   负责连接维持、协议解析、发布订阅路由、QoS 消息保障。
    *   **不处理**复杂的业务逻辑（如好友关系、群管理），只负责消息透传。
2.  **业务服务 (API Server)**:
    *   处理用户注册、登录（颁发 Token）、加好友、建群等 HTTP/gRPC 请求。
3.  **存储层**:
    *   **Redis**: 存储 MQTT Session（会话状态）、在线状态、飞行窗口（In-flight messages）。
    *   **Database**: 存储用户资料、关系链。
    *   **Time-Series/Log DB**: 存储历史聊天记录（写多读少）。

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


