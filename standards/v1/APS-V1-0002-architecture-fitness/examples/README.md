# Architecture Fitness Functions — Examples

This directory contains examples demonstrating valid usage of APS-V1-0002.

## Available Examples

| Example | Description |
|---------|-------------|
| [`fitness.toml`](./fitness.toml) | Multi-dimensional fitness configuration with threshold rules, dependency rules, dimension settings, and system-level fitness weights |
| [`fitness-exceptions.toml`](./fitness-exceptions.toml) | Exception tracking with ratchet budgets and mandatory issue references |
| [`fitness-report.json`](./fitness-report.json) | Complete validation report with per-dimension scores, system-level composite, trend tracking, and violation details |
| [`fitness-composite.json`](./fitness-composite.json) | Detailed system-level fitness composite showing weight contributions, tradeoff analysis, and recommendations |

## Using These Examples

1. Copy `fitness.toml` to your repository root
2. Adjust thresholds for your project's standards
3. Run `aps run topology analyze .` to generate `.topology/` artifacts
4. Run `aps run fitness validate .` to evaluate rules
5. Use `fitness-report.json` as a reference for the output format

## Adding Examples

When adding a new example:

1. Create the example file with descriptive comments
2. Ensure it conforms to the spec (`docs/01_spec.md`)
3. Update this README
