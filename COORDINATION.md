# Coordination: PR 61 (EXP-V1-0004 documentation standard) Addendum 3

Two parallel sessions are extending PR 61 per
`~/swarm-tasks/apss-unified-config.brief.md` Addendum 3 (operator,
2026-06-05). Earlier coordination files were deleted in commit
`aa3bcea`; this is a fresh start scoped to Addendum 3 work.

Already landed on `apss_doc-standard` before this note:

- Addendum 1 and Addendum 2 spec rewrites (commits up to `aa3bcea`),
  including PV01 renamed to the North Star (slug `north-star`,
  document at `docs/north-star.md`), AGENTS canonical with the
  `CLAUDE.md` symlink, `disable: false` removed from real and
  example configs.

The branch has NOT yet had `origin/main` merged into it.
`origin/main` carries the merged CF01 + DI01 substandards
(`fc2779c`).

## Addendum 3 split

### Claude (this session, completed in commit pending below)

Owns spec and contract text:

1. Filename casing aligned to merged CF01 (`apss.yaml` ->
   `APSS.yaml`) throughout the doc surface:
   - `docs/00_overview.md`, `docs/01_spec.md`,
     `docs/02_install_contract.md`
   - `substandards/ADR01-architecture-decision-records/docs/00_overview.md`,
     `docs/01_spec.md`, `templates/docs/adrs/README.md`
   - `substandards/PV01-purpose-and-vision/docs/00_overview.md`,
     `docs/01_spec.md`
   - `substandards/RETRO01-retrospectives/docs/00_overview.md`,
     `docs/01_spec.md`

2. ADR reference-accuracy validation specified:
   - Parent `docs/01_spec.md` Section 7.2 rewritten and renamed to
     "Reference accuracy". Section 7.2.1 adds the normative rule
     that a substandard MAY tighten accuracy to error severity with
     its own code, with ADR01 as the canonical example.
   - Parent `docs/01_spec.md` Section 9.4 ADR validator bullet
     extended to require the `ADR-NNN-<slug>` token resolution check
     and the new `ADR01-unknown-reference` (error) diagnostic.
   - `docs/02_install_contract.md` Section 2.5 ADR bullet updated
     identically.
   - ADR01 `docs/01_spec.md` Section 6 rewritten end to end:
     normative scan, resolution rules, diagnostic contents (file,
     line, token), placement guidance pointer, scope summary, and a
     migration note replacing the earlier `ADR01-dead-reference`
     warning with `ADR01-unknown-reference` (error).
   - ADR01 `docs/01_spec.md` Section 11 error code table updated.
   - ADR01 `docs/00_overview.md` error code table updated.

3. Backlink comment placement guidance:
   - Parent `docs/01_spec.md` Section 7.1.1 (new): two equally valid
     placements (top-of-file PREFERRED for whole-file ADRs;
     per-function or per-block ALSO ALLOWED for unit-scoped ADRs).
     The validator picks up tokens regardless of placement.
   - ADR01 `templates/docs/adrs/AGENTS.md` rewritten "The backlink
     rule" section: includes the two placements with worked examples
     in Rust and Python, plus the updated diagnostic naming.

4. Operator addition (2026-06-06): configurable scan globs.
   - Parent `docs/01_spec.md` Section 3.3 schema example: new
     `docs.backlinking.scan` key (list of include-globs) with the
     default list documented inline; absence means the default list
     applies (per Correction 1). The legacy `file_types` key is now
     a deprecated alias with `backlinking-file-types-deprecated`
     (warning) at config load when present; each entry `X` is
     treated as the glob `**/*.X` and unioned with `scan`.
   - Parent `docs/01_spec.md` Section 7.2 rewritten to reference
     `scan` first, `file_types` only for migration. The deprecation
     warning is documented in the parent diagnostic table (Section
     10.1).
   - ADR01 `docs/01_spec.md` Section 6.1 and 6.5 retargeted at the
     new `scan` set; the validator's walk respects whatever the
     parent config resolves.
   - Parent `docs/01_spec.md` Section 9.4 and `02_install_contract.md`
     Section 2.5 ADR bullets reference `scan` as the canonical
     source for the file set scanned, with `file_types` as the
     migration alias.

