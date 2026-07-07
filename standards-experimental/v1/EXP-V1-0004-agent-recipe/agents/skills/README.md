# Agent Recipe Standard Agent Skills (Experimental)

⚠️ This experiment is in incubation. Skills may change.

## Usage

An AI agent authoring or reviewing an agent recipe file (see `docs/01_spec.md`) should:

1. Confirm the document is a single YAML mapping with the required fields:
   `name`, `agent`, `model.name`, `model.effort`.
2. Confirm `agent` is one of the v1 harness values: `claude`, `codex`.
3. Confirm `model.effort` is one of `low`, `medium`, `high`.
4. If `skills` is present, confirm every entry is a non-empty string reference.
5. If `system_instructions` is present, confirm `mode` is `append` or `replace`
   and `content` is non-empty.
6. Reject any field not in the schema (`AGENT_RECIPE_UNKNOWN_FIELD`) rather than
   guessing at its intent.
7. Never place credentials, tokens, or other secrets inside a recipe document
   (see spec section 8.1); recipes are expected to be committed to version
   control.

No CLI subcommand is exposed for this experiment yet. The reference validator
is `agent_recipe::validate_document` in `src/lib.rs`, callable from Rust:

```rust
let diagnostics = agent_recipe::validate_document(yaml_text);
if diagnostics.has_errors() {
    // report diagnostics
}
```

