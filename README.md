# ZeroInsect - ROS System Capability Discovery & Planning Tool

> [中文文档](./README-CN.md) | English

ZeroInsect is a ROS (Robot Operating System) system capability discovery and action planning tool. It automatically discovers and builds a capability map of robot systems through runtime introspection and static analysis, supporting goal-oriented action planning and causal reasoning.

---

## Core Features

### 🗺️ Capability Map Construction
- **Auto Discovery**: Automatically discover nodes, topics, services, and actions from ROS runtime
- **Capability Classification**: Classify system capabilities into three categories: Sensing, Decision, and Actuation
- **Causal Association**: Establish causal relationships between actions, supporting goal-oriented action planning

### 🔍 Multi-source Data Collection
- **Runtime Introspection**: Discover real-time system status via DDS
- **Static Analysis**: Parse launch files, extract parameters and remappings
- **Caching System**: Multi-tier caching (memory + disk + TTL) for performance optimization

### 🎯 Intelligent Action Planning
- **Goal Parsing**: Support natural language goal descriptions
- **Path Search**: BFS/A* algorithms for action sequence search
- **Causal Reasoning**: Intelligent planning based on causal graph

### 💻 CLI Tools
- Interactive CLI commands
- Multiple query modes
- Beautiful output formatting

### 🔗 Skill System
- **Multi-source Discovery**: Discover skills from OpenCode, ZeroClaw, local directories
- **Skill Linking**: Link external skills via symlinks
- **ROS Integration**: Generate skills from ROS capabilities

---

## Use Cases

### 1. Robot System Audit
```bash
# Understand what capabilities the robot has
zeroinsect capability map
zeroinsect capability list
```
Use case: System capability audit before new robot deployment

### 2. Troubleshooting
```bash
# View topic publish/subscribe relationships
zeroinsect capability graph --from /scan
```
Use case: Locate data flow breakpoints, understand system architecture

### 3. Task Planning
```bash
# Let the robot move to target location
zeroinsect capability plan --goal "move to kitchen"
```
Use case: Automatic decomposition of high-level tasks into executable action sequences

### 4. Tool Integration
```rust
// Integrate with other systems via API
use zeroinsect::capability_map::{CapabilityClassifier, ActionPlanner, Goal};

let snapshot = introspector.capture_snapshot()?;
let classifier = CapabilityClassifier::new();
let cap_map = classifier.classify(&snapshot);
let planner = ActionPlanner::new(&cap_map);
let plan = planner.plan(&Goal::from_string("move to target"));
```
Use case: Building robot applications, ZeroClaw tool discovery

---

## Quick Start

### Installation

```bash
cargo build
```

### Basic Usage

```bash
# View capability map overview
cargo run -- capability map

# Filter by category
cargo run -- capability map --category sensing
cargo run -- capability map --category actuation

# View specific node
cargo run -- capability map --node /scan

# Plan actions
cargo run -- capability plan --goal "move to target location"

# View causal relationships
cargo run -- capability graph --from /cmd_vel
```

---

## Architecture

### Module Structure

```
src/
├── introspect/          # Capability discovery module
│   ├── runtime.rs      # Runtime introspection
│   ├── launch.rs       # Launch file parsing
│   ├── cache.rs        # Cache management
│   └── types.rs        # Data structures
├── capability_map/     # Capability map module
│   ├── graph.rs        # Graph structure and operations
│   ├── classifier.rs   # Capability classifier
│   └── planner.rs      # Action planner
├── skill_executor/     # Skill execution module
│   ├── registry.rs     # Skill registry
│   ├── loader.rs       # Skill loader
│   ├── executor.rs     # Skill executor
│   └── discovery.rs    # Skill discovery
├── tools/              # Tool definitions
├── ros2/               # ROS2 interface
├── bridge/             # ZeroClaw bridge
└── main.rs             # CLI entry point
```

### Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Data Collection Layer                     │
├──────────────────┬──────────────────┬──────────────────────┤
│ Runtime Intro.   │  Launch Files    │   Cache System       │
│   (DDS/rcl)      │  (quick-xml)     │  (memory + disk)     │
└────────┬─────────┴────────┬─────────┴──────────┬──────────┘
         │                   │                     │
         ▼                   ▼                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  Capability Aggregation Layer                │
│  - Capability Classification (Sensing/Decision/Actuation)   │
│  - Causal Edge Inference (Produces/Enables/Triggers)        │
│  - Graph Structure Construction                             │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      API / CLI Layer                         │
│  - CapabilityMap (Rust API)                                 │
│  - capability map/plan/graph/list (CLI)                     │
│  - skill list/call/info/discover/link (CLI)                 │
└─────────────────────────────────────────────────────────────┘
```

---

## CLI Commands

### capability map

View system capability map overview.

```bash
# Full overview
zeroinsect capability map

# Filter by category
zeroinsect capability map --category sensing     # Sensing capabilities
zeroinsect capability map --category decision    # Decision capabilities
zeroinsect capability map --category actuation   # Actuation capabilities

# View specific node details
zeroinsect capability map --node /scan
```

### capability plan

Plan action sequence based on goal.

```bash
# Natural language goals
zeroinsect capability plan --goal "move to kitchen"
zeroinsect capability plan --goal "grasp object"
zeroinsect capability plan --goal "navigate to target"
```

### capability graph

Query causal relationship graph.

```bash
# View edges from a node
zeroinsect capability graph --from /scan

# View edges to a node
zeroinsect capability graph --target /cmd_vel

