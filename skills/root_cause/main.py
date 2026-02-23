#!/usr/bin/env python3
import argparse
import json
import os
import sys

OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434")
USE_MOCK = os.environ.get("MOCK_MODE", "false").lower() == "true"

FAILURE_PATTERNS = {
    "motor_overheat": {"cause": "Insufficient cooling or overload", "confidence": 0.85, "factors": ["ambient_temperature", "workload", "cooling_system"]},
    "connection_timeout": {"cause": "Network issues or device overload", "confidence": 0.75, "factors": ["network_latency", "device_load", "firewall"]},
    "position_error": {"cause": "Sensor drift or mechanical issue", "confidence": 0.80, "factors": ["sensor_calibration", "mechanical_wear", "encoder_status"]},
    "low_battery": {"cause": "Insufficient charging or high consumption", "confidence": 0.90, "factors": ["battery_health", "charging_pattern", "power_draw"]},
}


def analyze_failure(event):
    error_type = event.get("error", event.get("type", "unknown"))
    pattern = FAILURE_PATTERNS.get(error_type, {"cause": "Unknown", "confidence": 0.5, "factors": []})
    recommendations = []
    if "overheat" in error_type.lower():
        recommendations = ["Check cooling system", "Reduce workload", "Improve ventilation"]
    elif "timeout" in error_type.lower():
        recommendations = ["Check network connection", "Increase timeout threshold", "Reduce device load"]
    elif "position" in error_type.lower():
        recommendations = ["Recalibrate sensors", "Check mechanical joints", "Verify encoder status"]
    elif "battery" in error_type.lower():
        recommendations = ["Check charging schedule", "Replace battery if old", "Review power consumption"]
    else:
        recommendations = ["Review device logs", "Check maintenance history", "Contact support"]
    return {
        "root_cause": pattern["cause"],
        "confidence": pattern["confidence"],
        "related_factors": pattern["factors"],
        "recommendations": recommendations,
    }


def main():
    parser = argparse.ArgumentParser(description="Root Cause Analysis")
    parser.add_argument("--event", type=str, help='JSON event object')
    parser.add_argument("--mock", action="store_true")
    args = parser.parse_args()
    if args.mock or not args.event:
        event = {"type": "failure", "device": "robot-arm-01", "error": "motor_overheat"}
    else:
        event = json.loads(args.event)
    result = analyze_failure(event)
    result["event"] = event
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
