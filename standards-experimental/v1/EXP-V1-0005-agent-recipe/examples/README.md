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
| `invalid/malformed-manifest/` | `RECIPE_MALFORMED_MANIFEST` |
| `invalid/unresolved-default-agent/` | `RECIPE_DEFAULT_AGENT_UNRESOLVED` |
| `invalid/unresolved-subagent/` | `RECIPE_SUBAGENT_UNRESOLVED` |
| `invalid/malformed-agent/` | `RECIPE_MALFORMED_HARNESS_YAML` |
| `invalid/empty-recipe-name/` | `RECIPE_EMPTY_RECIPE_NAME` |
| `invalid/empty-agent-name/` | `RECIPE_EMPTY_AGENT_NAME` |
| `invalid/empty-model-name/` | `RECIPE_EMPTY_MODEL_NAME` |
| `invalid/invalid-skill-ref/` | `RECIPE_INVALID_SKILL_REF` |
| `invalid/invalid-tool-ref/` | `RECIPE_INVALID_TOOL_REF` |
| `invalid/empty-instructions-content/` | `RECIPE_EMPTY_INSTRUCTIONS_CONTENT` |

`RECIPE_DUPLICATE_AGENT` (two agent files sharing a stem, e.g. `main.yaml` and
`main.yml`) is exercised by the `tests/fixtures/duplicate-agent/` loader
fixture rather than an example directory, since a `.yaml`/`.yml` collision is
awkward to ship as a reviewable example. `RECIPE_IO_ERROR` is not
example-backed (it requires an induced filesystem failure).

Additional schema-level fixtures exercising the loader (`load_recipe_dir`)
directly - including a minimal recipe with no optional `skills/`/`SYSTEM.md` -
live under `tests/fixtures/`; see `tests/README.md`.

## Purpose

Examples in experiments serve to:
1. Demonstrate proposed patterns
2. Gather feedback from users
3. Validate the approach before promotion
4. Exercise the validator's error-code coverage in automated tests
