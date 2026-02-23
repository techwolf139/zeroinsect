# generate_maintenance Skill

> Generate maintenance plans based on equipment health and history

## Metadata

- **name**: generate_maintenance
- **version**: 1.0.0
- **description**: Generate preventive maintenance schedules, parts lists, and downtime estimates
- **category**: business_analysis
- **tags**: maintenance, scheduling, planning

## Input Schema

```yaml
inputs:
  device_id:
    type: string
    required: true
  health_score:
    type: integer
    optional: true
  operating_hours:
    type: integer
    optional: true
```

## Output Schema

```yaml
outputs:
  next_maintenance:
    type: object
  maintenance_items:
    type: array
  parts_needed:
    type: array
  estimated_downtime:
    type: object
```

## Usage

```bash
python skills/generate_maintenance/main.py --device-id robot-arm-01 --health-score 65
```
