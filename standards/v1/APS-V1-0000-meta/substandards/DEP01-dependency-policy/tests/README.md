# DEP01 Tests

Test files land in Commit 2:

- `approved_deps_parse.rs` — round-trip `approved-deps.toml` parsing.
- `manifest_parsing.rs` — Cargo.toml / pyproject.toml / package.json readers.
- `rule_evaluation.rs` — end-to-end match of a fixture crate against a fixture approved list.
