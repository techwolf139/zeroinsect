# ROS 能力高级执行系统设计方案

> **目标**：通过 ZeroClaw 调用 Skill，实现对 ROS 能力的高级执行。将 ROS 能力封装为 ZeroClaw Skill 格式，供 ZeroClaw 的 LLM 调用。

> **版本**：v1.1 - 移除内置 LLM，依赖 ZeroClaw

---

## 一、需求分析

### 1.1 核心需求

| 需求 | 描述 | 优先级 |
|------|------|--------|
| **Skill 封装** | 将 ROS 能力封装为 ZeroClaw Skill 格式 | P0 |
| **能力 → Skill 映射** | 自动将 capability map 转换为 Skill | P0 |
| **Skill 执行** | 通过 Skill Executor 执行 ROS 能力 | P0 |
| **ZeroClaw 集成** | 注册到 ZeroClaw Tool/Skill System | P1 |

### 1.2 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                    ZeroClaw 平台                              │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐  │
│  │              LLM Agent Engine                        │  │
│  │   - 意图理解 (Intent Recognition)                    │  │
│  │   - 参数提取 (Parameter Extraction)                 │  │
│  │   - 任务分解 (Task Decomposition)                  │  │
│  │   - 结果解释 (Result Interpretation)                │  │
│  └────────────────────────┬────────────────────────────┘  │
│                           │                                │
│  ┌────────────────────────▼────────────────────────────┐    │
│  │              Skill System                           │    │
│  │   - Skill Registry (ZeroInsect 提供)              │    │
│  │   - Skill Executor (ZeroInsect 提供)              │    │
│  │   - Skill Chaining                                │    │
│  └────────────────────────────────────────────────────┘    │
└───────────────────────────┼─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   ZeroInsect 执行层                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐  │
│  │              Capability Map                           │  │
│  │   - 能力发现 (Discovery)                           │  │
│  │   - 能力分类 (Classification)                       │  │
│  │   - 因果推理 (Causal Inference)                     │  │
│  │   - 动作规划 (Action Planning)                      │  │
│  └─────────────────────────────────────────────────────┘  │
│                           │                                │
│  ┌────────────────────────▼────────────────────────────┐  │
│  │              ROS2 Bridge                           │  │
│  │   - Topic Publisher/Subscriber                    │  │
│  │   - Service Client                                 │  │
│  │   - Action Client                                  │  │
│  └────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 设计原则

- **不内置 LLM**：所有 LLM 功能由 ZeroClaw 提供
- **Skill 驱动**：ZeroInsect 作为 Skill Provider
- **零耦合**：ZeroInsect 只负责 ROS 执行，不处理认知
┌─────────────────────────────────────────────────────────────┐
│                    ZeroClaw 认知层                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐  │
│  │              LLM Agent Engine                        │  │
│  │   - 意图理解 (Intent Recognition)                    │  │
│  │   - 参数提取 (Parameter Extraction)                 │  │
│  │   - 任务分解 (Task Decomposition)                   │  │
│  │   - 结果解释 (Result Interpretation)                 │  │
│  └────────────────────────┬────────────────────────────┘  │
│                           │                               │
│  ┌────────────────────────▼────────────────────────────┐  │
│  │              Skill System                            │  │
│  │   - ROS Skill Registry                              │  │
│  │   - Skill Executor                                 │  │
│  │   - Skill Chaining                                 │  │
│  └────────────────────────┬────────────────────────────┘  │
└───────────────────────────┼─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   ZeroInsect 执行层                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐  │
│  │              Capability Map                          │  │
│  │   - 能力发现 (Discovery)                            │  │
│  │   - 能力分类 (Classification)                       │  │
│  │   - 因果推理 (Causal Inference)                     │  │
│  │   - 动作规划 (Action Planning)                      │  │
│  └─────────────────────────────────────────────────────┘  │
│                           │                               │
│  ┌────────────────────────▼────────────────────────────┐  │
│  │              ROS2 Bridge                            │  │
│  │   - Topic Publisher/Subscriber                    │  │
│  │   - Service Client                                │  │
│  │   - Action Client                                 │  │
│  └─────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 二、核心概念

