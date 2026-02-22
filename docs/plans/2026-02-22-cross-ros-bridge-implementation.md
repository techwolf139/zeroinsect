# Cross-ROS Bridge Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Cross-ROS Communication Bridge connecting ROS1 Noetic and ROS2 Humble systems using MQTT + Local LLM (Ollama) for schema matching and capability negotiation.

**Architecture:** 
- Bridge Hub: Central MQTT broker that routes messages between ROS Adapters
- ROS Adapters: Lightweight agents running on each ROS machine, connecting DDS to MQTT
- LLM Engine: Local Ollama for intent parsing, schema matching, and capability negotiation

**Tech Stack:** Rust (rumqttc, r2r), Python (rclpy), Ollama (llama3.2), RocksDB

---

## Phase 1: Bridge Hub (MQTT Broker Enhancement)

### Task 1: Add MQTT Client Capability to Hub

**Files:**
- Modify: `src/broker/server.rs` - Add MQTT client subscription capability
- Create: `src/llm/engine.rs` - LLM integration module

**Step 1: Add MQTT client setup**

The Hub needs to also function as an MQTT client to subscribe to command topics. Add:

```rust
// In src/llm/mod.rs
pub mod engine;
pub mod prompts;

// In src/llm/engine.rs
use reqwest::Client;

pub struct LlmEngine {
    client: Client,
    ollama_url: String,
    model: String,
}

impl LlmEngine {
    pub fn new(ollama_url: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            ollama_url: ollama_url.to_string(),
            model: model.to_string(),
        }
    }

    pub async fn parse_intent(&self, command: &str) -> Result<Intent, Box<dyn std::error::Error>> {
        // Call Ollama API
        let response = self.client
            .post(format!("{}/api/generate", self.ollama_url))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": format!("{}\n\nCommand: {}", PROMPT_INTENT_PARSING, command),
                "stream": false
            }))
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;
        
        // Parse JSON response
        Ok(serde_json::from_str(&response.response)?)
    }
}
```

**Step 2: Add command topic handling**

Modify `src/broker/server.rs` to subscribe to `bridge/command/+` topics:

```rust
// Add after connection handling
fn handle_command_topic(&self, topic: &str, payload: &[u8]) {
    if topic.starts_with("bridge/command/") {
        let device_id = topic.strip_prefix("bridge/command/").unwrap();
        let command = String::from_utf8_lossy(payload);
        
        // Queue for LLM processing
        self.llm_engine.parse_intent(&command);
    }
}
```

**Step 3: Test compilation**

Run: `cargo check --lib`
Expected: Compiles without errors

---

### Task 2: Add Intent Parsing Module

**Files:**
- Create: `src/llm/prompts.rs` - LLM prompt templates
- Create: `src/llm/types.rs` - Intent/Action types

**Step 1: Create types**

```rust
// src/llm/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub intent: String,
    pub parameters: serde_json::Value,
    pub confidence: f32,
    pub needs_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub status: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

**Step 2: Create prompts**

```rust
// src/llm/prompts.rs
pub const PROMPT_INTENT_PARSING: &str = r#"You are a robot command parser. 
Given a natural language command, extract the intent and parameters.

Output JSON:
{
  "intent": "MoveArm | GetStatus | Navigate | Stop | Custom",
  "parameters": {},
  "confidence": 0.0-1.0,
  "needs_confirmation": true/false
}"#;
```

---

## Phase 2: ROS Adapters

### Task 3: Create ROS2 Adapter (Rust)

**Files:**
- Create: `src/adapters/ros2_adapter.rs` - ROS2 client using r2r

**Step 1: Create adapter structure**

```rust
// src/adapters/ros2_adapter.rs
use r2r::R2RResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityManifest {
    pub ros_version: String,
    pub device_id: String,
    pub topics: Vec<TopicInfo>,
    pub services: Vec<ServiceInfo>,
    pub actions: Vec<ActionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub name: String,
    pub msg_type: String,
}

pub struct Ros2Adapter {
    node: r2r::Node,
    device_id: String,
    mqtt_client: AsyncClient,
}
```

**Step 2: Implement capability discovery**

```rust
impl Ros2Adapter {
    pub fn new(device_id: &str, mqttBroker: &str) -> R2RResult<Self> {
        let node = r2r::NodeBuilder::new()
            .name(&format!("ros2_adapter_{}", device_id))
            .build()?;
        
        Ok(Self { node, device_id: device_id.to_string(), mqtt_client: /* create MQTT client */ })
    }

