---
name: analyze_status
description: |
  使用本地 LLM (Ollama) 分析设备状态，包括实时指标、时序趋势和异常检测。
  输入设备 ID 和时间窗口，返回综合分析报告。
version: 1.0.0
metadata:
  trigger: 分析设备状态
  author: zeroinsect
  category: cognition
inputs:
  device_id:
    type: string
    required: true
    description: 设备 ID
  time_window:
    type: integer
    required: false
    default: 86400
    description: 分析时间窗口（秒），默认 24 小时
outputs:
  status:
    type: string
    description: 分析状态 (success/error)
  device_id:
    type: string
    description: 设备 ID
  timestamp:
    type: integer
    description: 分析时间戳
  realtime:
    type: object
    description: 实时状态分析
  trends:
    type: object
    description: 时序趋势分析
  anomalies:
    type: array
    description: 异常检测结果
  summary:
    type: string
    description: LLM 生成的分析总结
---

# Analyze Status Skill

你是一个设备状态分析专家。当收到设备数据时，你需要：

## 输入数据

- `device_id`: 设备唯一标识
- `time_window`: 分析的时间窗口（秒）
- `realtime_data`: 设备的实时状态（CPU、内存、温度、状态）
- `historical_data`: 时间窗口内的历史数据点

## 分析任务

### 1. 实时状态分析

评估当前设备状态：
- 运行状态（online/offline/error/idle）
- CPU 使用率是否正常（>80% 为高负载）
- 内存使用率是否正常（>80% 为高负载）
- 温度是否正常（>70°C 为过热）

### 2. 时序趋势分析

分析历史数据的变化趋势：
- 指标是上升、下降还是平稳
- 是否有周期性模式
- 预测下一步可能的走势

### 3. 异常检测

识别异常模式：
- 突然的数值跳变
- 持续的高负载状态
- 离线后重连
- 错误状态

## 输出格式

请返回 JSON 格式的分析结果：

```json
{
  "status": "success",
  "device_id": "设备ID",
  "timestamp": 1234567890,
  "realtime": {
    "status": "online",
    "cpu_usage": 45.2,
    "memory_usage": 62.8,
    "temperature": 48.5,
    "health": "good"
  },
  "trends": {
    "cpu": "stable",
    "memory": "increasing",
    "temperature": "decreasing"
  },
  "anomalies": [],
  "summary": "设备运行状态良好，各项指标正常..."
}
```

## 注意事项

- 如果某个指标缺失，使用 "unknown"
- 如果无法确定趋势，使用 "unknown"
- 异常为空数组表示未检测到异常
- summary 应该简洁明了，不超过 200 字
