# interpret-report

Read and explain a fitness report, focusing on actionable insights.

## Usage

```
User: "explain fitness report" | "what are the violations" | "show system fitness"
```

## Parameters

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| report_path | No | `fitness-report.json` | Path to the fitness report |

## Procedure

1. Read `fitness-report.json`
2. Present the system-level fitness score prominently
3. Show per-dimension scores with visual indicators (bar chart)
4. If trend data is available, show deltas and highlight regressions
5. For failed rules, list each violation with:
   - Entity path
   - Actual value vs threshold
   - Whether it's excepted or not
   - Which issue tracks it (if excepted)
6. For stale exceptions, recommend cleanup actions
7. Provide prioritized recommendations:
   - Which dimension to focus on (lowest score)
   - Which violations have the highest impact
   - Whether the system score trend is improving or declining

## Output Format

```
System Fitness: 0.78 / 0.70 (PASS)

Dimensions:
  MT01 Maintainability:  0.92  ████████████████████░░  (+0.03 ↑)
  MD01 Modularity:       0.71  ██████████████░░░░░░░░  (-0.05 ↓)
  ...

Failures (1):
  [FAIL] max-main-sequence-distance (MD01)
    packages.data-access: D = 0.85 (threshold: 0.7)
    → This module is in the Zone of Pain (concrete + stable)
    → Consider adding abstractions or reducing incoming dependencies

Recommendations:
  1. MD01 declined by 0.05 — investigate new coupling in recent commits
  2. packages.data-access needs refactoring (Zone of Pain)
```

## Error Handling

| Error | Recovery |
|-------|----------|
| Report file not found | Suggest running `validate-fitness` first |
| Report version mismatch | Warn about version and attempt best-effort parsing |
