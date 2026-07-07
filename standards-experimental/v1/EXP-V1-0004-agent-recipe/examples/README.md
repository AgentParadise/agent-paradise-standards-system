# Agent Recipe Standard Examples

As of the directory-shape rework (see `docs/01_spec.md`), a recipe is a
**directory**, not a single YAML file.

These directories are the **canonical fixtures** the directory validator
(`validate::validate_recipe_dir`) is tested against, and which downstream
consumers (Plan B / `itmux`) vendor (plan revision R9). Each `invalid/<case>/`
directory declares the error code it must trigger in its `README.md`
(`# Expect: <CODE>`); `tests/conformance_test.rs` walks both trees and asserts
`valid/` produces zero errors and each `invalid/` case emits its declared code.

## Valid Examples

| Example | Description |
|---------|--------------|
| `valid/pr-reviewer/` | A complete, conformant recipe directory: `recipe.yaml`, two agents (`main` with a `claude` harness, a subagent, skills, tools, and an append `system_instructions`; `reviewer` with a `codex` harness), a `SYSTEM.md`, and a bundled `skills/code-review/` package. |

## Invalid Examples

| Example | Expected code |
|---------|---------------|
| `invalid/missing-marker/` | `RECIPE_MISSING_MARKER` |
| `invalid/unresolved-default-agent/` | `RECIPE_DEFAULT_AGENT_UNRESOLVED` |
| `invalid/unresolved-subagent/` | `RECIPE_SUBAGENT_UNRESOLVED` |
| `invalid/malformed-agent/` | `RECIPE_MALFORMED_AGENT_YAML` |

Additional schema-level fixtures exercising the loader (`load_recipe_dir`)
directly - including a minimal recipe with no optional `skills/`/`SYSTEM.md` -
live under `tests/fixtures/`; see `tests/README.md`.

## Purpose

Examples in experiments serve to:
1. Demonstrate proposed patterns
2. Gather feedback from users
3. Validate the approach before promotion
4. Exercise the validator's error-code coverage in automated tests