# View all edges
zeroinsect capability graph
```

### capability list

List all ROS elements.

```bash
# List all topics
zeroinsect capability list

# Filter by category
zeroinsect capability list --category decision
```

### skill list

List skills from different sources.

```bash
# List ROS skills (default)
zeroinsect skill list

# List skills from specific source
zeroinsect skill list --source ros        # ROS runtime skills
zeroinsect skill list --source opencode   # OpenCode skills
zeroinsect skill list --source zeroclaw   # ZeroClaw/config skills
zeroinsect skill list --source local      # Local ./skills directory
zeroinsect skill list --source linked     # Symlinked skills
zeroinsect skill list --source all        # All sources

# Filter by category
zeroinsect skill list --category navigation
```

### skill discover

Discover skills from standard locations.

```bash
# Discover all skills
zeroinsect skill discover

# Filter by source
zeroinsect skill discover --source opencode
zeroinsect skill discover --source config
```

### skill link / unlink

Manage skill symlinks.

```bash
# Link external skill
zeroinsect skill link --path ~/.config/opencode/skills/my-skill

# Unlink skill
zeroinsect skill unlink --name my-skill
```

### skill info / call

Get skill info and execute skills.

```bash
# Get skill information
zeroinsect skill info --name ros2_topic_cmd_vel

# Call skill with parameters
zeroinsect skill call --name ros2_topic_cmd_vel --params "linear_x=0.5,angular_z=0.1"
```

---

## Rust API Usage

### Basic Workflow

```rust
use zeroinsect::introspect::runtime::RosRuntimeIntrospector;
use zeroinsect::capability_map::{CapabilityClassifier, ActionPlanner, Goal};

// 1. Get system snapshot
let mut introspector = RosRuntimeIntrospector::new();
let snapshot = introspector.capture_snapshot()?;

// 2. Build capability map
let classifier = CapabilityClassifier::new();
let cap_map = classifier.classify(&snapshot);

// 3. Plan actions
let planner = ActionPlanner::new(&cap_map);
let goal = Goal::from_string("move to target location");
let plan = planner.plan(&goal);

// 4. Execute action sequence
for step in plan.steps {
    println!("Execute: {}", step.name);
}
```

### Skill Discovery

```rust
use zeroinsect::skill_executor::{SkillDiscovery, SkillSource};

let discovery = SkillDiscovery::new();

// Discover from all standard locations
let skills = discovery.discover_from_standard_locations();

// Discover from specific source
let opencode_skills = discovery.discover_from_opencode();
let config_skills = discovery.discover_from_config();

// Link external skill
discovery.link_skill(
    &PathBuf::from("~/.config/opencode/skills/my-skill"),
    &PathBuf::from("./skills")
)?;
```

### Data Structures

```rust
// Capability node
pub struct CapabilityNode {
    pub id: String,
    pub name: String,
    pub category: CapabilityCategory,  // Sensing/Decision/Actuation
    pub ros_type: RosCapabilityType,   // Node/Topic/Service/Action
    pub node: String,
    pub description: String,
    pub preconditions: Vec<Condition>,
    pub effects: Vec<Effect>,
}

// Causal edge
pub struct CausalEdge {
    pub from: String,
    pub to: String,
    pub relation: CausalRelation,  // Enables/Produces/Consumes/Conflicts/Triggers
    pub probability: f32,
}

// Capability map
pub struct CapabilityMap {
    pub nodes: HashMap<String, CapabilityNode>,
    pub edges: Vec<CausalEdge>,
    pub topics: HashMap<String, TopicCapability>,
    pub services: HashMap<String, ServiceCapability>,
    pub actions: HashMap<String, ActionCapability>,
}

// Discovered skill
pub struct DiscoveredSkill {
    pub name: String,
    pub path: PathBuf,
    pub source: SkillSource,  // Local/OpenCode/ZeroClaw/Linked
    pub has_skill_toml: bool,
    pub has_skill_md: bool,
}
```

---

## Capability Classification Rules

### Topic Classification

| Category | Keywords | Examples |
|----------|----------|----------|
| Sensing | sensor, camera, laser, imu, odom, tf, joint_states | `/scan`, `/camera/image`, `/odom` |
| Actuation | cmd, velocity, trajectory, gripper, move | `/cmd_vel`, `/gripper/command` |

### Service Classification

| Category | Keywords | Examples |
|----------|----------|----------|
| Sensing | get, query, check, localize | `/get_state`, `/localize` |
| Decision | plan, compute, decide, optimize | `/plan_path`, `/compute_route` |
| Actuation | set, control, move, execute | `/set_velocity`, `/execute_action` |

---

## Extension Development

### Add New Capability Classifier

```rust
impl CapabilityClassifier {
    pub fn custom_classify(&self, name: &str) -> CapabilityCategory {
        // Implement custom classification logic
        // ...
    }
}
```

### Add New Planning Algorithm

```rust
pub struct CustomPlanner {
    // Custom planner
}

impl ActionPlannerTrait for CustomPlanner {
    fn plan(&self, goal: &Goal) -> ActionPlan {
        // Implement custom planning algorithm
        // ...
    }
}
```

---

## Testing

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test capability_map
cargo test introspect
cargo test skill_executor
```

---

## Related Documentation

- [ROS Capability Discovery Plan](./docs/plans/2026-02-21-ros-capability-discovery.md)
- [Capability Map Construction Plan](./docs/plans/2026-02-21-ros-capability-map.md)
- [ROS Skill Execution Plan](./docs/plans/2026-02-21-ros-skill-execution.md)

---

## License

MIT
