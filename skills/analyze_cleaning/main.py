#!/usr/bin/env python3
"""
Cleaning Efficiency Analysis Skill

Analyzes cleaning robot coverage, redundancy, and efficiency.
"""

import argparse
import json
import os
import sys
import time
from collections import defaultdict
from typing import Optional

import requests

DATA_LAKE_URL = os.environ.get("DATA_LAKE_URL", "http://localhost:8080")
OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434")
USE_MOCK = os.environ.get("MOCK_MODE", "false").lower() == "true"


def get_mock_data() -> dict:
    """Generate mock cleaning robot data."""
    # Simulate a roomba-style cleaning pattern
    positions = []
    ts = 1700000000
    
    # Create grid-like cleaning pattern
    for y in range(0, 10, 2):
        for x in range(0, 10):
            positions.append({"x": float(x), "y": float(y), "ts": ts})
            ts += 5
        # Reverse direction
        for x in range(9, -1, -1):
            positions.append({"x": float(x), "y": float(y + 1), "ts": ts})
            ts += 5
    
    return {
        "positions": positions,
        "events": [
            {"type": "cleaning_start", "ts": 1700000000},
            {"type": "cleaning_complete", "ts": ts},
            {"type": "obstacle_detected", "x": 5.5, "y": 5.5, "ts": 1700000500},
        ],
    }


def get_cleaning_data(device_id: str, time_window: int) -> dict:
    """Fetch cleaning robot data from Data Lake."""
    if USE_MOCK:
        return get_mock_data()

    now = int(time.time())
    start_ts = now - time_window

    # Fetch position data
    pos_url = f"{DATA_LAKE_URL}/api/device/{device_id}/sensor/position"
    pos_resp = requests.get(pos_url, params={"start_ts": start_ts, "end_ts": now})
    positions = pos_resp.json() if pos_resp.status_code == 200 else []

    # Fetch events
    evt_url = f"{DATA_LAKE_URL}/api/device/{device_id}/events"
    evt_resp = requests.get(evt_url, params={"start_ts": start_ts, "end_ts": now})
    events = evt_resp.json() if evt_resp.status_code == 200 else []

    return {"positions": positions, "events": events}


def calculate_coverage(positions: list, map_area: float = 100.0) -> dict:
    """Calculate cleaning coverage metrics."""
    if not positions:
        return {"area_cleaned_m2": 0, "coverage_percent": 0, "unvisited_zones": []}

    # Calculate bounding box
    xs = [p.get("x", 0) for p in positions]
    ys = [p.get("y", 0) for p in positions]
    
    min_x, max_x = min(xs), max(xs)
    min_y, max_y = min(ys), max(ys)
    
    # Calculate area covered using grid cells
    # Assume 0.5m x 0.5m grid cells
    cell_size = 0.5
    visited_cells = set()
    
    for pos in positions:
        cell_x = int(pos.get("x", 0) / cell_size)
        cell_y = int(pos.get("y", 0) / cell_size)
        visited_cells.add((cell_x, cell_y))
    
    area_cleaned = len(visited_cells) * cell_size * cell_size
    coverage_percent = min(100, (area_cleaned / map_area) * 100)

    # Find unvisited zones (large gaps)
    unvisited = []
    grid_cols = int((max_x - min_x) / cell_size) + 1
    grid_rows = int((max_y - min_y) / cell_size) + 1
    
    all_cells = {(x, y) for x in range(grid_cols) for y in range(grid_rows)}
    gap_cells = all_cells - visited_cells
    
    # Group contiguous unvisited areas
    if gap_cells:
        unvisited.append({
            "estimated_percent": round(len(gap_cells) / len(all_cells) * 100, 2),
            "note": "Some areas not visited during cleaning cycle"
        })

    return {
        "area_cleaned_m2": round(area_cleaned, 2),
        "coverage_percent": round(coverage_percent, 2),
        "unvisited_zones": unvisited,
        "bounding_box": {"width": max_x - min_x, "height": max_y - min_y},
    }


def calculate_redundancy(positions: list) -> dict:
    """Detect over-cleaning (redundant passes)."""
    if not positions:
        return {"overlap_percent": 0, "repeated_areas": [], "redundancy_score": 0}

    # Use grid to find repeated visits
    cell_size = 0.5
    cell_visits = defaultdict(int)
    
    for pos in positions:
        cell_x = int(pos.get("x", 0) / cell_size)
        cell_y = int(pos.get("y", 0) / cell_size)
        cell_visits[(cell_x, cell_y)] += 1

    # Count cells visited multiple times
    repeated = sum(1 for count in cell_visits.values() if count > 1)
    total = len(cell_visits)
    
    overlap_percent = (repeated / total * 100) if total > 0 else 0
    
    # Find most over-cleaned areas
    repeated_areas = [
        {"cell": f"({x},{y})", "visits": count}
        for (x, y), count in sorted(cell_visits.items(), key=lambda x: -x[1])[:5]
        if count > 1
    ]

    # Redundancy score: 0-100 (higher = more wasteful)
    redundancy_score = min(100, overlap_percent * 2)

    return {
        "overlap_percent": round(overlap_percent, 2),
        "repeated_areas": repeated_areas,
        "redundancy_score": round(redundancy_score, 2),
    }


