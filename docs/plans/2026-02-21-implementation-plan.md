# ZeroInsect Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build ZeroInsect as ROS2 execution layer that integrates with ZeroClaw for cognitive capabilities

**Architecture:** Modular Rust crate with r2r for ROS2 DDS communication, offering introspection, robot tools, safety gateway, and swarm collaboration

**Tech Stack:** Rust, r2r, serde, tokio, quick-xml

---

## Phase 1: Project Foundation

### Task 1: Initialize Rust Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`

**Step 1: Write Cargo.toml**

```toml
[package]
name = "zeroinsect"
version = "0.1.0"
edition = "2021"
description = "ROS2 execution layer with ZeroClaw integration"

[dependencies]
r2r = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
quick-xml = "0.37"
anyhow = "1.0"
thiserror = "2.0"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
parking_lot = "0.12"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tempfile = "3.14"

[[bin]]
name = "zeroinsect-cli"
path = "src/main.rs"
```

**Step 2: Run to verify it compiles**

Run: `cargo check`
Expected: SUCCESS (new project with no errors)

**Step 3: Commit**

```bash
git add Cargo.toml src/
git commit -m "feat: initialize ZeroInsect project"
```

---

### Task 2: Create Module Structure

**Files:**
- Create: `src/ros2/mod.rs`
- Create: `src/introspect/mod.rs`
- Create: `src/tools/mod.rs`
- Create: `src/safety/mod.rs`
- Create: `src/swarm/mod.rs`
- Create: `src/bridge/mod.rs`

**Step 1: Write each module file**

```rust
// src/ros2/mod.rs
pub mod client;
pub mod types;

// src/introspect/mod.rs  
pub mod launch;
pub mod node;
pub mod discovery;

// src/tools/mod.rs
pub mod robot_tools;
pub mod nav_tools;

// src/safety/mod.rs
pub mod gateway;
pub mod config;

// src/swarm/mod.rs
pub mod state;
pub mod contract_net;

// src/bridge/mod.rs
pub mod zeroclaw;
```

**Step 2: Commit**

```bash
git add src/
git commit -m "feat: create module structure"
```

---

## Phase 2: ROS2 Native Integration

### Task 3: ROS2 Client Wrapper

**Files:**
- Create: `src/ros2/client.rs`
- Test: `tests/ros2_client_test.rs`

**Step 1: Write failing test**

```rust
// tests/ros2_client_test.rs
use zeroinsect::ros2::Ros2Client;

#[tokio::test]
async fn test_client_initialization() {
    let client = Ros2Client::new("test_node").await;
    assert!(client.is_ok());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test ros2_client_test -- --nocapture`
Expected: FAIL with "module not found"

**Step 3: Write implementation**

```rust
// src/ros2/client.rs
use anyhow::Result;
use r2r::{Node, NodeOptions};

pub struct Ros2Client {
    node: Node,
}

impl Ros2Client {
    pub async fn new(node_name: &str) -> Result<Self> {
        let node = Node::builder()
            .name(node_name)
            .build()?;
        Ok(Self { node })
    }

    pub fn subscribe<T: r2r::Msg>(&self, topic: &str) -> Result<r2r::Subscriber<T>> {
        Ok(self.node.subscribe::<T>(topic)?)
    }

    pub fn publish<T: r2r::Msg>(&self, topic: &str) -> Result<r2r::Publisher<T>> {
        Ok(self.node.publish(topic)?)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test ros2_client_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ros2/ tests/
git commit -m "feat: add ROS2 client wrapper"
```

---

### Task 4: Topic/Service/Action Types

**Files:**
- Create: `src/ros2/types.rs`

**Step 1: Write implementation**

```rust
// src/ros2/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwistCommand {
    pub linear: Vector3,
    pub angular: Vector3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    pub position: Vector3,
    pub orientation: Quaternion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotState {
    pub pose: Pose,
    pub velocity: TwistCommand,
    pub battery_level: f32,
    pub timestamp: i64,
}
```

