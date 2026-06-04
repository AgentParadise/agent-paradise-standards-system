# Coordination: PR 61 (EXP-V1-0004 documentation standard) Addendum 2

Two parallel sessions are finishing PR 61 per
`~/swarm-tasks/apss-unified-config.brief.md` Addendum 2 (operator-binding,
2026-06-04 22:57). Agent Mail is unreliable; this file is the source of
truth for who edits what so we do not clobber each other.

Already landed on the branch before this note (do not redo):

- `bfff5ca` align ADR01 + examples config to `disable = false` convention.
- `1784797` re-home EXP-V1-0004 config to the `docs` section of `apss.yaml`.
- `67b62ca` catch trailing `.apss/config.toml` refs in `aps-cli/src/main.rs`.
- `9f17982` align ADR01 with canonical ADR community resource (templates
  for `docs/adrs/README.md`, `AGENTS.md`, `CLAUDE.md`/`GEMINI.md` symlinks,
  `ADR-000-template.md`; spec/install-contract wiring).

Remaining Addendum 2 items, split below.

## Session split

### Claude (this session) - spec and design heavy

Owns:

- `standards-experimental/v1/EXP-V1-0004-documentation/docs/00_overview.md`
  Reframe so the generic frontmatter-driven index + progressive disclosure
  is the stated primary purpose. Doc types are instances of the mechanism,
  not the point of it (Addendum 2 item 4).
- `standards-experimental/v1/EXP-V1-0004-documentation/substandards/PV01-purpose-and-vision/docs/00_overview.md`
- `standards-experimental/v1/EXP-V1-0004-documentation/substandards/PV01-purpose-and-vision/docs/01_spec.md`
  Rewrite to the chosen North Star shape (Option C, single document at
  `docs/north-star.md` with `## Mission`, `## Vision`, `## Position`).
- `standards-experimental/v1/EXP-V1-0004-documentation/substandards/PV01-purpose-and-vision/substandard.toml`
  Rename: `name = "North Star (Mission, Vision, Position)"`,
  `slug = "north-star"`.
- Parent-level references to PV01 in:
  - `docs/00_overview.md` (shipped doc types table).
  - `docs/01_spec.md` (Sections 3.3 default config, 8 doc type registry,
    8.3 substandard summaries, 9.4 "valid structure" bullet).
  - `docs/02_install_contract.md` (Sections 1.4 template inventory, 2.5
    valid-structure bullet, 7 cross references).
- PR body update with the proposal in
  `docs/proposals/north-star-shape-options.md` (informative, copied into PR
  description; the file itself stays so future readers can see the
  reasoning).

### Codex - mechanical refactors, validators, tests, fmt

Owns:

- `standards-experimental/v1/EXP-V1-0004-documentation/src/config.rs`
  Rename Rust field `purpose_and_vision: PurposeVisionConfig` to
  `north_star: NorthStarConfig` and the serde rename to `"north-star"`.
  Keep `default_vision_location` renamed to `default_north_star_location`
  returning `"docs/north-star.md"`. Update `PurposeVisionConfig` struct
  name to `NorthStarConfig` and its fields stay the same shape (`disable`,
  `location`).
- `standards-experimental/v1/EXP-V1-0004-documentation/substandards/PV01-purpose-and-vision/src/lib.rs`
  Update:
  - `DEFAULT_LOCATION` -> `"docs/north-star.md"`.
  - `REQUIRED_SECTIONS` -> `&["Mission", "Vision", "Position"]`.
  - Diagnostic codes renamed in lockstep:
    `PV01-missing-purpose-section` -> `PV01-missing-mission-section`,
    `PV01-missing-vision-section` stays,
    `PV01-missing-non-goals-section` -> `PV01-missing-position-section`
    (changes severity from warning to error per the new spec; Mission and
    Vision were already errors).
  - Doc comments and hint strings updated to mention `docs.north-star`,
    not `docs.purpose-and-vision`.
- `standards-experimental/v1/EXP-V1-0004-documentation/substandards/PV01-purpose-and-vision/Cargo.toml`
  Keep the crate name `documentation-purpose-and-vision` (the crate is
  identified by PV01; renaming the crate is a separate workspace churn
  beyond Addendum 2). The directory name and substandard.toml carry the
  rename.
- `standards-experimental/v1/EXP-V1-0004-documentation/substandards/PV01-purpose-and-vision/tests/scaffold_smoke.rs`
  Update the asserted DEFAULT_LOCATION and REQUIRED_SECTIONS, the PV01
  prefix check stays the same.
- `standards-experimental/v1/EXP-V1-0004-documentation/examples/apss.yaml`
  Replace the `purpose-and-vision` block with `north-star` and the new
  default location.
- `standards-experimental/v1/EXP-V1-0004-documentation/tests/config_parsing.rs`
  if any test asserts the old key or location, retarget it.
- `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`
  pass before pushing.

### Directory rename (`PV01-purpose-and-vision` -> `PV01-north-star`)

Out of scope for this round. Renaming the directory touches:

- `Cargo.toml` workspace members entry.
- Every relative `../substandards/PV01-purpose-and-vision/...` path in
  parent specs, the install contract, the parent crate's `lib.rs`, and
  upstream readme links.
- Backlink integrity for any code that already references the old path.

The substandard rename is fully expressed by the substandard.toml `name`
and `slug` and by the `docs.north-star` config key. The directory name
stays `PV01-purpose-and-vision` (matching the historical crate name) and
the substandard's identity travels through the slug. A directory rename
can ship as a follow-up PR with `git mv` plus a parent-spec sweep.

## Rules carried from the brief

1. No em or en dashes in spec text, comments, or commit messages.
2. Keep CI green on every push. Do not merge.
3. Commit and push incrementally with conventional commit messages.
4. Existing config content is preserved: same keys, same disable flags,
   same defaults. Only the substandard slug, file location, and required
   sections change for PV01.

## Conflict-flagging protocol

If a touch crosses the boundaries above, append a note to the bottom of
this file under a dated heading explaining what and why; the other
session reconciles on its next pass.

## Status log

- 2026-06-05 (Claude): Coordination note + North Star Option C selection
  pushed. PV01 spec rewrites in progress.
