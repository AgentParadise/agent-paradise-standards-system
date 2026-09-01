---
name: "Install Contract: Hook, Validator, Index"
description: "Normative contract for the docs validator, index generator, and git pre-commit hook the installer must produce"
---

# Install Contract (APS-V1-0003): Hook + Validator + Index

> **Status: forward specification, not yet implemented.** This document specifies the hook install contract. The `install`, `uninstall`, and `hook` subcommands are a forward specification and are NOT yet implemented by the handler, which currently provides `validate` and `index` only. The rest of this document is the design of record for the planned installer.

This document is the normative companion to `01_spec.md`. It defines the install entry point, the validator API, the index generator API, and the git pre-commit hook that ties them together. The working installer is a fast-follow PR; this document specifies what that installer MUST build.

The contract has four parts:

1. The install entry point.
2. The validator API and its diagnostics.
3. The index generator API.
4. The git pre-commit hook that wraps both.

> **Why a contract instead of an implementation:** a sharp contract is what lets the validator, the index generator, and the hook be developed and tested independently, lets CI invoke the same logic the hook does, and lets downstream tooling (vector indexers, doc search, semantic lookups) trust the structure they read.

---

## 1. Install Entry Point

```
apss run documentation install   [<repo-root>] [--force] [--no-config]
apss run documentation uninstall [<repo-root>]
```

### 1.1 `install` semantics

The installer MUST be idempotent. Running it twice MUST be equivalent to running it once.

Steps, in order:

1. **Resolve target.** If `<repo-root>` is omitted, use `git rev-parse --show-toplevel`. Fail with `install-no-git-root` if not in a git repository.
2. **Ensure the `docs` block in `apss.yaml`.** Configuration lives in a single root-level `apss.yaml` owned by the meta-standard (APS-V1-0000.CF01); this standard contributes only the `docs:` block. If `apss.yaml` does not exist, delegate creation to the CF01 installer. If `apss.yaml` exists, MUST NOT overwrite it; instead, add a `docs:` block populated with the Section 3.3 defaults when one is absent and otherwise leave the existing block untouched. `--force` MAY rewrite the `docs:` block but MUST back up `apss.yaml` to `apss.yaml.bak.<timestamp>` first. `--no-config` skips this step entirely.
3. **Install the pre-commit hook.** Write `.git/hooks/pre-commit` (mode `0755`). If the hook file does not exist, create it with the apss block as its only content. If it exists:
   - The hook MUST insert a block delimited by the sentinels:
     ```
     # >>> apss-docs-hook >>>
     apss run documentation hook --staged || exit $?
     # <<< apss-docs-hook <<<
     ```
   - The block MUST be placed at the end of any existing `#!` shebang block and before any user-defined hook body, so that a user hook that exits early does not skip APSS validation.
   - Re-running the installer MUST replace the existing apss block in place rather than appending a duplicate. Detection is by sentinel match.
4. **Materialise template files for active doc types.** Every active doc type substandard MAY ship a set of starter files (directory READMEs, agent-context files, document templates). For each templated file, the installer MUST create the destination file only when it is missing and MUST NOT overwrite an existing target file under any circumstance. If the destination directory is absent, create it first. `--force` does NOT change this rule for template files: an existing file is always preserved. The installer MUST emit `install-template-conflict` (warning) for each existing file it skipped, naming both the template and the target so the operator can compare manually. See Section 1.4 for the per-substandard template inventory.
5. **Print the resolved doc type registry and the ACTION REQUIRED banner.** After install completes, the CLI MUST print a one-line summary of every active doc type, its resolved location, and the templates it materialised, so the operator immediately sees what just became enforced. The CLI MUST ALSO print an "ACTION REQUIRED" block listing every file required by the standard's validator that the installer is forbidden from scaffolding (per Section 1.5 and the DOC03 self-reference rule), with a one-line content suggestion the operator can copy or adapt. As of this contract that list is the repository root `AGENTS.md` and `CLAUDE.md`; the banner MUST recommend writing a short orientation paragraph that names APSS, the docs root, every active doc-type location, and the backlink rule. The banner is printed regardless of whether the files already exist, so the operator is reminded of the DOC03 self-reference content rules even when the files exist but lack the required references.
6. **Exit code.** `0` on success, `2` on any unrecoverable install error. Diagnostics MUST use the human readable scheme.

### 1.2 `uninstall` semantics

`uninstall` MUST:

