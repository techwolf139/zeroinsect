#!/usr/bin/env python3
import argparse
import json
import os
import sys

OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434")
USE_MOCK = os.environ.get("MOCK_MODE", "false").lower() == "true"


def get_mock_data():
    return {
        "tasks": [
            {"id": "t1", "name": "Pick and Place", "duration": 120, "device": "robot-arm-01", "priority": 1},
            {"id": "t2", "name": "Welding", "duration": 180, "device": "robot-arm-02", "priority": 2},
            {"id": "t3", "name": "Inspection", "duration": 60, "device": "agv-01", "priority": 3},
            {"id": "t4", "name": "Material Transport", "duration": 90, "device": "agv-02", "priority": 1},
        ],
        "devices": [
            {"id": "robot-arm-01", "available": True, "slots": 1},
            {"id": "robot-arm-02", "available": True, "slots": 1},
            {"id": "agv-01", "available": True, "slots": 2},
            {"id": "agv-02", "available": False, "slots": 1},
        ],
    }


def optimize_schedule(tasks, devices):
    available_devices = {d["id"]: d for d in devices if d.get("available", True)}
    device_usage = {d["id"]: 0 for d in available_devices.values()}
    sorted_tasks = sorted(tasks, key=lambda t: t.get("priority", 5))
    schedule = []
    current_time = 0
    for task in sorted_tasks:
        device = task.get("device")
        if device not in available_devices:
            continue
        start_time = max(current_time, device_usage[device])
        end_time = start_time + task.get("duration", 60)
        schedule.append({
            "task_id": task.get("id"),
            "task_name": task.get("name"),
            "device": device,
            "start_time": start_time,
            "end_time": end_time,
        })
        device_usage[device] = end_time
        current_time = max(current_time, end_time)
    total_duration = max(device_usage.values()) if device_usage else 0
    return schedule, total_duration, device_usage


def main():
    parser = argparse.ArgumentParser(description="Optimize Schedule")
    parser.add_argument("--tasks", type=str, help="JSON array of tasks")
    parser.add_argument("--mock", action="store_true")
    args = parser.parse_args()
    if args.mock or not args.tasks:
        data = get_mock_data()
        tasks = data["tasks"]
        devices = data["devices"]
    else:
        tasks = json.loads(args.tasks)
        devices = []
    schedule, total_duration, usage = optimize_schedule(tasks, devices)
    result = {
        "schedule": schedule,
        "estimated_completion_seconds": total_duration,
        "estimated_completion_minutes": round(total_duration / 60, 2),
        "resource_utilization": usage,
    }
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
