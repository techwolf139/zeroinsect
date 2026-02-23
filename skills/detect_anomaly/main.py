#!/usr/bin/env python3
"""
Detect Anomaly Skill

使用本地 LLM (Ollama) 检测设备数据中的异常模式。

用法:
    python main.py --device-id device001 --time-window 3600
    python main.py --device-id device001 --sensitivity high
    python main.py --device-id device001 --mock  # 使用模拟数据测试
"""

import argparse
import json
import os
import sys
import time
from datetime import datetime
from typing import Any, Optional

import requests


OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434")
OLLAMA_MODEL = os.environ.get("OLLAMA_MODEL", "llama3.2")
DATA_LAKE_URL = os.environ.get("DATA_LAKE_URL", "http://localhost:8080")


def call_ollama(prompt: str) -> str:
    """调用 Ollama LLM 进行分析"""
    url = f"{OLLAMA_URL}/api/generate"
    payload = {
        "model": OLLAMA_MODEL,
        "prompt": prompt,
        "stream": False,
        "format": "json",
    }

    try:
        response = requests.post(url, json=payload, timeout=60)
        response.raise_for_status()
        return response.json().get("response", "")
    except requests.exceptions.RequestException as e:
        raise RuntimeError(f"Failed to call Ollama: {e}")


def parse_args() -> argparse.Namespace:
    """解析命令行参数"""
    parser = argparse.ArgumentParser(
        description="Detect anomalies in device data using LLM"
    )
    parser.add_argument(
        "--device-id",
        type=str,
        required=True,
        help="Device ID to analyze",
    )
    parser.add_argument(
        "--time-window",
        type=int,
        default=3600,
        help="Time window in seconds (default: 3600)",
    )
    parser.add_argument(
        "--sensitivity",
        type=str,
        default="medium",
        choices=["low", "medium", "high"],
        help="Anomaly detection sensitivity (default: medium)",
    )
    parser.add_argument(
        "--data-lake-url",
        type=str,
        default=DATA_LAKE_URL,
        help="Data Lake API URL",
    )
    parser.add_argument(
        "--mock",
        action="store_true",
        help="Use mock data for testing",
    )
    return parser.parse_args()


def get_device_data(device_id: str, time_window: int, base_url: str) -> dict:
    """从 Data Lake 获取设备数据"""
    # 计算时间范围
    end_ts = int(time.time())
    start_ts = end_ts - time_window

    # 获取设备状态
    state_url = f"{base_url}/api/device/{device_id}/state"
    try:
        state_response = requests.get(state_url, timeout=10)
        state_data = state_response.json() if state_response.status_code == 200 else None
    except requests.exceptions.RequestException:
        state_data = None

    # 获取历史状态范围
    range_url = f"{base_url}/api/device/{device_id}/state/range"
    try:
        range_response = requests.get(
            range_url,
            params={"start_ts": start_ts, "end_ts": end_ts},
            timeout=10,
        )
        states = range_response.json() if range_response.status_code == 200 else []
    except requests.exceptions.RequestException:
        states = []

    # 获取传感器数据
    sensors = {}
    sensor_types = ["temperature", "vibration", "cpu", "imu", "humidity", "pressure"]

    for sensor_type in sensor_types:
        sensor_url = f"{base_url}/api/device/{device_id}/sensor/{sensor_type}"
        try:
            sensor_response = requests.get(
                sensor_url,
                params={"start_ts": start_ts, "end_ts": end_ts},
                timeout=10,
            )
            if sensor_response.status_code == 200:
                data = sensor_response.json()
                if data:
                    # 收集所有传感器值
                    values = []
                    for item in data:
                        if "values" in item:
                            values.extend(item["values"])
                    if values:
                        sensors[sensor_type] = values
        except requests.exceptions.RequestException:
            continue

    return {
        "device_id": device_id,
        "latest_state": state_data,
        "states": states,
        "sensors": sensors,
        "time_window": time_window,
    }


def get_mock_data() -> dict:
    """获取模拟数据用于测试"""
    return {
        "device_id": "mock-device-01",
        "latest_state": {
            "device_id": "mock-device-01",
            "timestamp": int(time.time()),
            "status": "online",
            "cpu_usage": 95.5,
            "memory_usage": 88.0,
            "temperature": 85.0,
            "last_command": "process_data",
        },
        "states": [
            {"timestamp": int(time.time()) - 300, "cpu_usage": 45.2, "memory_usage": 62.5, "temperature": 38.5, "status": "online"},
            {"timestamp": int(time.time()) - 240, "cpu_usage": 55.0, "memory_usage": 65.0, "temperature": 42.0, "status": "online"},
            {"timestamp": int(time.time()) - 180, "cpu_usage": 70.0, "memory_usage": 72.0, "temperature": 55.0, "status": "online"},
            {"timestamp": int(time.time()) - 120, "cpu_usage": 85.0, "memory_usage": 80.0, "temperature": 70.0, "status": "online"},
            {"timestamp": int(time.time()) - 60, "cpu_usage": 95.5, "memory_usage": 88.0, "temperature": 85.0, "status": "online"},
        ],
        "sensors": {
            "temperature": [38.5, 42.0, 55.0, 70.0, 85.0],
            "vibration": [0.01, 0.02, 0.05, 0.15, 0.45],
            "cpu": [45.2, 55.0, 70.0, 85.0, 95.5],
        },
        "time_window": 3600,
    }


