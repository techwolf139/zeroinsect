# ROS 功能集发现系统设计方案

> **目标**：通过运行时自省和静态分析结合的方式，自动发现并记录本地 ROS 系统的功能集，保存为本地 JSON/YAML 文件，供 ZeroClaw 调用

> **版本**：v1.2 - 科学迭代版本（含数据一致性优化）

---

## 一、设计原则

### 1.1 核心原则

| 原则 | 描述 |
|------|------|
| **数据一致性** | 快照原子获取，增量变更检测 |
| **资源高效** | 批量获取减少 DDS 调用，本地缓存避免重复 |
| **分层管理** | 内存缓存 → 磁盘持久化 → 文件输出 |

### 1.2 数据获取策略

```
┌─────────────────────────────────────────────────────────────┐
│                    数据获取策略                               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. 启动时 (全量快照)                                        │
│     ┌─────────────────────────────────────┐                │
│     │  capture_snapshot() 原子操作         │                │
│     │  - list_nodes() 批量获取            │                │
│     │  - list_topics() 批量获取          │                │
│     │  - list_services() 批量获取        │                │
│     │  - list_actions() 批量获取          │                │
│     │  (时间差 < 100ms)                  │                │
│     └─────────────────────────────────────┘                │
│                                                              │
│  2. 运行中 (增量/轮询)                                       │
│     ┌─────────────────────────────────────┐                │
│     │  定时轮询 (默认 30s)                │                │
│     │  或事件驱动订阅变化                   │                │
│     └─────────────────────────────────────┘                │
│                                                              │
│  3. 缓存策略                                                │
│     ┌─────────────────────────────────────┐                │
│     │  内存缓存 (热) - 频繁访问           │                │
│     │  磁盘缓存 (温) - 启动加载           │                │
│     │  TTL 过期 (冷) - 定时刷新           │                │
│     └─────────────────────────────────────┘                │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 一致性保障机制

| 机制 | 实现方式 |
|------|----------|
| **原子快照** | 一次性获取所有维度，时间差 < 100ms |
| **版本号** | 每次快照 +1，用于变化检测 |
| **变更检测** | 对比前后快照，只记录增量 |
| **校验和** | 生成 snapshot hash，用于一致性验证 |

---

## 二、迭代计划

### Iteration 1: 数据结构基础（基础层）
优先定义核心数据类型，确保上下游模块数据一致

### Iteration 2: 运行时自省（核心功能）
实现 ROS2 DDS 运行时发现能力

### Iteration 3: 静态分析增强（扩展能力）
扩展 Launch 解析，支持更多场景

### Iteration 4: 能力聚合（价值层）
生成可调用工具卡片

### Iteration 5: 输出与管理（交付层）
CLI 命令和文件输出

---

## 二、数据结构定义（Iteration 1）

### 2.1 核心类型

```rust
// src/introspect/types.rs

/// ROS 节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub namespace: String,
    pub full_name: String,
    pub executable: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// ROS Topic 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub name: String,
    pub type_name: String,
    pub publishers: Vec<String>,
    pub subscribers: Vec<String>,
}

/// ROS Service 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub type_name: String,
    pub provider_nodes: Vec<String>,
}

/// ROS Action 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    pub name: String,
    pub type_name: String,
    pub server_nodes: Vec<String>,
    pub client_nodes: Vec<String>,
}

/// ROS 系统快照（原子获取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub nodes: HashMap<String, NodeInfo>,
    pub topics: HashMap<String, TopicInfo>,
    pub services: HashMap<String, ServiceInfo>,
    pub actions: HashMap<String, ActionInfo>,
    pub checksum: String,
}

impl RosSnapshot {
    /// 原子性获取完整快照
    pub fn capture() -> Result<Self>;
    
    /// 计算校验和
    pub fn compute_checksum(&self) -> String;
    
    /// 检测变更（对比两个快照）
    pub fn diff(&self, other: &RosSnapshot) -> SnapshotDiff;
}

/// 快照变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub added_nodes: Vec<NodeInfo>,
    pub removed_nodes: Vec<String>,
    pub added_topics: Vec<TopicInfo>,
    pub removed_topics: Vec<String>,
    pub added_services: Vec<ServiceInfo>,
    pub removed_services: Vec<String>,
}

/// 缓存管理器
pub struct CacheManager {
    memory_cache: Option<RosSnapshot>,
    disk_cache_path: PathBuf,
    ttl_seconds: u64,
}

impl CacheManager {
    pub fn new(cache_dir: &str, ttl_seconds: u64) -> Self;
    
    /// 获取缓存（内存优先）
    pub fn get(&self) -> Option<&RosSnapshot>;
    
