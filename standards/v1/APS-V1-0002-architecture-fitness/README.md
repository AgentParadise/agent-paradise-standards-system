# Architecture Fitness Functions

**ID:** `APS-V1-0002`
**Type:** Standard
**Slug:** `architecture-fitness`
**Version:** `1.0.0`

Declarative, automated, continuous assertions on architectural properties, organized into composable dimensional substandards. Promoted from EXP-V1-0003 with expanded scope.

## Index

- [standard.toml](standard.toml)
- [Specification](docs/01_spec.md) (rule format, exception format, report format, system fitness composite, dimension lifecycle)
- [Overview](docs/00_overview.md)
- [Metrics Catalog](docs/02_metrics-catalog.md)
- [Integration Guide](docs/INTEGRATION.md) (topology then fitness then CI)
- [ADRs](docs/adrs/) (0001 promotion lineage, 0002 MT01 + MD01 active, 0003 six-dimension promotion)
- [Examples](examples/) (`fitness.toml`, `fitness-exceptions.toml`, `fitness-report.json`)
- [Tests](tests/) (engine + schema round-trip + structural patterns)
- [Schemas](schemas/) (`fitness-config.schema.json`, `fitness-exceptions.schema.json`, `fitness-report.schema.json`)
- [Agent Skills](agents/skills/) (`validate-fitness`, `interpret-report`, `configure-dimensions`)
- [Substandards](substandards/) (one crate per dimension: MT01, MD01, ST01, SC01, LG01, AC01, PF01, AV01)

## Current dimension status (normative)

Six dimensions are `active` and strictly enforced: MT01 Maintainability, MD01 Modularity & Coupling, ST01 Structural Integrity, SC01 Security, LG01 Legality, AC01 Accessibility. PF01 Performance and AV01 Availability remain `incubating` because their thresholds (latency, throughput, SLOs) are project-specific. See [Appendix D](docs/01_spec.md#appendix-d-current-implementation-status) for per-dimension R1-R5 disclosure.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate standard APS-V1-0002
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```

Run the fitness engine itself against a target project:

```bash
cargo run -p aps-cli -- run fitness validate <path>
```

Add `--previous-report path/to/prior.json` for trend deltas (or the back-compat alias `--previous`).
