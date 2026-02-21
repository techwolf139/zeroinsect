# ZeroInsect + ZeroClaw 融合系统设计方案

> **目标**：以 ZeroClaw 为认知引擎，ZeroInsect 为 ROS2 执行层，构建机器人操作与认知一体化框架

## 一、技术融合策略

### 1.1 ZeroClaw 已有功能（复用）

| 功能模块 | 状态 | 说明 |
|----------|------|------|
| **Agent Engine** | ✅ 完整 | LLM 对话、智能推理、Tool Calling |
| **Tool System** | ✅ 30+ 工具 | shell, http, file, git, cron, browser 等 |
| **Skill System** | ✅ 完整 | SKILL.md/SKILL.toml 技能定义与加载 |
| **SkillForge** | ✅ 完整 | 从 GitHub 自动发现、评估、集成技能 |
| **Memory System** | ✅ 完整 | SQLite/Vector 记忆存储与检索 |
| **Integrations** | ✅ 80+ 服务 | 第三方 API 集成 |
| **Channels** | ✅ 20+ 通道 | Telegram, Discord, WhatsApp 等 |
| **Security** | ✅ 沙箱 | Landlock, Bubblewrap, Firejail |

### 1.2 ZeroInsect 专属功能（自研）

| 功能模块 | 说明 |
|----------|------|
| **ROS2 原生集成** | 基于 r2r/ros2-client 的 DDS 零拷贝通信 |
| **机器人工具** | 运动控制、传感器读取、导航、抓取等 |
| **物理安全网关** | 速度/力度限制、电子围栏、碰撞检测 |
| **具身 CoT** | 物理仿真验证、感知-动作闭环 |
| **蜂群协作** | CRDT 状态同步、合同网协议任务分配 |
| **边缘感知处理** | 激光雷达、视觉、IMU 数据实时处理 |

### 1.3 系统架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        ZeroClaw (认知层)                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │
│  │  Agent   │  │  Skills  │  │  Memory  │  │  Integrations │  │
│  │  Engine  │  │ SkillForge│  │ (Vector) │  │   (80+ API)  │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘  │
│                                                                  │
│  ZeroClaw Tools: shell, http, git, cron, browser, file, etc.   │
└──────────────────────────────┬──────────────────────────────────┘
                               │ IPC / gRPC
┌──────────────────────────────▼──────────────────────────────────┐
│                    ZeroInsect (执行层)                           │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              ROS2 Native Bridge (r2r)                    │   │
│  │     零拷贝 DDS 通信 │ Topic/Service/Action 代理          │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │  机器人工具  │  │ 物理安全网关  │  │    具身思维链引擎       │ │
│  │ move_base   │  │ 速度限制      │  │ 物理仿真 → 动作验证     │ │
│  │ grasp       │  │ 电子围栏      │  │ 感知融合 → 决策生成     │ │
│  │ sensor_read │  │ 碰撞检测      │  │                        │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              蜂群协作层 (CRDT + Contract Net)            │   │
│  │    分布式状态 │ 任务竞拍 │ 异构节点协商                    │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## 二、ZeroClaw 调用方案

### 2.1 进程间通信架构

```toml
# 配置文件: ~/.zeroclaw/tools/zeroclaw_ros2.toml

[[tool]]
name = "ros2_publish"
description = "Publish message to ROS2 topic"
command = "zeroclaw-ros2-bridge"
args = { action = "publish", topic = "{topic}", data = "{data}" }

[[tool]]
name = "ros2_call_service"
description = "Call ROS2 service"
command = "zeroclaw-ros2-bridge"
args = { action = "service", service = "{service}", request = "{request}" }

[[tool]]
name = "ros2_execute_action"
description = "Execute ROS2 action goal"
command = "zeroclaw-ros2-bridge"
args = { action = "action", action_name = "{name}", goal = "{goal}" }

[[tool]]
name = "get_robot_state"
description = "Get current robot state (position, velocity, battery)"
command = "zeroclaw-ros2-bridge"
args = { action = "state" }

[[tool]]
name = "get_scene_description"
description = "Get semantic description of current scene from sensors"
command = "zeroclaw-ros2-bridge"
args = { action = "scene" }

[[tool]]
name = "validate_motion_command"
description = "Validate motion command against safety constraints"
command = "zeroclaw-ros2-bridge"
args = { action = "validate", command = "{cmd}" }
```

