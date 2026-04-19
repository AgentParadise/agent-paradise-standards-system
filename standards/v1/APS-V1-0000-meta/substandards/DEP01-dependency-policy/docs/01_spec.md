# APS-V1-0000.DEP01 — Specification

Normative specification. Keywords MUST, MUST NOT, SHOULD, MAY follow RFC 2119.

## §1 Repository requirements

§1.1 A repository adopting DEP01 **MUST** contain `approved-deps.toml` at the repository root.

§1.2 `approved-deps.toml` **MUST** have `schema = "aps.approved-deps/v1"` as its first assignment.

§1.3 Every `[[dep]]` entry **MUST** provide `name`, `justification`, `category`, `allowed_for`, and `transitive_audit_date`. `notes` is optional.

§1.4 `category` **MUST** be one of `"standard"` or `"tooling"`.

§1.5 `allowed_for` is an array of crate-name glob patterns. `"*"` matches any crate.

§1.6 `transitive_audit_date` **MUST** be an ISO-8601 date (e.g., `2026-04-18`).

## §2 Shippable standard crates

§2.1 A crate whose path matches `standards/v1/APS-*/**` is a **shippable standard**.

§2.2 A shippable standard **MUST NOT** declare a `[dependencies]` entry whose name is not present in `approved-deps.toml`.

§2.3 A shippable standard **MUST NOT** declare a `[dev-dependencies]` entry whose name is not present in `approved-deps.toml`, **unless** the entry has `category = "tooling"` AND the consuming crate's package name appears in that entry's `allowed_for` list.

§2.4 A `category = "tooling"` entry with `allowed_for = ["*"]` **MUST NOT** appear. Tooling deps require explicit scope.

## §3 Tooling crates

§3.1 A crate whose path matches `crates/**` is a **tooling crate**.

§3.2 A tooling crate **MAY** declare any dependency whose approved-list `allowed_for` list names it (or contains `"*"` for `category = "standard"` entries).

## §4 Audit cadence

§4.1 Every entry's `transitive_audit_date` **SHOULD** be less than 12 months old. Stale audits **MAY** emit a warning diagnostic (`APPROVED_DEP_AUDIT_STALE`) but **MUST NOT** block merges.

§4.2 When an entry's transitive tree changes materially (major-version bumps introducing new crates), maintainers **MUST** refresh the audit and update `transitive_audit_date`.

## §5 Exception handling

§5.1 Adding a new approved entry requires: (a) identifying the proximate need, (b) one-level transitive review, (c) written justification in the `justification` field, (d) explicit `allowed_for` scoping.

§5.2 Removal of an approved entry **MUST** be paired with removal of all consuming dependencies.

## §6 Fitness integration

§6.1 DEP01 supplies a rule template consumable under dimension LG01 as `[[rules.dependency_manifest]]`. The rule:

- Reads the configured `approved_list` path.
- Globs configured `manifests` relative to the workspace root.
- For each manifest, extracts direct dependencies and emits a violation per unapproved entry or scope breach.

§6.2 If `approved_list` is missing the rule emits `status = "skip"` — never `fail`. A missing policy file is a documentation gap, not a compliance breach.

## §7 Error codes

| Code | Meaning |
|------|---------|
| `UNAPPROVED_DEPENDENCY` | Dep name not present in `approved-deps.toml`. |
| `DEPENDENCY_NOT_ALLOWED_FOR_CRATE` | Approved, but consuming crate not in `allowed_for`. |
| `DEPENDENCY_WRONG_CATEGORY` | Tooling dep used by a shippable standard (or vice-versa). |
| `APPROVED_DEP_AUDIT_STALE` | `transitive_audit_date` older than audit window. |
