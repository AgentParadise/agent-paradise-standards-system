# Skill: Run Standard CLI

## Purpose

Run a standard's CLI commands via `aps run`.

## Commands

```bash
# List available standards
aps run --list

# Run a standard command
aps run <slug> <command> [args...]

# Examples
aps run topology analyze .
aps run topology validate .topology/
aps run topology diff base/ pr/ --format json
```

## Common Patterns

### Analyze and Validate

```bash
# Generate artifacts
aps run topology analyze . --output .topology/

# Validate they were created correctly
aps run topology validate .topology/
```

### CI Diff Check

```bash
# Analyze base branch
git checkout main
aps run topology analyze --output .topology-base/

# Analyze PR branch
git checkout pr-branch
aps run topology analyze --output .topology-pr/

# Compare
aps run topology diff .topology-base/ .topology-pr/ --format json > diff.json
```

## Output Formats

Use `--json` for machine-readable output:

```bash
aps run topology validate .topology/ --json
```

Output:
```json
{
  "status": "success",
  "command": "validate",
  "diagnostics": []
}
```
