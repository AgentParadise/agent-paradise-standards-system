# Skill: Draft a new approved-deps entry

## Purpose

Given a proposed new third-party dependency, draft a complete `[[dep]]` entry for `approved-deps.toml` with justification, category decision, scope, and a one-level transitive audit.

## Inputs

- Dep name (e.g., `jsonschema`)
- Consumer crate name(s) (e.g., `aps-schema-test`)
- Stated need (why is this dep being added?)

## Procedure

1. **Alternatives check.** Can the stdlib or an already-approved entry cover this? If yes, recommend that instead and stop.
2. **Transitive audit.** Run `cargo tree -p <consumer> --edges no-dev --depth 2` (or the dep's own tree if not yet added). Note the direct children. Call out anything heavy (C deps, async runtimes, platform shims like icu_*, fraction, large regex engines).
3. **Category.**
   - `standard` — light transitive tree, widely-maintained, ergonomic addition to a shippable standard.
   - `tooling` — heavy, test-only, or CLI-only. MUST have non-`"*"` `allowed_for`.
4. **Scope.** `allowed_for` = exact consumer names, or `"*"` (standard only).
5. **Draft.** Produce the `[[dep]]` block. Example:
   ```toml
   [[dep]]
   name = "<dep>"
   category = "standard" | "tooling"
   justification = "<one-sentence need + any notable alternatives weighed>"
   allowed_for = ["<crate-a>", "<crate-b>"]
   transitive_audit_date = "<today ISO-8601>"
   ```

## Output

Return the draft `[[dep]]` block plus a one-paragraph review note for the PR description: what transitive children were found, whether any raised concern, and why the category + scope decision is appropriate.
