# Managing `approved-deps.toml`

## Adding a new dependency

1. Identify the proximate need. What specifically breaks without this dep? Is a stdlib solution acceptable?
2. Inspect the transitive tree one level deep: `cargo tree -p <crate> --edges no-dev --depth 2`. Note the direct children.
3. Write a justification describing (a) the need, (b) the alternatives considered, (c) any concerning transitive children.
4. Decide `category`:
   - **`standard`**: lightweight, stable, widely-used (e.g., `serde`, `thiserror`). Usable by shippable standards.
   - **`tooling`**: heavy or test-only (e.g., `jsonschema`, `clap`, `handlebars`). Scope strictly.
5. Decide `allowed_for`. Use `"*"` only for `category = "standard"` entries that are truly universal; otherwise enumerate the consuming crate names.
6. Record `transitive_audit_date` as today's ISO-8601 date.
7. Append the `[[dep]]` block and run `cargo run -p aps-cli -- run fitness validate` — the LG01 rule should pass.

## Renewing a stale audit

If `APPROVED_DEP_AUDIT_STALE` surfaces for an entry:
1. Re-run `cargo tree` and diff against the previous audit tree.
2. Investigate any new transitive additions.
3. If clean, update `transitive_audit_date` to today.

## Removing an approved entry

1. Find every consumer: `rg '<dep-name>\s*=' --type toml`.
2. Remove or replace each consumer's dependency declaration.
3. Delete the `[[dep]]` block from `approved-deps.toml`.
4. Commit atomically.
