---
name: "Install Contract: Hook, Validator, Index"
description: "Normative contract for the docs validator, index generator, and git pre-commit hook the installer must produce"
---

# Install Contract (EXP-V1-0004): Hook + Validator + Index

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
aps run docs install   [<repo-root>] [--force] [--no-config]
aps run docs uninstall [<repo-root>]
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
     aps run docs hook --staged || exit $?
     # <<< apss-docs-hook <<<
     ```
   - The block MUST be placed at the end of any existing `#!` shebang block and before any user-defined hook body, so that a user hook that exits early does not skip APSS validation.
   - Re-running the installer MUST replace the existing apss block in place rather than appending a duplicate. Detection is by sentinel match.
4. **Materialise template files for active doc types.** Every active doc type substandard MAY ship a set of starter files (directory READMEs, agent-context files, document templates). The installer MUST copy these into the target repository for each active doc type whose target directory either does not exist or is empty of the substandard's templated files. Copying MUST NOT overwrite an existing target file under any circumstance. `--force` does NOT change this rule for template files: an existing file is always preserved. The installer MUST emit `install-template-conflict` (warning) for each existing file it skipped, naming both the template and the target so the operator can compare manually. See Section 1.4 for the per-substandard template inventory.
5. **Print the resolved doc type registry.** After install completes, the CLI MUST print a one-line summary of every active doc type, its resolved location, and the templates it materialised, so the operator immediately sees what just became enforced.
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
`templates/<relative-target-path>` and are materialised verbatim into the
target repository at the corresponding path under the docs root. Symlinks
in the source tree (for example `CLAUDE.md` and `GEMINI.md` symlinks to
`AGENTS.md` in the ADR templates) are preserved on filesystems that
support symlinks; on Windows the installer MUST instead copy the link
target's contents.

The shipped inventory at the time of this contract:

- **EXP-V1-0004.ADR01 (Architecture Decision Records)** ships, relative
  to the ADR directory resolved from `docs.adr.directory`:
  - `README.md` - directory README summarising what an ADR is, when to
    write one, the lifecycle (`status` field), and the project naming
    convention.
  - `AGENTS.md` - the canonical agent-context block for the ADR
    directory: where ADRs live, when to use one, and the parent-level
    backlink rule.
  - `CLAUDE.md` (symlink to `AGENTS.md`) - so Claude Code reads the
    same context.
  - `GEMINI.md` (symlink to `AGENTS.md`) - so Gemini reads the same
    context.
  - `ADR-000-template.md` - Nygard-style template with the required
    frontmatter (`name`, `description`, `status`) and the `## Context`,
    `## Decision`, `## Consequences` sections.

- **EXP-V1-0004.PV01 (North Star: Mission, Vision, Position)** and
  **EXP-V1-0004.RETRO01 (Retrospectives)** MAY ship their own
  templates following the same convention; the shipped inventory for
  these is documented in their respective substandard specs and is
  out of scope for the install-contract surface beyond the rule "copy
  what the substandard ships at `templates/`, skip on conflict".

All substandard templates MUST live under `<substandard-crate>/templates/`
inside the standard package, version-controlled alongside the
substandard's spec, so the installer ships a single coherent bundle.

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

- `Full`: walk the entire docs root and every active doc type directory. Used by `aps run docs validate` and by CI.
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

- `aps run docs validate` exits `0` iff `summary.errors == 0`. Warnings do not cause a non-zero exit.
- A panic, uncaught IO error, or regex compile failure on a built-in pattern MUST be reported as `validator-internal-error` (error severity) and MUST result in a non-zero exit. The validator MUST NOT exit `0` after eating an internal error.

### 2.5 What "valid structure" means

The validator MUST enforce, for each active doc type:

- **ADR (`EXP-V1-0004.ADR01`)**: directory exists, every file matches the configured naming regex, every ADR has the required frontmatter and `status`, required topic keywords are satisfied, context files exist, no dead or superseded backlinks.
- **Purpose and Vision (`EXP-V1-0004.PV01`)**: a single document exists at the configured location with required frontmatter, `## Purpose`, `## Vision`, `## Non-Goals`, and a current `status`.
- **Retrospectives (`EXP-V1-0004.RETRO01`)**: directory exists, each file matches the naming regex, files are append only (`Changed` scope: no historical retro file appears in the staged diff with content modifications outside the appended sections), and required sections are present.

