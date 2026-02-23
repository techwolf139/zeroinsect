#!/usr/bin/env python3
import argparse
import json
import os
import sys

DATA_LAKE_URL = os.environ.get("DATA_LAKE_URL", "http://localhost:8080")
OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434")
USE_MOCK = os.environ.get("MOCK_MODE", "false").lower() == "true"

COMPONENT_LIFECYCLES = {
    "motor": {"expected_hours": 5000, "degradation_rate": 0.02},
    "bearing": {"expected_hours": 3000, "degradation_rate": 0.03},
    "sensor": {"expected_hours": 8000, "degradation_rate": 0.01},
    "battery": {"expected_hours": 1500, "degradation_rate": 0.05},
    "belt": {"expected_hours": 2000, "degradation_rate": 0.04},
}


def get_mock_data():
    return {
        "components": [
            {"name": "motor", "operating_hours": 4200, "failure_count": 1},
            {"name": "bearing", "operating_hours": 2800, "failure_count": 0},
            {"name": "sensor", "operating_hours": 7500, "failure_count": 0},
        ],
    }


def predict_failure(component, current_hours):
    lifecycle = COMPONENT_LIFECYCLES.get(component, {"expected_hours": 3000, "degradation_rate": 0.03})
    remaining = lifecycle["expected_hours"] - current_hours
    risk = min(100, (current_hours / lifecycle["expected_hours"]) * 100)
    return {
        "component": component,
        "current_hours": current_hours,
        "expected_life_hours": lifecycle["expected_hours"],
        "remaining_hours": max(0, remaining),
        "risk_percent": round(risk, 2),
        "recommendation": "Replace soon" if risk > 80 else "Monitor" if risk > 50 else "OK",
    }


def find_maintenance_window(predictions):
    windows = []
    urgent = [p for p in predictions if p["risk_percent"] > 70]
    if urgent:
        windows.append({"type": "urgent", "reason": "High risk components", "timing": "Within 1 week"})
    medium = [p for p in predictions if 50 < p["risk_percent"] <= 70]
    if medium:
        windows.append({"type": "scheduled", "reason": "Monitored components", "timing": "Within 1 month"})
    return windows


def main():
    parser = argparse.ArgumentParser(description="Predictive Maintenance")
    parser.add_argument("--device-id", required=True)
    parser.add_argument("--mock", action="store_true")
    args = parser.parse_args()
    if args.mock:
        global USE_MOCK
        USE_MOCK = True
    data = get_mock_data() if USE_MOCK else {}
    components = data.get("components", [])
    predictions = []
    for comp in components:
        pred = predict_failure(comp.get("name", "unknown"), comp.get("operating_hours", 0))
        predictions.append(pred)
    predictions.sort(key=lambda x: x["risk_percent"], reverse=True)
    windows = find_maintenance_window(predictions)
    result = {
        "device_id": args.device_id,
        "predictions": predictions,
        "maintenance_windows": windows,
        "priority_components": [p["component"] for p in predictions[:3]],
    }
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
