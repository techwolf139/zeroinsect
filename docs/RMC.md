# RMC (Rust-MQTT-Chat) Documentation

## Overview

RMC is a high-performance, lightweight MQTT-based chat system built in Rust. It implements a full MQTT broker with authentication, session management, and message routing capabilities.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     RMC Architecture                         │
├─────────────────────────────────────────────────────────────┤
│  Transport Layer (TCP)                                      │
│  └── src/network/tcp.rs                                     │
├─────────────────────────────────────────────────────────────┤
│  Broker Core                                                │
│  ├── src/broker/server.rs    - Main broker server          │
│  ├── src/broker/packet.rs   - MQTT packet parser           │
│  ├── src/broker/dispatcher.rs - Message dispatcher         │
│  └── src/broker/auth.rs     - Authentication (JWT)        │
├─────────────────────────────────────────────────────────────┤
│  Session Management                                         │
│  └── src/session/                                          │
│      ├── manager.rs          - Session manager              │
│      └── state.rs            - Session state                │
├─────────────────────────────────────────────────────────────┤
│  Topic Routing                                             │
│  └── src/router/                                           │
│      ├── radix_tree.rs       - Topic tree (wildcard support)│
│      └── matcher.rs          - Topic matcher               │
├─────────────────────────────────────────────────────────────┤
│  Storage (RocksDB)                                         │
│  └── src/storage/                                          │
│      ├── kv_store.rs         - Key-value store             │
│      └── schema.rs           - Data schemas                │
└─────────────────────────────────────────────────────────────┘
```

## Key Features

### 1. MQTT Protocol Support
- MQTT v3.1.1 / v5.0 compatible
- Support for QoS 0, 1, 2 (QoS 1 implemented)
- Clean session handling

### 2. Authentication
- Username/password authentication
- JWT token support
- Argon2 password hashing

### 3. Session Management
- In-memory session storage using DashMap
- Connection state tracking
- Clean session support

### 4. Topic Routing
- Radix tree-based topic matching
- Wildcard support (`+` single-level, `#` multi-level)
- Real-time subscription updates

### 5. Message Delivery
- Publish/Subscribe pattern
- Offline message storage
- Message history

## MQTT Topic Design

| Feature | Topic Pattern | Description |
|---------|--------------|-------------|
| Private Chat | `chat/u/{target_id}` | One-on-one messaging |
| Group Chat | `chat/g/{group_id}` | Group messaging |
| System Notification | `sys/u/{user_id}` | System notifications |
| User Status | `status/u/{user_id}` | Online/offline status |

## Usage

### Starting the Broker

```rust
use zeroinsect::broker::server::BrokerServer;
use zeroinsect::storage::kv_store::KvStore;

#[tokio::main]
async fn main() -> Result<()> {
    let store = KvStore::new()?;
    let server = BrokerServer::new(store);
    server.start("127.0.0.1:1883").await?;
    Ok(())
}
```

### Authentication

```rust
use zeroinsect::broker::auth::Authenticator;

// Register user
let user_id = auth.register_user("username", "password").await?;

// Authenticate
let result = auth.authenticate("username", "password").await;

// Generate JWT token
let token = auth.generate_token(&user_id).await?;
```

### Publishing Messages

```rust
use zeroinsect::broker::packet::PacketParser;
use mqttrs::QoS;

let packet = PacketParser::encode_publish(
    "chat/u/user123",
    b"Hello world",
    QoS::AtLeastOnce,
    Some(1),
);
```

### Subscribing to Topics

Clients can subscribe to topics using MQTT SUBSCRIBE packet with wildcard support:
- `chat/u/+` - All private chats
- `chat/g/#` - All group chats

## Testing

Run all tests:
```bash
cargo test
```

Run RMC-specific tests:
```bash
cargo test --test integration_test
```

## Dependencies

- **mqttrs** - MQTT protocol implementation
- **rocksdb** - Embedded key-value storage
- **tokio** - Async runtime
- **argon2** - Password hashing
- **jsonwebtoken** - JWT support
- **dashmap** - Concurrent HashMap

## Performance Characteristics

- Zero-copy packet parsing where possible
- Async I/O with Tokio
- In-memory session management
- RocksDB for persistent storage
- Thread-safe with DashMap and RwLock
