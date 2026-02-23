#!/usr/bin/env python3
import argparse
import json
import os
import sys
import time

import requests

DATA_LAKE_URL = os.environ.get("DATA_LAKE_URL", "http://localhost:8080")
OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434")
USE_MOCK = os.environ.get("MOCK_MODE", "false").lower() == "true"

MAINTENANCE_INTERVALS = {
    "robot-arm": {"minor": 500, "major": 2000, "overhaul": 5000},
    "agv": {"minor": 250, "major": 1000, "overhaul": 3000},
    "cleaner": {"minor": 100, "major": 500, "overhaul": 1500},
    "default": {"minor": 200, "major": 1000, "overhaul": 3000},
}


def get_device_type(device_id):
    if "robot" in device_id.lower():
        return "robot-arm"
    elif "agv" in device_id.lower():
        return "agv"
    elif "cleaner" in device_id.lower():
        return "cleaner"
    return "default"


def get_mock_history(device_id):
    return {
        "operating_hours": 450,
        "last_maintenance": 1700000000 - 30 * 24 * 3600,
        "maintenance_history": [
            {"type": "minor", "date": "2026-01-01", "parts": ["filter", "lubricant"]},
            {"type": "major", "date": "2025-10-15", "parts": ["motor", "bearing", "filter"]},
        ],
    }


def get_device_history(device_id):
    if USE_MOCK:
        return get_mock_history(device_id)
    url = f"{DATA_LAKE_URL}/api/device/{device_id}/maintenance/history"
    resp = requests.get(url)
    return resp.json() if resp.status_code == 200 else {}


def calculate_next_maintenance(device_type, operating_hours, health_score):
    intervals = MAINTENANCE_INTERVALS.get(device_type, MAINTENANCE_INTERVALS["default"])
    if health_score and health_score < 50:
        next_type = "minor"
        interval = intervals["minor"] * 0.5
    elif operating_hours > intervals["overhaul"] * 0.8:
        next_type = "overhaul"
        interval = intervals["overhaul"]
    elif operating_hours > intervals["major"] * 0.8:
        next_type = "major"
        interval = intervals["major"]
    else:
        next_type = "minor"
        interval = intervals["minor"]
    hours_until = max(0, interval - operating_hours)
    next_date = int(time.time()) + hours_until * 3600
    return {
        "type": next_type,
        "hours_until": hours_until,
        "scheduled_date": time.strftime("%Y-%m-%d", time.localtime(next_date)),
    }


def get_maintenance_items(device_type, maintenance_type):
    items_db = {
        "robot-arm": {
            "minor": [{"item": "Visual inspection", "duration_min": 15}, {"item": "Lubrication check", "duration_min": 20}, {"item": "Filter replacement", "duration_min": 10}],
            "major": [{"item": "All minor items", "duration_min": 45}, {"item": "Motor inspection", "duration_min": 60}, {"item": "Bearing replacement", "duration_min": 45}],
            "overhaul": [{"item": "Full disassembly", "duration_min": 180}, {"item": "All parts replacement", "duration_min": 120}],
        },
        "agv": {
            "minor": [{"item": "Battery check", "duration_min": 10}, {"item": "Wheel inspection", "duration_min": 15}, {"item": "Sensor clean", "duration_min": 10}],
            "major": [{"item": "Motor check", "duration_min": 45}, {"item": "Navigation calibration", "duration_min": 30}, {"item": "Battery replacement", "duration_min": 30}],
            "overhaul": [{"item": "Full system check", "duration_min": 120}, {"item": "Component replacement", "duration_min": 90}],
        },
    }
    return items_db.get(device_type, {}).get(maintenance_type, [])


def estimate_downtime(maintenance_items):
    total_minutes = sum(item.get("duration_min", 30) for item in maintenance_items)
    return {"minutes": total_minutes, "hours": round(total_minutes / 60, 2)}


def main():
    parser = argparse.ArgumentParser(description="Generate Maintenance Plan")
    parser.add_argument("--device-id", required=True)
    parser.add_argument("--health-score", type=int, help="Current health score (0-100)")
    parser.add_argument("--operating-hours", type=int, help="Total operating hours")
    parser.add_argument("--mock", action="store_true")
    args = parser.parse_args()
    if args.mock:
        global USE_MOCK
        USE_MOCK = True
    device_type = get_device_type(args.device_id)
    history = get_device_history(args.device_id)
    operating_hours = args.operating_hours or history.get("operating_hours", 0)
    health_score = args.health_score or 80
    next_maint = calculate_next_maintenance(device_type, operating_hours, health_score)
    maint_items = get_maintenance_items(device_type, next_maint["type"])
    parts_needed = list(set(item.get("parts", []) if isinstance(item, dict) else [] for item in history.get("maintenance_history", [])))
    parts_needed = [p for sublist in parts_needed for p in sublist] if parts_needed else ["Standard consumables"]
    downtime = estimate_downtime(maint_items)
    result = {
        "device_id": args.device_id,
        "device_type": device_type,
        "operating_hours": operating_hours,
        "next_maintenance": next_maint,
        "maintenance_items": maint_items,
        "parts_needed": parts_needed,
        "estimated_downtime": downtime,
    }
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