For every doc type, the validator MUST also enforce the parent rules: frontmatter present and well formed, README index present and up to date, per directory context files present.

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

- `aps run docs index` (dry run) exits `0` regardless of whether content would change, as long as no file read fails.
- `aps run docs index --write` exits `0` when every write succeeds, even if no file actually changed. A write failure MUST emit `index-write-failed` and exit non zero.

---

## 4. Git Pre-Commit Hook Contract

The hook is the operator facing surface of the install. Its job is to keep indexes fresh and the doc structure valid at every commit.

### 4.1 Entry point

```
aps run docs hook --staged
```

The installed `.git/hooks/pre-commit` block MUST do nothing more than call this command and forward its exit code. The hook's logic lives in the Rust binary so it can be tested and version controlled.

### 4.2 Steps (normative)

1. **Resolve scope.** `repo_root = git rev-parse --show-toplevel`; `staged = git diff --cached --name-only --diff-filter=ACMR`. If `repo_root` is missing, exit `2` with `hook-not-in-repo`.
2. **Load config.** If `apss.yaml` fails to load, emit `invalid-apss-yaml` and exit `2`. The hook MUST NOT proceed with defaults when the config file exists but is malformed; the operator should fix it before committing. (The hook reads the `docs` block out of the file the meta-validator already cascade-resolved.)
3. **Refresh indexes.** Compute the set of docs directories whose contents appear in `staged`. Call the index generator with `mode = Write` for that set. For each rewritten `README.md`, the hook MUST run `git add <path>` so the regenerated index is part of the commit. If a write fails, exit `2`.
4. **Validate.** Call `validate(repo_root, config, Changed { staged_paths: staged })`.
5. **Report.** Print all error and warning diagnostics in human readable form (color when TTY, plain otherwise). When stdout is being piped, also write the `machine_readable` JSON to a temporary file referenced in the human output, so CI can pick it up.
6. **Exit.**
   - `0` when `summary.errors == 0` (warnings allowed).
   - `1` when `summary.errors > 0`.
   - `2` for any internal hook error (config load failure, index write failure, missing `aps` binary).

### 4.3 Concurrency and recursion

- The hook MUST be safe to run from `git commit -p` and from inside an interactive rebase. It MUST NOT call `git commit` itself.
- The hook MUST NOT call itself recursively. Re-staging `README.md` files (step 3) MUST use `git add`, not `git commit`.
- The hook MUST tolerate a missing `aps` binary by exiting `2` with `hook-missing-aps` rather than blocking with a cryptic shell error.

### 4.4 Escape hatches

- `git commit --no-verify` continues to skip the hook entirely. This is a human operator escape hatch. The standard MUST NOT teach agents to use `--no-verify`.
- Setting `docs.disable: true` in `apss.yaml` is the supported way to keep the hook installed but silent for a temporary period (for example, during a large migration).

### 4.5 Hook diagnostics

| Code | Severity | Description |
|------|----------|-------------|
| `hook-not-in-repo` | error | `git rev-parse --show-toplevel` failed. |
| `hook-missing-aps` | error | The `aps` binary is not on `PATH`. |
| `hook-staged-rewrite-failed` | error | A `git add` for a regenerated index failed. |

These are emitted in addition to the validator and generator diagnostics above; the hook is just the runner.

---

## 5. End-to-end Example

A typical commit flow with the standard installed:

1. Operator edits `docs/adrs/ADR-001-security.md` and commits.
2. Pre-commit hook fires `aps run docs hook --staged`.
3. The hook regenerates `docs/adrs/README.md` (the index), `git add`s it.
4. The hook runs the validator in `Changed` scope. ADR01 checks pass. Backlink checks see no new dangling references. Frontmatter and `status` are valid.
5. The hook prints a one-line success banner and exits `0`. The commit completes with the regenerated index included.

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
- Substandards: [`../substandards/ADR01-architecture-decision-records`](../substandards/ADR01-architecture-decision-records), [`../substandards/PV01-purpose-and-vision`](../substandards/PV01-purpose-and-vision), [`../substandards/RETRO01-retrospectives`](../substandards/RETRO01-retrospectives).
