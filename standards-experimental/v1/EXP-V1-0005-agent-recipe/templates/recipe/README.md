# Recipe Template

`skeleton/` is the canonical scaffold for a new recipe directory. It is the
source of truth for the files `agent_recipe::generate::scaffold_recipe` emits
(backed by `apss-dev run agent-recipe create <name>`).

The skeleton files are embedded into the crate via `include_str!` so the
generator is self-contained and works from any working directory, while the
on-disk copy here stays human-readable and reviewable. The two cannot drift:
the generator reads exactly these bytes at compile time.

Layout:

```text
skeleton/
  recipe.yaml            # RecipeManifest; {{name}} is substituted at generate time
  agents/
    main.yaml            # a claude default agent + a commented codex example
  skills/
    .gitkeep             # keeps the (initially empty) skills/ directory tracked
  SYSTEM.md              # starter shared base instructions
```

Only `recipe.yaml` contains a template variable (`{{name}}`); the other files
are literal starter content. Generator output is guaranteed to pass
`validate_recipe_dir` (see the round-trip test in `tests/round_trip_test.rs`).