### 2.2 ZeroClaw 启动配置

```toml
# ~/.zeroclaw/config.toml

[agent]
model = "claude-3-5-sonnet"
temperature = 0.7

[tools]
# 使用本地 zeroclaw-ros2-bridge 二进制
external_tools_dir = "/opt/zeroclaw-ros2/bin"

[skills]
# 机器人专用技能目录
robot_skills_dir = "/opt/zeroclaw-ros2/skills"

[memory]
# 机器人长期记忆 (可选向量数据库)
type = "sqlite"
path = "/var/lib/zeroclaw/robot_memory.db"
```

### 2.3 技能定义示例

```yaml
# /opt/zeroclaw-ros2/skills/navigation/SKILL.toml

[skill]
name = "navigation"
description = "Robot navigation and path planning skills"
version = "1.0.0"
tags = ["robotics", "ros2", "navigation"]

[[tool]]
name = "navigate_to"
description = "Navigate robot to specified location"
kind = "ros2_action"
action = "navigate_to_pose"
params = { frame_id = "map", tolerance = 0.3 }

[[tool]]
name = "avoid_obstacle"
description = "Plan path avoiding detected obstacles"
kind = "ros2_service"
service = "planner"
method = "make_plan"

[[tool]]
name = "localize"
description = "Get robot's current position estimate"
kind = "ros2_topic"
topic = "/amcl_pose"
```

## 三、核心功能模块

### 3.1 ROS2 原生集成

**实现方式**：直接使用 r2r crate 操作 DDS

```rust
use r2r::{Node, Topic, Service, ActionClient};

// 零拷贝图像传输
fn image_callback(image_data: &[u8]) -> Vec<u8> {
    // 直接共享内存指针，避免拷贝
    image_data.to_vec()
}

// 高频控制循环
async fn control_loop(node: &Node) {
    let cmd_vel = node.subscribe::<geometry_msgs::msg::Twist>("/cmd_vel").unwrap();
    loop {
        if let Ok(vel) = cmd_vel.recv() {
            // 毫秒级响应
            execute_motor_command(vel).await;
        }
    }
}
```

### 3.2 机器人专用工具

| 工具名称 | 功能 | 对应 ROS2 |
|----------|------|-----------|
| `get_robot_state` | 获取位置、速度、电量 | `/odom`, `/battery_state` |
| `get_scene_description` | 场景语义描述 | 融合激光雷达+视觉 |
| `navigate_to_pose` | 导航到目标点 | `/navigate_to_pose` action |
| `grasp_object` | 执行抓取动作 | `/grasp` action |
| `detect_objects` | 目标检测 | `/detected_objects` topic |
| `validate_motion` | 运动命令安全校验 | 本地安全网关 |

### 3.3 ROS 系统自省能力

**目标**：自动发现、分析和接入 ROS 系统中已实现的功能

#### 3.3.1 Launch 文件解析器

```rust
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchFile {
    pub path: String,
    pub nodes: Vec<LaunchNode>,
    pub parameters: Vec<Param>,
    pub arguments: Vec<Argument>,
    pub includes: Vec<Include>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchNode {
    pub name: String,
    pub package: String,
    pub executable: String,
    pub parameters: Vec<NodeParam>,
    pub remappings: Vec<Remap>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapability {
    pub name: String,
    pub topics: Vec<TopicInfo>,
    pub services: Vec<ServiceInfo>,
    pub actions: Vec<ActionInfo>,
    pub parameters: Vec<ParamInfo>,
}

// 解析 launch 文件
pub fn parse_launch_file(path: &str) -> Result<LaunchFile, Error> {
    let content = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&content);
    // XML 解析逻辑...
}

// 递归解析 include
pub fn parse_launch_recursive(path: &str) -> Result<LaunchFile, Error> {
    let mut launch = parse_launch_file(path)?;
    for include in &launch.includes {
        let included = parse_launch_recursive(&include.file)?;
        launch.nodes.extend(included.nodes);
    }
    Ok(launch)
}
```