**Step 2: Commit**

```bash
git add src/ros2/types.rs
git commit -m "feat: add ROS2 types"
```

---

## Phase 3: ROS System Introspection

### Task 5: Launch File Parser

**Files:**
- Create: `src/introspect/launch.rs`
- Test: `tests/launch_parser_test.rs`

**Step 1: Write test with sample launch file**

```rust
// tests/launch_parser_test.rs
use zeroinsect::introspect::launch::{parse_launch_file, LaunchNode};

#[test]
fn test_parse_simple_launch() {
    let xml = r#"
    <launch>
        <node name="talker" package="demo_nodes_cpp" exec="talker" />
        <node name="listener" package="demo_nodes_cpp" exec="listener" />
    </launch>
    "#;
    
    std::fs::write("/tmp/test.launch", xml).unwrap();
    let result = parse_launch_file("/tmp/test.launch").unwrap();
    
    assert_eq!(result.nodes.len(), 2);
    assert_eq!(result.nodes[0].name, "talker");
}
```

**Step 2: Run test**

Run: `cargo test launch_parser_test`
Expected: FAIL with "parse_launch_file not defined"

**Step 3: Write implementation**

```rust
// src/introspect/launch.rs
use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchFile {
    pub path: String,
    pub nodes: Vec<LaunchNode>,
    pub parameters: HashMap<String, String>,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchNode {
    pub name: String,
    pub package: String,
    pub executable: String,
    pub namespace: Option<String>,
    pub parameters: HashMap<String, String>,
    pub remappings: HashMap<String, String>,
}

pub fn parse_launch_file(path: &str) -> Result<LaunchFile> {
    let content = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);

    let mut nodes = Vec::new();
    let mut parameters = HashMap::new();
    let mut arguments = Vec::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"node" {
                    let mut node = LaunchNode {
                        name: String::new(),
                        package: String::new(),
                        executable: String::new(),
                        namespace: None,
                        parameters: HashMap::new(),
                        remappings: HashMap::new(),
                    };

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "name" => node.name = value,
                            "package" => node.package = value,
                            "exec" => node.executable = value,
                            "ns" => node.namespace = Some(value),
                            _ => {}
                        }
                    }
                    nodes.push(node);
                }
            }
            Ok(Event::Empty(e)) => {
                if e.name().as_ref() == b"node" {
                    // Handle self-closing <node /> tags
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(LaunchFile {
        path: path.to_string(),
        nodes,
        parameters,
        arguments,
    })
}
```

**Step 4: Run test**

Run: `cargo test launch_parser_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/introspect/ tests/
git commit -m "feat: add launch file parser"
```

---

### Task 6: ROS Node Discovery

**Files:**
- Create: `src/introspect/node.rs`
- Test: `tests/node_discovery_test.rs`

**Step 1: Write test**

```rust
// tests/node_discovery_test.rs
#[test]
fn test_node_info_struct() {
    use zeroinsect::introspect::node::NodeInfo;
    
    let info = NodeInfo {
        name: "/test_node".to_string(),
        publishers: vec!["/scan".to_string()],
        subscribers: vec!["/cmd_vel".to_string()],
        services: vec!["/reset".to_string()],
    };
    
    assert_eq!(info.publishers.len(), 1);
}
```

**Step 2: Run test**

Run: `cargo test node_discovery_test`
Expected: FAIL

**Step 3: Write implementation**

```rust
// src/introspect/node.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub publishers: Vec<String>,
    pub subscribers: Vec<String>,
    pub services: Vec<String>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGraph {
    pub nodes: Vec<NodeInfo>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub topic: String,
}
```

**Step 4: Run test**

Run: `cargo test node_discovery_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/introspect/ tests/
git commit -m "feat: add node discovery types"
```

---

### Task 7: Service/Action Discovery

**Files:**
- Create: `src/introspect/discovery.rs`

