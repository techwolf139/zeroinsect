# Business Data Analysis Skills Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 基于现有 Data Lake 架构，构建业务数据分析 Skills 体系，实现设备运营分析、效率优化、预测性维护等业务场景。

**Architecture:** 
- Data Lake (RocksDB) 存储原始数据
- Python Skills 调用 Ollama LLM 进行智能分析
- HTTP API 提供统一数据访问接口
- 支持离线/在线模式切换

**Tech Stack:** Python, Ollama (llama3.2), RocksDB, HTTP API, pytest

---

## 当前架构分析

### 已实现的 Skills

| Skill | 功能 | 状态 |
|-------|------|------|
| `analyze_status` | 设备状态分析 | ✅ 已实现 |
| `detect_anomaly` | 异常检测 | ✅ 已实现 |
| `predict_failure` | 故障预测 | ✅ 已实现 |

### 数据流

```
Data Lake (RocksDB)
       │
       ▼ HTTP API (port 8080)
       │
       ▼ requests.get/post
Python Skills
       │
       ▼ /api/generate
Ollama LLM
       │
       ▼ JSON
分析结果
```

---

## 待实现业务场景

### Phase 1: 设备运营分析 (8 Tasks)

### Task 1: 设备利用率分析 Skill

**Files:**
- Create: `skills/analyze_utilization/SKILL.md`
- Create: `skills/analyze_utilization/main.py`

**Step 1: 创建 SKILL.md**

```yaml
---
name: analyze_utilization
description: 分析设备利用率，包括 CPU、内存、运行时间利用率
inputs:
  device_id:
    type: string
    required: true
  time_window:
    type: integer
    default: 86400
outputs:
  utilization: object
  recommendations: array
---
```

**Step 2: 创建 main.py**

```python
#!/usr/bin/env python3
"""
Analyze Device Utilization Skill

分析设备利用率并提供优化建议
"""

import argparse
import json
import os
import sys
import time
import requests

DATA_LAKE_URL = os.environ.get("DATA_LAKE_URL", "http://localhost:8080")
OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434")

def get_device_states(device_id: str, time_window: int) -> list:
    """获取设备状态历史"""
    now = int(time.time())
    url = f"{DATA_LAKE_URL}/api/device/{device_id}/state/range"
    resp = requests.get(url, params={"start_ts": now - time_window, "end_ts": now})
    return resp.json() if resp.status_code == 200 else []

def analyze_utilization(states: list) -> dict:
    """使用 LLM 分析利用率"""
    if not states:
        return {"status": "error", "message": "No data"}
    
    prompt = f"""分析以下设备利用率数据:
json.dumps(states{, indent=2)}

返回 JSON:
{{
  "utilization": {{"cpu_avg": float, "memory_avg": float, "uptime_percent": float}},
  "recommendations": ["建议1", "建议2"]
}}"""
    
    # 调用 Ollama...
    pass

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--device-id", required=True)
    parser.add_argument("--time-window", type=int, default=86400)
    args = parser.parse_args()
    
    states = get_device_states(args.device_id, args.time_window)
    result = analyze_utilization(states)
    print(json.dumps(result, indent=2))

if __name__ == "__main__":
    main()
```

**Step 3: 测试**

```bash
python skills/analyze_utilization/main.py --device-id robot-arm-01 --mock
```

### Task 2: 设备运行时间分析 Skill

**Files:**
- Create: `skills/analyze_uptime/SKILL.md`
- Create: `skills/analyze_uptime/main.py`

**分析内容:**
- 在线/离线时长统计
- 平均故障间隔时间 (MTBF)
- 可用性百分比

### Task 3: 能耗分析 Skill

**Files:**
- Create: `skills/analyze_energy/SKILL.md`
- Create: `skills/analyze_energy/main.py`

**分析内容:**
- 功耗趋势
- 异常能耗检测
- 节能建议

### Task 4: 设备健康评分 Skill

**Files:**
- Create: `skills/health_score/SKILL.md`
- Create: `skills/health_score/main.py`

**输出:**
- 综合健康评分 (0-100)
- 各维度得分
- 改进建议

---

### Phase 2: 业务场景分析 (8 Tasks)

### Task 5: 机械臂操作分析 Skill

