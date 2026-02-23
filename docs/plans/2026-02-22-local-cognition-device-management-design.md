---
name: local-cognition-device-management
description: |
  本地认知设备管理系统，使用本地 LLM (Ollama) 理解设备节点状态和维护数据。
  所有认知和决策能力都实现为可复用的 Skills。
  
  Triggers: 分析设备状态, 检测异常, 预测故障, 任务调度, 资源优化
---

# Local Cognition Device Management System

## Overview

A **Local Cognition Device Management System** that uses local LLM (Ollama) to understand device node states and maintain data. All cognitive and decision capabilities are implemented as reusable Skills.

### Core Goals

- **Device State Understanding**: LLM understands real-time device status
- **Data Pattern Analysis**: Analyze sensor data streams for anomalies and trends
- **Semantic Knowledge Graph**: Build device capability associations
- **Autonomous Decision Making**: Schedule and optimize based on understanding

## 1. Overview

A **Local Cognition Device Management System** that uses local LLM (Ollama) to understand device node states and maintain data. All cognitive and decision capabilities are implemented as reusable Skills.

### 1.1 Core Goals

- **Device State Understanding**: LLM understands real-time device status
- **Data Pattern Analysis**: Analyze sensor data streams for anomalies and trends
- **Semantic Knowledge Graph**: Build device capability associations
- **Autonomous Decision Making**: Schedule and optimize based on understanding

### 1.2 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Local LLM (Ollama)                            │
│                    ┌─────────────────┐                          │
│                    │   Skill Engine  │                          │
│                    │  • analyze      │                          │
│                    │  • predict      │                          │
│                    │  • schedule     │                          │
│                    │  • optimize     │                          │
│                    └────────┬────────┘                          │
└─────────────────────────────┼───────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         ▼                    ▼                    ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  MQTT Hub    │     │  ROS2 Hub    │     │  Data Lake  │
│  (Device)   │     │  (Runtime)   │     │  (RocksDB)  │
└──────────────┘     └──────────────┘     └──────────────┘
```

## 2. Skill System

### 2.1 Skill Directory Structure

```
skills/
├── analyze_status/           # Analyze device status
├── detect_anomaly/          # Detect anomaly patterns
├── predict_failure/          # Predict failure
├── schedule_task/            # Task scheduling
├── optimize_resource/        # Resource optimization
└── custom/                   # User custom skills
    ├── patrol_route/
    └── emergency_stop/
```

### 2.2 skill.toml Format

```yaml
[skill]
name = "analyze_status"
version = "1.0.0"
description = "Analyze device status using LLM"

[skill.execution]
entry = "main.py"
runtime = "python3"

[skill.inputs]
required = ["device_id", "status_data"]
optional = ["historical_data"]

[skill.outputs]
- "analysis_result"
- "confidence_score"
- "recommendations"

[skill.config]
llm_model = "llama3.2"
llm_endpoint = "http://localhost:11434"
timeout = 30
```

### 2.3 Skill Interface

```python
class Skill:
    def execute(self, context: SkillContext) -> SkillResult:
        """
        Args:
            context.input: dict - Input data
            context.device_state: DeviceState - Current device states
            context.historical: DataLake - Historical data access
            
        Returns:
            SkillResult with output data
        """
        raise NotImplementedError
```

## 3. Data Lake

### 3.1 Storage Structure

Using **RocksDB** for time-series data storage:

```rust
// Device State Store
cf_name: "device_state"
key: "device_{id}_state_{timestamp}"
value: DeviceState JSON

// Sensor Data Store
cf_name: "sensor_data"
key: "sensor_{id}_{timestamp}"
value: SensorData JSON

// Historical Analytics
cf_name: "analytics"
key: "analyze_{device_id}_{time_window}"
value: AnalyticsResult JSON
```

### 3.2 Data Models

```rust
struct DeviceState {
    device_id: String,
    timestamp: u64,
    status: DeviceStatus,  // online, offline, error, idle
    cpu_usage: f32,
    memory_usage: f32,
    temperature: f32,
    last_command: Option<String>,
}

struct SensorData {
    device_id: String,
    sensor_type: String,  // imu, temperature, voltage
    values: Vec<f32>,
    timestamp: u64,
}

struct DeviceKnowledge {
    device_id: String,
    capabilities: Vec<Capability>,
    relationships: Vec<Relationship>,
    last_analysis: Option<AnalysisResult>,
}
```

### 3.3 Operations

| Operation | Description |
|-----------|-------------|
| `store_state()` | Store current device state |
| `store_sensor()` | Store sensor reading |
| `query_range()` | Query data by time range |
| `get_latest()` | Get latest state |
| `aggregate()` | Aggregate over time window |

## 4. Knowledge Graph

### 4.1 Graph Structure

```rust
struct KnowledgeGraph {
    nodes: HashMap<String, DeviceNode>,
    edges: Vec<RelationshipEdge>,
}

struct DeviceNode {
    device_id: String,
    device_type: DeviceType,
    capabilities: Vec<Capability>,
    state: DeviceState,
    metadata: HashMap<String, String>,
}