### Codex (this session, picking up after the spec lands)

Owns the merge, the validator implementation, and the mechanical
sweeps. Order matters; do them in this sequence so the spec text
this session pushes does not get clobbered by a later merge resolve.

1. **Merge `origin/main` into `apss_doc-standard` (merge, not rebase).**
   Conflicts seen on the first attempt (aborted by Claude before
   touching anything Codex owns):
   - `AGENTS.md`: main rewrote the root context file with a new
     project overview; HEAD still carries the RIPER-5 protocol body.
     Resolve in favour of the project text on main; the RIPER-5
     section is owned by the operator workflow docs, not this PR.
   - `CLAUDE.md`: distinct types (main has a regular file; HEAD has
     a symlink). Resolve to whatever main intends; the docs standard
     is not normative on the repo root layout (DOC03 only requires
     existence + the APSS/docs-root/doc-type references).
   - `Cargo.lock`: regenerate after the Rust merge resolves.
   - `crates/aps-cli/Cargo.toml`,
     `crates/aps-cli/src/main.rs`,
     `crates/aps-cli/tests/fixtures/mod.rs`,
     `crates/aps-cli/tests/template_test.rs`: code merges.
   After resolving, run `cargo test --workspace`,
   `cargo clippy --workspace -- -D warnings`, and
   `cargo fmt --all -- --check` before pushing.

2. **Implement `ADR01-unknown-reference` in the validator.**
   - The reference-extraction regex MUST be derived from
     `docs.adr.naming_pattern` (so a project with a custom prefix
     still gets accurate detection). For the default pattern
     `ADR-\d{3,5}-[a-zA-Z0-9-]+\.md`, the extracted token is
     `ADR-NNN-<slug>` (3 to 5 digit number).
   - Scan walks the repository, skipping hidden directories,
     `docs.readme.exclude_dirs`, and the ADR directory itself.
   - For every token found, look up the matching file name
     `<token>.md` in `<docs.root>/<docs.adr.directory>/`. If the file
     does not exist OR its name does not satisfy
     `docs.adr.naming_pattern`, emit `ADR01-unknown-reference`
     (error) with the source file path, line number, and offending
     token verbatim.
   - Update the existing `DEAD_ADR_REFERENCE` constant in
     `substandards/ADR01-architecture-decision-records/src/lib.rs`:
     rename to `UNKNOWN_ADR_REFERENCE` and change the value to
     `"ADR01-unknown-reference"`. Upgrade the call site from warning
     to error severity. Drop the old constant; the spec migration
     note (Section 6.6) confirms it is a one-way upgrade.
   - Update the doc comment in
     `standards-experimental/v1/EXP-V1-0004-documentation/src/config.rs`
     line 278 that mentions `ADR01-dead-reference` to point at the
     new code.

3. **Tests for the reference accuracy validator.**
   - Resolves to a real file: passes silently.
   - Resolves to a file whose name does not satisfy
     `docs.adr.naming_pattern`: error.
   - Resolves to a file that does not exist: error.
   - Default pattern: token like `ADR-001-security`.
   - Custom pattern with `DEC-` prefix: token like `DEC-042-routing`.
   - Backlinking disabled: no errors, no warnings.
   - Token inside a comment in the file's source language: detected.
   - Diagnostic carries file path, line number, and the offending
     token verbatim.
   - **Scan defaults applied when `docs.backlinking.scan` is absent**
     (file types covered by the default globs all scanned).
   - **Scan override**: a custom `scan` list (for example
     `["src/**/*.rs", "scripts/*.sh"]`) restricts the walk and an ADR
     token outside the override is NOT detected.
   - **Deprecated `file_types` migration**: a config that sets only
     `file_types` still scans the equivalent file set and the
     validator emits `backlinking-file-types-deprecated` (warning).
   - **`scan` plus `file_types` union**: when both are set the walked
     set is the union; the deprecation warning still fires.