- Locate the pre-commit hook and remove the entire `# >>> apss-docs-hook >>>` to `# <<< apss-docs-hook <<<` block, including the sentinels.
- Leave the rest of `.git/hooks/pre-commit` intact.
- Leave `apss.yaml` and its `docs:` block intact (config is the operator's, not the installer's).
- Be a no-op when the sentinels are not present.

### 1.3 Install-related diagnostics

| Code | Severity | Description |
|------|----------|-------------|
| `install-no-git-root` | error | The target path is not inside a git repository. |
| `install-hook-write-failed` | error | Could not write `.git/hooks/pre-commit`. |
| `install-config-conflict` | error | `apss.yaml` exists with a `docs:` block and `--force` was not specified. |
| `install-template-conflict` | warning | A template file was skipped because the target already exists. The target path and the template path MUST appear in the message so the operator can reconcile manually. |
| `install-template-write-failed` | error | Could not write a template file the installer attempted to create. |

### 1.4 Template inventory per active doc type

Each substandard's templates ship inside the substandard's crate at
`templates/<relative-target-path>` and are materialised into the
target repository at the corresponding path **after the docs-root
rewrite described below.**

**Template path resolution under custom `docs.root` and
`docs.<slug>.directory` (normative).** Templates are stored on disk
with the literal default path (e.g. ADR01 ships
`templates/docs/adrs/AGENTS.md`) so the source tree is
self-explanatory. On install, the installer MUST rewrite the leading
components of the template's relative path as follows:

1. Strip the leading `docs/` segment that matches the parent
   standard's default `docs.root` value, and replace it with the
   resolved `<docs.root>` value from `apss.yaml`.
2. For substandards that contribute a directory key
   (`docs.adr.directory`, `docs.retrospectives.directory`, etc.),
   strip the substandard's default directory segment (e.g. `adrs/`
   for ADR01) and replace it with the resolved configured value.
3. Any further sub-path segments after the substandard's directory
   prefix are preserved verbatim.

So a project that sets `docs.root: documentation` and
`docs.adr.directory: decisions` receives the ADR templates at
`documentation/decisions/README.md`,
`documentation/decisions/AGENTS.md`, etc. A project that leaves the
defaults receives `docs/adrs/README.md` and so on. The "verbatim"
clause in earlier drafts applied to **file content**, not to the
path; this section is the authoritative path rule.

`AGENTS.md` is the canonical agent context file. `CLAUDE.md` ships as a
regular file whose bytes are identical to the adjacent `AGENTS.md`.
Gemini reads `AGENTS.md` natively, so this standard ships NO
`GEMINI.md`; agents that want a separate Gemini context file are out of
scope and MUST NOT be added by the installer.

The standard's templates MUST NOT contain symlinks, and the installer
MUST NOT create one, on any platform. There is no OS-conditional
branch: the behaviour is identical on Linux, macOS, and Windows. The
reason is that the hazard is at clone time, not install time - see
parent spec Section 6.4. A `CLAUDE.md` that is an `@AGENTS.md` import
stub is likewise non-conformant.

The shipped inventory at the time of this contract:

- **APS-V1-0003 parent (docs-root bootstrap)** ships, relative to the
  resolved `docs.root`:
  - `README.md` - the docs-root README with a placeholder
    `## Subdirectories` block (linked under the docs root) and the
    placeholder `## Index` section that the index generator
    overwrites on first hook run.
  - `AGENTS.md` - canonical agent-context file for the docs root,
    pointing at the README and naming the active doc-type
    directories.
  - `CLAUDE.md` (byte-identical copy of `AGENTS.md`).

  These three templates exist so a fresh adopter does not trip
  `readme-missing` (error) at the docs root on the first commit
  (Reviewer finding A4). They are scaffolded once on first install
  per the create-if-missing rule in Section 1.5 and are never
  touched afterwards.

- **APS-V1-0003.AD01 (Architecture Decision Records)** ships, relative
  to the ADR directory resolved from `docs.adr.directory`:
  - `README.md` - directory README summarising what an ADR is, when to
    write one, the lifecycle (`status` field), and the project naming
    convention.
  - `AGENTS.md` - the canonical agent-context block for the ADR
    directory: where ADRs live, when to use one, the parent-level
    backlink rule, and references back to the ADR01 substandard spec.
  - `CLAUDE.md` (byte-identical copy of `AGENTS.md`) - so Claude Code
    reads the AGENTS.md content on every platform.
  - `ADR-000-template.md.example` - Nygard-style template with the
    required frontmatter (`name`, `description`, `status`) and the
    `## Context`, `## Decision`, `## Consequences` sections. The
    installer MUST materialise this with the literal `.example`
    suffix so the ADR01 naming validator skips it (see ADR01 spec
    Section 2 exclusion rule) and the parent indexer omits it from
    the `## Index` table. A project that wants to copy the template
    renames it to `ADR-<NNN>-<slug>.md` and fills in the frontmatter.

