# Fitness Functions Examples

This directory contains example configurations demonstrating the experimental standard.

## Available Examples

| File | Description |
|------|-------------|
| [`fitness.toml`](fitness.toml) | Example rule configuration with threshold and dependency rules |
| [`fitness-exceptions.toml`](fitness-exceptions.toml) | Example exception file with ratchet budgets and issue references |
| [`fitness-report.json`](fitness-report.json) | Example validation report output |

## Usage

Copy `fitness.toml` and `fitness-exceptions.toml` to your repository root, then:

```bash
# Generate topology artifacts first
aps run topology analyze . --output .topology

# Run fitness validation
aps run fitness validate .
```

## Adapting for Your Codebase

1. **Adjust thresholds** — Set `max`/`min` values appropriate for your codebase
2. **Set field paths** — Use dot-notation to target nested metrics (e.g., `metrics.cognitive`)
3. **Configure excludes** — Skip test files, generated code, or vendored dependencies
4. **Add exceptions** — Track existing violations with issue references
