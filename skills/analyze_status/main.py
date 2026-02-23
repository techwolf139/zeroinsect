#!/usr/bin/env python3
"""
Analyze Status Skill

使用本地 LLM (Ollama) 分析设备状态，包括实时指标、时序趋势和异常检测。

用法:
    python main.py --device-id device001 --time-window 3600
    python main.py --device-id device001 --data-lake-url http://localhost:8080
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
        description="Analyze device status using LLM"
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
        help="Time window in seconds (default: 86400 = 24 hours)",
    )
    parser.add_argument(
        "--data-lake-url",
        type=str,
        default=DATA_LAKE_URL,
        help="Data Lake API URL",
    )
    parser.add_argument(
        "--ollama-url",
        type=str,
        default=OLLAMA_URL,
        help="Ollama API URL",
    )
    parser.add_argument(
        "--ollama-model",
        type=str,
        default=OLLAMA_MODEL,
        help="Ollama model name",
    )
    parser.add_argument(
        "--mock",
        action="store_true",
        help="Use mock data instead of real Data Lake",
    )
    return parser.parse_args()


def get_device_state_api(device_id: str, base_url: str) -> dict:
    """从 Data Lake API 获取设备实时状态"""
    url = f"{base_url}/api/device/{device_id}/state"
    try:
        response = requests.get(url, timeout=5)
        if response.status_code == 404:
            return None
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        raise RuntimeError(f"Failed to get device state: {e}")


def get_historical_data_api(device_id: str, time_window: int, base_url: str) -> list:
    """从 Data Lake API 获取历史数据"""
    now = int(time.time())
    start_ts = now - time_window
    url = f"{base_url}/api/device/{device_id}/state/range"
    params = {
        "start_ts": start_ts,
        "end_ts": now,
    }
    try:
        response = requests.get(url, params=params, timeout=5)
        if response.status_code == 404:
            return []
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        raise RuntimeError(f"Failed to get historical data: {e}")


def get_device_state_mock(device_id: str) -> dict:
    """获取设备实时状态（模拟数据）"""
    import random
    
    states = ["online", "online", "online", "idle", "offline", "error"]
    return {
        "device_id": device_id,
        "timestamp": int(time.time()),
        "status": random.choice(states),
        "cpu_usage": round(random.uniform(10, 90), 1),
        "memory_usage": round(random.uniform(20, 80), 1),
        "temperature": round(random.uniform(30, 65), 1),
        "last_command": "status_check",
    }


def get_historical_data_mock(device_id: str, time_window: int) -> list:
    """获取历史数据（模拟数据）"""
    import random
    
    now = int(time.time())
    data = []
    num_points = min(time_window // 3600, 24)
    
    for i in range(num_points):
        data.append({
            "timestamp": now - (num_points - i) * 3600,
            "cpu_usage": round(random.uniform(10, 90), 1),
            "memory_usage": round(random.uniform(20, 80), 1),
            "temperature": round(random.uniform(30, 65), 1),
            "status": random.choice(["online", "online", "idle"]),
        })
    
    return data


def analyze_with_llm(
    device_id: str,
    realtime_data: dict,
    historical_data: list,
    skill_content: str,
) -> dict:
    """使用 LLM 分析设备状态"""
    
    # 构建 prompt
    prompt = f"""你是一个设备状态分析专家。请分析以下设备数据并返回 JSON 格式的分析结果。

## 设备实时状态
{json.dumps(realtime_data, indent=2, ensure_ascii=False)}

## 历史数据（{len(historical_data)} 个数据点）
{json.dumps(historical_data[:10], indent=2, ensure_ascii=False)}

## 分析要求

1. 实时状态分析：评估当前 CPU、内存、温度状态
2. 时序趋势分析：分析历史数据的变化趋势
3. 异常检测：识别异常模式
4. 生成总结（不超过 200 字）

## 输出格式