- **APS-V1-0003.PV01 (North Star: Mission, Vision, Position)** and
  **APS-V1-0003.RT01 (Retrospectives)** MAY ship their own
  templates following the same convention; the shipped inventory for
  these is documented in their respective substandard specs and is
  out of scope for the install-contract surface beyond the rule "copy
  what the substandard ships at `templates/`, skip on conflict".

The repository root `AGENTS.md` and root `CLAUDE.md` are NOT shipped
as templates and the installer MUST NOT scaffold them. Their content
is project-specific per Section 1.5 and the operator's Correction 2.
The DOC03 root-context diagnostics are emitted at **warning**
severity (parent spec Section 6.1, 6.2) so a missing root file does
not block the first commit; install step 5 prints an "ACTION
REQUIRED" banner naming the missing root files instead.

All substandard templates MUST live under `<substandard-crate>/templates/`
inside the standard package, version-controlled alongside the
substandard's spec, so the installer ships a single coherent bundle.
The parent's docs-root bootstrap templates live under the parent
crate's `templates/`.

### 1.5 AGENTS.md and CLAUDE.md scaffolding (create-if-missing, never-overwrite)

This is a normative contract rule, broken out of Section 1.4 because
the operator surface depends on it being unambiguous.

**Scope.** This rule covers every `AGENTS.md` and every adjacent
`CLAUDE.md` copy under the docs root that the standard's templates
target. The shipped templates today are the docs-area files at
`docs/adrs/AGENTS.md` and `docs/adrs/CLAUDE.md` (ADR01 substandard);
future substandards add to this list through Section 1.4.

**Scaffold when absent.** When the installer runs and the target
`AGENTS.md` does not exist, the installer MUST actively scaffold it
from the substandard's template, including the explanatory context the
template carries (for ADR01: what ADRs are, where they live, when to
write one, the backlink rule, and a reference back to the ADR01
substandard spec). The installer MUST also create the adjacent
`CLAUDE.md` by copying the just-created `AGENTS.md` byte for byte, as a
regular file, on every platform. This is a real install step, not a
validation warning.

**The four presence states (normative).** The generic template rule in
Section 1.1 step 4 ("create the destination only when missing") is not
sufficient for this pair, because a `CLAUDE.md` created from the
*shipped* template beside a project-customised `AGENTS.md` is divergent
the instant it is written. The installer MUST therefore special-case
the pair:

| On disk | Installer action |
|---|---|
| Both `AGENTS.md` and `CLAUDE.md` present | Create neither. Emit `install-template-conflict` (warning) per file. Do NOT reconcile divergence; that is `claude-md --fix`'s job, not the installer's. |
| `AGENTS.md` present, `CLAUDE.md` absent | Create `CLAUDE.md` as a byte-identical copy of the **on-disk adjacent `AGENTS.md`**, NEVER from the shipped `CLAUDE.md` template. Emit `install-template-conflict` for the skipped `AGENTS.md` only. |
| `AGENTS.md` absent, `CLAUDE.md` present | Scaffold `AGENTS.md` from the shipped template. Do NOT touch the existing `CLAUDE.md` - never-overwrite applies. The pair MAY then be divergent - the existing `CLAUDE.md` could already equal the shipped template and be mode `100644` - so the validator, not the installer, decides; the installer MUST print a line naming `apss run documentation claude-md --fix` as the repair. |
| Both absent | Scaffold `AGENTS.md` from the shipped template, then create `CLAUDE.md` as a byte-identical copy of the `AGENTS.md` it just wrote. Copying the file it wrote, rather than the shipped `CLAUDE.md` template, keeps the two identical by construction even if the two shipped templates ever drift apart. |

In every row where the installer creates `CLAUDE.md`, the source is
the adjacent `AGENTS.md` **on disk**, never the shipped template. That
single rule is what keeps the pair identical by construction wherever
the installer is the one creating it. It does NOT make all four states
convergent: the installer deliberately leaves an existing pair
unreconciled, so divergence is the validator's to report and
`claude-md --fix`'s to repair.

