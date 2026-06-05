# APS-V1-0000.CF01: Unified Install Seam (Normative)

**Version**: 1.0.0
**Status**: Active
**Parent**: APS-V1-0000.CF01 (Project Configuration)

Sibling normative spec to `01_spec.md` and the other CF01 sibling
specs. Equal precedence under APS-V1-0000 §1.1.

This document specifies the CF01 side of the unified installer
introduced by Addendum 1 of the operator brief (2026-06-04 22:47):

> The unification extends to THREE concerns, not two: configuration
> plus distribution plus installation are one system. APSS.yaml is
> the MANIFEST, not just settings.

DI01 owns the resolve and publish ends; CF01 owns the manifest;
the installer is the glue. The DI01 side is specified in
`standards/v1/APS-V1-0000-meta/substandards/DI01-distribution/docs/02_unified_install_seam.md`.

## Terminology

RFC 2119 keywords apply. The placeholder `<bootstrap>` is used in CLI
examples while repo issue 64 (APS vs APSS naming) is open.

---

## 1. APSS.yaml as Manifest

APSS.yaml plays the role of an npm `package.json`'s `dependencies`
field for the APSS ecosystem. CF01 owns the manifest surface; DI01
owns version resolution and lockfile production.

The manifest is the union of:

1. **Activation declarations.** Every standard whose slug appears as
   a key in APSS.yaml (active or disabled) is part of the project's
   declared set. Active standards that have no section are still in
   the declared set: the default-on philosophy (`01_spec.md` §5)
   means active membership is inferred from the resolved standards
   list, not from the presence of a section.
2. **Per-standard configuration.** The contents of each slug section
   (see `03_contribution_schema.md`).
3. **Version constraints.** Optional `version:` Cargo-style semver
   requirements per standard (`03_contribution_schema.md` §3.1).

How the "declared set" is computed is intentionally explicit:

```
declared_set = explicit_keys(APSS.yaml) UNION baseline_active_set
```

`explicit_keys(APSS.yaml)` is the set of registered slugs present as
top-level keys (excluding reserved CF01 keys). `baseline_active_set`
is the set of standards a project ships with by default; for the
APSS standards repository it equals the set of all standards
discovered by the slug registry (see `02_slug_registry.md`). For
external consumer projects the baseline is empty and the manifest
MUST list every standard the project adopts.

For an external consumer project (not the APSS repo itself):

- A standard is active if and only if its slug appears as a top-level
  key in APSS.yaml AND that section is not `disable: true`.
- A substandard is active if its parent is active AND the substandard
  section is not `disable: true`.

For the APSS standards repository:

- Every shipped standard is active by default (so meta-validation
  always runs across the whole tree).
- APSS.yaml in the APSS repo MAY exist but is not required; if
  present, sections override defaults.

This split is the resolution of an apparent contradiction in
default-on: external projects opt in per standard, the APSS repo
itself opts in to everything.

---

## 2. The Unified Installer Contract

### 2.1 Purpose

A single installer command reads APSS.yaml and installs (or updates,
or uninstalls) every active standard's installable artifacts. The
brief calls this `apss install`; CF01 specs it as the unified
installer regardless of the final binary name.

### 2.2 Inputs

The installer consumes:

- APSS.yaml (the manifest, owned by CF01),
- apss.lock if present (owned by DI01),
- the local on-disk copies of standard crates (or the registry, per
  DI01 §2),
- per-standard install contracts (per-standard docs file; see §3).

### 2.3 Outputs

The installer produces:

- an updated apss.lock,
- the composed binary (`.apss/bin/<bootstrap>`, owned by DI01 §6),
- the side effects declared by each active standard's install
  contract: git hooks, scaffolds, generated files, schema files,
  language-server config, etc.

### 2.4 Idempotency

The installer MUST be idempotent. Running it twice with the same
APSS.yaml and the same standard versions MUST produce the same
filesystem state (modulo timestamps that are not under version
control). Per-standard install contracts MUST also be idempotent;
see §3.2.

### 2.5 Uninstall on Removal