4. **Spec the new `scan` key in the Rust config model.**
   - Add `pub scan: Vec<String>` to the `BacklinkingConfig` struct in
     `standards-experimental/v1/EXP-V1-0004-documentation/src/config.rs`
     with a `#[serde(default = "default_backlinking_scan")]` matching
     the defaults documented in Section 3.3 of the parent spec.
   - Keep `file_types` for back-compat; surface
     `backlinking-file-types-deprecated` (warning) at config load
     when `file_types` is non-empty.
   - The merged glob set (scan union with file_types as `**/*.X`) is
     what the validator walks; centralise the merge inside the config
     module so the ADR01 validator just reads the resolved globs.

5. **Sweeps.**
   - Add `examples/APSS.yaml` (Codex deleted `examples/apss.yaml` in
     `aa3bcea`; the new example should follow CF01's canonical
     casing and the absence-equals-enabled empty-section convention
     from Correction 1). Keep the file minimal: showing the docs
     section schema with only override fields, no `disable: false`
     boilerplate.
   - Update `examples/README.md` to drop the old `apss.config/v1`
     schema id; the merged CF01 spec specifies `apss.project/v1`
     instead.

## Out of scope: semantic vectorization (repo issue #66)

The semantic-vectorization layer that surfaced in earlier addenda is
explicitly out of scope for PR 61. The operator filed repo issue #66
to track it separately. Neither session should add vectorization
code, vectorization tests, vectorization config fields, or
vectorization spec text in this PR. If a follow-up needs an empty
hook on the docs validator surface to keep #66 cheap to land later,
that is fine; do not anticipate the work.

## Merge-state warning for Codex

Claude attempted `git merge origin/main --no-ff` to scope conflicts
and immediately ran `git merge --abort` since merge mechanics belong
to Codex. The abort succeeded the first time. A later invocation of
something (likely a hook) re-entered the merge silently; Claude
detected the unmerged state via `MERGE_HEAD` and aborted again
before continuing. When Codex starts the real merge, run
`git status` first to confirm a clean tree, and stash any
intermediate edits before re-running the merge so the conflict set
stays clean.

## Open question for the operator (please resolve before merge)

The merged CF01 spec on `origin/main` Section 2.7 still carries
the `disable: false` opt-out convention as the global default for
every standard's section. Correction 1 (operator, 2026-06-05)
reversed that for EXP-V1-0004 to absence-equals-enabled. The two
specs now disagree. EXP-V1-0004 follows the operator override; CF01
on main has not yet been corrected. Decisions needed:

- Option A: CF01 gets a matching correction in a follow-up PR;
  EXP-V1-0004 stays absence-equals-enabled. (Recommended.)
- Option B: EXP-V1-0004 reverts to `disable: false` to match CF01.
- Option C: Both conventions coexist (an awkward split that this
  session believes the operator did not intend).

This session leaves EXP-V1-0004 absence-equals-enabled per Correction
1 and surfaces the conflict here rather than guessing.

## Final-gate blocker 2026-06-06 (Codex)

Operator final-gate review found a hard blocker plus two doc defects.
Claude has fixed the two doc defects in the same commit as this note;
Codex owns the BLOCKER. Details:

### BLOCKER: filename-casing drift in the docs standard's Rust

The merged CF01 meta-standard canonically loads `APSS.yaml`
(uppercase) via `crates/apss-core/src/config.rs` line 19
(`CONFIG_FILENAME = "APSS.yaml"`). This standard's Rust still
hardcodes lowercase `apss.yaml` in:

- `standards-experimental/v1/EXP-V1-0004-documentation/src/config.rs`
  line 24:
  `pub const CONFIG_FILENAME: &str = "apss.yaml";`
