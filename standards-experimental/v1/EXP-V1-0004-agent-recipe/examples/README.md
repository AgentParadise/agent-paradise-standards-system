# Agent Recipe Standard Examples

As of the directory-shape rework (see `docs/01_spec.md`), a recipe is a
**directory**, not a single YAML file.

## Available Examples

| Example | Description |
|---------|--------------|
| `valid/pr-reviewer/` | A complete, conformant recipe directory: `recipe.yaml`, two agents (`main` with a `claude` harness, a subagent, skills, tools, and an append `system_instructions`; `reviewer` with a `codex` harness), a `SYSTEM.md`, and a bundled `skills/code-review/` package. |

`examples/invalid/<case>/` directories (each documenting which error code(s)
it is expected to trigger) and the conformance test that walks both
`examples/valid/` and `examples/invalid/` are added by Task 2 of the
directory-recipe rework plan, alongside `validate_recipe_dir`.

Schema-level fixtures exercising the loader (`load_recipe_dir`) directly -
including the failure cases (missing marker, unresolved `default_agent`,
malformed agent YAML) - live under `tests/fixtures/` instead; see
`tests/README.md`.

## Purpose

Examples in experiments serve to:
1. Demonstrate proposed patterns
2. Gather feedback from users
3. Validate the approach before promotion
4. Exercise the validator's error-code coverage in automated tests