def calculate_efficiency(positions: list, events: list) -> dict:
    """Calculate cleaning efficiency metrics."""
    if not positions:
        return {"area_per_minute": 0, "avg_speed_mps": 0, "efficiency_score": 0}

    # Time span
    start_ts = positions[0].get("ts", 0)
    end_ts = positions[-1].get("ts", 0)
    duration_minutes = (end_ts - start_ts) / 60

    # Unique positions (coverage)
    unique_positions = len(set((p.get("x", 0), p.get("y", 0)) for p in positions))
    
    # Estimate area cleaned
    cell_size = 0.5
    area_cleaned = unique_positions * cell_size * cell_size

    # Average speed
    total_distance = 0
    for i in range(1, len(positions)):
        dx = positions[i].get("x", 0) - positions[i-1].get("x", 0)
        dy = positions[i].get("y", 0) - positions[i-1].get("y", 0)
        total_distance += (dx**2 + dy**2) ** 0.5
    
    duration_seconds = max(1, end_ts - start_ts)
    avg_speed = total_distance / duration_seconds
    
    # Area per minute
    area_per_minute = area_cleaned / duration_minutes if duration_minutes > 0 else 0

    # Efficiency score (combination of coverage and speed)
    # Higher is better, but too high might mean rushing
    efficiency_score = min(100, area_per_minute * 10)

    return {
        "area_cleaned_m2": round(area_cleaned, 2),
        "duration_minutes": round(duration_minutes, 2),
        "area_per_minute": round(area_per_minute, 2),
        "avg_speed_mps": round(avg_speed, 2),
        "efficiency_score": round(efficiency_score, 2),
    }


def generate_recommendations(coverage: dict, redundancy: dict, efficiency: dict) -> list:
    """Generate optimization recommendations."""
    recommendations = []

    # Coverage recommendations
    if coverage.get("coverage_percent", 0) < 80:
        recommendations.append({
            "category": "coverage",
            "priority": "high",
            "message": f"Coverage is only {coverage.get('coverage_percent')}%. Consider optimizing path planning."
        })

    # Redundancy recommendations
    if redundancy.get("overlap_percent", 0) > 30:
        recommendations.append({
            "category": "redundancy",
            "priority": "medium",
            "message": f"Overlapping cleaning at {redundancy.get('overlap_percent')}%. Improve pattern efficiency."
        })

    # Efficiency recommendations
    if efficiency.get("efficiency_score", 0) < 30:
        recommendations.append({
            "category": "efficiency",
            "priority": "medium",
            "message": "Low efficiency. Consider faster cleaning mode for large areas."
        })
    elif efficiency.get("efficiency_score", 0) > 90:
        recommendations.append({
            "category": "efficiency",
            "priority": "info",
            "message": "Very efficient cleaning pattern. Consider applying to other robots."
        })

    if not recommendations:
        recommendations.append({
            "category": "general",
            "priority": "info",
            "message": "Cleaning robot operating efficiently."
        })

    return recommendations


def main():
    parser = argparse.ArgumentParser(description="Cleaning Efficiency Analysis")
    parser.add_argument("--device-id", required=True, help="Cleaning robot device ID")
    parser.add_argument("--time-window", type=int, default=3600, help="Time window in seconds")
    parser.add_argument("--map-area", type=float, default=100.0, help="Total floor area in m²")
    parser.add_argument("--mock", action="store_true", help="Use mock data")
    args = parser.parse_args()

    if args.mock:
        global USE_MOCK
        USE_MOCK = True

    # Get data
    data = get_cleaning_data(args.device_id, args.time_window)

    # Calculate metrics
    coverage = calculate_coverage(data.get("positions", []), args.map_area)
    redundancy = calculate_redundancy(data.get("positions", []))
    efficiency = calculate_efficiency(data.get("positions", []), data.get("events", []))

    # Build result
    result = {
        "device_id": args.device_id,
        "time_window_seconds": args.time_window,
        "coverage": coverage,
        "redundancy": redundancy,
        "efficiency": efficiency,
        "recommendations": generate_recommendations(coverage, redundancy, efficiency),
    }

    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
