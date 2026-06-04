# Agent Skills — Code Topology and Coupling Analysis

This directory contains skill definitions for AI agents working with code topology artifacts.

## Available Skills

| Skill | Purpose | Trigger Phrases |
|-------|---------|-----------------|
| [`analyze-topology`](./analyze-topology.md) | Generate `.topology/` artifacts | "analyze the codebase", "find complex code" |
| [`query-coupling`](./query-coupling.md) | Query coupling relationships | "what depends on X", "how coupled are X and Y" |
| [`visualize-architecture`](./visualize-architecture.md) | Generate visualizations | "show the architecture", "create a diagram" |

## Skill Workflow

```
┌─────────────────────┐
│  analyze-topology   │  ← First: Generate artifacts
└──────────┬──────────┘
           │
           ▼
    ┌──────────────┐
    │  .topology/  │  ← Artifacts stored
    └──────┬───────┘
           │
     ┌─────┴─────┐
     ▼           ▼
┌─────────┐  ┌──────────────────────┐
│ query-  │  │ visualize-           │
│ coupling│  │ architecture         │
└─────────┘  └──────────────────────┘
     │                │
     ▼                ▼
  Metrics         Diagrams/3D
```

## Common Patterns

### Initial Codebase Analysis

```
User: I'm new to this codebase. Help me understand it.

Agent workflow:
1. Run analyze-topology to generate artifacts
2. Run query-coupling to find high-complexity hotspots
3. Run visualize-architecture to show module relationships
4. Present findings with actionable insights
```

### Pre-Refactoring Assessment

```
User: I want to refactor the auth module. What should I know?

Agent workflow:
1. Check if .topology/ exists (run analyze-topology if not)
2. Run query-coupling for auth module
   - Find dependents (what would break)
   - Find dependencies (what auth needs)
   - Get Martin's metrics (stability assessment)
3. Present risk assessment
```

### Architecture Documentation

```
User: Create architecture diagrams for our docs.

Agent workflow:
1. Run visualize-architecture with mermaid projector
2. Generate module-level diagram
3. Generate call graph for critical paths
4. Output markdown-embeddable diagrams
```

## Artifact Locations

Skills read/write artifacts at these locations:

| Artifact | Path | Purpose |
|----------|------|---------|
| Manifest | `.topology/manifest.toml` | Analysis metadata |
| Function metrics | `.topology/metrics/functions.json` | Per-function complexity |
| Module metrics | `.topology/metrics/modules.json` | Martin's metrics |
| Coupling matrix | `.topology/graphs/coupling-matrix.json` | For 3D visualization |
| Historical | `.topology/snapshots/*.json` | Trend analysis |

## Skill Parameters

All skills accept these common parameters:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `topology_path` | `./.topology` | Path to artifacts directory |
| `verbose` | `false` | Show detailed progress |

## Error Recovery

| Situation | Recommended Action |
|-----------|-------------------|
| No `.topology/` directory | Run `analyze-topology` first |
| Stale artifacts | Re-run analysis or warn user |
| Parse errors | Report which files failed, continue with others |
| Unsupported language | Skip files, list in manifest warnings |

## Integration Notes

### For Agent Developers

Skills are designed to be composable. Common patterns:

```python
# Pseudocode for agent implementation
if needs_fresh_analysis():
    run_skill("analyze-topology", path=codebase_path)

coupling = run_skill("query-coupling", module="auth")
if coupling.instability > 0.8:
    warn("High instability - changes may have ripple effects")
```

### For Tool Developers

Skills expect these CLI commands to exist:

```bash
# Analysis
topology analyze <path> [--languages <langs>] [--exclude <patterns>]

# Query
topology query coupling <module> [--type dependents|dependencies|metrics]

# Visualization
topology project --projector <name> --format <fmt> --output <file>
```