Removing a standard from APSS.yaml and re-running the installer MUST
uninstall that standard's hooks and scaffolds cleanly. "Cleanly"
means:

- file-level: any file the install contract created and has not been
  edited by the user is removed,
- file-level: any file the install contract created but the user has
  edited is preserved and a warning is emitted,
- hook-level: any git hook or CI step the install contract owns is
  removed if it still matches the installed version's checksum,
  preserved otherwise (with a warning).

Disabling a standard via `disable: true` MUST have the same
uninstall semantics as removing it from APSS.yaml. The two paths
exist so that consumers have a discoverable way to record "we
deliberately do not use docs" in their manifest.

### 2.6 CLI Surface

```
<bootstrap> install                 # primary path; reads APSS.yaml
<bootstrap> install --locked        # CI mode; fail if apss.lock changes
<bootstrap> install --offline       # use only cached crates
<bootstrap> install --dry-run       # print what would happen, no writes
<bootstrap> install --update <slug> # update a single standard
```

DI01 §3.2 already lists these commands. CF01 re-states them here to
emphasize that they are driven by APSS.yaml, and that the manifest
is the sole declarative input.

`<bootstrap> run <slug> <cmd>` (DI01 §3.3) remains the escape hatch for
invoking a single standard directly without re-installing. Per the
brief, individual installation (e.g. `<bootstrap> run docs install`)
also remains supported as an escape hatch; the unified installer is
the documented primary path.

---

## 3. Per-Standard Install Contracts

### 3.1 Where They Live

Every standard that produces install-time side effects MUST ship an
install contract at:

```
<package>/docs/02_install_contract.md
```

The file is normative for that standard. It is the lifecycle hook
the unified installer invokes. EXP-V1-0004 already follows this
convention (the brief calls it out by name).

Standards with no install-time side effects MAY omit the file but
SHOULD include a one-line marker
(`docs/02_install_contract.md` with content `<!-- no install
contract -->`) so the absence is intentional and not an oversight.

### 3.2 What the Contract Specifies

A standard's install contract MUST specify:

1. **Install steps.** The actions the unified installer performs
   when this standard transitions from absent to present, or when its
   version changes. Steps MUST be deterministic given the
   `StandardConfig` resolved from APSS.yaml.
2. **Uninstall steps.** The actions performed when the standard
   transitions from present to absent (removed from manifest or
   `disable: true`). MUST undo §1 cleanly per §2.5.
3. **Update steps.** What changes between two installed versions of
   the same standard. MAY default to "uninstall + install" if the
   standard has no incremental update story.
4. **Inputs read.** What the install contract reads from APSS.yaml.
   Typically a subset of the standard's contribution schema.
5. **Outputs written.** Every file path the install contract creates
   or modifies. Required so the uninstall step is auditable.
6. **Failure mode.** What happens if an install step fails midway:
   atomic (all-or-nothing) or staged (record progress for resume).
   The unified installer MUST handle both.

The contract MUST NOT presuppose how the unified installer invokes
it. The installer MAY shell out to the standard's CLI subcommand,
call into the standard crate via the composed binary, or use a Rust
trait method, depending on DI01's implementation choices. The
contract is at the semantic level.

### 3.3 Trait Surface (Recommended)

Standards SHOULD implement the following trait so the installer can
drive them programmatically without per-standard shell-outs:

```rust
pub trait Installable: Standard {
    fn install(&self, ctx: &InstallContext) -> Result<InstallReport, InstallError>;
    fn uninstall(&self, ctx: &InstallContext) -> Result<InstallReport, InstallError>;
    fn update(&self, ctx: &InstallContext, from: &semver::Version) -> Result<InstallReport, InstallError>;
}

pub struct InstallContext<'a> {
    pub project_root: &'a Path,
    pub apss_toml: &'a Path,
    pub resolved_config: toml::Value,
    pub effective_version: semver::Version,
    pub dry_run: bool,
    pub previous_outputs: Option<&'a InstallReport>,
}

pub struct InstallReport {
    pub outputs: Vec<InstallOutput>,
    pub notes: Vec<String>,
}

pub struct InstallOutput {
    pub path: PathBuf,
    pub kind: OutputKind,
    pub checksum: String,
}

pub enum OutputKind {
    File,
    Symlink,
    GitHook,
    CiStep,
    Scaffold,
}
```

