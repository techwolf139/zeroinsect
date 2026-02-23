# analyze_agv_path Skill

> Analyze AGV (Automated Guided Vehicle) path efficiency and coverage

## Metadata

- **name**: analyze_agv_path
- **version**: 1.0.0
- **description**: Analyze AGV path coverage, efficiency, collision risk, and battery consumption
- **category**: business_analysis
- **tags**: agv, path, logistics, efficiency

## Input Schema

```yaml
inputs:
  device_id:
    type: string
    required: true
    description: AGV device ID
  time_window:
    type: integer
    default: 3600
    description: Analysis time window in seconds
  map_data:
    type: object
    optional: true
    description: Map boundary and obstacles data
```

## Output Schema

```yaml
outputs:
  coverage:
    type: object
    description: Path coverage metrics
  efficiency:
    type: object
    description: Movement efficiency metrics
  collision_risk:
    type: object
    description: Collision risk assessment
  battery_optimization:
    type: object
    description: Battery consumption analysis
  recommendations:
    type: array
    description: Optimization suggestions
```

## Analysis Metrics

### Coverage Metrics
- Area coverage percentage
- Unvisited zones detection
- Redundant path detection

### Efficiency Metrics
- Average velocity
- Idle time percentage
- Optimal path adherence
- Turn count analysis

### Collision Risk
- Near-miss events count
- Stop frequency
- Speed in congested areas

### Battery Optimization
- Energy consumption per meter
- Charging frequency
- Optimal route suggestions

## Usage

```bash
# Analyze specific AGV
python skills/analyze_agv_path/main.py --device-id agv-01

# With time window
python skills/analyze_agv_path/main.py --device-id agv-01 --time-window 7200

# Mock mode for testing
python skills/analyze_agv_path/main.py --device-id agv-01 --mock
```

## Data Sources

- Position data: `/api/device/{device_id}/sensor/position`
- Velocity data: `/api/device/{device_id}/sensor/velocity`
- Battery data: `/api/device/{device_id}/sensor/battery`
- Events: `/api/device/{device_id}/events`

## Dependencies

- requests
- numpy
- ollama (for LLM analysis)