    /// 设置内存缓存
    pub fn set_memory(&mut self, snapshot: RosSnapshot);
    
    /// 加载磁盘缓存
    pub fn load_disk(&self) -> Option<RosSnapshot>;
    
    /// 保存到磁盘
    pub fn save_disk(&self, snapshot: &RosSnapshot) -> Result<()>;
    
    /// 检查是否过期
    pub fn is_expired(&self) -> bool;
}
```

---

## 三、迭代实施计划

### Iteration 1: 数据结构基础

| 任务 | 文件 | 描述 |
|------|------|------|
| T1.1 | `src/introspect/types.rs` | 定义核心类型（NodeInfo, TopicInfo, ServiceInfo, ActionInfo） |
| T1.2 | `src/introspect/types.rs` | 定义 RosSnapshot 快照结构 + 校验和计算 |
| T1.3 | `src/introspect/types.rs` | 定义 SnapshotDiff 变更检测结构 |
| T1.4 | `src/introspect/types.rs` | 定义 CacheManager 缓存管理结构 |
| T1.5 | `src/introspect/mod.rs` | 导出新类型 |

### Iteration 2: 运行时自省

| 任务 | 文件 | 描述 |
|------|------|------|
| T2.1 | `src/introspect/runtime.rs` | 创建 RosRuntimeIntrospector 结构 |
| T2.2 | `src/introspect/runtime.rs` | 实现 list_nodes() - 获取所有节点 |
| T2.3 | `src/introspect/runtime.rs` | 实现 list_topics() - 获取所有话题及类型 |
| T2.4 | `src/introspect/runtime.rs` | 实现 list_services() - 获取所有服务及类型 |
| T2.5 | `src/introspect/runtime.rs` | 实现 list_actions() - 获取所有动作及类型 |
| T2.6 | `src/introspect/runtime.rs` | 实现 capture_snapshot() - 原子化快照获取 |
| T2.7 | `src/introspect/runtime.rs` | 实现 diff() - 快照变更检测 |

### Iteration 3: 静态分析增强

| 任务 | 文件 | 描述 |
|------|------|------|
| T3.1 | `src/introspect/launch.rs` | 扩展支持 `<param>` 标签解析 |
| T3.2 | `src/introspect/launch.rs` | 扩展支持 `<remap>` 标签解析 |
| T3.3 | `src/introspect/launch.rs` | 实现 launch 文件递归扫描 |
| T3.4 | `src/introspect/launch.rs` | 实现工作空间自动发现 |
| T3.5 | `src/introspect/cache.rs` | 实现内存缓存管理（get/set/clear） |
| T3.6 | `src/introspect/cache.rs` | 实现磁盘缓存持久化（load/save） |
| T3.7 | `src/introspect/cache.rs` | 实现 TTL 过期检查 |

### Iteration 4: 能力聚合

| 任务 | 文件 | 描述 |
|------|------|------|
| T4.1 | `src/introspect/capability.rs` | 创建 CapabilityCard 结构 |
| T4.2 | `src/introspect/capability.rs` | 实现 Topic -> 能力卡片转换 |
| T4.3 | `src/introspect/capability.rs` | 实现 Service -> 能力卡片转换 |
| T4.4 | `src/introspect/capability.rs` | 实现 Action -> 能力卡片转换 |

### Iteration 5: 输出与管理

| 任务 | 文件 | 描述 |
|------|------|------|
| T5.1 | `src/introspect/output.rs` | 创建 OutputManager 结构 |
| T5.2 | `src/introspect/output.rs` | 实现 JSON 文件序列化 |
| T5.3 | `src/main.rs` | 添加 `discover` CLI 子命令 |
| T5.4 | `src/main.rs` | 添加缓存参数 `--cache-ttl`, `--no-cache` |
| T5.5 | `src/main.rs` | 添加 `--runtime`, `--static`, `--output-dir` 参数 |

---

## 四、数据流图

```
┌─────────────────────────────────────────────────────────────┐
│                    数据采集层                                 │
├──────────────────┬──────────────────┬────────────────────┤
│  运行时自省       │  Launch 文件解析   │  源码分析           │
│  (r2r/rclpy)    │  (quick-xml)      │  (AST)             │
└────────┬─────────┴────────┬─────────┴─────────┬────────────┘
         │                  │                   │
         ▼                  ▼                   ▼
┌─────────────────────────────────────────────────────────────┐
│                    能力聚合层                                 │
│  - 节点能力合并                                              │
│  - 接口类型推断                                               │
│  - 调用方式生成                                               │
└────────┬────────────────────────────────────────────────┬────┘
         │                                                 │
         ▼                                                 ▼