**Never overwrite.** When the target `AGENTS.md` already exists, the
installer MUST NOT overwrite or modify it. Full stop. `--force` does
not change this rule. The existing file MAY have different content
from the standard template; that is the project's business and is
not the installer's call to reconcile. The same rule applies to an
existing `CLAUDE.md`, whatever form it takes on disk. Divergence
between an existing `AGENTS.md` and an existing `CLAUDE.md` is not the
installer's problem either: it is reported by the validator as
`claude-md-divergent` (error, parent spec Section 6.4) and repaired by
`apss run documentation claude-md --fix`, never by the installer overwriting a
file it did not create. The installer MUST emit
`install-template-conflict` (warning) for each existing file it
skipped, naming both the template and the target so the operator can
diff them manually.

**Two different comparisons, and only one of them is forbidden.** The
original "existence only" wording predates the copy rule and, read
literally today, would forbid the equality check that parent spec
Section 6.4 mandates. It does not. The two comparisons are distinct
and are governed separately:

1. **File vs. shipped template - FORBIDDEN.** The validator MUST NOT
   compare any on-disk `AGENTS.md` or `CLAUDE.md` against the content
   this standard ships in `templates/`. An `AGENTS.md` that has been
   rewritten entirely by the project passes validation as long as it
   exists and carries valid frontmatter per the parent indexing rules
   (Section 4 of `01_spec.md`). Drift from the template is the
   project's business. The installer's `install-template-conflict`
   warning is the only surface for it.
2. **`CLAUDE.md` vs. its adjacent `AGENTS.md` - REQUIRED.** Whenever
   both files exist, the validator MUST compare them byte for byte and
   MUST emit `claude-md-divergent` (error) when they differ or when
   `CLAUDE.md` is not a regular file committed as mode `100644`. This
   is not a template comparison: it involves no shipped content at
   all, only two files the project already owns, and it has exactly
   one correct repair. Parent spec Section 6.4 is normative.

For `AGENTS.md` alone, existence and well-formed frontmatter remain
the whole of the check.

**Root context files (DOC03) stay project-specific.** The repository
root `AGENTS.md` and root `CLAUDE.md` carry project-specific
orientation and are owned by the project, not the standard. The
parent's DOC03 self-reference check (`01_spec.md` Section 6.3)
verifies only that the root files reference APSS, the docs root, and
the active doc type locations; the standard does not ship a template
for the root `AGENTS.md` and the installer MUST NOT scaffold it. The
substandard-supplied docs-area `AGENTS.md` files (for example
`docs/adrs/AGENTS.md`) are the ones that carry the explanatory context
about ADRs, doc types, and the backlink rule.

---

## 2. Validator API

The validator is the single source of truth. The CLI, the hook, and any third party tool MUST call the same entry point with the same arguments and get the same diagnostics.

### 2.1 Public function

```
fn validate(repo_root: &Path, config: &ApssConfig, scope: ValidationScope) -> ValidationReport;
```

### 2.2 Input: `ValidationScope`

```
enum ValidationScope {
    Full,
    Changed { staged_paths: Vec<PathBuf> },
}
```

- `Full`: walk the entire docs root and every active doc type directory. Used by `apss run documentation validate` and by CI.
- `Changed`: only inspect docs touched by `staged_paths`. The hook MUST use this scope. The validator MUST still load enough surrounding state (for example, the doc type directories themselves) to detect dead backlinks introduced by the change set.

When `scope = Changed` and the staged set contains an `apss.yaml` modification, the validator MUST run the `Full` set of checks; config changes can invalidate the entire tree.

### 2.3 Output: `ValidationReport`

```
struct ValidationReport {
    diagnostics: Vec<Diagnostic>,
    summary: Summary,
    machine_readable: serde_json::Value,
}

struct Diagnostic {
    code: String,         // e.g. "ADR01-dir-not-found"
    severity: Severity,   // Error or Warning
    path: Option<PathBuf>,
    line: Option<u32>,
    message: String,
    hint: Option<String>, // one-liner with the recommended fix
}

struct Summary {
    errors: u32,
    warnings: u32,
}
```

The `machine_readable` field MUST contain the same content as `diagnostics`/`summary`, rendered as stable JSON. The JSON keys MUST be the human readable diagnostic codes (Section 10 of `01_spec.md`). Numeric aliases MAY appear in a side-by-side `legacy_codes` map but MUST NOT be the primary key.

