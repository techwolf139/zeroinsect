#!/usr/bin/env python3
"""
AGV Path Analysis Skill

Analyzes AGV path coverage, efficiency, collision risk, and battery optimization.
"""

import argparse
import json
import os
import sys
import time
from dataclasses import dataclass
from typing import Optional

import requests

DATA_LAKE_URL = os.environ.get("DATA_LAKE_URL", "http://localhost:8080")
OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434")
USE_MOCK = os.environ.get("MOCK_MODE", "false").lower() == "true"


@dataclass
class Position:
    x: float
    y: float
    timestamp: int


def get_mock_data() -> dict:
    """Generate mock AGV data for testing."""
    return {
        "positions": [
            {"x": 0.0, "y": 0.0, "ts": 1700000000},
            {"x": 1.0, "y": 0.5, "ts": 1700000010},
            {"x": 2.0, "y": 1.0, "ts": 1700000020},
            {"x": 3.0, "y": 1.0, "ts": 1700000030},
            {"x": 4.0, "y": 1.5, "ts": 1700000040},
            {"x": 5.0, "y": 2.0, "ts": 1700000050},
            {"x": 5.0, "y": 3.0, "ts": 1700000060},
            {"x": 5.0, "y": 4.0, "ts": 1700000070},
            {"x": 4.5, "y": 4.5, "ts": 1700000080},
            {"x": 4.0, "y": 5.0, "ts": 1700000090},
        ],
        "velocities": [
            {"linear": 0.5, "angular": 0.0, "ts": 1700000000},
            {"linear": 0.6, "angular": 0.1, "ts": 1700000010},
            {"linear": 0.5, "angular": 0.0, "ts": 1700000020},
            {"linear": 0.4, "angular": 0.2, "ts": 1700000030},
            {"linear": 0.5, "angular": 0.1, "ts": 1700000040},
        ],
        "battery": [
            {"level": 100, "ts": 1700000000},
            {"level": 98, "ts": 1700000100},
            {"level": 95, "ts": 1700000200},
            {"level": 90, "ts": 1700000300},
        ],
        "events": [
            {"type": "stop", "reason": "obstacle", "ts": 1700000045},
            {"type": "near_miss", "ts": 1700000065},
        ],
    }


def get_agv_data(device_id: str, time_window: int) -> dict:
    """Fetch AGV data from Data Lake."""
    if USE_MOCK:
        return get_mock_data()

    now = int(time.time())
    start_ts = now - time_window

    # Fetch position data
    pos_url = f"{DATA_LAKE_URL}/api/device/{device_id}/sensor/position"
    pos_resp = requests.get(pos_url, params={"start_ts": start_ts, "end_ts": now})
    positions = pos_resp.json() if pos_resp.status_code == 200 else []

    # Fetch velocity data
    vel_url = f"{DATA_LAKE_URL}/api/device/{device_id}/sensor/velocity"
    vel_resp = requests.get(vel_url, params={"start_ts": start_ts, "end_ts": now})
    velocities = vel_resp.json() if vel_resp.status_code == 200 else []

    # Fetch battery data
    bat_url = f"{DATA_LAKE_URL}/api/device/{device_id}/sensor/battery"
    bat_resp = requests.get(bat_url, params={"start_ts": start_ts, "end_ts": now})
    battery = bat_resp.json() if bat_resp.status_code == 200 else []

    # Fetch events
    evt_url = f"{DATA_LAKE_URL}/api/device/{device_id}/events"
    evt_resp = requests.get(evt_url, params={"start_ts": start_ts, "end_ts": now})
    events = evt_resp.json() if evt_resp.status_code == 200 else []

    return {
        "positions": positions,
        "velocities": velocities,
        "battery": battery,
        "events": events,
    }


def calculate_coverage(positions: list, map_bounds: dict = None) -> dict:
    """Calculate path coverage metrics."""
    if not positions:
        return {"coverage_percent": 0, "unvisited_zones": [], "redundant_paths": 0}

    # Simple coverage calculation based on bounding box
    xs = [p.get("x", 0) for p in positions]
    ys = [p.get("y", 0) for p in positions]

    min_x, max_x = min(xs), max(xs)
    min_y, max_y = min(ys), max(ys)

    # Estimate coverage area
    area_covered = (max_x - min_x) * (max_y - min_y)
    map_area = 100.0  # Default 10x10 map
    if map_bounds:
        map_area = map_bounds.get("width", 10) * map_bounds.get("height", 10)

    coverage_percent = min(100, (area_covered / map_area) * 100)

    # Calculate path length
    path_length = 0
    for i in range(1, len(positions)):
        dx = positions[i].get("x", 0) - positions[i-1].get("x", 0)
        dy = positions[i].get("y", 0) - positions[i-1].get("y", 0)
        path_length += (dx**2 + dy**2) ** 0.5

    return {
        "coverage_percent": round(coverage_percent, 2),
        "path_length_meters": round(path_length, 2),
        "bounding_box": {
            "min_x": min_x,
            "max_x": max_x,
            "min_y": min_y,
            "max_y": max_y,
        },
    }


def calculate_efficiency(positions: list, velocities: list) -> dict:
    """Calculate movement efficiency metrics."""
    if not velocities:
        return {"avg_velocity": 0, "idle_time_percent": 100, "turn_count": 0}

    # Calculate average velocity
    total_linear = sum(v.get("linear", 0) for v in velocities)
    avg_velocity = total_linear / len(velocities) if velocities else 0

    # Count stops (velocity near 0)
    stops = sum(1 for v in velocities if v.get("linear", 0) < 0.05)
    idle_percent = (stops / len(velocities) * 100) if velocities else 100

    # Count direction changes (turns)
    turns = 0
    for i in range(1, len(velocities)):
        curr = velocities[i].get("angular", 0)
        prev = velocities[i-1].get("angular", 0)
        if abs(curr - prev) > 0.2:
            turns += 1

    return {
        "avg_velocity_mps": round(avg_velocity, 2),
        "idle_time_percent": round(idle_percent, 2),
        "turn_count": turns,
        "efficiency_score": round(max(0, 100 - idle_percent - turns * 2), 2),
    }


