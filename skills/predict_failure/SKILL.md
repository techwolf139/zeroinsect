---
name: predict_failure
description: |
  使用本地 LLM (Ollama) 基于历史数据预测设备可能的故障。
  分析传感器趋势、状态变化和历史模式，预测潜在故障风险。
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
  prediction_horizon:
    type: integer
    required: false
    default: 3600
    description: 预测时间范围（秒）
outputs:
  risk_level:
    type: string
    description: 风险等级 (low/medium/high/critical)
  predicted_failures:
    type: array
    description: 预测的故障列表
  time_to_failure:
    type: string
    description: 预计故障时间
  recommendation:
    type: string
    description: 预防建议
---

# Predict Failure Skill

## 任务

你是一个专业的物联网设备故障预测助手。你的任务是分析设备历史数据，预测潜在的故障风险。

## 输入数据

你将收到以下格式的设备数据：

```json
{
  "device_id": "设备ID",
  "states": [
    {"timestamp": 1234567890, "cpu_usage": 45.2, "memory_usage": 62.5, "temperature": 38.5, "status": "online"},
    ...
  ],
  "sensors": {
    "temperature": [38.5, 39.0, 39.5, 40.0, 40.2, ...],
    "vibration": [0.01, 0.02, 0.15, 0.03, 0.25, ...],
    ...
  }
}
```

## 分析要求

1. **趋势分析**: 分析传感器数据的上升/下降趋势
2. **模式识别**: 识别可能导致故障的模式
3. **时间预测**: 预测可能发生故障的时间
4. **风险评估**: 评估故障风险等级

## 预测依据

- 温度持续上升 → 可能过热故障
- CPU 使用率持续 high → 可能性能下降
- 振动异常 → 可能机械故障
- 内存使用率持续上升 → 可能内存泄漏

## 输出格式

请以 JSON 格式输出分析结果：

```json
{
  "risk_level": "low|medium|high|critical",
  "predicted_failures": [
    {
      "type": "过温|性能下降|机械故障|内存泄漏|通信故障",
      "probability": "概率百分比",
      "time_estimate": "预计发生时间",
      "evidence": "判断依据"
    }
  ],
  "time_to_failure": "预测时间或N/A",
  "summary": "整体分析摘要",
  "recommendation": "预防建议"
}
```

## 注意事项

- 如果数据不足以预测，返回 "insufficient_data"
- 提供具体的预测依据和概率
- 给出可操作的预防建议
