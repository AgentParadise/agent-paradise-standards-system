# APS-V1-0002.PF01 — Performance Dimension

**Version**: 1.0.0
**Parent**: APS-V1-0002 (Architecture Fitness Functions)
**Default Status**: Opt-in (requires performance test infrastructure)

---

## 1. Scope

This substandard governs **performance regression detection**. It asserts that key performance indicators (latency, throughput, error rates) do not regress beyond acceptable thresholds.

**Data source**: Load test and benchmark output via adapter.

## 2. Adapter Contract

### 2.1 Supported Tools

| Tool | Type | Normalizer | Status |
|------|------|-----------|--------|
| k6 | Load testing | `builtin:k6` | Planned |
| Criterion.rs | Rust benchmarks | `builtin:criterion` | Planned |
| Custom | Any benchmark JSON | `script:<path>` | Available |

### 2.2 Normalized Output Format

```json
{
  "endpoints": [
    {
      "id": "GET /api/orders",
      "metrics": {
        "p50_latency_ms": 45,
        "p95_latency_ms": 120,
        "p99_latency_ms": 350,
        "throughput_rps": 1500,
        "error_rate": 0.002,
        "p99_regression_pct": 5.0
      }
    }
  ]
}
```

## 3. Default Rules

```toml
[[rules.threshold]]
id = "max-p99-latency"
name = "Maximum P99 Latency"
dimension = "PF01"
source = "adapters/pf01/performance.json"
field = "metrics.p99_latency_ms"
max = 500
scope = "endpoint"
severity = "error"

[[rules.threshold]]
id = "max-error-rate"
name = "Maximum Error Rate"
dimension = "PF01"
source = "adapters/pf01/performance.json"
field = "metrics.error_rate"
max = 0.01
scope = "endpoint"
severity = "error"

[[rules.threshold]]
id = "max-p99-regression"
name = "Maximum P99 Latency Regression"
dimension = "PF01"
source = "adapters/pf01/performance.json"
field = "metrics.p99_regression_pct"
max = 20
scope = "endpoint"
severity = "warning"
```

## 4. Scoring

```
PF01_score = 1.0 - (unexcepted_violations / total_endpoints_tested)
```

## 5. When to Enable

PF01 SHOULD be enabled for:
- API services with latency SLAs
- User-facing applications where performance affects UX
- Data processing pipelines with throughput requirements

PF01 requires a repeatable performance test suite (e.g., k6 scripts, Criterion benchmarks) to produce meaningful results.