### 2.4 Exit behavior

- `apss run documentation validate` exits `0` iff `summary.errors == 0`. Warnings do not cause a non-zero exit.
- A panic, uncaught IO error, or regex compile failure on a built-in pattern MUST be reported as `validator-internal-error` (error severity) and MUST result in a non-zero exit. The validator MUST NOT exit `0` after eating an internal error.

### 2.5 What "valid structure" means

The validator MUST enforce, for each active doc type:

- **ADR (`APS-V1-0003.AD01`)**: directory exists, every non-`.example` file matches the configured naming regex, every ADR has the required frontmatter and `status` (with the per-doc-type lifecycle vocabulary from Section 8.1 of `01_spec.md`: ADRs use `accepted` for the in-force value), required topic keywords are satisfied, context files exist, every `ADR-NNN-...` token (3 to 5 digit number) found in the file set defined by `docs.backlinking.scan` (defaults documented in Section 3.3 of `01_spec.md`; deprecated `file_types` honoured with a `backlinking-file-types-deprecated` warning) resolves to a real ADR file in `docs.adr.directory` whose name satisfies `docs.adr.naming_pattern` (diagnostic: `ADR01-unknown-reference`, error), and resolved references to ADRs with `status: superseded` are flagged as `ADR01-superseded-reference` (warning, hint naming the `superseded_by` target) and references to `status: deprecated` ADRs are flagged as `ADR01-deprecated-reference` (warning, hint suggesting retarget or annotate as intentional). See Section 7.2 of `01_spec.md` for the reference-accuracy contract.
- **North Star (`APS-V1-0003.PV01`)**: a single document exists at the configured location with required frontmatter, `## Mission`, `## Vision`, `## Position`, and a current `status`.
- **Retrospectives (`APS-V1-0003.RT01`)**: directory exists, each file matches the naming regex, files are append only (`Changed` scope: no historical retro file appears in the staged diff with content modifications outside the appended sections), and required sections are present.

For every doc type, the validator MUST also enforce the parent rules: frontmatter present and well formed, README index present and up to date, per directory context files present.

**Context files: no template comparison, but a required adjacent-file
comparison.** Per Section 1.5, the validator MUST verify that the
substandard's docs-area `AGENTS.md` and the adjacent `CLAUDE.md` exist
at the configured locations, and:

- The validator MUST NOT compare either file's content against the
  substandard's **shipped template**. A project that has authored its
  own `AGENTS.md` for a docs subdirectory passes as long as the file is
  present and carries valid frontmatter per Section 4 of `01_spec.md`.
  The installer's `install-template-conflict` warning is the only
  surface for template drift; the validator stays silent on it.
- The validator MUST compare `CLAUDE.md` against **its adjacent
  `AGENTS.md`** byte for byte whenever both exist, and MUST emit
  `claude-md-divergent` (error) on any mismatch, on a non-regular
  file, or on a git index mode other than `100644`. See parent spec
  Section 6.4 for the condition, the severity rationale, and the
  repair algorithm.

The distinction is the whole point: the standard does not own what
`AGENTS.md` says, but it does own the requirement that `CLAUDE.md`
says exactly the same thing.

> **Not yet implemented.** As with the rest of this document,
> `claude-md-divergent` is a forward specification. No shipped code
> emits it today.

---

## 3. Index Generator API

### 3.1 Public function

```
fn generate(repo_root: &Path, config: &ApssConfig, dirs: &[PathBuf], mode: GeneratorMode)
    -> GeneratorReport;

enum GeneratorMode {
    DryRun,
    Write,
}
```

### 3.2 Output

```
struct GeneratorReport {
    files: Vec<(PathBuf, String)>, // (README.md path, new content)
    diagnostics: Vec<Diagnostic>,  // e.g. index-write-failed
}
```

### 3.3 Determinism

- For a given `(repo_root, config, dirs)` tuple, `generate` MUST be deterministic. Two consecutive runs MUST produce byte identical `files` content.
- `DryRun` and `Write` MUST produce the same `files[*].1` (content) for the same inputs.
- The validator's `index-stale` check MUST be implemented as `generate(DryRun).files[i].1 != fs::read_to_string(files[i].0)`. There MUST NOT be a separate "is stale?" implementation that can drift.

### 3.4 Empty directories

