# APS-V1-0002.SC01 — Security Dimension

**Version**: 1.0.0
**Parent**: APS-V1-0002 (Architecture Fitness Functions)

---

## 1. Scope

This substandard governs **dependency vulnerability scanning and supply chain safety**. It asserts that known security vulnerabilities in dependencies are detected, tracked, and resolved.

**Data source**: Security scanner output via adapter. No native topology integration — all data comes from external tools.

## 2. Adapter Contract

### 2.1 Supported Scanners

| Scanner | Language | Normalizer | Status |
|---------|----------|-----------|--------|
| cargo-audit | Rust | `builtin:cargo-audit` | Planned |
| npm audit | JavaScript/TypeScript | `builtin:npm-audit` | Planned |
| pip-audit | Python | `builtin:pip-audit` | Planned |
| Custom | Any | `script:<path>` | Available |

### 2.2 Normalized Output Format

```json
{
  "dependencies": [
    {
      "id": "serde:1.0.100",
      "metrics": {
        "critical_vulnerabilities": 0,
        "high_vulnerabilities": 0,
        "medium_vulnerabilities": 1,
        "low_vulnerabilities": 0,
        "total_vulnerabilities": 1,
        "max_cvss": 5.3
      }
    }
  ]
}
```

### 2.3 Severity Mapping

| CVSS Score | Fitness Severity |
|-----------|-----------------|
| 9.0–10.0 (Critical) | error |
| 7.0–8.9 (High) | error |
| 4.0–6.9 (Medium) | warning |
| 0.1–3.9 (Low) | warning |

## 3. Default Rules

```toml
[[rules.threshold]]
id = "zero-critical-vulns"
name = "Zero Critical Vulnerabilities"
dimension = "SC01"
source = "adapters/sc01/vulnerabilities.json"
field = "metrics.critical_vulnerabilities"
max = 0
scope = "dependency"
severity = "error"

[[rules.threshold]]
id = "zero-high-vulns"
name = "Zero High Vulnerabilities"
dimension = "SC01"
source = "adapters/sc01/vulnerabilities.json"
field = "metrics.high_vulnerabilities"
max = 0
scope = "dependency"
severity = "error"

[[rules.threshold]]
id = "max-medium-vulns"
name = "Maximum Medium Vulnerabilities per Dependency"
dimension = "SC01"
source = "adapters/sc01/vulnerabilities.json"
field = "metrics.medium_vulnerabilities"
max = 0
scope = "dependency"
severity = "warning"
```

## 4. Scoring

SC01 scoring reflects the severity-weighted vulnerability landscape:

```
SC01_score = 1.0 - (unexcepted_violations / total_dependencies_scanned)
```

## 5. Exception Handling

Security exceptions are time-sensitive. When excepting a vulnerability:

```toml
[zero-critical-vulns."openssl:1.1.1"]
value = 1
issue = "#302"
# Note: security exceptions should be reviewed weekly
```

Security exceptions SHOULD be reviewed more frequently than other dimensions due to the evolving nature of vulnerability disclosures.