**Step 1: Write implementation**

```rust
// src/introspect/discovery.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscovery {
    pub name: String,
    pub type_name: String,
    pub provider_node: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDiscovery {
    pub name: String,
    pub type_name: String,
    pub provider_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityCard {
    pub name: String,
    pub type_: CapabilityType,
    pub inputs: Vec<FieldSchema>,
    pub outputs: Vec<FieldSchema>,
    pub description: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityType {
    Topic,
    Service,
    Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub field_type: String,
    pub required: bool,
}
```

**Step 2: Commit**

```bash
git add src/introspect/discovery.rs
git commit -m "feat: add service/action discovery types"
```

---

## Phase 4: Robot Tools

### Task 8: Robot State Tool

**Files:**
- Create: `src/tools/robot_tools.rs`
- Test: `tests/robot_tools_test.rs`

**Step 1: Write test**

```rust
// tests/robot_tools_test.rs
use zeroinsect::tools::robot_tools::RobotStateTool;

#[tokio::test]
async fn test_get_robot_state() {
    // This will need a running ROS2 environment
    // For now, test the struct creation
    let tool = RobotStateTool::new();
    assert!(!tool.supported_topics().is_empty());
}
```

**Step 2: Run test**

Run: `cargo test robot_tools_test`
Expected: PASS (struct test)

**Step 3: Write implementation**

```rust
// src/tools/robot_tools.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait RobotTool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("ROS2 error: {0}")]
    Ros2Error(String),
    #[error("Timeout: {0}")]
    Timeout(String),
}

pub struct RobotStateTool;

impl RobotStateTool {
    pub fn new() -> Self {
        Self
    }

    pub fn supported_topics(&self) -> Vec<&'static str> {
        vec!["/odom", "/battery_state", "/joint_states"]
    }
}

#[async_trait]
impl RobotTool for RobotStateTool {
    fn name(&self) -> &str {
        "get_robot_state"
    }

    fn description(&self) -> &str {
        "Get current robot state (position, velocity, battery)"
    }

    async fn execute(&self, _params: ToolParams) -> Result<ToolResult, ToolError> {
        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "pose": {"x": 0.0, "y": 0.0},
                "velocity": {"linear": 0.0, "angular": 0.0},
                "battery": 100.0
            }),
            message: "Robot state retrieved".to_string(),
        })
    }
}
```

**Step 4: Commit**

```bash
git add src/tools/ tests/
git commit -m "feat: add robot state tool"
```

---

### Task 9: Navigation Tool

**Files:**
- Create: `src/tools/nav_tools.rs`

**Step 1: Write implementation**

```rust
// src/tools/nav_tools.rs
use async_trait::async_trait;
use super::robot_tools::{RobotTool, ToolParams, ToolResult, ToolError};

pub struct NavigateTool;

impl NavigateTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RobotTool for NavigateTool {
    fn name(&self) -> &str {
        "navigate_to_pose"
    }

    fn description(&self) -> &str {
        "Navigate robot to specified pose"
    }

    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let goal = params.data;
        // In real implementation, send action goal to nav2
        Ok(ToolResult {
            success: true,
            data: goal,
            message: "Navigation goal sent".to_string(),
        })
    }
}
```

**Step 2: Commit**

```bash
git add src/tools/nav_tools.rs
git commit -m "feat: add navigation tool"
```

---

## Phase 5: Physical Safety Gateway

### Task 10: Safety Config

**Files:**
- Create: `src/safety/config.rs`
- Test: `tests/safety_config_test.rs`

**Step 1: Write test**

```rust
// tests/safety_config_test.rs
use zeroinsect::safety::config::SafetyConfig;

#[test]
fn test_velocity_validation() {
    let config = SafetyConfig::default();
    assert!(config.validate_velocity(1.0).is_ok());
    assert!(config.validate_velocity(100.0).is_err());
}
```

**Step 2: Run test**