When a docs directory has no indexable `.md` siblings, the generator MUST still emit a stable index placeholder (default: `## Index\n\n_No indexable documents in this directory yet._\n`). Dry run and write MUST produce that placeholder identically. The validator MUST treat the placeholder as a valid index (no `index-missing`, no `index-stale`).

### 3.5 Exit behavior

- `apss run documentation index` (dry run) exits `0` regardless of whether content would change, as long as no file read fails.
- `apss run documentation index --write` exits `0` when every write succeeds, even if no file actually changed. A write failure MUST emit `index-write-failed` and exit non zero.

---

## 4. Git Pre-Commit Hook Contract

The hook is the operator facing surface of the install. Its job is to keep indexes fresh and the doc structure valid at every commit.

### 4.1 Entry point

```
apss run documentation hook --staged
```

The installed `.git/hooks/pre-commit` block MUST do nothing more than call this command and forward its exit code. The hook's logic lives in the Rust binary so it can be tested and version controlled.

### 4.2 Steps (normative)

0. **Honour `docs.disable: true`.** If the resolved `docs` block sets
   `disable: true`, the hook MUST short-circuit immediately, print a
   single-line "docs validation disabled via `docs.disable: true`"
   notice to stderr, and exit `0` WITHOUT running the index
   generator (step 3) or the validator (step 4). The kill switch
   covers both the read side (validation) and the write side
   (index regeneration) so a migration window the operator wanted
   quiet does not silently rewrite `README.md` files.
1. **Resolve scope.** `repo_root = git rev-parse --show-toplevel`; `staged = git diff --cached --name-only --diff-filter=ACMR`. If `repo_root` is missing, exit `2` with `hook-not-in-repo`.
2. **Load config.** If `apss.yaml` fails to load, emit `invalid-apss-yaml` and exit `2`. The hook MUST NOT proceed with defaults when the config file exists but is malformed; the operator should fix it before committing. (The hook reads the `docs` block out of the file the meta-validator already cascade-resolved.)
3. **Refresh indexes.** Compute the **set of docs directories
   whose contents appear in `staged`**, defined as the parent
   directory of every staged path that lies under `docs.root`, plus
   `docs.root` itself when any path under it is staged. Call the
   index generator with `mode = Write` for that set. For each
   rewritten `README.md`, the hook MUST run `git add <path>` so the
   regenerated index is part of the commit. If a write fails,
   exit `2`. See Section 4.3 for the `git commit -p` interaction.
4. **Reconcile `CLAUDE.md`.** Call `claude-md --fix` (parent spec
   Section 6.4.3) over the directories in scope. For every `CLAUDE.md`
   it replaces, the hook MUST stage the result as a `100644` blob and
   MUST print the discarded diff and the "you meant to edit
   `AGENTS.md`" notice required by Section 6.4.3 - never a summary
   line, never suppressed by a quiet flag. Where `AGENTS.md` exists
   and `CLAUDE.md` is absent, `--fix` creates `CLAUDE.md` from the
   on-disk `AGENTS.md`. Where `AGENTS.md` is absent, `--fix` does
   nothing. If a replacement or its verification fails, exit `2` with
   `hook-claude-md-fix-failed`. This step runs BEFORE validation so
   the validator sees the repaired tree and the commit is
   self-consistent.
5. **Validate.** Call `validate(repo_root, config, Changed { staged_paths: staged })`.
6. **Report.** Print all error and warning diagnostics in human readable form (color when TTY, plain otherwise). When stdout is being piped, also write the `machine_readable` JSON to a temporary file referenced in the human output, so CI can pick it up.
7. **Exit.**
   - `0` when `summary.errors == 0` (warnings allowed).
   - `1` when `summary.errors > 0`.
   - `2` for any internal hook error (config load failure, index write failure, `claude-md --fix` failure, missing `apss` binary).

The hook is convenience, not the guarantee. A fresh clone has no hook
until `install` runs, `--no-verify` bypasses it, and a CI checkout has
no hook at all. Adopters MUST additionally run
`apss run documentation claude-md --check` in CI; that run is what
actually holds the Section 6.4 invariant.

### 4.3 Concurrency and recursion

- The hook MUST be safe to run from `git commit -p` and from inside an interactive rebase. It MUST NOT call `git commit` itself.
- The hook MUST NOT call itself recursively. Re-staging `README.md` files (step 3) MUST use `git add`, not `git commit`.
- The hook MUST tolerate a missing `apss` binary by exiting `2` with `hook-missing-apss` rather than blocking with a cryptic shell error.

