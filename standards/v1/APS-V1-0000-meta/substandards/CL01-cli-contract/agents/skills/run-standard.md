# Skill: Run Standard CLI

## Purpose

Run a standard's CLI commands via `apss run`.

## Commands

```bash
# List available standards
apss run --list

# Run a standard command
apss run <slug> <command> [args...]

# Examples
apss run topology analyze .
apss run topology validate .topology/
apss run topology diff base/ pr/ --format json
```

## Common Patterns

### Analyze and Validate

```bash
# Generate artifacts
apss run topology analyze . --output .topology/

# Validate they were created correctly
apss run topology validate .topology/
```

### CI Diff Check

```bash
# Analyze base branch
git checkout main
apss run topology analyze --output .topology-base/

# Analyze PR branch
git checkout pr-branch
apss run topology analyze --output .topology-pr/

# Compare
apss run topology diff .topology-base/ .topology-pr/ --format json > diff.json
```

## Output Formats

Use `--json` for machine-readable output:

```bash
apss run topology validate .topology/ --json
```

Output:
```json
{
  "status": "success",
  "command": "validate",
  "diagnostics": []
}
```