### 2.1 Skill 定义

```yaml
# skill.yaml - ROS 能力 Skill 格式

skill:
  name: "ros2_navigation"
  version: "1.0.0"
  description: "Robot navigation capability"
  category: "motion"
  tags: ["ros2", "navigation", "mobile"]

# 能力元数据
capability:
  type: "action"           # action/service/topic
  ros_name: "/navigate_to_pose"
  ros_type: "nav2_msgs/action/NavigateToPose"
  node: "/nav2/navigation"
  
# 输入参数定义
parameters:
  - name: "target_pose"
    type: "pose"
    required: true
    description: "Target pose in map frame"
    
  - name: "tolerance"
    type: "float"
    default: 0.3
    description: "Position tolerance in meters"

# 输出结果定义
returns:
  - name: "success"
    type: "bool"
    
  - name: "message"
    type: "string"
    
  - name: "result_pose"
    type: "pose"

# 前置条件
preconditions:
  - capability: "/scan"
    type: "topic"
    description: "激光雷达数据可用"
    
  - capability: "/amcl_pose"
    type: "topic"  
    description: "定位数据可用"

# 效果/后置条件
effects:
  - capability: "/cmd_vel"
    type: "topic"
    description: "生成运动指令"
```

### 2.2 Skill Chain 定义

```yaml
# skill_chain.yaml - 复合技能定义

chain:
  name: "move_to_and_grasp"
  description: "移动到目标位置并抓取"
  type: "sequential"  # sequential / parallel / conditional

steps:
  - skill: "ros2_navigation"
    input:
      target_pose: "${chain.target_pose}"
      
  - skill: "ros2_detect_object"
    input:
      region_of_interest: "${steps[0].robot_position}"
    output:
      bind_to: "object_pose"
      
  - skill: "ros2_grasp"
    input:
      target: "${object_pose}"
```

### 2.3 LLM Prompt 模板

```python
# 意图理解 Prompt
INTENT_PROMPT = """
你是一个机器人任务规划助手。用户会用自然语言描述任务。
请分析用户的意图，并将其转换为结构化的技能调用。

## 可用技能列表
{skill_list}

## 用户输入
{user_input}

## 输出格式
请以 JSON 格式输出：
{{
  "intent": "任务意图摘要",
  "skill_name": "要调用的技能名称",
  "parameters": {{ "参数名": "参数值" }},
  "confidence": 0.0-1.0,
  "needs_clarification": false,
  "clarification_questions": []
}}
"""

# 结果解释 Prompt
RESULT_PROMPT = """
用户的原始请求是：{user_request}
技能执行结果是：{execution_result}

请用自然语言向用户解释执行结果。
如果执行失败，说明原因并提供建议。
"""

# 任务分解 Prompt
DECOMPOSE_PROMPT = """
用户请求：{user_request}
这是一个复杂任务，需要分解为多个步骤。

## 可用技能
{available_skills}

## 分解要求
1. 将任务分解为顺序或并行的技能调用
2. 考虑技能之间的依赖关系
3. 输出技能调用序列
"""
```

---

## 三、模块设计

### 3.1 模块结构

```
src/skill_executor/
├── mod.rs           # 模块导出
├── registry.rs      # Skill 注册表
├── loader.rs        # Skill 加载器
├── executor.rs      # Skill 执行器
├── chain.rs         # Skill 链执行
├── llm.rs           # LLM 接口
└── prompt.rs        # Prompt 模板
```

### 3.2 核心数据结构

