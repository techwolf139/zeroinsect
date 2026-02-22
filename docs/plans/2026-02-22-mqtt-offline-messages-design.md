# MQTT 离线消息支持设计方案

**创建日期**: 2026-02-22  
**状态**: 已确认  
**版本**: 1.0

---

## 一、需求概述

实现 MQTT 服务端和客户端的离线消息支持：

- 服务端：在接收方离线时缓存消息，重连后自动投递
- 客户端：本地缓存待发送消息，网络恢复后自动发送
- QoS 1（至少一次）
- 消息保留 7 天后自动清理

---

## 二、整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                      服务端 (Broker)                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐   │
│  │ 连接管理     │    │ 离线消息存储  │    │  定时清理    │   │
│  │             │    │ (RocksDB)   │    │  (7天)      │   │
│  └──────┬──────┘    └──────┬──────┘    └─────────────┘   │
│         │                   │                               │
│         ▼                   ▼                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │            消息分发器 (Dispatcher)                   │    │
│  │  • 检测订阅者在线状态                                │    │
│  │  • 离线则存入 RocksDB                               │    │
│  │  • 重连后自动投递                                   │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                      客户端 (Client)                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐   │
│  │ MQTT 连接   │    │ 本地消息队列 │    │  网络状态   │   │
│  │ (rumqttc)  │    │ (临时文件)   │    │  检测       │   │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘   │
│         │                   │                   │           │
│         ▼                   ▼                   ▼           │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              消息管理器 (MessageManager)             │    │
│  │  • 发送时存入本地队列                               │    │
│  │  • 网络恢复后自动发送队列中消息                      │    │
│  │  • 处理 QoS 确认                                   │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

---

## 三、服务端设计

### 3.1 核心逻辑

1. **连接时**：根据 `CleanSession` 标志决定是否持久化会话
2. **消息分发时**：如果接收方离线（无活跃连接），存入 RocksDB
3. **客户端重连时**：完成订阅后，自动从 RocksDB 拉取离线消息并投递
4. **定时清理**：每天凌晨执行清理，删除超过 7 天的离线消息

### 3.2 已有实现 (kv_store.rs)

```rust
// 已实现
pub fn save_offline_msg(&self, target_user_id: &str, msg: &ChatMessage) -> Result<()>
pub fn pop_offline_msgs(&self, target_user_id: &str) -> Result<Vec<ChatMessage>>
pub fn count_offline_msgs(&self, target_user_id: &str) -> Result<usize>
```

### 3.3 需要新增

```rust
// kv_store.rs - 新增方法
pub fn cleanup_old_offline_messages(&self, days: i64) -> Result<usize>
```

### 3.4 服务端触发逻辑

**位置**: `src/broker/server.rs`

在客户端完成订阅（收到 SUBACK）后：

```rust
// 伪代码
async fn on_client_subscribed(&self, user_id: &str) {
    // 检查是否有离线消息
    let messages = self.store.pop_offline_msgs(user_id)?;
    
    for msg in messages {
        // 投递消息
        self.deliver_message(&msg, user_id).await?;
    }
}
```

---

## 四、客户端设计

### 4.1 本地存储 (临时文件)

使用 JSON 文件存储待发送消息：

```json
// ~/.mqtt_chat/pending/{uuid}.json
{
  "id": "uuid-string",
  "topic": "chat/u/Bob",
  "payload": "{\"from\":\"Alice\",\"content\":\"Hello\"}",
  "qos": 1,
  "created_at": 1708588800,
  "retry_count": 0
}
```

**存储结构**：
- 目录: `~/.mqtt_chat/pending/`
- 每个消息一个 JSON 文件
- 文件名使用 UUID

### 4.2 核心模块

