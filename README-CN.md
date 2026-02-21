# ZeroInsect - ROS 系统能力发现与规划工具

> 中文文档 | [English](./README.md)

ZeroInsect 是一个 ROS (Robot Operating System) 系统能力发现与动作规划工具。它通过运行时自省和静态分析，自动发现并构建机器人系统的能力地图，支持有目的的动作规划和因果关系推理。

---

## 核心特性

### 🗺️ 能力地图构建
- **自动发现**：从 ROS 运行时自动发现节点、话题、服务、动作
- **能力分类**：将系统能力分为感知(Sensing)、决策(Decision)、执行(Actuation)三类
- **因果关联**：建立动作之间的因果关系，支持有目的的动作规划

### 🔍 多源数据采集
- **运行时自省**：通过 DDS 发现系统实时状态
- **静态分析**：解析 Launch 文件、提取参数和映射关系
- **缓存机制**：多层缓存（内存+磁盘+TTL）优化性能

### 🎯 智能动作规划
- **目标解析**：支持自然语言目标描述
- **路径搜索**：BFS/A* 算法搜索动作序列
- **因果推理**：基于因果图的智能规划

### 💻 CLI 工具
- 交互式 CLI 命令
- 支持多种查询方式
- 美观的输出格式

---

## 适用场景

### 1. 机器人系统审计
```
# 了解机器人具备哪些能力
zeroinsect capability map
zeroinsect capability list
```
适用于：新机器人部署前的系统能力审计

### 2. 故障排查
```
# 查看话题的发布/订阅关系
zeroinsect capability graph --from /scan
```
适用于：定位数据流中断点、理解系统架构

### 3. 任务规划
```
# 让机器人移动到目标位置
zeroinsect capability plan --goal "移动到厨房"
```
适用于：高层任务自动分解为可执行动作序列

### 4. 工具集成
```rust
// 通过 API 集成到其他系统
use zeroinsect::capability_map::{CapabilityClassifier, ActionPlanner, Goal};

let snapshot = introspector.capture_snapshot()?;
let classifier = CapabilityClassifier::new();
let cap_map = classifier.classify(&snapshot);
let planner = ActionPlanner::new(&cap_map);
let plan = planner.plan(&Goal::from_string("move to target"));
```
适用于：构建机器人应用、ZeroClaw 工具发现

---

## 快速开始

### 安装

```bash
cargo build
```

### 基本使用

```bash
# 查看能力地图概览
cargo run -- capability map

# 按类别筛选
cargo run -- capability map --category sensing
cargo run -- capability map --category actuation

# 查看特定节点
cargo run -- capability map --node /scan

# 规划动作
cargo run -- capability plan --goal "移动到目标位置"

# 查看因果关系
cargo run -- capability graph --from /cmd_vel
```

---

## 架构设计

### 模块结构

```
src/
├── introspect/          # 能力发现模块
│   ├── runtime.rs      # 运行时自省
│   ├── launch.rs       # Launch 文件解析
│   ├── cache.rs        # 缓存管理
│   └── types.rs        # 数据结构
├── capability_map/     # 能力地图模块
│   ├── graph.rs        # 图结构与操作
│   ├── classifier.rs   # 能力分类器
│   └── planner.rs      # 动作规划器
├── tools/              # 工具定义
├── ros2/              # ROS2 接口
├── bridge/            # ZeroClaw 桥接
└── main.rs            # CLI 入口
```

### 数据流

```
┌─────────────────────────────────────────────────────────────┐
│                      数据采集层                              │
├──────────────────┬──────────────────┬──────────────────────┤
│   运行时自省      │  Launch 文件     │   缓存系统           │
│  (DDS/rcl)      │  (quick-xml)    │  (内存+磁盘)         │
└────────┬─────────┴────────┬─────────┴──────────┬─────────┘
         │                   │                     │
         ▼                   ▼                     ▼
┌─────────────────────────────────────────────────────────────┐
│                      能力聚合层                              │
│  - 能力分类 (Sensing/Decision/Actuation)                   │
│  - 因果边推断 (Produces/Enables/Triggers)                  │
│  - 图结构构建                                             │
└─────────────────────────────┬───────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                      API/CLI 层                             │
│  - CapabilityMap (Rust API)                                │
│  - capability map/plan/graph/list (CLI)                    │
└─────────────────────────────────────────────────────────────┘
```