- `crates/aps-cli/src/main.rs` lines 1446 and 1464 where the docs
  validate path is wired.

On a case-sensitive filesystem (Linux CI, most production hosts) the
docs validator silently ignores a project's real `APSS.yaml`.

**Fix per the no-magic-strings rule:**

1. Delete `pub const CONFIG_FILENAME: &str = "apss.yaml";` in
   `standards-experimental/v1/EXP-V1-0004-documentation/src/config.rs`.
2. Import the canonical constant from `apss-core`:
   `use apss_core::config::CONFIG_FILENAME;` (or
   `apss_core::CONFIG_FILENAME` if re-exported) and use it wherever
   the docs standard reads or writes the config file. There MUST be
   no local duplicate. Adjust the `load_config` function and any
   helper that currently joins the literal `"apss.yaml"`.
3. Update `crates/aps-cli/src/main.rs` lines 1446 and 1464 to use
   the same imported constant. Drop any local string literal.
4. Sweep every remaining lowercase `apss.yaml` doc comment and hint
   string in the docs standard's Rust to the canonical casing:
   - `standards-experimental/v1/EXP-V1-0004-documentation/src/config.rs`
     module doc, `DocsConfig` doc, `BacklinkingConfig` doc,
     `PurposeVisionConfig` doc, `load_config` doc.
   - `standards-experimental/v1/EXP-V1-0004-documentation/src/lib.rs`
     line 128: `Load the validator, reading config from APSS.yaml.`
   - `standards-experimental/v1/EXP-V1-0004-documentation/src/readme.rs`
     line 30 hint string: `configure docs.root in APSS.yaml`.
   - `standards-experimental/v1/EXP-V1-0004-documentation/tests/config_parsing.rs`
     module doc and inline comments at lines 197 and 224.
   - `standards-experimental/v1/EXP-V1-0004-documentation/substandards/ADR01-architecture-decision-records/src/lib.rs`
     doc comment on `from_repo` and hint strings at lines 99 and 118.
   - `standards-experimental/v1/EXP-V1-0004-documentation/substandards/PV01-purpose-and-vision/src/lib.rs`
     line 56 hint string.
   - `standards-experimental/v1/EXP-V1-0004-documentation/substandards/RETRO01-retrospectives/src/lib.rs`
     line 61 hint string.

**Regression test (required):**

Add a unit or integration test in
`standards-experimental/v1/EXP-V1-0004-documentation/tests/config_parsing.rs`
that:

1. Creates a temp directory.
2. Writes the REAL uppercase file name `APSS.yaml` (not lowercase) to
   that directory with a valid `docs:` block.
3. Calls `load_config(temp.path())` and asserts the returned config
   loads the docs block from the file.
4. Asserts the inverse: a lowercase `apss.yaml` in the same temp dir
   without an uppercase counterpart MUST NOT be picked up as the
   canonical config (otherwise the validator silently honours the
   wrong file on case-insensitive filesystems and silently ignores
   it on case-sensitive ones).

The test exists so this drift can never silently recur and so any
future refactor that re-introduces a lowercase magic string fails
CI.

### DEFECT 2 (Claude, fixed in this commit)

`standards-experimental/v1/EXP-V1-0004-documentation/examples/apss.yaml`
renamed via `git mv` to `examples/APSS.yaml`. Both that file and
`examples/README.md` now carry the canonical merged schema id
`apss.project/v1`, with the example structure updated to reflect
AGENTS canonical / CLAUDE.md symlink and the absence-equals-enabled
convention.

### DEFECT 3 (Claude, fixed in this commit)

PR 61 body test-plan bullet that claimed Clippy and Format were red
is corrected to show all five CI checks green.

## Rules carried from the brief

1. Do not merge PR 61. Keep CI green on every push.
2. No em or en dashes anywhere.
3. Conventional commit messages.
4. Commit and push incrementally with clear messages.
