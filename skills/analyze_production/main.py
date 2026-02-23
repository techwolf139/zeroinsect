#!/usr/bin/env python3
import argparse
import json
import os
import sys
import time
from collections import defaultdict

import requests

DATA_LAKE_URL = os.environ.get("DATA_LAKE_URL", "http://localhost:8080")
OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434")
USE_MOCK = os.environ.get("MOCK_MODE", "false").lower() == "true"


def get_mock_data():
    return {
        "cycles": [
            {"id": 1, "start": 1700000000, "end": 1700000060, "status": "complete", "units": 10},
            {"id": 2, "start": 1700000060, "end": 1700000120, "status": "complete", "units": 12},
            {"id": 3, "start": 1700000120, "end": 1700000200, "status": "complete", "units": 9},
            {"id": 4, "start": 1700000200, "end": 1700000260, "status": "complete", "units": 11},
            {"id": 5, "start": 1700000260, "end": 1700000350, "status": "incomplete", "units": 5},
        ],
        "stops": [
            {"ts": 1700000350, "duration": 120, "reason": "maintenance"},
            {"ts": 1700000600, "duration": 60, "reason": "material_change"},
        ],
        "quality": {"total": 47, "good": 44, "rework": 3},
    }


def get_production_data(device_id: str, time_window: int):
    if USE_MOCK:
        return get_mock_data()
    now = int(time.time())
    start_ts = now - time_window
    cycle_url = f"{DATA_LAKE_URL}/api/device/{device_id}/production/cycles"
    stop_url = f"{DATA_LAKE_URL}/api/device/{device_id}/production/stops"
    quality_url = f"{DATA_LAKE_URL}/api/device/{device_id}/quality"
    cycles = requests.get(cycle_url, params={"start_ts": start_ts, "end_ts": now}).json() if requests.get(cycle_url).status_code == 200 else []
    stops = requests.get(stop_url, params={"start_ts": start_ts, "end_ts": now}).json() if requests.get(stop_url).status_code == 200 else []
    quality = requests.get(quality_url, params={"start_ts": start_ts, "end_ts": now}).json() if requests.get(quality_url).status_code == 200 else {}
    return {"cycles": cycles, "stops": stops, "quality": quality}


def analyze_cycle_time(cycles):
    if not cycles:
        return {"avg_seconds": 0, "min": 0, "max": 0, "trend": "unknown"}
    durations = [c["end"] - c["start"] for c in cycles if c.get("end") and c.get("start")]
    if not durations:
        return {"avg_seconds": 0, "min": 0, "max": 0, "trend": "unknown"}
    return {
        "avg_seconds": round(sum(durations) / len(durations), 2),
        "min": min(durations),
        "max": max(durations),
        "total_cycles": len(cycles),
    }


def calculate_oee(cycles, stops, quality, time_window):
    total_time = time_window
    running_time = sum(c.get("end", 0) - c.get("start", 0) for c in cycles if c.get("end") and c.get("start"))
    stop_time = sum(s.get("duration", 0) for s in stops)
    planned_time = max(total_time - stop_time, 1)
    availability = min(100, (running_time / planned_time) * 100) if planned_time > 0 else 0
    target_rate = 10
    actual_units = sum(c.get("units", 0) for c in cycles)
    target_units = (planned_time / 60) * target_rate
    performance = min(100, (actual_units / target_units) * 100) if target_units > 0 else 0
    total_units = quality.get("total", actual_units)
    good_units = quality.get("good", actual_units)
    quality_rate = min(100, (good_units / total_units) * 100) if total_units > 0 else 100
    oee = (availability * performance * quality_rate) / 10000
    return {
        "availability": round(availability, 2),
        "performance": round(performance, 2),
        "quality": round(quality_rate, 2),
        "oee": round(oee, 2),
    }


def find_bottlenecks(cycles, stops):
    bottlenecks = []
    if len(cycles) > 1:
        wait_times = []
        for i in range(1, len(cycles)):
            wait = cycles[i].get("start", 0) - cycles[i-1].get("end", 0)
            if wait > 0:
                wait_times.append(wait)
        if wait_times and max(wait_times) > 30:
            bottlenecks.append({"location": "station_transfer", "avg_wait_seconds": round(sum(wait_times)/len(wait_times), 2)})
    if stops:
        stop_reasons = defaultdict(int)
        for s in stops:
            stop_reasons[s.get("reason", "unknown")] += s.get("duration", 0)
        if stop_reasons:
            max_reason = max(stop_reasons.items(), key=lambda x: x[1])
            bottlenecks.append({"location": max_reason[0], "total_stop_time": max_reason[1]})
    return bottlenecks


def forecast_capacity(oee, avg_cycle_time, total_units):
    if not avg_cycle_time or avg_cycle_time == 0:
        return {"units_per_hour": 0, "daily_capacity": 0}
    units_per_hour = (3600 / avg_cycle_time) * (oee / 100)
    return {
        "units_per_hour": round(units_per_hour, 2),
        "daily_capacity": round(units_per_hour * 24, 0),
    }


def main():
    parser = argparse.ArgumentParser(description="Production Cycle Analysis")
    parser.add_argument("--device-id", required=True)
    parser.add_argument("--time-window", type=int, default=3600)
    parser.add_argument("--mock", action="store_true")
    args = parser.parse_args()
    if args.mock:
        global USE_MOCK
        USE_MOCK = True
    data = get_production_data(args.device_id, args.time_window)
    cycle_analysis = analyze_cycle_time(data.get("cycles", []))
    oee = calculate_oee(data.get("cycles", []), data.get("stops", []), data.get("quality", {}), args.time_window)
    bottlenecks = find_bottlenecks(data.get("cycles", []), data.get("stops", []))
    capacity = forecast_capacity(oee.get("oee", 0), cycle_analysis.get("avg_seconds", 60), sum(c.get("units", 0) for c in data.get("cycles", [])))
    result = {
        "device_id": args.device_id,
        "cycle_time": cycle_analysis,
        "oee": oee,
        "bottlenecks": bottlenecks,
        "capacity_forecast": capacity,
    }
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