```rust
// Skill 定义
pub struct RosSkill {
    pub metadata: SkillMetadata,
    pub capability: CapabilityRef,
    pub parameters: Vec<ParameterDef>,
    pub returns: Vec<ReturnDef>,
    pub preconditions: Vec<Condition>,
    pub effects: Vec<Effect>,
}

pub struct SkillMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: SkillCategory,
    pub tags: Vec<String>,
}

pub enum SkillCategory {
    Motion,
    Perception,
    Manipulation,
    Navigation,
    Communication,
    Custom(String),
}

// Skill 调用请求
pub struct SkillRequest {
    pub skill_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: ExecutionContext,
}

// Skill 调用结果
pub struct SkillResponse {
    pub success: bool,
    pub result: serde_json::Value,
    pub message: String,
    pub execution_time_ms: u64,
}

// Skill 链
pub struct SkillChain {
    pub name: String,
    pub chain_type: ChainType,
    pub steps: Vec<ChainStep>,
}

pub enum ChainType {
    Sequential,
    Parallel,
    Conditional { condition: String },
}

pub struct ChainStep {
    pub skill: String,
    pub input_mapping: HashMap<String, String>,  // ${steps[0].output}
    pub output_binding: Option<String>,
    pub on_error: ChainErrorHandling,
}
```

### 3.3 LLM 接口

```rust
pub trait LlmClient: Send + Sync {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse, LlmError>;
    async fn structured_output<T: DeserializeOwned>(&self, 
        messages: Vec<ChatMessage>, 
        schema: &Schema
    ) -> Result<T, LlmError>;
}

pub struct ExecutionContext {
    pub user_id: String,
    pub session_id: String,
    pub history: Vec<Interaction>,
    pub robot_state: RobotStateSnapshot,
    pub environment: EnvironmentInfo,
}

pub struct Interaction {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}
```

---

## 四、迭代计划

### Iteration 1: Skill 注册与加载

**目标**：建立 Skill 注册表，支持从文件加载 Skill 定义

| 任务 | 文件 | 描述 |
|------|------|------|
| T1.1 | `src/skill_executor/registry.rs` | 创建 RosSkill 数据结构 |
| T1.2 | `src/skill_executor/registry.rs` | 实现 SkillRegistry (注册/查询) |
| T1.3 | `src/skill_executor/loader.rs` | 实现 YAML/JSON Skill 加载器 |
| T1.4 | `src/skill_executor/loader.rs` | 实现从 CapabilityMap 自动生成 Skill |
| T1.5 | `src/skill_executor/mod.rs` | 模块入口 |

**验收标准**：
- [ ] SkillRegistry 可以注册/查询 Skill
- [ ] 可以从 YAML 文件加载 Skill 定义
- [ ] 可以从 CapabilityMap 自动生成 Skill

### Iteration 2: Skill 执行器

**目标**：实现 Skill 执行逻辑，支持 ROS2 调用

| 任务 | 文件 | 描述 |
|------|------|------|
| T2.1 | `src/skill_executor/executor.rs` | 创建 SkillExecutor 结构 |
| T2.2 | `src/skill_executor/executor.rs` | 实现参数验证 |
| T2.3 | `src/skill_executor/executor.rs` | 实现 Topic 发布执行 |
| T2.4 | `src/skill_executor/executor.rs` | 实现 Service 调用执行 |
| T2.5 | `src/skill_executor/executor.rs` | 实现 Action 执行 |
| T2.6 | `src/skill_executor/executor.rs` | 实现前置条件检查 |

**验收标准**：
- [ ] 可以执行 Topic 发布类型的 Skill
- [ ] 可以执行 Service 调用类型的 Skill
- [ ] 可以执行 Action 类型的 Skill
- [ ] 执行前检查前置条件

### Iteration 3: Skill 链执行

**目标**：支持复合 Skill 执行（顺序/并行/条件）

| 任务 | 文件 | 描述 |
|------|------|------|
| T3.1 | `src/skill_executor/chain.rs` | 定义 SkillChain 结构 |
| T3.2 | `src/skill_executor/chain.rs` | 实现顺序执行 |
| T3.3 | `src/skill_executor/chain.rs` | 实现并行执行 |
| T3.4 | `src/skill_executor/chain.rs` | 实现参数绑定传递 |
| T3.5 | `src/skill_executor/chain.rs` | 实现错误处理和回滚 |

**验收标准**：
- [ ] 顺序执行多个 Skill
- [ ] 并行执行独立 Skill
- [ ] 上一步输出作为下一步输入

### Iteration 4: ZeroClaw 集成 (替代 LLM 迭代)

