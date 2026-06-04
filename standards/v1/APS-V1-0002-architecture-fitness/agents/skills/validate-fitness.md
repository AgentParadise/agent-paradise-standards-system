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
| dimensions | No | all enabled | Comma-separated dimension codes to evaluate (e.g., `MT01,MD01,SC01`) |
| config | No | `fitness.toml` | Path to fitness configuration |
| report | No | none | Path to write JSON report |
| previous-report | No | none | Path to a previous `fitness-report.json` for trend analysis. Resolved relative to `path` when not absolute. |

The six active dimensions (MT01, MD01, ST01, SC01, LG01, AC01) are evaluated in strict-artifact mode: a missing source artifact or adapter output for an enabled active dimension fails the rule rather than skipping silently (`PROMOTION_REQUIREMENT_UNMET`, §12). Incubating dimensions (PF01, AV01) downgrade their rules to advisory severities and skip silently when artifacts are missing.

## Procedure

1. Check that `fitness.toml` exists at the target path
2. Check that `.topology/` directory exists (run `aps run topology analyze .` if missing)
3. Run `aps run fitness validate <path>` with appropriate options. For trend analysis, pass `--previous-report <file>` (the CLI flag is `--previous-report`, not `--previous`).
4. Read the output and report:
   - Overall pass/fail status
   - System-level fitness score and threshold
   - Per-dimension scores, marking each dimension's `promotion_status` (`active` / `incubating`) and `enforcement` (`enforced` / `advisory`) so users know which findings are blocking versus advisory
   - Any unexcepted violations with entity paths and actual values
   - Stale exceptions that should be cleaned up
   - Trend deltas if a previous report was provided

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