#### 3.3.2 ROS 节点自省

```rust
use r2r::RclrsError;

// 通过 r2r 获取运行时节点信息
pub struct RosIntrospector {
    node: r2r::Node,
}

impl RosIntrospector {
    // 列出所有活动节点
    pub fn list_nodes(&self) -> Result<Vec<NodeInfo>, RclrsError> {
        // 使用 ros2 service call /ros2_dds/participant/lister_nodes
    }

    // 获取节点详细信息
    pub fn get_node_info(&self, node_name: &str) -> Result<NodeDetail, Error> {
        Ok(NodeDetail {
            publishers: self.get_node_publishers(node_name)?,
            subscribers: self.get_node_subscriptions(node_name)?,
            services: self.get_node_services(node_name)?,
            actions: self.get_node_actions(node_name)?,
        })
    }

    // 构建节点关系图
    pub fn build_node_graph(&self) -> NodeGraph {
        let nodes = self.list_nodes().unwrap_or_default();
        let topics = self.list_topics();
        // 构建 publisher -> topic -> subscriber 关系
    }
}
```

#### 3.3.3 服务与 Action 发现

```rust
// 服务类型发现
pub fn discover_services(&self) -> Vec<ServiceDiscovery> {
    vec![
        ServiceDiscovery {
            name: "/navigation/srv/Pause".into(),
            type_name: "nav2_msgs/srv/Pause".into(),
            provider_node: "nav2_controller".into(),
            description: "Pause navigation".into(),
        },
        // ...
    ]
}

// Action 发现
pub fn discover_actions(&self) -> Vec<ActionDiscovery> {
    vec![
        ActionDiscovery {
            name: "/navigate_to_pose".into(),
            type_name: "nav2_msgs/action/NavigateToPose".into(),
            provider_nodes: vec!["nav2_controller".into(), "planner".into()],
            goals: vec!["Navigate to pose goal"],
        },
    ]
}

// 服务能力卡片生成
pub fn generate_capability_card(&self, service: &ServiceDiscovery) -> CapabilityCard {
    CapabilityCard {
        name: service.name.clone(),
        type_: CapabilityType::Service,
        inputs: infer_service_inputs(&service.type_name),
        outputs: infer_service_outputs(&service.type_name),
        description: service.description.clone(),
        confidence: 0.9,
    }
}
```

#### 3.3.4 脚本分析与接入

```python
# Python 节点分析 (通过 AST)
import ast

def analyze_python_node(file_path: str) -> NodeAnalysis:
    with open(file_path) as f:
        tree = ast.parse(f.read())
    
    # 查找 ROS 节点创建
    # 查找 pub/sub/service 定义
    # 提取参数配置
    return NodeAnalysis(
        publishers=extract_publishers(tree),
        subscribers=extract_subscribers(tree),
        services=extract_services(tree),
        actions=extract_actions(tree),
        parameters=extract_parameters(tree),
    )

# C++ 节点分析
def analyze_cpp_node(file_path: str) -> NodeAnalysis:
    # 使用正则表达式提取
    # rclcpp::Publisher::SharedPtr
    # rclcpp::Subscription::SharedPtr
    # rclcpp::Service::SharedPtr
    pass
```

#### 3.3.5 自省工具列表

