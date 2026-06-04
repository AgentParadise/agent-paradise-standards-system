# Skill: Query Coupling

## Overview

Query coupling relationships and metrics from existing `.topology/` artifacts without re-running analysis.

## When to Use

- When asked about module dependencies or coupling
- When investigating why a change might have ripple effects
- When planning refactoring to decouple modules
- When asked "what depends on X?" or "what does X depend on?"

## Prerequisites

- `.topology/` directory must exist with valid artifacts
- Run `analyze-topology` first if artifacts are missing

## Inputs

| Input | Required | Description |
|-------|----------|-------------|
| `topology_path` | No | Path to `.topology/` directory (default: `./.topology`) |
| `module` | No | Specific module to query |
| `query_type` | No | Type of query: `dependents`, `dependencies`, `coupling`, `metrics` |

## Outputs

| Output | Description |
|--------|-------------|
| Coupling strength | Numeric coupling value (0-1) between modules |
| Dependency list | Modules that depend on or are depended upon |
| Martin's metrics | Ca, Ce, Instability, Abstractness, Distance |

## Query Types

### `dependents` — Who depends on this module?

```
Query: What depends on the crypto module?

Result:
- auth (coupling: 0.75, imports: 2)
  - Uses: hmac::sign, hmac::verify
```

### `dependencies` — What does this module depend on?

```
Query: What does api depend on?

Result:
- auth (coupling: 0.60, imports: 3)
- db (coupling: 0.55, imports: 2)
- utils (coupling: 0.35, imports: 1)
```

### `coupling` — How coupled are two modules?

```
Query: How coupled are auth and crypto?

Result:
- Coupling strength: 0.75 (high)
- auth → crypto: 2 imports, 3 function calls
- crypto → auth: 0 imports, 1 function call
- Recommendation: Consider if crypto should be internal to auth
```

### `metrics` — What are the quality metrics for a module?

```
Query: Show metrics for the utils module.

Result:
Module: utils
├── Cyclomatic: 1 (avg), 1 (max) ✅
├── Cognitive: 0 (avg), 0 (max) ✅
├── Instability (I): 0.0 — Very stable
├── Abstractness (A): 0.0 — Concrete
└── Distance (D): 1.0 ⚠️ — Zone of Pain

Warning: utils is concrete and stable. Changes are risky.
```

## Example Usage

```
User: Is it safe to refactor the crypto module?

Agent: Let me check what depends on crypto.

[Queries coupling from .topology/]

**Crypto Module Analysis:**

Dependents (Ca=2):
- auth (coupling: 0.75) — Heavy user of crypto
- No other direct dependents

Martin's Metrics:
- Instability: 0.33 (relatively stable)
- Distance: 0.67 ⚠️ — In Zone of Pain

**Assessment:** Refactoring crypto is moderately risky.
The auth module is heavily coupled (0.75) and would need updates.
Consider adding abstraction (traits/interfaces) before major changes.
```

## Error Handling

| Error | Action |
|-------|--------|
| No .topology directory | Suggest running `analyze-topology` |
| Module not found | List available modules |
| Stale artifacts | Warn about last analysis date |

## Related Skills

- `analyze-topology` — Generate topology artifacts
- `visualize-architecture` — See coupling as 3D visualization