def build_prompt(device_data: dict, sensitivity: str) -> str:
    """构建分析 prompt"""
    prompt = f"""你是一个专业的物联网设备异常检测助手。请分析以下设备数据，检测异常模式。

## 设备数据
```json
{json.dumps(device_data, indent=2, ensure_ascii=False)}
```

## 异常检测敏感度
{sensitivity} (low=宽松检测, medium=标准检测, high=严格检测)

## 分析要求
1. 时序异常: 检测传感器数据中的突变、趋势偏离
2. 阈值异常: 检测是否超过安全阈值
3. 模式异常: 检测不符合正常运行模式的异常行为

## 温度阈值参考
- 临界: 80°C
- 警告: 60°C
- 正常: < 45°C

## CPU 使用率阈值
- 临界: 90%
- 警告: 75%
- 正常: < 60%

## 振动阈值
- 临界: 0.5
- 警告: 0.3
- 正常: < 0.1

请以 JSON 格式输出分析结果：
```json
{{
  "anomalies": [
    {{
      "type": "threshold|trend|pattern",
      "sensor": "传感器类型或general",
      "description": "异常描述",
      "value": "异常值",
      "threshold": "阈值",
      "severity": "low|medium|high"
    }}
  ],
  "severity": "normal|warning|critical",
  "summary": "整体分析摘要",
  "recommendation": "建议操作"
}}
```

只输出 JSON，不要其他内容。
"""
    return prompt


def parse_ollama_response(response: str) -> dict:
    """解析 Ollama 返回的 JSON 响应"""
    try:
        # 尝试直接解析
        return json.loads(response)
    except json.JSONDecodeError:
        # 尝试提取 JSON 块
        import re
        json_match = re.search(r'\{[^{}]*\}', response, re.DOTALL)
        if json_match:
            try:
                return json.loads(json_match.group())
            except json.JSONDecodeError:
                pass
        
        # 返回错误响应
        return {
            "anomalies": [],
            "severity": "unknown",
            "summary": f"Failed to parse LLM response: {response[:200]}",
            "recommendation": "Please check LLM service",
        }


def main():
    args = parse_args()

    print(f"🔍 Detecting anomalies for device: {args.device_id}")
    print(f"   Time window: {args.time_window}s")
    print(f"   Sensitivity: {args.sensitivity}")
    print()

    # 获取设备数据
    if args.mock:
        print("📦 Using mock data...")
        device_data = get_mock_data()
    else:
        print("📡 Fetching data from Data Lake...")
        device_data = get_device_data(args.device_id, args.time_window, args.data_lake_url)

    if not device_data.get("latest_state") and not device_data.get("sensors"):
        print("❌ No data available for device")
        sys.exit(1)

    print(f"   Latest state: {device_data.get('latest_state', {}).get('status', 'N/A')}")
    print(f"   Sensors: {list(device_data.get('sensors', {}).keys())}")
    print()

    # 构建 prompt
    prompt = build_prompt(device_data, args.sensitivity)

    # 调用 LLM
    print("🤖 Analyzing with LLM...")
    try:
        response = call_ollama(prompt)
        result = parse_ollama_response(response)
    except RuntimeError as e:
        print(f"❌ Error: {e}")
        sys.exit(1)

    # 输出结果
    print("=" * 50)
    print("📊 ANOMALY DETECTION RESULTS")
    print("=" * 50)

    severity = result.get("severity", "unknown")
    severity_emoji = {
        "normal": "✅",
        "warning": "⚠️",
        "critical": "🚨",
    }.get(severity, "❓")

    print(f"\n{severity_emoji} Overall Severity: {severity.upper()}")

    anomalies = result.get("anomalies", [])
    if anomalies:
        print(f"\n🔴 Detected {len(anomalies)} anomalies:")
        for i, anomaly in enumerate(anomalies, 1):
            print(f"   {i}. [{anomaly.get('severity', '?').upper()}] {anomaly.get('description', '')}")
            print(f"      Type: {anomaly.get('type', 'unknown')}, Sensor: {anomaly.get('sensor', 'N/A')}")
            if anomaly.get('value') and anomaly.get('threshold'):
                print(f"      Value: {anomaly.get('value')} (threshold: {anomaly.get('threshold')})")
    else:
        print("\n✅ No anomalies detected")

    print(f"\n📝 Summary: {result.get('summary', 'N/A')}")
    print(f"\n💡 Recommendation: {result.get('recommendation', 'N/A')}")

    # 保存结果到文件
    output_file = f"anomaly_result_{args.device_id}_{int(time.time())}.json"
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2, ensure_ascii=False)
    print(f"\n💾 Results saved to: {output_file}")


if __name__ == "__main__":
    main()