请返回以下 JSON 结构（必须严格 JSON 格式，不要有其他内容）：

```json
{{
  "status": "success",
  "device_id": "{device_id}",
  "timestamp": {int(time.time())},
  "realtime": {{
    "status": "设备状态",
    "cpu_usage": 数值,
    "memory_usage": 数值,
    "temperature": 数值,
    "health": "good/warning/critical"
  }},
  "trends": {{
    "cpu": "stable/increasing/decreasing/unknown",
    "memory": "stable/increasing/decreasing/unknown",
    "temperature": "stable/increasing/decreasing/unknown"
  }},
  "anomaly": [],
  "summary": "分析总结"
}}
```

请直接返回 JSON，不要有其他内容："""

    # 调用 LLM
    result = None
    try:
        result = call_ollama(prompt)
        
        # 尝试解析 JSON
        # 去除可能的 markdown 代码块标记
        result = result.strip()
        if result.startswith("```"):
            lines = result.split("\n")
            result = "\n".join(lines[1:-1] if lines[-1].startswith("```") else lines[1:])
        
        return json.loads(result)
        
    except json.JSONDecodeError as e:
        # 如果解析失败，返回错误信息
        return {
            "status": "error",
            "device_id": device_id,
            "timestamp": int(time.time()),
            "error": f"Failed to parse LLM response: {e}",
            "raw_response": result[:500],
        }
    except Exception as e:
        return {
            "status": "error",
            "device_id": device_id,
            "timestamp": int(time.time()),
            "error": str(e),
        }


def main():
    """主函数"""
    args = parse_args()
    
    # 设置全局变量
    global OLLAMA_URL, OLLAMA_MODEL, DATA_LAKE_URL
    OLLAMA_URL = args.ollama_url
    OLLAMA_MODEL = args.ollama_model
    DATA_LAKE_URL = args.data_lake_url
    
    print(f"Analyzing device: {args.device_id}")
    print(f"Time window: {args.time_window} seconds")
    print(f"Ollama: {OLLAMA_URL}/{OLLAMA_MODEL}")
    print(f"Data Lake: {DATA_LAKE_URL}")
    print("-" * 40)
    
    # 获取设备数据
    print("Fetching device data...")
    if args.mock:
        realtime_data = get_device_state_mock(args.device_id)
        historical_data = get_historical_data_mock(args.device_id, args.time_window)
    else:
        realtime_data = get_device_state_api(args.device_id, DATA_LAKE_URL)
        if realtime_data is None:
            print("Device not found, using mock data...")
            realtime_data = get_device_state_mock(args.device_id)
        historical_data = get_historical_data_api(args.device_id, args.time_window, DATA_LAKE_URL)
        if not historical_data:
            print("No historical data found, using mock data...")
            historical_data = get_historical_data_mock(args.device_id, args.time_window)
    
    print(f"Realtime status: {realtime_data['status']}")
    print(f"CPU: {realtime_data['cpu_usage']}%, Memory: {realtime_data['memory_usage']}%, Temp: {realtime_data['temperature']}°C")
    print(f"Historical data points: {len(historical_data)}")
    print("-" * 40)
    
    # 读取 Skill 定义
    skill_path = os.path.join(os.path.dirname(__file__), "SKILL.md")
    skill_content = ""
    if os.path.exists(skill_path):
        with open(skill_path, "r", encoding="utf-8") as f:
            skill_content = f.read()
    
    # 使用 LLM 分析
    print("Analyzing with LLM...")
    result = analyze_with_llm(
        args.device_id,
        realtime_data,
        historical_data,
        skill_content,
    )
    
    # 输出结果
    print("-" * 40)
    print("Analysis Result:")
    print(json.dumps(result, indent=2, ensure_ascii=False))
    
    # 返回状态码
    if result.get("status") == "error":
        sys.exit(1)
    sys.exit(0)


if __name__ == "__main__":
    main()
