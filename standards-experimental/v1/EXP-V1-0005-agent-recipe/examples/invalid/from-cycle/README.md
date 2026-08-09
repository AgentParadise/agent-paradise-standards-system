# Expect: RECIPE_FROM_CYCLE

`agents/a.yaml` has `from: b` and `agents/b.yaml` has `from: a`. Resolving
either agent's inheritance chain revisits the other, which this standard
rejects rather than recursing forever.
