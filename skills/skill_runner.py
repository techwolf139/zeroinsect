#!/usr/bin/env python3
"""
Skill Runner - Unified interface for all business analysis skills
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

SKILLS_DIR = Path(__file__).parent

SKILL_REGISTRY = {
    "analyze_status": "analyze_status/main.py",
    "detect_anomaly": "detect_anomaly/main.py",
    "predict_failure": "predict_failure/main.py",
    "analyze_utilization": "analyze_utilization/main.py",
    "analyze_uptime": "analyze_uptime/main.py",
    "analyze_energy": "analyze_energy/main.py",
    "health_score": "health_score/main.py",
    "analyze_robot_arm": "analyze_robot_arm/main.py",
    "analyze_agv_path": "analyze_agv_path/main.py",
    "analyze_cleaning": "analyze_cleaning/main.py",
    "analyze_production": "analyze_production/main.py",
    "generate_maintenance": "generate_maintenance/main.py",
    "optimize_schedule": "optimize_schedule/main.py",
    "root_cause": "root_cause/main.py",
    "predictive_maintenance": "predictive_maintenance/main.py",
}


def list_skills():
    print("Available Skills:")
    print("-" * 50)
    for name in sorted(SKILL_REGISTRY.keys()):
        print(f"  {name}")
    print("-" * 50)
    print(f"Total: {len(SKILL_REGISTRY)} skills")


def run_skill(skill_name, args):
    if skill_name not in SKILL_REGISTRY:
        print(f"Error: Unknown skill '{skill_name}'")
        print("Run with --list to see available skills")
        return 1
    
    skill_path = SKILLS_DIR / SKILL_REGISTRY[skill_name]
    if not skill_path.exists():
        print(f"Error: Skill file not found: {skill_path}")
        return 1
    
    cmd = [sys.executable, str(skill_path)] + args
    result = subprocess.run(cmd)
    return result.returncode


def main():
    parser = argparse.ArgumentParser(description="Business Analysis Skill Runner")
    parser.add_argument("--list", action="store_true", help="List available skills")
    parser.add_argument("skill", nargs="?", help="Skill name to run")
    parser.add_argument("args", nargs=argparse.REMAINDER, help="Arguments for skill")
    
    args = parser.parse_args()
    
    if args.list:
        list_skills()
        return 0
    
    if not args.skill:
        parser.print_help()
        return 1
    
    return run_skill(args.skill, args.args)


if __name__ == "__main__":
    sys.exit(main())