**目标**：将 Skill 注册到 ZeroClaw Tool/Skill System

> 注意：LLM 功能由 ZeroClaw 提供，ZeroInsect 只提供 ROS 执行能力

| 任务 | 文件 | 描述 |
|------|------|------|
| T4.1 | `src/skill_executor/exporter.rs` | 导出 Skill 为 ZeroClaw 格式 |
| T4.2 | `src/skill_executor/exporter.rs` | 生成 SKILL.toml 文件 |
| T4.3 | `src/main.rs` | 添加 `skill export` 命令 |
| T4.4 | `src/main.rs` | 生成 ZeroClaw 工具定义 |

**验收标准**：
- [ ] 可导出 Skill 到 ZeroClaw 兼容格式
- [ ] 自动生成 SKILL.toml
- [ ] 注册到 ZeroClaw Tool System

### Iteration 5: CLI 集成

**目标**：提供 CLI 命令调用 Skill

| 任务 | 文件 | 描述 |
|------|------|------|
| T5.1 | `src/main.rs` | 添加 `skill` 子命令 |
| T5.2 | `src/main.rs` | 实现 `skill list` |
| T5.3 | `src/main.rs` | 实现 `skill call` |
| T5.4 | `src/main.rs` | 实现 `skill execute` (通过 ZeroClaw) |
| T5.5 | `src/main.rs` | 实现 `skill chain` |

**验收标准**：
- [ ] `skill list` 显示所有可用 Skill
- [ ] `skill call --name xxx --param k=v` 调用 Skill
- [ ] `skill execute` 委托给 ZeroClaw 执行

---

## 五、数据流设计

### 5.1 自然语言任务执行流程

```
┌─────────────────────────────────────────────────────────────┐
│               自然语言任务执行流程                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  用户: "让机器人移动到厨房，然后抓取红色的盒子"               │
│                                                              │
│  1. 意图理解 (LLM)                                         │
│     ┌─────────────────────────────────────┐                │
│     │  解析用户输入                        │                │
│     │  - 意图: move_and_grasp             │                │
│     │  - 参数: target=kitchen, object=red_box│               │
│     │  - 置信度: 0.92                    │                │
│     └──────────────┬──────────────────────┘                │
│                    │                                        │
│                    ▼                                        │
│  2. 技能匹配                                                │
│     ┌─────────────────────────────────────┐                │
│     │  从 Registry 匹配可用 Skill          │                │
│     │  - ros2_navigation                  │                │
│     │  - ros2_detect_object               │                │
│     │  - ros2_grasp                       │                │
│     └──────────────┬──────────────────────┘                │
│                    │                                        │
│                    ▼                                        │
│  3. 参数提取与验证                                          │
│     ┌─────────────────────────────────────┐                │
│     │  验证参数完整性                      │                │
│     │  - target_pose: ✓ (from context)    │                │
│     │  - object_color: ✓ (from vision)   │                │
│     └──────────────┬──────────────────────┘                │
                    │                                        │
                    ▼                                        │
│  4. Skill Chain 执行                                        │
│     ┌─────────────────────────────────────┐                │
│     │  顺序执行:                          │                │
│     │  1. navigate(target=kitchen)      │                │
│     │     ↓ 成功                          │                │
│     │  2. detect_object(color=red)      │                │
│     │     ↓ 成功                          │                │
│     │  3. grasp(object=box_123)          │                │
│     └──────────────┬──────────────────────┘                │
                    │                                        │
                    ▼                                        │
│  5. 结果返回 ZeroClaw (LLM 解释)                          │
│     ┌─────────────────────────────────────┐                │
│     │  ZeroClaw LLM 将结果转换为自然语言   │                │
│     │  "机器人已成功移动到厨房位置，        │                │
│     │   检测到红色盒子位于(1.2, 0.5)处，  │                │
│     │   并已完成抓取操作。"                │                │
│     └─────────────────────────────────────┘                │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Skill 调用时序图

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  User    │     │   LLM    │     │ Registry │     │Executor  │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ "移动到厨房"   │                │                │
     │──────────────>│                │                │
     │                │                │                │
     │  意图解析      │                │                │
     │──────────────>│                │                │
     │                │                │                │
     │  {skill: nav, │                │                │
     │   params: {...}}               │                │
     │<─────────────│                │                │
     │                │                │                │
     │                │  查询 Skill    │                │
     │                │───────────────>│                │
     │                │                │                │
     │                │  Skill 定义    │                │
     │                │<───────────────│                │
     │                │                │                │
     │                │  验证参数      │                │
     │                │───────────────────────────────>│
     │                │                │                │
     │                │  检查前置条件   │                │
     │                │───────────────────────────────>│
     │                │                │                │
     │                │  ROS2 Action 调用             │
     │                │───────────────────────────────>│
     │                │                │                │
     │                │                │    Action Client
     │                │                │─────────────────────>│ ROS2
     │                │                │                │   Network
     │                │                │                │<───────
     │                │                │                │
     │                │  执行结果      │                │
     │                │<───────────────────────────────│
     │                │                │                │
     │  执行成功，已  │                │                │
     │  移动到厨房   │                │                │
     │<──────────────│                │                │
     │                │                │                │
```