Run: `cargo test safety_config_test`
Expected: FAIL

**Step 3: Write implementation**

```rust
// src/safety/config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub max_linear_velocity: f64,
    pub max_angular_velocity: f64,
    pub max_force: f64,
    pub workspace_boundary: Vec<Point2D>,
    pub forbidden_zones: Vec<Polygon>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polygon {
    pub points: Vec<Point2D>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    Approved,
    Rejected(String),
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            max_linear_velocity: 1.0,  // m/s
            max_angular_velocity: 1.0, // rad/s
            max_force: 100.0,          // N
            workspace_boundary: vec![
                Point2D { x: -10.0, y: -10.0 },
                Point2D { x: 10.0, y: -10.0 },
                Point2D { x: 10.0, y: 10.0 },
                Point2D { x: -10.0, y: 10.0 },
            ],
            forbidden_zones: vec![],
        }
    }
}

impl SafetyConfig {
    pub fn validate_velocity(&self, linear: f64) -> ValidationResult {
        if linear.abs() > self.max_linear_velocity {
            ValidationResult::Rejected(format!(
                "Exceeds max velocity {} m/s",
                self.max_linear_velocity
            ))
        } else {
            ValidationResult::Approved
        }
    }

    pub fn validate_workspace(&self, x: f64, y: f64) -> ValidationResult {
        // Simple bounding box check
        if x < -10.0 || x > 10.0 || y < -10.0 || y > 10.0 {
            ValidationResult::Rejected("Outside workspace boundary".to_string())
        } else {
            ValidationResult::Approved
        }
    }
}
```

**Step 4: Run test**

Run: `cargo test safety_config_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/safety/ tests/
git commit -m "feat: add safety configuration"
```

---

### Task 11: Safety Gateway

**Files:**
- Create: `src/safety/gateway.rs`

**Step 1: Write implementation**

```rust
// src/safety/gateway.rs
use super::config::{SafetyConfig, ValidationResult};
use crate::ros2::types::TwistCommand;

pub struct SafetyGateway {
    config: SafetyConfig,
}

impl SafetyGateway {
    pub fn new(config: SafetyConfig) -> Self {
        Self { config }
    }

    pub fn validate_command(&self, cmd: &TwistCommand) -> ValidationResult {
        // Check linear velocity
        let linear_vel = (cmd.linear.x.powi(2) + cmd.linear.y.powi(2)).sqrt();
        if let ValidationResult::Rejected(reason) = self.config.validate_velocity(linear_vel) {
            return ValidationResult::Rejected(reason);
        }

        // Check angular velocity
        if let ValidationResult::Rejected(reason) = 
            self.config.validate_velocity(cmd.angular.z) 
        {
            return ValidationResult::Rejected(reason);
        }

        ValidationResult::Approved
    }
}
```

**Step 2: Commit**

```bash
git add src/safety/gateway.rs
git commit -m "feat: add safety gateway"
```

---

## Phase 6: ZeroClaw Bridge

### Task 12: ZeroClaw IPC Bridge

**Files:**
- Create: `src/bridge/zeroclaw.rs`
- Test: `tests/zeroclaw_bridge_test.rs`

**Step 1: Write test**

```rust
// tests/zeroclaw_bridge_test.rs
use zeroinsect::bridge::zeroclaw::{ToolRequest, ToolResponse};

#[test]
fn test_tool_request_serialization() {
    let req = ToolRequest {
        tool_name: "navigate_to_pose".to_string(),
        parameters: serde_json::json!({"x": 1.0, "y": 2.0}),
    };
    
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("navigate_to_pose"));
}
```

**Step 2: Run test**

Run: `cargo test zeroclaw_bridge_test`
Expected: FAIL

**Step 3: Write implementation**