struct RelationshipEdge {
    from: String,
    to: String,
    relation_type: RelationType,  // depends_on, produces, controls
    weight: f32,
}
```

### 4.2 Semantic Associations

```
Device A (sensor) ──produces──► Data X ──consumes──► Device B (processor)
                                                    │
                                                    └──controls──► Device C (actuator)
```

### 4.3 LLM-Enhanced Understanding

```python
# Use LLM to understand device relationships
prompt = f"""Given these devices and their capabilities:
{devices}

Identify semantic relationships and suggest:
1. Data flow between devices
2. Potential dependencies
3. Optimization opportunities"""
```

## 5. Cognitive Engine

### 5.1 Processing Pipeline

```
Raw Data ──► Preprocess ──► Select Skill ──► Execute ──► Postprocess ──► Result
                │                │             │            │
                ▼                ▼             ▼            ▼
           [Filter]      [Skill Match]   [LLM推理]    [Parse Output]
```

### 5.2 Built-in Skills

#### analyze_status

```python
# Input: device_id, current status
# Output: analysis_result, confidence, recommendations

1. Get device state from Data Lake
2. Build LLM prompt with status + history
3. Query Ollama for analysis
4. Parse and return results
```

#### detect_anomaly

```python
# Input: sensor_data stream
# Output: anomaly_detected, anomaly_type, severity

1. Get recent sensor data
2. Compare with baseline patterns
3. Use LLM to identify anomalies
4. Return anomaly report
```

#### predict_failure

```python
# Input: device_id, historical_data
# Output: failure_probability, time_to_failure, risk_factors

1. Get historical performance data
2. Use LLM to analyze trends
3. Predict failure probability
4. Recommend maintenance
```

## 6. Decision Engine

### 6.1 Decision Pipeline

```
Task/Event ──► Analyze Context ──► Select Decision Skill ──► Execute ──► Action
                  │                     │                   │
                  ▼                     ▼                   ▼
            [Understand]          [Skill Match]       [Execute Action]
```

### 6.2 Built-in Decision Skills

#### schedule_task

```python
# Input: task_list, available_devices
# Output: schedule_plan

1. Get task requirements
2. Query device states
3. Use LLM to optimize assignment
4. Return schedule
```

#### optimize_resource

```python
# Input: current_allocation, task_queue
# Output: optimization_suggestions

1. Analyze resource usage
2. Identify bottlenecks
3. Use LLM to suggest optimizations
4. Return optimization plan
```

## 7. Integration with Existing System

### 7.1 Reuse ZeroInsect Components

| Component | Usage |
|-----------|-------|
| `skill_executor` | Execute Skills |
| `broker` | MQTT message hub |
| `llm/engine` | LLM integration |
| `storage/kv_store` | Data Lake base |

### 7.2 New Components to Add

```
src/
├── cognition/
│   ├── mod.rs
│   ├── data_lake.rs      # Time-series storage
│   ├── knowledge_graph.rs # Semantic associations
│   ├── cognitive_engine.rs # Processing pipeline
│   └── decision_engine.rs  # Decision pipeline
│
├── skills/
│   ├── mod.rs
│   └── registry.rs        # Skill registry
│
examples/
└── skills/               # Pre-built skills
    ├── analyze_status/
    ├── detect_anomaly/
    ├── predict_failure/
    ├── schedule_task/
    └── optimize_resource/
```

## 8. Implementation Phases

### Phase 1: Data Lake

- [ ] Create data_lake module
- [ ] Implement device state storage
- [ ] Implement sensor data storage
- [ ] Add time-range queries
- [ ] Integration with broker

### Phase 2: Knowledge Graph

- [ ] Create knowledge_graph module
- [ ] Implement node/edge structures
- [ ] Add semantic relationship inference
- [ ] Integration with LLM

### Phase 3: Skill System Enhancement

- [ ] Create skills/ directory structure
- [ ] Implement analyze_status skill
- [ ] Implement detect_anomaly skill
- [ ] Implement predict_failure skill

### Phase 4: Decision Skills

- [ ] Implement schedule_task skill
- [ ] Implement optimize_resource skill
- [ ] Create decision pipeline

### Phase 5: Integration

- [ ] Connect MQTT/ROS to cognition
- [ ] Connect decision engine to execution
- [ ] End-to-end testing

## 9. Usage Examples

### 9.1 Analyze Device Status

```python
# Via MQTT
Topic: bridge/command/{device_id}
Payload: "analyze status"

# Via CLI
cargo run -- analyze device001
```

### 9.2 Detect Anomaly

```python
# Automatic detection
# When sensor data exceeds threshold
# Trigger detect_anomaly skill
```

### 9.3 Schedule Task

```python
# Via MQTT
Topic: bridge/command/scheduler
Payload: '{"task": "patrol", "devices": ["robot1", "robot2"]}'
```

## 10. Dependencies

- **ollama**: Local LLM inference
- **rocksdb**: Time-series storage
- **mqttrs**: MQTT client
- **r2r**: ROS2 interface
- **reqwest**: HTTP client for Ollama