---

## 六、与现有模块集成

### 6.1 依赖关系

```
┌─────────────────────────────────────────────────────────────┐
│                    模块依赖关系                               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   capability_map (已有)                                     │
│       │                                                     │
│       └──► skill_executor (新增)                            │
│                     │                                       │
│                     ├──► registry.rs ←── introspect        │
│                     ├──► executor.rs ←── ros2 client       │
│                     ├──► chain.rs                          │
│                     ├──► llm.rs                           │
│                     └──► prompt.rs                         │
│                                                              │
│   tools (已有)                                              │
│       │                                                     │
│       └──► RobotTool ◄────────────────────────────────────┘
│                           │                                  │
│   bridge (已有)              │                                  │
│       │                     │                                  │
│       ▼                     ▼                                  │
│   ZeroClaw ◄───────────────────────────────────────────────►│
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 API 对接

```rust
// 从 CapabilityMap 自动生成 Skill
use crate::capability_map::{CapabilityClassifier, CapabilityMap};
use crate::skill_executor::{RosSkill, SkillRegistry};

let cap_map = classifier.classify(&snapshot);
let registry = SkillRegistry::new();

// 从 capability 生成 skill
for node in cap_map.nodes.values() {
    let skill = RosSkill::from_capability(node);
    registry.register(skill);
}

// 执行 Skill
use crate::skill_executor::{SkillExecutor, SkillRequest};

let executor = SkillExecutor::new(registry);
let request = SkillRequest {
    skill_name: "ros2_navigation".to_string(),
    parameters: serde_json::json!({
        "target_pose": {"x": 1.0, "y": 2.0, "theta": 0.0}
    }),
    context: ExecutionContext::default(),
};

let response = executor.execute(request).await?;
```

---

## 七、Skill 定义示例

### 7.1 导航 Skill

```yaml
skill:
  name: "ros2_navigation"
  version: "1.0.0"
  description: "Navigate robot to specified pose"
  category: "navigation"
  tags: ["ros2", "navigation", "move_base"]

capability:
  type: "action"
  ros_name: "/navigate_to_pose"
  ros_type: "nav2_msgs/action/NavigateToPose"
  node: "/nav2/navigation"

parameters:
  - name: "target_pose"
    type: "pose"
    required: true
    description: "Target pose in map frame"
    
  - name: "tolerance"
    type: "float"
    default: 0.3
    description: "Position tolerance in meters"
    
  - name: "timeout"
    type: "float"
    default: 30.0
    description: "Timeout in seconds"

returns:
  - name: "success"
    type: "bool"
    
  - name: "result_pose"
    type: "pose"
    description: "Final robot pose"
    
  - name: "message"
    type: "string"

preconditions:
  - capability: "/scan"
    type: "topic"
    description: "Laser scan available"
    
  - capability: "/amcl_pose"
    type: "topic"
    description: "Localization available"

effects:
  - capability: "/cmd_vel"
    type: "topic"
    description: "Generates velocity commands"