**Files:**
- Create: `skills/analyze_robot_arm/SKILL.md`
- Create: `skills/analyze_robot_arm/main.py`

**分析内容:**
- 关节角度范围
- 运动平滑度
- 任务完成效率
- 异常振动检测

**数据源:**
```python
# 从 Data Lake 获取
joint_position = get_sensor_data("robot-arm-01", "joint_position")
joint_velocity = get_sensor_data("robot-arm-01", "joint_velocity")
torque = get_sensor_data("robot-arm-01", "torque")
```

### Task 6: AGV 路径分析 Skill

**Files:**
- Create: `skills/analyze_agv_path/SKILL.md`
- Create: `skills/analyze_agv_path/main.py`

**分析内容:**
- 路径覆盖率
- 行驶效率
- 碰撞风险评估
- 电池消耗优化

### Task 7: 清洁效率分析 Skill

**Files:**
- Create: `skills/analyze_cleaning/SKILL.md`
- Create: `skills/analyze_cleaning/main.py`

**分析内容:**
- 清洁覆盖率
- 重复清洁检测
- 清洁效率评分
- 任务时间优化

### Task 8: 生产节拍分析 Skill

**Files:**
- Create: `skills/analyze_production/SKILL.md`
- Create: `skills/analyze_production/main.py`

**分析内容:**
- 周期时间
- 瓶颈识别
- OEE (设备综合效率)
- 产能预测

---

### Phase 3: 智能建议 (6 Tasks)

### Task 9: 维护计划生成 Skill

**Files:**
- Create: `skills/generate_maintenance/SKILL.md`
- Create: `skills/generate_maintenance/main.py`

**输出:**
- 建议维护时间
- 维护项目清单
- 备件需求
- 预估停机时间

### Task 10: 资源调度优化 Skill

**Files:**
- Create: `skills/optimize_schedule/SKILL.md`
- Create: `skills/optimize_schedule/main.py`

**输入:**
- 任务列表
- 设备状态
- 约束条件

**输出:**
- 最优调度方案
- 预期完成时间
- 资源利用率

### Task 11: 异常根因分析 Skill

**Files:**
- Create: `skills/root_cause/SKILL.md`
- Create: `skills/root_cause/main.py`

**功能:**
- 接收异常事件
- 关联历史数据
- 输出根因分析

### Task 12: 预测性维护 Skill

**Files:**
- Create: `skills/predictive_maintenance/SKILL.md`
- Create: `skills/predictive_maintenance/main.py`

**功能:**
- 基于历史预测部件寿命
- 更换建议时间
- 维护优先级排序

---

### Phase 4: 集成与测试 (8 Tasks)

### Task 13: Skill 调用封装

**Files:**
- Create: `skills/skill_runner.py`

```python
class SkillRunner:
    """统一 Skill 调用封装"""
    
    SKILLS = {
        "analyze_status": "skills/analyze_status/main.py",
        "analyze_utilization": "skills/analyze_utilization/main.py",
        "detect_anomaly": "skills/detect_anomaly/main.py",
        # ...
    }
    
    def run(self, skill_name: str, **kwargs) -> dict:
        """统一调用入口"""
        pass
```

### Task 14: CLI 命令集成

**Files:**
- Modify: `src/main.rs`

```bash
# 新增命令
cargo run -- skill analyze robot-arm-01 utilization
cargo run -- skill analyze robot-arm-01 robot_arm_operation
cargo run -- skill analyze agv-01 path
```

### Task 15: 批量分析脚本

**Files:**
- Create: `scripts/batch_analyze.py`

```bash
# 分析所有设备
python scripts/batch_analyze.py --skill analyze_utilization

# 分析特定设备组
python scripts/batch_analyze.py --device-group production_robots
```

### Task 16: 测试用例编写

**测试文件:**
- `skills/tests/test_business_analysis.py`

**测试用例 (30+):**

```python
# 1. 设备利用率分析
def test_utilization_with_normal_data():
    # 正常数据 → 正常分析
    
def test_utilization_with_no_data():
    # 无数据 → 返回错误
    
def test_utilization_with_partial_data():
    # 部分数据 → 尽量分析

# 2. 机械臂操作分析
def test_robot_arm_waving():
    # 挥臂动作 → 正常分析
    
def test_robot_arm_sweeping():
    # 清扫动作 → 正常分析

# 3. AGV 路径分析
def test_agv_coverage():
    # 覆盖率计算
    
def test_agv_efficiency():
    # 效率分析

# ... 共 30+ 测试用例
```