`previous_outputs` is what makes idempotency and clean uninstall
possible: the installer persists each standard's last `InstallReport`
under `.apss/install/<slug>.json` and passes it back on the next
run. Uninstall consults this report to know what to remove.

The persisted reports live under the existing `.apss/` dotdir which
is for GENERATED artifacts only (brief binding decision 1); they MUST
be gitignored. The dotdir is NOT used for configuration.

---

## 4. Resolve and Install Order

The unified installer's run is split into two stages with a clear seam:

```
+--------+   +------------+   +-----------+
| Resolve|-->| Lockfile   |-->| Install   |
| (DI01) |   | (DI01)     |   | (per-std) |
+--------+   +------------+   +-----------+
```

1. **Resolve (DI01).** Read APSS.yaml. Resolve standard versions
   against the registry and the existing apss.lock. Apply
   `--locked` if set. Write apss.lock if changes are allowed.
2. **Install (per-standard).** For each active standard in the
   manifest, in deterministic order (sorted by slug ascending):
   1. Load the previous `InstallReport` if present.
   2. Compute the desired outputs from the install contract and
      `resolved_config`.
   3. Apply the diff: create new outputs, replace changed outputs,
      remove orphaned outputs.
   4. Persist the new `InstallReport`.
3. **Reconcile.** For each previously installed standard not in the
   active set, run `uninstall()` and discard its report.

The order is intentional: resolve must complete fully before any
install side effect runs, so a failed resolve never half-installs.

### 4.1 Failure Handling

If any standard's `install()` returns an error, the installer MUST:

- continue installing the remaining standards (so the user sees the
  full damage),
- collect every error into a final report,
- exit with code 1,
- NOT write `.apss/bin/<bootstrap>` so the composed binary is not
  silently stale.

The user fixes the failures and re-runs; idempotency guarantees no
partial state survives in problematic ways.

---

## 5. The DI01 Seam, Explicitly

The brief asks for the seam between DI01 and the installer to be
explicit. Here it is:

| Concern | Owner | Surface |
|---------|-------|---------|
| Where standards come from (vendoring, registry, pinning) | DI01 | `apss.lock`, `[source]` resolution, registry config |
| What standards are active and how they are configured | CF01 | APSS.yaml manifest, contribution schemas |
| How each standard installs its hooks and scaffolds | per-standard | `docs/02_install_contract.md`, `Installable` trait |
| Orchestration: resolve then install | unified installer | the `<bootstrap> install` command, owned by DI01 §3 with manifest semantics owned by CF01 |

The installer MUST NOT bypass DI01 for source resolution and MUST
NOT bypass CF01 for manifest interpretation. Per-standard install
contracts MUST NOT reach into the registry directly; they receive
already-resolved versions through the `InstallContext`.

---

## 6. Migration Path

The migration window from EXP-V1-0004's `.apss/config.toml` to the
single `APSS.yaml` manifest is specified in the migration note attached
to `01_spec.md`. Per brief decision 1, the `.apss/` dotdir is retained
for generated artifacts (lockfile resolution outputs, install reports)
but is never configuration.

External consumers MAY use a shim during the window: if `.apss/config.toml`
exists, the installer MUST report the legacy generated-directory config and
require the consumer to move that content into `APSS.yaml`.

| Code | Severity | Rule |
|------|----------|------|
| `CF_LEGACY_APSS_CONFIG_TOML` | Error | Legacy `.apss/config.toml` exists under the generated artifact directory. |

This avoids the failure mode where a forgotten old file silently
shadows the new one.

---

## 7. Binary Naming

Repo issue 64 logs an open question: APS vs APSS naming for the
binary. This spec deliberately writes `<bootstrap>` everywhere. Once
issue 64 closes, all instances may be substituted by the agreed
name without changing the underlying mechanism. The substitution is
mechanical and does not require a CF01 version bump.
