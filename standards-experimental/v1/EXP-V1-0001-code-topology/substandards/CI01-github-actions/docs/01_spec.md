# CI01 Specification — Threshold Configuration

## 1. Configuration File

The CI check reads thresholds from `.topology/config.toml`:

```toml
# .topology/config.toml
schema_version = "1.0.0"

[thresholds]
# Function-level thresholds
max_cyclomatic_warning = 10
max_cyclomatic_failure = 20
max_cognitive_warning = 15
max_cognitive_failure = 30

# Module-level thresholds  
max_coupling_delta_warning = 0.10  # 10% increase
max_coupling_delta_failure = 0.25  # 25% increase
max_distance_warning = 0.5
max_distance_failure = 0.8

[behavior]
# Whether to fail the check on warnings
fail_on_warning = false

# Whether to post a comment on PRs
post_comment = true

# Include Mermaid diagrams in comments
include_diagrams = true

# Only check changed files (faster, less comprehensive)
incremental_only = false

[ignore]
# Paths to ignore in analysis
paths = [
    "tests/",
    "benches/",
    "examples/",
]

# Functions to ignore (by pattern)
functions = [
    "*::main",       # CLI entry points are naturally complex
    "*::test_*",     # Test functions
]
```

## 2. Diff Report Schema

The diff analyzer outputs a JSON report:

```json
{
  "schema_version": "1.0.0",
  "status": "warning",
  "summary": {
    "base_functions": 150,
    "pr_functions": 156,
    "base_total_cc": 234,
    "pr_total_cc": 248,
    "delta_cc": 14,
    "new_hotspots": 2,
    "coupling_changes": 3
  },
  "hotspots": [
    {
      "function": "parser::parse_complex",
      "file": "src/parser.rs",
      "line": 45,
      "base_cc": null,
      "pr_cc": 18,
      "threshold": 15,
      "severity": "warning"
    }
  ],
  "coupling_changes": [
    {
      "module_a": "parser",
      "module_b": "utils",
      "base_strength": 0.3,
      "pr_strength": 0.45,
      "delta": 0.15,
      "severity": "warning"
    }
  ],
  "zone_changes": [
    {
      "module": "core::data",
      "base_zone": "healthy",
      "pr_zone": "zone_of_pain",
      "base_distance": 0.2,
      "pr_distance": 0.7,
      "severity": "warning"
    }
  ]
}
```

## 3. Severity Levels

| Level | Exit Code | Effect |
|-------|-----------|--------|
| `pass` | 0 | Check passes, optional comment |
| `warning` | 0 (or 1 if `fail_on_warning`) | Check passes with warnings |
| `failure` | 1 | Check fails, blocks merge |

## 4. PR Comment Format

The CI posts a structured comment:

### Header

```markdown
## 🔍 Topology Analysis — {status_emoji} {status_text}
```

### Summary Table

```markdown
| Metric | Base | PR | Δ |
|--------|------|----|---|
| Functions | {base} | {pr} | {delta} |
| Total CC | {base} | {pr} | {delta} {emoji} |
| Max CC | {base} | {pr} | {delta} {emoji} |
| Avg Coupling | {base} | {pr} | {delta} |
```

### Hotspots Section

```markdown
### 🔥 New Hotspots

<details>
<summary>⚠️ <code>{function}</code> — CC: {cc} (threshold: {threshold})</summary>

**File**: `{file}:{line}`

**Suggestions**:
- Consider extracting helper functions
- Reduce nesting depth
- Simplify conditional logic

</details>
```

### Module Changes Diagram

```markdown
### 📦 Module Changes

```mermaid
graph LR
    {module_a}:::warning -->|+{delta}%| {module_b}
    classDef warning fill:#ffa94d
```
```

## 5. Command Interface

The workflow calls these commands:

```bash
# Analyze a project
aps topology analyze --output .topology-pr/

# Compare two topology directories
aps topology diff .topology-base/ .topology-pr/ --format json > diff.json

# Check against thresholds
aps topology check diff.json --config .topology/config.toml

# Generate PR comment markdown
aps topology comment diff.json --config .topology/config.toml > comment.md
```

## 6. Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All checks pass |
| 1 | Failure threshold exceeded |
| 2 | Configuration error |
| 3 | Analysis error |