### Task 17: 集成测试

**测试文件:**
- `skills/tests/test_integration.py`

```python
def test_data_lake_to_skill_pipeline():
    # 1. 写入测试数据到 Data Lake
    # 2. 调用 Skill 分析
    # 3. 验证结果
    
def test_offline_mode():
    # 模拟离线模式
    # 使用 mock 数据
```

### Task 18: 性能测试

**测试内容:**
- 并发调用多个 Skills
- 大数据量分析响应时间
- 内存使用监控

---

## 文件结构

```
skills/
├── analyze_status/           # ✅ 已实现
│   ├── SKILL.md
│   └── main.py
├── detect_anomaly/          # ✅ 已实现
│   ├── SKILL.md
│   └── main.py
├── predict_failure/         # ✅ 已实现
│   ├── SKILL.md
│   └── main.py
│
├── analyze_utilization/     # Task 1
│   ├── SKILL.md
│   └── main.py
├── analyze_uptime/         # Task 2
├── analyze_energy/         # Task 3
├── health_score/           # Task 4
├── analyze_robot_arm/      # Task 5
├── analyze_agv_path/       # Task 6
├── analyze_cleaning/       # Task 7
├── analyze_production/     # Task 8
├── generate_maintenance/   # Task 9
├── optimize_schedule/      # Task 10
├── root_cause/            # Task 11
├── predictive_maintenance/ # Task 12
│
├── skill_runner.py         # Task 13
├── tests/
│   ├── test_suite.py      # 现有 32 测试
│   ├── test_business_analysis.py  # Task 16
│   └── test_integration.py       # Task 17
│
└── scripts/
    └── batch_analyze.py   # Task 15
```

---

## 使用示例

### 单设备分析

```bash
# 分析设备利用率
python skills/analyze_utilization/main.py --device-id robot-arm-01

# 检测异常
python skills/detect_anomaly/main.py --device-id robot-arm-01 --sensitivity high

# 预测故障
python skills/predict_failure/main.py --device-id agv-01 --prediction-horizon 7200
```

### 批量分析

```bash
# 分析所有生产设备
python scripts/batch_analyze.py --skill analyze_utilization --device-group production

# 生成维护计划
python skills/generate_maintenance/main.py --device-group all
```

### 通过 CLI

```bash
# 通过 Rust CLI 调用
cargo run -- skill analyze robot-arm-01 utilization
cargo run -- skill analyze agv-01 path --output json
```

---

## 验证命令

```bash
# 1. 启动 Data Lake
cargo run -- data-lake --port 8080 &

# 2. 启动 Ollama (可选，使用 mock 模式)
# ollama serve

# 3. 运行测试
cd skills && pytest tests/ -v

# 4. 手动测试
python skills/analyze_utilization/main.py --device-id robot-arm-01 --mock
python skills/analyze_robot_arm/main.py --device-id robot-arm-01 --mock
```

---

## 完成 Checkbox

- [ ] Task 1: 设备利用率分析 Skill
- [ ] Task 2: 设备运行时间分析 Skill
- [ ] Task 3: 能耗分析 Skill
- [ ] Task 4: 设备健康评分 Skill
- [ ] Task 5: 机械臂操作分析 Skill
- [ ] Task 6: AGV 路径分析 Skill
- [ ] Task 7: 清洁效率分析 Skill
- [ ] Task 8: 生产节拍分析 Skill
- [ ] Task 9: 维护计划生成 Skill
- [ ] Task 10: 资源调度优化 Skill
- [ ] Task 11: 异常根因分析 Skill
- [ ] Task 12: 预测性维护 Skill
- [ ] Task 13: Skill 调用封装
- [ ] Task 14: CLI 命令集成
- [ ] Task 15: 批量分析脚本
- [ ] Task 16: 业务分析测试用例 (30+)
- [ ] Task 17: 集成测试
- [ ] Task 18: 性能测试