| 工具名称 | 功能 |
|----------|------|
| `parse_launch_file` | 解析单个 launch 文件 |
| `parse_launch_recursive` | 递归解析 launch 及 include |
| `list_ros_nodes` | 列出所有活动节点 |
| `get_node_detail` | 获取节点详情（pub/sub/svc） |
| `build_node_graph` | 构建节点关系拓扑图 |
| `discover_services` | 发现所有可用服务 |
| `discover_actions` | 发现所有可用 actions |
| `discover_topics` | 发现所有话题及类型 |
| `analyze_python_node` | 分析 Python 节点源码 |
| `analyze_cpp_node` | 分析 C++ 节点源码 |
| `generate_capability_card` | 生成服务能力卡片 |
| `infer_service_schema` | 推断服务请求/响应结构 |

#### 3.3.6 能力接入工作流

```
┌─────────────────────────────────────────────────────────────────┐
│                    ROS 自省与接入工作流                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. 发现阶段                                                    │
│     ┌──────────────┐                                           │
│     │ 扫描工作空间  │ → 发现 .launch, .py, .cpp 文件           │
│     └──────┬───────┘                                           │
│            ↓                                                    │
│  2. 解析阶段                                                    │
│     ┌──────────────┐                                           │
│     │ 解析 Launch  │ → 提取节点、参数、remap                   │
│     └──────┬───────┘                                           │
│            ↓                                                    │
│  3. 自省阶段                                                    │
│     ┌──────────────┐                                           │
│     │ 运行时发现   │ → 连接 ROS2 daemon 获取实际服务            │
│     └──────┬───────┘                                           │
│            ↓                                                    │
│  4. 接入阶段                                                    │
│     ┌──────────────┐                                           │
│     │ 生成能力卡片 │ → 注册到 ZeroClaw Tool System              │
│     └──────────────┘                                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### 3.3.7 ZeroClaw 集成示例

```toml
# 自动发现后生成的能力工具

[[tool]]
name = "ros2_discovered_navigate"
description = "Navigate robot to pose (discovered from nav2_bringup)"
command = "zeroclaw-ros2-bridge"
args = { action = "action", action_name = "/navigate_to_pose", goal = "{goal}" }

[[tool]]
name = "ros2_discovered_localize"
description = "Get robot localization (discovered from amcl)"
command = "zeroclaw-ros2-bridge"
args = { action = "topic", topic = "/amcl_pose" }

[[tool]]
name = "ros2_discovered_map"
description = "Get current map (discovered from map_server)"
command = "zeroclaw-ros2-bridge"
args = { action = "service", service = "/map_server/load_map" }
```

### 3.3 物理安全网关

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub max_linear_velocity: f64,   // m/s
    pub max_angular_velocity: f64, // rad/s
    pub max_force: f64,             // N
    pub workspace_boundary: Vec<Point2D>,
    pub forbidden_zones: Vec<Polygon>,
}

impl SafetyConfig {
    pub fn validate_command(&self, cmd: &MotionCommand) -> ValidationResult {
        if cmd.linear_velocity.abs() > self.max_linear_velocity {
            return ValidationResult::Rejected("Exceeds max velocity".into());
        }
        if !self.workspace_boundary.contains(&cmd.target) {
            return ValidationResult::Rejected("Outside workspace".into());
        }
        for zone in &self.forbidden_zones {
            if zone.contains(&cmd.target) {
                return ValidationResult::Rejected("In forbidden zone".into());
            }
        }
        ValidationResult::Approved
    }
}
```

### 3.4 具身思维链 (Embodied CoT)

**工作流程**：

```
ZeroClaw 推理 → ZeroInsect 物理验证 → ROS2 执行 → 反馈

1. 感知 (Sense):  ZeroInsect 收集传感器数据
2. 认知 (Think):  ZeroClaw Agent 分析任务
3. 验证 (Simulate): ZeroInsect 调用物理引擎验证路径
4. 执行 (Act):    通过 ROS2 下发动作指令
5. 反馈 (Feedback): 执行结果回传给 ZeroClaw
```

### 3.5 蜂群协作

**CRDT 状态同步**：