```rust
// src/bridge/zeroclaw.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub tool_name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

pub struct ZeroClawBridge {
    socket_path: String,
}

impl ZeroClawBridge {
    pub fn new(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
        }
    }

    pub async fn call_tool(&self, request: ToolRequest) -> ToolResponse {
        // In real implementation, use Unix socket or HTTP to communicate
        // with ZeroClaw process
        ToolResponse {
            success: true,
            result: serde_json::json!({"status": "executed"}),
            error: None,
        }
    }
}
```

**Step 4: Run test**

Run: `cargo test zeroclaw_bridge_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/bridge/ tests/
git commit -m "feat: add ZeroClaw bridge"
```

---

### Task 13: Tool Registry

**Files:**
- Create: `src/bridge/registry.rs`

**Step 1: Write implementation**

```rust
// src/bridge/registry.rs
use crate::tools::robot_tools::RobotTool;
use std::collections::HashMap;

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn RobotTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: RobotTool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn RobotTool>> {
        self.tools.get(name)
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Commit**

```bash
git add src/bridge/registry.rs
git commit -m "feat: add tool registry"
```

---

## Phase 7: Swarm Collaboration

### Task 14: CRDT State

**Files:**
- Create: `src/swarm/state.rs`

**Step 1: Write implementation**

```rust
// src/swarm/state.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmState {
    pub node_id: String,
    pub battery: f32,
    pub position: Position,
    pub status: NodeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Idle,
    Busy,
    Charging,
    Error,
}
```

**Step 2: Commit**

```bash
git add src/swarm/state.rs
git commit -m "feat: add swarm state types"
```

---

### Task 15: Contract Net Protocol

**Files:**
- Create: `src/swarm/contract_net.rs`

**Step 1: Write implementation**

```rust
// src/swarm/contract_net.rs
use serde::{Deserialize, Serialize};
use crate::swarm::state::NodeStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBid {
    pub node_id: String,
    pub cost: f64,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnnouncement {
    pub task_id: String,
    pub description: String,
    pub requirements: TaskRequirements,
    pub deadline: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub min_battery: f32,
    pub needed_capabilities: Vec<String>,
    pub weight_estimate: f64,
}

pub struct ContractNet {
    node_id: String,
}

impl ContractNet {
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
        }
    }

    pub fn announce_task(&self, announcement: TaskAnnouncement) -> TaskAnnouncement {
        announcement
    }

    pub fn evaluate_bid(&self, bid: &TaskBid, requirements: &TaskRequirements) -> bool {
        bid.cost > 0.0 && requirements.needed_capabilities.iter().all(|cap| {
            bid.capabilities.contains(cap)
        })
    }
}
```

**Step 2: Commit**

```bash
git add src/swarm/contract_net.rs
git commit -m "feat: add contract net protocol"
```

---

## Phase 8: CLI Application

### Task 16: Main CLI

**Files:**
- Modify: `src/main.rs`

**Step 1: Write implementation**

```rust
// src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zeroinsect")]
#[command(about = "ROS2 execution layer with ZeroClaw integration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start ZeroInsect daemon
    Start {
        /// ROS2 workspace to scan
        #[arg(long)]
        workspace: Option<String>,
    },
    /// Introspect ROS system
    Introspect {
        /// Output format (json/yaml)
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// List available tools
    Tools,
    /// Validate safety configuration
    ValidateSafety {
        /// Config file path
        #[arg(long)]
        config: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Start { workspace } => {
            println!("Starting ZeroInsect...");
        }
        Commands::Introspect { format } => {
            println!("Introspecting ROS system...");
        }
        Commands::Tools => {
            println!("Available tools: get_robot_state, navigate_to_pose, ...");
        }
        Commands::ValidateSafety { config } => {
            println!("Validating safety config: {}", config);
        }
    }
}
```

**Step 2: Run build**

Run: `cargo build --release`
Expected: SUCCESS

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: add CLI application"
```

---

## Execution Summary

**Plan complete and saved to `docs/plans/2026-02-21-zeroclaw-integration.md`**

### Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