def assess_collision_risk(events: list) -> dict:
    """Assess collision risk based on events."""
    near_misses = sum(1 for e in events if e.get("type") == "near_miss")
    stops = sum(1 for e in events if e.get("type") == "stop")
    total_events = len(events)

    # Risk score: 0-100
    risk_score = min(100, near_misses * 20 + stops * 10)

    return {
        "near_miss_count": near_misses,
        "stop_count": stops,
        "total_events": total_events,
        "risk_level": "low" if risk_score < 30 else "medium" if risk_score < 70 else "high",
        "risk_score": risk_score,
    }


def analyze_battery(battery: list, path_length: float) -> dict:
    """Analyze battery consumption."""
    if not battery or len(battery) < 2:
        return {"consumption_per_meter": 0, "estimated_range": 0}

    start_level = battery[0].get("level", 100)
    end_level = battery[-1].get("level", 100)
    consumption = start_level - end_level

    if path_length > 0:
        consumption_per_meter = consumption / path_length
    else:
        consumption_per_meter = 0

    # Estimate remaining range at current consumption
    if consumption > 0:
        estimated_range = (end_level / consumption) * path_length if consumption > 0 else float('inf')
    else:
        estimated_range = float('inf')

    return {
        "consumption_percent": consumption,
        "consumption_per_meter": round(consumption_per_meter, 4),
        "current_level": end_level,
        "estimated_range_meters": round(estimated_range, 2) if estimated_range != float('inf') else "unlimited",
    }


def generate_recommendations(coverage: dict, efficiency: dict, collision: dict, battery: dict) -> list:
    """Generate optimization recommendations using LLM or rules."""
    recommendations = []

    # Coverage recommendations
    if coverage.get("coverage_percent", 0) < 50:
        recommendations.append({
            "category": "coverage",
            "priority": "high",
            "message": f"Coverage is only {coverage.get('coverage_percent')}%. Consider more diverse routing."
        })

    # Efficiency recommendations
    if efficiency.get("idle_time_percent", 0) > 30:
        recommendations.append({
            "category": "efficiency",
            "priority": "medium",
            "message": f"Idle time is {efficiency.get('idle_time_percent')}%. Investigate causes of stops."
        })

    # Collision recommendations
    if collision.get("risk_level") == "high":
        recommendations.append({
            "category": "safety",
            "priority": "high",
            "message": "High collision risk detected. Review path planning and add safety zones."
        })

    # Battery recommendations
    if battery.get("estimated_range_meters") != "unlimited" and battery.get("estimated_range_meters", 0) < 100:
        recommendations.append({
            "category": "battery",
            "priority": "medium",
            "message": f"Low battery warning. Estimated range: {battery.get('estimated_range_meters')}m"
        })

    if not recommendations:
        recommendations.append({
            "category": "general",
            "priority": "info",
            "message": "AGV is operating within normal parameters."
        })

    return recommendations


def analyze_with_llm(data: dict, recommendations: list) -> str:
    """Use LLM for advanced analysis (optional)."""
    try:
        prompt = f"""Analyze this AGV path data and provide insights:

Coverage: {json.dumps(data.get('coverage', {}))}
Efficiency: {json.dumps(data.get('efficiency', {}))}
Collision Risk: {json.dumps(data.get('collision_risk', {}))}
Battery: {json.dumps(data.get('battery', {}))}

Current recommendations: {json.dumps(recommendations)}

Provide additional insights in 2-3 sentences."""

        resp = requests.post(
            f"{OLLAMA_URL}/api/generate",
            json={"model": "llama3.2", "prompt": prompt, "stream": False}
        )
        if resp.status_code == 200:
            return resp.json().get("response", "")
    except Exception:
        pass
    return ""


def main():
    parser = argparse.ArgumentParser(description="AGV Path Analysis")
    parser.add_argument("--device-id", required=True, help="AGV device ID")
    parser.add_argument("--time-window", type=int, default=3600, help="Time window in seconds")
    parser.add_argument("--map-width", type=float, help="Map width in meters")
    parser.add_argument("--map-height", type=float, help="Map height in meters")
    parser.add_argument("--mock", action="store_true", help="Use mock data")
    args = parser.parse_args()

    if args.mock:
        global USE_MOCK
        USE_MOCK = True

    # Get data
    data = get_agv_data(args.device_id, args.time_window)

    # Calculate metrics
    map_bounds = {"width": args.map_width or 10, "height": args.map_height or 10}
    coverage = calculate_coverage(data.get("positions", []), map_bounds)
    efficiency = calculate_efficiency(data.get("positions", []), data.get("velocities", []))
    collision_risk = assess_collision_risk(data.get("events", []))
    battery = analyze_battery(data.get("battery", []), coverage.get("path_length_meters", 0))

    # Build result
    result = {
        "device_id": args.device_id,
        "time_window_seconds": args.time_window,
        "coverage": coverage,
        "efficiency": efficiency,
        "collision_risk": collision_risk,
        "battery_optimization": battery,
    }

    # Generate recommendations
    recommendations = generate_recommendations(coverage, efficiency, collision_risk, battery)
    result["recommendations"] = recommendations

    # Optional LLM enhancement
    llm_insight = analyze_with_llm(result, recommendations)
    if llm_insight:
        result["llm_insight"] = llm_insight

    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
