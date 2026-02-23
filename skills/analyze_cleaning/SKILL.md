# analyze_cleaning Skill

> Analyze cleaning robot efficiency and coverage

## Metadata

- **name**: analyze_cleaning
- **version**: 1.0.0
- **description**: Analyze cleaning robot efficiency, coverage, redundancy, and task optimization
- **category**: business_analysis
- **tags**: cleaning, robot, coverage, efficiency

## Input Schema

```yaml
inputs:
  device_id:
    type: string
    required: true
    description: Cleaning robot device ID
  time_window:
    type: integer
    default: 3600
    description: Analysis time window in seconds
  map_data:
    type: object
    optional: true
    description: Floor map boundaries
```

## Output Schema

```yaml
outputs:
  coverage:
    type: object
    description: Cleaning coverage metrics
  redundancy:
    type: object
    description: Over-cleaning detection
  efficiency:
    type: object
    description: Cleaning efficiency score
  recommendations:
    type: array
    description: Optimization suggestions
```

## Analysis Metrics

### Coverage
- Total area cleaned (m²)
- Coverage percentage
- Unvisited zones

### Redundancy
- Repeated cleaning areas
- Overlap percentage
- Inefficient patterns

### Efficiency
- Area per minute
- Cleaning speed analysis
- Task completion time

## Usage

```bash
python skills/analyze_cleaning/main.py --device-id cleaner-01 --mock
python skills/analyze_cleaning/main.py --device-id cleaner-01 --time-window 7200
```
