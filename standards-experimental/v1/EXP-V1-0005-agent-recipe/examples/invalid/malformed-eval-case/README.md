# Expect: RECIPE_MALFORMED_EVAL_CASE

`evals/broken-case/` has an `input.json` but no `expected.md`. A missing
half of an eval case MUST be reported, not silently skipped: silent
skipping is the failure mode where the bar quietly shrinks while the suite
still reports green.
