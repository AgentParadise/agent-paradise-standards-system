# APS-V1-0002.AV01 — Availability Dimension

**Version**: 1.0.0
**Parent**: APS-V1-0002 (Architecture Fitness Functions)
**Default Status**: Opt-in (requires chaos engineering or monitoring infrastructure)

---

## 1. Scope

This substandard governs **resilience and availability assertions**. It verifies that systems meet availability targets and can recover gracefully from failures, using chaos engineering experiments and uptime monitoring data.

**Data source**: Chaos engineering results and uptime metrics via adapter.

## 2. Adapter Contract

### 2.1 Supported Tools

| Tool | Type | Normalizer | Status |
|------|------|-----------|--------|
| Chaos Monkey | Chaos engineering | `builtin:chaos-monkey` | Planned |
| Custom experiments | Chaos engineering | `script:<path>` | Available |
| Uptime monitors | Monitoring API | `script:<path>` | Available |

### 2.2 Normalized Output Format

```json
{
  "services": [
    {
      "id": "order-service",
      "metrics": {
        "uptime_pct": 99.95,
        "rto_seconds": 30,
        "rpo_seconds": 0,
        "chaos_experiments_passed": 5,
        "chaos_experiments_total": 6,
        "chaos_pass_rate": 0.833,
        "mttr_seconds": 45
      }
    }
  ]
}
```

## 3. Default Rules

```toml
[[rules.threshold]]
id = "min-uptime"
name = "Minimum Uptime Percentage"
dimension = "AV01"
source = "adapters/av01/availability.json"
field = "metrics.uptime_pct"
min = 99.5
scope = "service"
severity = "warning"

[[rules.threshold]]
id = "max-rto"
name = "Maximum Recovery Time Objective"
dimension = "AV01"
source = "adapters/av01/availability.json"
field = "metrics.rto_seconds"
max = 300
scope = "service"
severity = "error"

[[rules.threshold]]
id = "min-chaos-pass-rate"
name = "Minimum Chaos Experiment Pass Rate"
dimension = "AV01"
source = "adapters/av01/availability.json"
field = "metrics.chaos_pass_rate"
min = 0.8
scope = "service"
severity = "warning"
```

## 4. Scoring

```
AV01_score = 1.0 - (unexcepted_violations / total_services_monitored)
```

## 5. When to Enable

AV01 SHOULD be enabled for:
- Production services with SLA commitments
- Systems where downtime has significant business impact
- Microservice architectures where partial failure is expected

AV01 requires either chaos engineering infrastructure (to run experiments) or uptime monitoring integration (to collect availability metrics).
