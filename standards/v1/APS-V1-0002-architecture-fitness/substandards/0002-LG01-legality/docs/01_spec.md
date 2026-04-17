# APS-V1-0002.LG01 — Legality Dimension

**Version**: 1.0.0
**Parent**: APS-V1-0002 (Architecture Fitness Functions)

---

## 1. Scope

This substandard governs **open-source license compliance and intellectual property safety**. It asserts that all dependencies use licenses compatible with the project's licensing model.

**Data source**: License scanner output via adapter.

## 2. Adapter Contract

### 2.1 Supported Scanners

| Scanner | Language | Normalizer | Status |
|---------|----------|-----------|--------|
| cargo-deny | Rust | `builtin:cargo-deny` | Planned |
| license-checker | JavaScript | `builtin:license-checker` | Planned |
| Custom | Any | `script:<path>` | Available |

### 2.2 Normalized Output Format

```json
{
  "dependencies": [
    {
      "id": "serde:1.0.100",
      "metrics": {
        "license": "MIT",
        "license_category": "permissive",
        "license_known": 1,
        "license_copyleft": 0,
        "license_proprietary": 0
      }
    }
  ]
}
```

### 2.3 License Categories

| Category | Examples | Typical Policy |
|----------|---------|---------------|
| `permissive` | MIT, Apache-2.0, BSD | Generally allowed |
| `weak_copyleft` | LGPL, MPL | Often allowed with care |
| `copyleft` | GPL, AGPL | Forbidden in proprietary projects |
| `proprietary` | Custom commercial | Requires explicit approval |
| `unknown` | No license detected | Forbidden by default |

## 3. Default Rules

```toml
[[rules.threshold]]
id = "no-copyleft"
name = "No Copyleft Dependencies"
dimension = "LG01"
source = "adapters/lg01/licenses.json"
field = "metrics.license_copyleft"
max = 0
scope = "dependency"
severity = "error"

[[rules.threshold]]
id = "no-unknown-license"
name = "No Unknown Licenses"
dimension = "LG01"
source = "adapters/lg01/licenses.json"
field = "metrics.license_known"
min = 1
scope = "dependency"
severity = "warning"
```

## 4. Scoring

```
LG01_score = 1.0 - (unexcepted_violations / total_dependencies_scanned)
```

## 5. Project License Context

License rules are inherently context-dependent. A copyleft dependency is a violation for a proprietary project but perfectly acceptable for a GPL project. The default rules assume a permissive/proprietary licensing model. Projects using copyleft licenses SHOULD override these rules.
