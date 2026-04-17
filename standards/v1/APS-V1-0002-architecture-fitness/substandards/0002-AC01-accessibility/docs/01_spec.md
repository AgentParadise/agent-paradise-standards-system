# APS-V1-0002.AC01 — Accessibility Dimension

**Version**: 1.0.0
**Parent**: APS-V1-0002 (Architecture Fitness Functions)
**Default Status**: Opt-in (requires web frontend)

---

## 1. Scope

This substandard governs **web accessibility compliance (WCAG)**. It asserts that user interfaces meet accessibility standards, ensuring inclusive design for users with disabilities.

**Data source**: Accessibility scanner output via adapter.

## 2. Adapter Contract

### 2.1 Supported Scanners

| Scanner | Type | Normalizer | Status |
|---------|------|-----------|--------|
| axe-core | Automated a11y testing | `builtin:axe-core` | Planned |
| pa11y | Automated a11y testing | `builtin:pa11y` | Planned |
| Lighthouse | Performance + a11y audit | `builtin:lighthouse` | Planned |

### 2.2 Normalized Output Format

```json
{
  "pages": [
    {
      "id": "/login",
      "metrics": {
        "wcag_a_violations": 0,
        "wcag_aa_violations": 2,
        "wcag_aaa_violations": 5,
        "color_contrast_violations": 1,
        "total_violations": 8,
        "accessibility_score": 0.85
      }
    }
  ]
}
```

## 3. Default Rules

```toml
[[rules.threshold]]
id = "zero-wcag-a"
name = "Zero WCAG Level A Violations"
dimension = "AC01"
source = "adapters/ac01/accessibility.json"
field = "metrics.wcag_a_violations"
max = 0
scope = "page"
severity = "error"

[[rules.threshold]]
id = "max-wcag-aa"
name = "Maximum WCAG Level AA Violations"
dimension = "AC01"
source = "adapters/ac01/accessibility.json"
field = "metrics.wcag_aa_violations"
max = 0
scope = "page"
severity = "warning"
```

## 4. Scoring

```
AC01_score = 1.0 - (unexcepted_violations / total_pages_scanned)
```

## 5. When to Enable

AC01 SHOULD be enabled for:
- Web applications with user-facing interfaces
- Public-facing websites
- Government or regulated industry projects (Section 508, EN 301 549)

AC01 MAY be disabled for:
- Backend services with no UI
- CLI tools
- Libraries without visual components
