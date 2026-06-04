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
  for `docs/adrs/README.md`, `AGENTS.md`, `CLAUDE.md` symlink,
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
- 2026-06-05 (Claude): Operator corrections 1 and 2 applied to specs.
  See "Operator corrections 2026-06-05" below for the Codex follow-up
  list that lands the matching mechanical changes.

## Operator corrections 2026-06-05

Two corrections from the operator after the first North Star spec
rewrite, plus a reinforcement on the install contract.

### Correction 1: absence-equals-enabled convention

Modelled on environment variables. The empty section is the happy
path. `disable: false` is the default the validator applies for
absence and MUST NOT appear in any real or example config. A key is
written only to opt out (`disable: true`) or to override a non-
`disable` default.

Spec text updated in this commit:

- Parent `docs/01_spec.md` Section 3.2 (renamed "Default Behavior
  (absence equals enabled)") with the explicit rule and the two
  reasons a key gets written.
- Section 3.3 schema example rewritten to show empty-section happy
  paths instead of `disable: false` boilerplate; every default-on
  surface now says "# disable defaults to false" as a comment, not as
  a YAML field.
- Section 3.4 carries the new rule for tooling and examples.
- Section 8.2 (adding a new doc type) no longer instructs new
  substandards to start their config block with `disable: false`.
- Parent `docs/00_overview.md` bullet 1 of "What else this standard
  provides" carries the convention.
- PV01 `docs/00_overview.md` and `docs/01_spec.md` Section 7 rewritten
  to recommend empty-section default and to forbid `disable: false`.
- RETRO01 `docs/00_overview.md` Configuration block rewritten the
  same way.

### Correction 2: AGENTS canonical, no GEMINI.md

`AGENTS.md` is the canonical agent context file. `CLAUDE.md` is the
ONLY symlink. Gemini reads `AGENTS.md` natively, so the standard ships
NO `GEMINI.md` anywhere.

Spec text updated in this commit:

- Parent `docs/02_install_contract.md` Section 1.4 inventory: the
  ADR01 inventory lists `README.md`, `AGENTS.md`, `CLAUDE.md`
  (symlink), `ADR-000-template.md`. No `GEMINI.md`. The intro
  paragraph carries the AGENTS-canonical statement and forbids the
  installer from adding a `GEMINI.md`.
- Parent `docs/02_install_contract.md` Section 1.5 (new, normative):
  the AGENTS.md and CLAUDE.md scaffolding contract. Create-if-missing
  on first install, never overwrite on any subsequent run (full
  stop), validation checks existence only, root context files stay
  project specific. This is the operator reinforcement landed
  verbatim as its own contract rule.
- Section 2.5 of the install contract carries the existence-only
  validator rule and points back to Section 1.5.
- Parent `docs/01_spec.md` Section 5.2 rewritten to make AGENTS.md
  canonical, CLAUDE.md a symlink, and to forbid GEMINI.md.
- ADR01 `docs/00_overview.md` template list rewritten without
  GEMINI.md, with the create-if-missing never-overwrite call out.
- ADR01 template file
  `templates/docs/adrs/AGENTS.md` rewritten to drop the GEMINI.md
  reference and to state the install contract rule in the file
  itself.
- Parent `docs/00_overview.md` bullet 3 updated for AGENTS-canonical
  framing and Section 1.5 pointer.

### Codex follow-up list (mechanical pieces)

The spec is now the source of truth. Codex owns the mechanical sweeps
that bring code, tests, and examples into line:

1. **Delete the ADR01 GEMINI.md symlink.**
   `git rm standards-experimental/v1/EXP-V1-0004-documentation/substandards/ADR01-architecture-decision-records/templates/docs/adrs/GEMINI.md`
   (it is currently a symlink to `AGENTS.md`). The directory now ships
   only `README.md`, `AGENTS.md`, `CLAUDE.md` symlink, and
   `ADR-000-template.md`.
2. **Sweep `examples/apss.yaml`.** Remove every `disable: false` line.
   The example MUST show empty sections (or no section at all) for
   defaults a project adopts. The example may keep overrides like
   `directory:` and `naming_pattern:` if they illustrate a non-default
   value; if they show the default value, prefer commenting them out
   with a "# default; remove to keep using the default" annotation.
3. **Sweep `tests/config_parsing.rs`.** Tests that asserted parsing of
   `disable: false` blocks can stay since they exercise the parser's
   tolerance, but at least one new test MUST assert that an empty
   `docs: {}` block, a missing `docs` block, and a missing `apss.yaml`
   all resolve to identical default configs (the absence-equals-
   enabled invariant in code).
4. **No change required in `src/config.rs`.** The Rust struct
   `Default` impls already encode the same absence-equals-enabled
   behaviour at deserialisation; this is a YAML-author convention, not
   a Rust ABI change. If a doc comment on a `Default` impl still says
   "the example config writes `disable: false`", update the doc
   comment.
5. **`cargo fmt --all` and `cargo clippy --workspace -- -D warnings`**
   pass before pushing. CI runs the no-`--all-targets` form for
   clippy, matching local invocation.

Boundary unchanged from the earlier split: Codex keeps PV01
`src/lib.rs` (`DEFAULT_LOCATION`, `REQUIRED_SECTIONS`, the two
renamed diagnostic code constants), the matching scaffold smoke
tests, and the `purpose_and_vision` -> `north_star` field rename in
`src/config.rs`. Claude keeps spec text and contract text.