┌─────────────────────────────────────────────────────────────┐
│                    输出层                                     │
│  - ros_capabilities.json (可调用工具集)                      │
│  - ros_nodes.json (节点清单)                                │
│  - ros_topics.json (话题清单)                               │
│  - ros_services.json (服务清单)                              │
│  - ros_actions.json (动作清单)                               │
└─────────────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────────────┐
│                    数据采集层                                 │
├──────────────────┬──────────────────┬────────────────────┤
│  运行时自省       │  Launch 文件解析   │  源码分析           │
│  (r2r/rclpy)    │  (quick-xml)      │  (AST)             │
└────────┬─────────┴────────┬─────────┴─────────┬────────────┘
         │                  │                   │
         ▼                  ▼                   ▼
┌─────────────────────────────────────────────────────────────┐
│                    能力聚合层                                 │
│  - 节点能力合并                                              │
│  - 接口类型推断                                               │
│  - 调用方式生成                                               │
└────────┬────────────────────────────────────────────────┬────┘
         │                                                 │
         ▼                                                 ▼
┌─────────────────────────────────────────────────────────────┐
│                    输出层                                     │
│  - ros_capabilities.json (可调用工具集)                      │
│  - ros_nodes.json (节点清单)                                │
│  - ros_topics.json (话题清单)                               │
│  - ros_services.json (服务清单)                              │
│  - ros_actions.json (动作清单)                               │
└─────────────────────────────────────────────────────────────┘
```

## 二、模块设计

### 2.1 运行时自省模块

```rust
// src/introspect/runtime.rs

pub struct RosRuntimeIntrospector {
    node: Option<r2r::Node>,
}

impl RosRuntimeIntrospector {
    pub fn new() -> Result<Self, Error>;
    
    /// 获取所有活跃节点
    pub fn list_nodes(&self) -> Result<Vec<NodeInfo>>;
    
    /// 获取所有 Topic 及类型
    pub fn list_topics(&self) -> Result<Vec<TopicInfo>>;
    
    /// 获取所有 Service 及类型
    pub fn list_services(&self) -> Result<Vec<ServiceInfo>>;
    
    /// 获取所有 Action 及类型
    pub fn list_actions(&self) -> Result<Vec<ActionInfo>>;
    
    /// 获取节点详情（发布者、订阅者、服务）
    pub fn get_node_detail(&self, node_name: &str) -> Result<NodeDetail>;
}
```

### 2.2 Launch 解析增强

```rust
// src/introspect/launch.rs (扩展现有模块)

pub struct LaunchAnalyzer {
    workspace_path: PathBuf,
}

impl LaunchAnalyzer {
    pub fn new(workspace: &str) -> Self;
    
    /// 扫描工作空间所有 launch 文件
    pub fn scan_launch_files(&self) -> Result<Vec<LaunchFile>>;
    
    /// 递归解析 launch 及 include
    pub fn parse_recursive(&self, path: &str) -> Result<LaunchFile>;
    
    /// 提取节点声明的 parameters
    pub fn extract_parameters(&self, launch: &LaunchFile) -> Vec<ParameterDef>;
    
