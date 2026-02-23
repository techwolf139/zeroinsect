# root_cause Skill

> Analyze failures and determine root causes

## Metadata

- **name**: root_cause
- **version**: 1.0.0
- **description**: Analyze failure events and determine root causes using historical data
- **category**: business_analysis
- **tags**: root_cause, failure, debugging, analysis

## Input Schema

```yaml
inputs:
  event:
    type: object
    required: true
```

## Output Schema

```yaml
outputs:
  root_cause:
    type: string
  confidence:
    type: float
  related_factors:
    type: array
  recommendations:
    type: array
```

## Usage

```bash
python skills/root_cause/main.py --event '{"type": "failure", "device": "robot-arm-01", "error": "motor_overheat"}'
```
