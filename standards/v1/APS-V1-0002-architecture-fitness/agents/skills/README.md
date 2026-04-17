# Architecture Fitness Functions — Agent Skills

Skills for AI agents to interact with the architectural fitness governance framework.

## Available Skills

| Skill | Purpose | Trigger Phrases |
|-------|---------|-----------------|
| [`validate-fitness`](./validate-fitness.md) | Run fitness validation and interpret results | "validate fitness", "check architecture", "run fitness functions" |
| [`interpret-report`](./interpret-report.md) | Read and explain a fitness report | "explain fitness report", "what are the violations", "show system fitness" |
| [`configure-dimensions`](./configure-dimensions.md) | Set up fitness governance for a project | "configure fitness", "set up architectural governance", "add fitness rules" |

## Skill Workflow

```
configure-dimensions → fitness.toml created
                            ↓
validate-fitness     → aps run fitness validate .
                            ↓
interpret-report     → read fitness-report.json, explain findings
```

## Artifact Locations

| Artifact | Path | Description |
|----------|------|-------------|
| Configuration | `fitness.toml` | Rule definitions and dimension settings |
| Exceptions | `fitness-exceptions.toml` | Tracked violations |
| Report | `fitness-report.json` | Validation output |
| Topology metrics | `.topology/metrics/` | Source data from APS-V1-0001 |

## Common Patterns

### First-time setup
1. Run `configure-dimensions` to create fitness.toml
2. Run `validate-fitness` to see initial violations
3. Add exceptions for known violations that can't be fixed immediately

### Ongoing governance
1. Run `validate-fitness` on each PR
2. Use `interpret-report` to understand failures
3. Fix violations or add tracked exceptions with issue references

### Tradeoff analysis
1. Run `validate-fitness` with `--previous` to see trends
2. Use `interpret-report` to understand dimension deltas
3. Focus on dimensions with declining scores
