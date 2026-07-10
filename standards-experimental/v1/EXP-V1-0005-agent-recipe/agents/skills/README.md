# Agent Recipe Standard Agent Skills (Experimental)

⚠️ This experiment is in incubation. Skills may change.

## Usage

An AI agent authoring or reviewing an agent recipe directory (see
`docs/01_spec.md`) should:

1. Confirm the directory has a `recipe.yaml` marker with `name`, `version`,
   and `default_agent`.
2. Confirm `default_agent` names a file that actually exists under
   `agents/<default_agent>.yaml`.
3. For every `agents/*.yaml` file, confirm it has `name`, `agent`
   (`claude` | `codex`), and `model.name` / `model.effort`
   (`low` | `medium` | `high`).
4. If `skills` is present, confirm every entry is a non-empty string
   reference; entries resolve to `skills/<ref>/` inside the recipe if that
   subdirectory exists, else the ref is treated as an external skill
   path/name.
5. If `system_instructions` is present, confirm `mode` is `append` or
   `replace` and `content` is non-empty.
6. Reject any field not in the schema - the Rust types use
   `#[serde(deny_unknown_fields)]`, so this happens automatically on load.
7. Never place credentials, tokens, or other secrets inside a recipe
   directory; recipes are expected to be committed to version control.

A CLI is exposed through the development runner:

```bash
# Scaffold a new conformant recipe directory
apss-dev run agent-recipe create <name> [--dir <parent>]

# Validate a recipe directory (exit 0 = clean, 1 = errors, 3 = usage error)
apss-dev run agent-recipe validate <recipe-dir>
```

(slug aliases: `recipe`, `exp-v1-0005`)

For programmatic use, the reference loader is
`agent_recipe::load_recipe_dir` in `src/schema.rs`, callable from Rust:

```rust
use std::path::Path;

match agent_recipe::load_recipe_dir(Path::new("path/to/recipe")) {
    Ok(recipe) => {
        // recipe.manifest, recipe.agents, recipe.skills, recipe.system_md
    }
    Err(error) => {
        // error.code() is a stable machine-readable code (see
        // schema::error_codes), error itself carries the offending path.
    }
}
```