```

### 7.2 抓取 Skill

```yaml
skill:
  name: "ros2_grasp"
  version: "1.0.0"
  description: "Grasp object at specified location"
  category: "manipulation"
  tags: ["ros2", "manipulation", "grasp"]

capability:
  type: "action"
  ros_name: "/grasp_object"
  ros_type: "grasp_msgs/action/Grasp"
  node: "/manipulator/grasp_server"

parameters:
  - name: "target_position"
    type: "point"
    required: true
    description: "3D position of object"
    
  - name: "approach_height"
    type: "float"
    default: 0.1
    description: "Approach height in meters"
    
  - name: "grasp_force"
    type: "float"
    default: 10.0
    description: "Grasping force in Newtons"

returns:
  - name: "success"
    type: "bool"
    
  - name: "grasp_quality"
    type: "float"
    description: "Grasp quality score 0-1"

effects:
  - capability: "/gripper/command"
    type: "topic"
    description: "Gripper position commands"
```

---

## 八、CLI 命令设计

### 8.1 命令列表

```bash
# 列出所有可用 Skill
zeroinsect skill list
zeroinsect skill list --category navigation
zeroinsect skill list --tag ros2

# 查看 Skill 详情
zeroinsect skill info ros2_navigation

# 调用 Skill (结构化参数)
zeroinsect skill call ros2_navigation --param target_pose.x=1.0 --param target_pose.y=2.0

# 执行 Skill (自然语言)
zeroinsect skill execute "让机器人移动到厨房"

# 执行 Skill 链
zeroinsect skill chain move_and_grasp --param target=kitchen

# 查看执行历史
zeroinsect skill history
```

### 8.2 输出示例

```bash
$ zeroinsect skill list

=== Available Skills ===

[Navigation]
  ✦ ros2_navigation          Navigate to pose
  ✦ ros2_localize            Get robot position
  ✦ ros2_plan_path          Plan path to goal

[Manipulation]
  ✦ ros2_grasp               Grasp object
  ✦ ros2_place               Place object
  ✦ ros2_move_gripper       Control gripper

[Perception]
  ✦ ros2_detect_object       Detect objects
  ✦ ros2_get_depth          Get depth image

$ zeroinsect skill execute "让机器人移动到厨房"

🤔 理解意图: 导航到厨房位置
📋 选择技能: ros2_navigation
⚙️  参数: {target: "kitchen", tolerance: 0.3}

▶️  执行中...
   ✓ 前置条件检查通过
   ✓ 调用 /navigate_to_pose action
   ✓ 等待导航完成...

✅ 执行成功
📍 当前位置: kitchen (1.2, 3.4, 0.0)

💬 机器人已成功移动到厨房位置。
```

---

## 九、测试计划

### 9.1 单元测试

| 模块 | 测试用例 |
|------|----------|
| registry | Skill 注册/查询、重复注册 |
| loader | YAML 解析、错误处理 |
| executor | 参数验证、ROS2 调用 |
| chain | 顺序/并行执行、参数传递 |
| llm | API 调用、响应解析 |

### 9.2 集成测试

| 测试场景 | 预期结果 |
|----------|----------|
| 自然语言 "移动到厨房" | 正确调用 navigation skill |
| 连续执行多个 skill | 正确传递参数 |
| 前置条件不满足时执行 | 返回错误并说明原因 |

---

## 十、风险与挑战

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| LLM API 延迟 | 响应时间增加 | 本地缓存、异步处理 |
| LLM 解析错误 | Skill 调用失败 | 回退到结构化输入 |
| ROS2 执行超时 | 任务卡住 | 超时控制、状态监控 |

---

## 十一、里程碑

| 里程碑 | 预计时间 | 交付物 |
|--------|----------|--------|
| M1: Skill 注册 | 1 天 | Registry + Loader |
| M2: Skill 执行 | 2 天 | Executor |
| M3: Skill 链 | 1 天 | Chain |
| M4: LLM 集成 | 2 天 | LLM 接口 + Prompt |
| M5: CLI | 1 天 | CLI 命令 |

**总工期**：约 7 天
