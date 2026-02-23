# analyze_production Skill

> Analyze production cycle efficiency and OEE metrics

## Metadata

- **name**: analyze_production
- **version**: 1.0.0
- **description**: Analyze production cycle time, bottlenecks, OEE, and capacity forecasting
- **category**: business_analysis
- **tags**: production, oee, cycle_time, manufacturing

## Input Schema

```yaml
inputs:
  device_id:
    type: string
    required: true
    description: Production device/line ID
  time_window:
    type: integer
    default: 3600
    description: Analysis time window in seconds
```

## Output Schema

```yaml
outputs:
  cycle_time:
    type: object
    description: Cycle time analysis
  bottlenecks:
    type: array
    description: Identified bottlenecks
  oee:
    type: object
    description: Overall Equipment Effectiveness
  capacity_forecast:
    type: object
    description: Production capacity prediction
```

## Analysis Metrics

### Cycle Time
- Average cycle time
- Min/Max cycle time
- Cycle time trend

### OEE (Overall Equipment Effectiveness)
- Availability (running time / planned time)
- Performance (actual / target speed)
- Quality (good units / total units)

### Bottlenecks
- Station with longest wait time
- Resource contention points

## Usage

```bash
python skills/analyze_production/main.py --device-id line-01 --mock
```