**`git commit -p` interaction (normative).** Step 3 re-stages
regenerated `README.md` files unconditionally so the commit's index
state matches the validator's view of the docs tree. Under `git
commit -p` this means a regenerated docs README is **always**
included in the commit even when the operator did not explicitly
stage the hunk. The hook MUST print one line per re-staged file in
the form `regenerated and added: <path>` so an operator using `-p`
can `git restore --staged <path>` before completing the commit. The
auto-add is intentional: index drift inside a commit that touched
docs is a worse default than a one-line banner, and the standard's
core contract is "the index is correct at every commit". Operators
who want strict `-p` semantics for indexes can set
`docs.index.disable: true` for the affected commit and re-enable
afterward.

### 4.4 Escape hatches

- `git commit --no-verify` continues to skip the hook entirely. This is a human operator escape hatch. The standard MUST NOT teach agents to use `--no-verify`.
- Setting `docs.disable: true` in `apss.yaml` is the supported way to keep the hook installed but silent for a temporary period (for example, during a large migration). Per Section 4.2 step 0 the kill switch short-circuits the hook (no index regeneration, no validation, exit 0) so the migration is genuinely quiet.

### 4.5 Hook diagnostics

| Code | Severity | Description |
|------|----------|-------------|
| `hook-not-in-repo` | error | `git rev-parse --show-toplevel` failed. |
| `hook-missing-apss` | error | The `apss` binary is not on `PATH`. |
| `hook-staged-rewrite-failed` | error | A `git add` for a regenerated index failed. |
| `hook-claude-md-fix-failed` | error | `claude-md --fix` could not replace a `CLAUDE.md`, or the post-write index verification did not report mode `100644` with the `AGENTS.md` blob hash. |

These are emitted in addition to the validator and generator diagnostics above; the hook is just the runner.

---

## 5. End-to-end Example

A typical commit flow with the standard installed:

1. Operator edits `docs/adrs/ADR-001-security.md` and commits.
2. Pre-commit hook fires `apss run documentation hook --staged`.
3. The hook regenerates `docs/adrs/README.md` (the index), `git add`s it.
4. The hook runs `claude-md --fix`. `docs/adrs/CLAUDE.md` is already byte-identical to `docs/adrs/AGENTS.md`, so nothing changes and nothing is printed.
5. The hook runs the validator in `Changed` scope. ADR01 checks pass. Backlink checks see no new dangling references. Frontmatter and `status` are valid.
6. The hook prints a one-line success banner and exits `0`. The commit completes with the regenerated index included.

If the operator instead edited `docs/adrs/CLAUDE.md` (the derived file)
rather than `docs/adrs/AGENTS.md`:

1. The hook regenerates the index as usual.
2. `claude-md --fix` sees the pair diverge. It reads `AGENTS.md`, unlinks `CLAUDE.md`, writes a fresh `100644` regular file with the `AGENTS.md` bytes, stages it, and re-reads the index to confirm the mode and blob hash.
3. It prints the full diff of what it discarded, names `docs/adrs/CLAUDE.md` as the file overwritten and `docs/adrs/AGENTS.md` as the canonical file the operator meant to edit, and does not abbreviate either.
4. Validation runs against the repaired tree and passes. The commit completes, containing the operator's *unintended* edit nowhere - which is why step 3 is loud.

If the operator instead saved an ADR without a `status` field:

1. The hook regenerates the index (best-effort still).
2. The validator emits `ADR01-status-missing` (error).
3. The hook exits `1`, the commit is blocked, the diagnostic includes the file path and a hint to add `status: proposed`.

---

## 6. Out of scope for this PR

This document is the contract. The actual installer, hook binary, and CLI sub-commands ship in a follow-up PR. Reviewers should evaluate this document for completeness and tightness of the contract, not for the presence of working code.

## 7. Cross references

- Parent spec: [`01_spec.md`](01_spec.md).
- Diagnostic code scheme: Section 10 of `01_spec.md`.
- Doc type registry: Section 8 of `01_spec.md`.
- Substandards: [`../substandards/AD01-architecture-decision-records`](../substandards/AD01-architecture-decision-records), [`../substandards/PV01-purpose-and-vision`](../substandards/PV01-purpose-and-vision), [`../substandards/RT01-retrospectives`](../substandards/RT01-retrospectives).