---

## CLI 命令详解

### capability map

查看系统能力地图概览。

```bash
# 完整概览
zeroinsect capability map

# 按类别筛选
zeroinsect capability map --category sensing      # 感知能力
zeroinsect capability map --category decision   # 决策能力
zeroinsect capability map --category actuation  # 执行能力

# 查看特定节点详情
zeroinsect capability map --node /scan
```

### capability plan

根据目标规划动作序列。

```bash
# 自然语言目标
zeroinsect capability plan --goal "移动到厨房"
zeroinsect capability plan --goal "抓取物体"
zeroinsect capability plan --goal "导航到目标点"
```

### capability graph

查询因果关系图。

```bash
# 查看从某节点出发的边
zeroinsect capability graph --from /scan

# 查看到达某节点的边
zeroinsect capability graph --target /cmd_vel

# 查看所有边
zeroinsect capability graph
```

### capability list

列出所有 ROS 元素。

```bash
# 列出所有话题
zeroinsect capability list

# 按类别筛选
zeroinsect capability list --category decision
```

---

## Rust API 使用

### 基本使用流程

```rust
use zeroinsect::introspect::runtime::RosRuntimeIntrospector;
use zeroinsect::capability_map::{CapabilityClassifier, ActionPlanner, Goal};

// 1. 获取系统快照
let mut introspector = RosRuntimeIntrospector::new();
let snapshot = introspector.capture_snapshot()?;

// 2. 构建能力地图
let classifier = CapabilityClassifier::new();
let cap_map = classifier.classify(&snapshot);

// 3. 规划动作
let planner = ActionPlanner::new(&cap_map);
let goal = Goal::from_string("移动到目标位置");
let plan = planner.plan(&goal);

// 4. 执行动作序列
for step in plan.steps {
    println!("执行: {}", step.name);
}
```

### 数据结构

```rust
// 能力节点
pub struct CapabilityNode {
    pub id: String,
    pub name: String,
    pub category: CapabilityCategory,  // Sensing/Decision/Actuation
    pub ros_type: RosCapabilityType,  // Node/Topic/Service/Action
    pub node: String,
    pub description: String,
    pub preconditions: Vec<Condition>,
    pub effects: Vec<Effect>,
}

// 因果边
pub struct CausalEdge {
    pub from: String,
    pub to: String,
    pub relation: CausalRelation,  // Enables/Produces/Consumes/Conflicts/Triggers
    pub probability: f32,
}

// 能力地图
pub struct CapabilityMap {
    pub nodes: HashMap<String, CapabilityNode>,
    pub edges: Vec<CausalEdge>,
    pub topics: HashMap<String, TopicCapability>,
    pub services: HashMap<String, ServiceCapability>,
    pub actions: HashMap<String, ActionCapability>,
}
```

---

## 能力分类规则

### 话题分类

| 类别 | 关键词 | 示例 |
|------|--------|------|
| 感知 (Sensing) | sensor, camera, laser, imu, odom, tf, joint_states | `/scan`, `/camera/image`, `/odom` |
| 执行 (Actuation) | cmd, velocity, trajectory, gripper, move | `/cmd_vel`, `/gripper/command` |

### 服务分类

| 类别 | 关键词 | 示例 |
|------|--------|------|
| 感知 (Sensing) | get, query, check, localize | `/get_state`, `/localize` |
| 决策 (Decision) | plan, compute, decide, optimize | `/plan_path`, `/compute_route` |
| 执行 (Actuation) | set, control, move, execute | `/set_velocity`, `/execute_action` |

---

## 扩展开发

### 添加新的能力分类器

```rust
impl CapabilityClassifier {
    pub fn custom_classify(&self, name: &str) -> CapabilityCategory {
        // 实现自定义分类逻辑
        // ...
    }
}
```

### 添加新的规划算法

```rust
pub struct CustomPlanner {
    // 自定义规划器
}

impl ActionPlannerTrait for CustomPlanner {
    fn plan(&self, goal: &Goal) -> ActionPlan {
        // 实现自定义规划算法
        // ...
    }
}
```

---

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test capability_map
cargo test introspect
```

---

## 相关文档

- [ROS 能力发现方案](./docs/plans/2026-02-21-ros-capability-discovery.md)
- [能力地图构建方案](./docs/plans/2026-02-21-ros-capability-map.md)

---

## 许可证

MIT License