```rust
use crdt::{GCounter, LWWRegister, ORSet};

// 共享状态
struct SwarmState {
    // 每个节点的电池电量
    battery_levels: GCounter,
    // 节点当前位置
    positions: LWWRegister<String, Pose>,
    // 已发现目标
    discovered_targets: ORSet<String>,
}
```

**合同网协议任务分配**：

```
节点A (发现者): "发现重物(50kg)，坐标(X,Y)"
     ↓招标
节点B,C,D (评估者): 评估自身能力(电量、扭矩、距离)
     ↓投标 (报价)
节点A: 选择最优组合 (B+D)
     ↓签约
执行任务，实时同步状态
```

## 四、应用场景

### 4.1 灾后救援蜂群

| 阶段 | ZeroClaw | ZeroInsect |
|------|----------|------------|
| 探索 | 分析视觉/热成像数据，识别幸存者 | 控制多机器人编队 |
| 通信 | 生成救援请求，调度资源 | 建立 Mesh 网络 |
| 救援 | 规划挖掘路径，评估安全 | 执行抓取、搬运 |

### 4.2 柔性制造

| 阶段 | ZeroClaw | ZeroInsect |
|------|----------|------------|
| 换产 | 解析 ERP 订单，下载新技能 | 热更新 ROS2 Action |
| 执行 | 实时质量监控，动态调整 | 高精度力控 |
| 安全 | 生成安全策略 | Rust 规则校验 |

### 4.3 智慧城市运维

| 阶段 | ZeroClaw | ZeroInsect |
|------|----------|------------|
| 巡检 | 分析异常，预测故障 | 路径规划 |
| 协商 | 跨机器人任务谈判 | 状态广播 |
| 处置 | 生成方案，下发指令 | 协调执行 |

## 五、部署拓扑

### 5.1 单机模式

```
┌─────────────────────────────────────┐
│          ZeroClaw Agent             │
│    (任务规划 + LLM 推理)            │
└──────────────┬──────────────────────┘
               │ IPC
┌──────────────▼──────────────────────┐
│        ZeroInsect                   │
│  ┌────────────────────────────────┐ │
│  │   ROS2 Bridge (r2r)           │ │
│  │   机器人工具 | 安全网关         │ │
│  └────────────────────────────────┘ │
│               ↓                     │
│  ┌────────────────────────────────┐ │
│  │   物理世界 (执行器/传感器)     │ │
│  └────────────────────────────────┘ │
└─────────────────────────────────────┘
```

### 5.2 蜂群模式

```
                    ┌──────────────┐
                    │ ZeroClaw Hub │
                    │ (边缘服务器)  │
                    └──────┬───────┘
                           │ 5G/WiFi
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼────┐      ┌────▼────┐      ┌────▼────┐
    │ZeroInsect│      │ZeroInsect│      │ZeroInsect│
    │ 节点-A   │◄────►│ 节点-B   │◄────►│ 节点-C   │
    │ (机器狗) │ CRDT │ (机器狗) │ CRDT │ (无人机) │
    └─────────┘      └─────────┘      └─────────┘
```

## 六、总结

### 技术复用优势

| 方面 | 复用 ZeroClaw |
|------|---------------|
| **开发效率** | 无需重新实现 Agent、Skill、Memory 等成熟模块 |
| **资源占用** | 共用同一进程，总计 < 20MB |
| **扩展性** | 80+ 集成、SkillForge 自动发现即插即用 |
| **安全性** | 已有沙箱机制可直接复用 |

### ZeroInsect 专注领域

1. **ROS2 深度集成**：DDS 零拷贝、高频控制
2. **ROS 系统自省**：Launch/节点/服务自动发现与分析
3. **机器人工具**：运动控制、传感器处理
4. **物理安全**：速度/力度限制、电子围栏
5. **具身 CoT**：物理仿真验证
6. **蜂群协作**：CRDT + 合同网

---

*文档版本：v2.1*
*更新时间：2026-02-21*
*新增 ROS 系统自省能力章节*