```rust
// examples/mqtt_chat.rs - 新增模块

use std::path::PathBuf;
use std::fs;

struct MessageManager {
    pending_dir: PathBuf,   // ~/.mqtt_chat/pending/
    client: AsyncClient,   // MQTT 客户端
}

impl MessageManager {
    fn new(client: AsyncClient) -> Self {
        let pending_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".mqtt_chat")
            .join("pending");
        
        // 确保目录存在
        fs::create_dir_all(&pending_dir).ok();
        
        Self { pending_dir, client }
    }
    
    // 发送消息时调用
    async fn send_message(&self, topic: &str, payload: &str) -> Result<()> {
        // 1. 先存入本地文件
        let msg = PendingMessage::new(topic, payload);
        self.save_to_pending(&msg).await?;
        
        // 2. 尝试发送到服务器
        match self.client.publish(topic, QoS::AtLeastOnce, false, payload).await {
            Ok(_) => {
                // 3. 发送成功，删除本地文件
                self.remove_from_pending(&msg.id).await?;
                Ok(())
            }
            Err(e) => {
                // 4. 发送失败，保留在本地
                Err(e)
            }
        }
    }
    
    // 网络恢复时调用
    async fn flush_pending_messages(&self) -> Result<()> {
        let messages = self.get_all_pending().await?;
        
        for msg in messages {
            match self.client.publish(&msg.topic, QoS::AtLeastOnce, false, &msg.payload).await {
                Ok(_) => {
                    self.remove_from_pending(&msg.id).await?;
                }
                Err(_) => {
                    // 更新重试计数
                    self.increment_retry(&msg.id).await?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct PendingMessage {
    id: String,
    topic: String,
    payload: String,
    qos: u8,
    created_at: i64,
    retry_count: u8,
}

### 4.3 网络状态检测

使用 MQTT 的 `ConnectionError` 事件监听网络状态变化：

```rust
// 监听连接断开
tokio::select! {
    event = eventloop.poll() => {
        match event {
            Err(ConnectionError::Io(_)) => {
                // 网络断开，启动本地缓存模式
                self.set_network_offline().await;
            }
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                // 网络恢复
                self.set_network_online().await;
                self.flush_pending_messages().await?;
            }
            _ => {}
        }
    }
}
```

---

## 五、消息流程

### 场景 A：客户端发送消息，接收方离线

```
1. 客户端 A 发送消息到 topic chat/u/Bob
2. 服务端 Dispatcher 检查订阅者
3. 发现 Bob 不在线（无连接）
4. 将消息存入 RocksDB (inbox:bob:...)
5. 返回 QoS 1 ACK 给客户端 A
```

### 场景 B：客户端 B 上线并接收离线消息

```
1. 客户端 B 连接 (CleanSession=false)
2. 服务端创建会话，标记为持久会话
3. 客户端 B 订阅主题
4. 服务端等待 SUBACK 完成
5. 服务端从 RocksDB 拉取 Bob 的离线消息
6. 逐个投递消息给客户端 B
7. 投递成功后从 RocksDB 删除
```

### 场景 C：客户端本地缓存发送

```
1. 用户输入消息，点击发送
2. 消息写入 ~/.mqtt_chat/pending/{uuid}.json
3. 尝试发送到服务器
4. 如果网络不通，返回失败
5. 前端显示"消息已缓存"
6. 网络恢复后，后台自动发送队列中的消息
7. 收到 ACK 删除本地文件
```

---

## 六、错误处理

| 场景 | 处理方式 |
|------|----------|
| 服务端存储失败 | 返回错误给发送方 |
| 客户端重连后拉取失败 | 重试 3 次，失败后放弃 |
| 客户端本地发送失败 | 保留在队列，下次重连继续 |
| 消息过期 (7天) | 定时任务自动清理 |

---

## 七、实现任务清单

### 服务端

- [ ] 1. 实现 `KvStore::cleanup_old_offline_messages()` 方法
- [ ] 2. 修改 `server.rs` 在 SUBACK 后触发离线消息投递
- [ ] 3. 添加定时清理任务

### 客户端 (examples/mqtt_chat.rs)

- [ ] 4. 添加 `dirs` 依赖 (用于获取用户目录)
- [ ] 5. 实现本地消息文件管理
- [ ] 6. 实现网络状态检测
- [ ] 7. 实现重连后自动发送队列消息

---

## 八、技术选型

| 组件 | 选型 | 理由 |
|------|------|------|
| 服务端存储 | RocksDB (已有) | 高性能嵌入式 KV 存储 |
| 客户端存储 | 临时文件 (JSON) | 简单、无依赖、易实现 |
| QoS 级别 | 1 (AtLeastOnce) | 平衡可靠性与性能 |
| 消息保留 | 7 天 | 合理的时间窗口 |

---

**文档状态**: 已确认，等待实现
