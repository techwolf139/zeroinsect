#!/usr/bin/env python3
"""
Predict Failure Skill

使用本地 LLM (Ollama) 基于历史数据预测设备可能的故障。

用法:
    python main.py --device-id device001 --time-window 86400
    python main.py --device-id device001 --prediction-horizon 7200
    python main.py --device-id device001 --mock  # 使用模拟数据测试
"""

import argparse
import json
import os
import sys
import time
from typing import Any

import requests


OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434")
OLLAMA_MODEL = os.environ.get("OLLAMA_MODEL", "llama3.2")
DATA_LAKE_URL = os.environ.get("DATA_LAKE_URL", "http://localhost:8080")


def call_ollama(prompt: str) -> str:
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
    parser = argparse.ArgumentParser(
        description="Predict device failures using LLM"
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
        default=86400,
        help="Analysis time window in seconds (default: 86400 = 24 hours)",
    )
    parser.add_argument(
        "--prediction-horizon",
        type=int,
        default=3600,
        help="Prediction horizon in seconds (default: 3600 = 1 hour)",
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
    end_ts = int(time.time())
    start_ts = end_ts - time_window

    state_url = f"{base_url}/api/device/{device_id}/state"
    try:
        state_response = requests.get(state_url, timeout=10)
        state_data = state_response.json() if state_response.status_code == 200 else None
    except requests.exceptions.RequestException:
        state_data = None

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
            {"timestamp": int(time.time()) - 7200, "cpu_usage": 45.2, "memory_usage": 62.5, "temperature": 38.5, "status": "online"},
            {"timestamp": int(time.time()) - 5400, "cpu_usage": 50.0, "memory_usage": 65.0, "temperature": 40.0, "status": "online"},
            {"timestamp": int(time.time()) - 3600, "cpu_usage": 60.0, "memory_usage": 68.0, "temperature": 45.0, "status": "online"},
            {"timestamp": int(time.time()) - 1800, "cpu_usage": 75.0, "memory_usage": 75.0, "temperature": 55.0, "status": "online"},
            {"timestamp": int(time.time()) - 900, "cpu_usage": 85.0, "memory_usage": 82.0, "temperature": 70.0, "status": "online"},
            {"timestamp": int(time.time()) - 300, "cpu_usage": 92.0, "memory_usage": 86.0, "temperature": 80.0, "status": "online"},
            {"timestamp": int(time.time()) - 60, "cpu_usage": 95.5, "memory_usage": 88.0, "temperature": 85.0, "status": "online"},
        ],
        "sensors": {
            "temperature": [38.5, 40.0, 45.0, 55.0, 70.0, 80.0, 85.0],
            "cpu": [45.2, 50.0, 60.0, 75.0, 85.0, 92.0, 95.5],
            "memory": [62.5, 65.0, 68.0, 75.0, 82.0, 86.0, 88.0],
        },
        "time_window": 86400,
    }


def build_prompt(device_data: dict, prediction_horizon: int) -> str:
    prompt = f"""你是一个专业的物联网设备故障预测助手。请分析以下设备历史数据，预测潜在的故障风险。

## 设备数据
```json
{json.dumps(device_data, indent=2, ensure_ascii=False)}
```

## 预测时间范围
预测未来 {prediction_horizon} 秒 ({prediction_horizon/3600:.1f} 小时) 内的故障风险

## 趋势分析要点
1. 温度上升趋势 → 过热风险
2. CPU 使用率持续 high → 性能下降风险
3. 内存使用率上升 → 内存泄漏风险
4. 振动加剧 → 机械故障风险

## 风险等级定义
- critical (临界): 24 小时内极可能故障
- high (高): 72 小时内可能故障
- medium (中): 1 周内可能故障
- low (低): 暂无明显风险

请以 JSON 格式输出分析结果：
```json
{{
  "risk_level": "low|medium|high|critical",
  "predicted_failures": [
    {{
      "type": "过温|性能下降|机械故障|内存泄漏|通信故障",
      "probability": "概率百分比",
      "time_estimate": "预计发生时间",
      "evidence": "判断依据"
    }}
  ],
  "time_to_failure": "预测时间或N/A",
  "summary": "整体分析摘要",
  "recommendation": "预防建议"
}}
```

只输出 JSON，不要其他内容。
"""
    return prompt


def parse_ollama_response(response: str) -> dict:
    try:
        return json.loads(response)
    except json.JSONDecodeError:
        import re
        json_match = re.search(r'\{[^{}]*\}', response, re.DOTALL)
        if json_match:
            try:
                return json.loads(json_match.group())
            except json.JSONDecodeError:
                pass
        
        return {
            "risk_level": "unknown",
            "predicted_failures": [],
            "time_to_failure": "N/A",
            "summary": f"Failed to parse LLM response: {response[:200]}",
            "recommendation": "Please check LLM service",
        }


def main():
    args = parse_args()

    print(f"🔮 Predicting failures for device: {args.device_id}")
    print(f"   Time window: {args.time_window}s ({args.time_window/86400:.1f} days)")
    print(f"   Prediction horizon: {args.prediction_horizon}s ({args.prediction_horizon/3600:.1f} hours)")
    print()

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

    prompt = build_prompt(device_data, args.prediction_horizon)

    print("🤖 Analyzing with LLM...")
    try:
        response = call_ollama(prompt)
        result = parse_ollama_response(response)
    except RuntimeError as e:
        print(f"❌ Error: {e}")
        sys.exit(1)

    print("=" * 50)
    print("🔮 FAILURE PREDICTION RESULTS")
    print("=" * 50)

    risk_level = result.get("risk_level", "unknown")
    risk_emoji = {
        "low": "✅",
        "medium": "⚠️",
        "high": "🚨",
        "critical": "🛑",
    }.get(risk_level, "❓")

    print(f"\n{risk_emoji} Risk Level: {risk_level.upper()}")

    failures = result.get("predicted_failures", [])
    if failures:
        print(f"\n🔴 Predicted {len(failures)} potential failures:")
        for i, failure in enumerate(failures, 1):
            print(f"   {i}. [{failure.get('type', 'unknown')}]")
            print(f"      Probability: {failure.get('probability', 'N/A')}")
            print(f"      Time: {failure.get('time_estimate', 'N/A')}")
            print(f"      Evidence: {failure.get('evidence', 'N/A')}")
    else:
        print("\n✅ No significant failures predicted")

    print(f"\n📝 Summary: {result.get('summary', 'N/A')}")
    print(f"\n💡 Recommendation: {result.get('recommendation', 'N/A')}")

    output_file = f"prediction_result_{args.device_id}_{int(time.time())}.json"
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2, ensure_ascii=False)
    print(f"\n💾 Results saved to: {output_file}")


if __name__ == "__main__":
    main()