    pub async fn discover_capabilities(&self) -> CapabilityManifest {
        let topics = self.node
            .topics()
            .into_iter()
            .map(|(name, msg_type)| TopicInfo { name, msg_type })
            .collect();
        
        CapabilityManifest {
            ros_version: "humble".to_string(),
            device_id: self.device_id.clone(),
            topics,
            services: vec![],
            actions: vec![],
        }
    }

    pub async fn publish_capabilities(&self) {
        let manifest = self.discover_capabilities().await;
        let topic = format!("bridge/capabilities/{}", self.device_id);
        // Publish to MQTT
    }
}
```

---

### Task 4: Create ROS1 Adapter (Python)

**Files:**
- Create: `adapters/ros1_adapter.py` - ROS1 client using rclpy

**Step 1: Create Python adapter**

```python
#!/usr/bin/env python3
"""ROS1 Adapter - Connects ROS1 Noetic to MQTT Bridge"""

import rclpy
from rclpy.node import Node
import json
import paho.mqtt.client as mqtt

class ROS1Adapter(Node):
    def __init__(self, device_id: str, mqtt_broker: str):
        super().__init__(f'ros1_adapter_{device_id}')
        self.device_id = device_id
        self.mqtt_client = mqtt.Client()
        self.mqtt_client.connect(mqtt_broker, 1883)
        
    def discover_capabilities(self):
        """Discover ROS1 topics, services, actions"""
        topics = self.get_topic_names_and_types()
        # Convert to capability manifest
        return {
            "ros_version": "noetic",
            "device_id": self.device_id,
            "topics": [{"name": t[0], "msg_type": t[1]} for t in topics],
            "services": [],  # TODO: discover services
            "actions": []    # TODO: discover actions
        }
    
    def publish_capabilities(self):
        manifest = self.discover_capabilities()
        topic = f"bridge/capabilities/{self.device_id}"
        self.mqtt_client.publish(topic, json.dumps(manifest))

def main(args=None):
    rclpy.init(args=args)
    adapter = ROS1Adapter("robot_arm", "localhost")
    adapter.publish_capabilities()
    rclpy.spin(adapter)

if __name__ == '__main__':
    main()
```

---

## Phase 3: LLM Integration

### Task 5: Integrate Ollama for Schema Matching

**Files:**
- Modify: `src/llm/engine.rs` - Add schema matching function

**Step 1: Add schema matching**

```rust
pub async fn match_schemas(
    &self, 
    ros1_msg: &str, 
    ros2_msg: &str
) -> Result<SchemaMatch, Box<dyn std::error::Error>> {
    let prompt = format!(r#"Compare these ROS message definitions:

ROS1 Message:
```
{}
```

ROS2 Message:
```
{}
```

Are they compatible? Provide JSON with field mappings.""", 
        ros1_msg, ros2_msg);
    
    // Call Ollama and parse response
}
```

---

## Phase 4: MQTT Command Reception

### Task 6: Add Command Processing Pipeline

**Files:**
- Modify: `src/broker/server.rs` - Add command topic subscription
- Create: `src/command/mod.rs` - Command processing

**Step 1: Add command handler**

```rust
// src/command/mod.rs
pub struct CommandProcessor {
    llm_engine: LlmEngine,
    adapter_registry: AdapterRegistry,
}

impl CommandProcessor {
    pub async fn process(&self, device_id: &str, command: &str) -> CommandResult {
        // 1. Parse intent with LLM
        let intent = self.llm_engine.parse_intent(command).await?;
        
        // 2. Get device capabilities
        let capabilities = self.adapter_registry.get(device_id)?;
        
        // 3. Plan action
        let action = self.plan_action(&intent, &capabilities).await?;
        
        // 4. Execute via adapter
        self.adapter_registry.execute(device_id, &action).await
    }
}
```

---

## Implementation Order

1. **Task 1**: Add MQTT client + LLM engine to Hub
2. **Task 2**: Create intent/types/prompts modules
3. **Task 3**: Create ROS2 Adapter (Rust)
4. **Task 4**: Create ROS1 Adapter (Python)
5. **Task 5**: Schema matching with Ollama
6. **Task 6**: Command processing pipeline

---

## Testing Strategy

Each task:
1. Write unit test for the new functionality
2. Run `cargo test` or `pytest` 
3. Verify compilation
4. Commit with conventional message

---

**Plan complete saved to: `docs/plans/2026-02-22-cross-ros-bridge-implementation.md`**

Two execution options:

1. **Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

2. **Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