    /// 生成节点能力摘要
    pub fn generate_node_capability(&self, launch: &LaunchFile) -> Vec<Capability>;
}
```

### 2.3 能力卡片生成

```rust
// src/introspect/capability.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityCard {
    pub name: String,
    pub capability_type: CapabilityType,
    pub ros_type: String,
    pub description: String,
    pub parameters: Vec<ParameterSchema>,
    pub call_template: CallTemplate,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityType {
    TopicPublisher,
    TopicSubscriber,
    Service,
    Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallTemplate {
    pub tool_name: String,
    pub args_template: serde_json::Value,
}

impl CapabilityCard {
    /// 从 Topic 生成能力卡片
    pub fn from_topic(topic: &TopicInfo) -> Self;
    
    /// 从 Service 生成能力卡片
    pub fn from_service(service: &ServiceInfo) -> Self;
    
    /// 从 Action 生成能力卡片
    pub fn from_action(action: &ActionInfo) -> Self;
}
```

### 2.4 输出管理器

```rust
// src/introspect/output.rs

pub struct OutputManager {
    output_dir: PathBuf,
}

impl OutputManager {
    pub fn new(output_dir: &str) -> Self;
    
    pub fn save_nodes(&self, nodes: &[NodeInfo]) -> Result<()>;
    pub fn save_topics(&self, topics: &[TopicInfo]) -> Result<()>;
    pub fn save_services(&self, services: &[ServiceInfo]) -> Result<()>;
    pub fn save_actions(&self, actions: &[ActionInfo]) -> Result<()>;
    pub fn save_capabilities(&self, capabilities: &[CapabilityCard]) -> Result<()>;
    
    /// 生成汇总报告
    pub fn generate_summary(&self) -> Result<RosSystemSummary>;
}
```

## 三、数据结构

### 3.1 核心类型定义

```rust
// src/introspect/types.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub namespace: String,
    pub full_name: String,
    pub publishers: Vec<String>,
    pub subscribers: Vec<String>,
    pub services: Vec<String>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub name: String,
    pub type_name: String,
    pub publishers: Vec<String>,
    pub subscribers: Vec<String>,
    pub qos: QoSProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub type_name: String,
    pub provider_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    pub name: String,
    pub type_name: String,
    pub server_nodes: Vec<String>,
    pub client_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QoSProfile {
    pub reliability: String,
    pub durability: String,
    pub history: String,
    pub depth: u32,
}
```

### 3.2 输出 JSON 格式

```json
// ros_capabilities.json 示例
{
  "generated_at": "2026-02-21T10:30:00Z",
  "robot_name": "robot_01",
  "capabilities": [
    {
      "name": "publish_scan",
      "capability_type": "TopicPublisher",
      "ros_type": "sensor_msgs/msg/LaserScan",
      "description": "Publish laser scan data",
      "parameters": [],
      "call_template": {
        "tool_name": "ros2_publish",
        "args_template": {
          "topic": "/scan",
          "msg_type": "sensor_msgs/msg/LaserScan"
        }
      },
      "confidence": 1.0
    },
    {
      "name": "call_navigate",
      "capability_type": "Service",
      "ros_type": "nav2_msgs/srv/NavigateToPose",
      "description": "Navigate to pose service",
      "parameters": [
        {"name": "pose", "type": "geometry_msgs/msg/PoseStamped"},
        {"name": "bt_xml", "type": "string", "required": false}
      ],
      "call_template": {
        "tool_name": "ros2_call_service",
        "args_template": {
          "service": "/navigate_to_pose",
          "request": {}
        }
      },
      "confidence": 0.95
    }
  ]
}
```

## 四、CLI 命令

### 4.1 命令列表

| 命令 | 功能 |
|------|------|
| `discover` | 执行完整发现流程 |
| `discover --runtime` | 仅运行时自省 |
| `discover --static` | 仅静态分析 |
| `discover --output json` | 指定输出格式 |
| `discover --output-dir ./ros_data` | 指定输出目录 |

### 4.2 使用示例

```bash
# 完整发现
zeroinsect-cli discover --workspace /opt/ros2_ws

# 仅运行时
zeroinsect-cli discover --runtime

# 输出到指定目录
zeroinsect-cli discover --output-dir ~/.zeroinsect/ros_capabilities
```

## 五、ZeroClaw 集成

### 5.1 工具生成

发现的能力自动转换为 ZeroClaw 外部工具：

```toml
# ~/.zeroclaw/tools/ros2_discovered.toml

[[tool]]
name = "ros2_scan"
description = "Publish laser scan (discovered from rplidar node)"
command = "zeroclaw-ros2-bridge"
args = { action = "publish", topic = "/scan", msg_type = "sensor_msgs/msg/LaserScan" }

[[tool]]
name = "ros2_navigate"
description = "Navigate to pose (discovered from nav2_controller)"
command = "zeroclaw-ros2-bridge"
args = { action = "service", service = "/navigate_to_pose" }
```

### 5.2 调用方式

```python
# ZeroClaw Agent 调用示例
agent: "Navigate the robot to position (1.0, 2.0)"

# ZeroClaw 自动发现可用工具
# -> ros2_navigate 可用
# -> 调用 ros2_call_service("/navigate_to_pose", {pose: ...})
```

## 六、实施计划

### Phase 1: 运行时自省
- [ ] 实现 r2r 运行时客户端封装
- [ ] 实现节点列表获取
- [ ] 实现 Topic/Service/Action 发现

### Phase 2: 静态分析增强
- [ ] 扩展 Launch 解析器（支持更多标签）
- [ ] 实现 launch 文件递归扫描
- [ ] 实现参数和 remapping 提取

### Phase 3: 能力聚合
- [ ] 实现能力卡片生成器
- [ ] 实现类型推断（从 msg/srv/action 类型）
- [ ] 实现调用模板生成

### Phase 4: 输出与管理
- [ ] 实现 JSON/YAML 输出
- [ ] 实现 CLI 命令
- [ ] 实现 ZeroClaw 工具转换

---

*文档版本：v1.0*
*更新时间：2026-02-21*
