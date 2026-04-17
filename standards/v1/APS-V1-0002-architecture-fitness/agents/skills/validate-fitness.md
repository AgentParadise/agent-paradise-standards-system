# validate-fitness

Run architecture fitness validation and report results.

## Usage

```
User: "validate fitness" | "check architecture" | "run fitness functions"
```

## Parameters

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| path | No | `.` | Repository root path |
| dimensions | No | all enabled | Comma-separated dimension codes to evaluate |
| config | No | `fitness.toml` | Path to fitness configuration |
| report | No | none | Path to write JSON report |
| previous | No | none | Path to previous report for trend analysis |

## Procedure

1. Check that `fitness.toml` exists at the target path
2. Check that `.topology/` directory exists (run `aps run topology analyze .` if missing)
3. Run `aps run fitness validate <path>` with appropriate options
4. Read the output and report:
   - Overall pass/fail status
   - System-level fitness score and threshold
   - Per-dimension scores
   - Any unexcepted violations with entity paths and actual values
   - Stale exceptions that should be cleaned up
   - Trend deltas if previous report provided

## Outputs

- Exit code: 0 (pass), 1 (fail), 2 (warnings only)
- Console: Human-readable summary with dimension scores
- JSON report: Full details if `--report` specified

## Error Handling

| Error | Recovery |
|-------|----------|
| Missing fitness.toml | Suggest running `configure-dimensions` skill |
| Missing topology dir | Suggest running `aps run topology analyze .` |
| Adapter not found | Report which adapter is missing and dimension affected |
| System score below threshold | Identify weakest dimensions and suggest focus areas |
