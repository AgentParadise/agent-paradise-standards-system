# APS-V1-0000.DI01: Unified Install Seam (Normative)

**Version**: 1.0.0
**Status**: Active
**Parent**: APS-V1-0000.DI01 (Distribution and Installation)

Sibling normative spec to `01_spec.md`. Equal precedence under
APS-V1-0000 §1.1.

This document specifies the DI01 side of the unified installer
introduced by Addendum 1 of the operator brief (2026-06-04 22:47).
The CF01 side lives at
`standards/v1/APS-V1-0000-meta/substandards/CF01-project-config/docs/06_unified_install_seam.md`.

Reading order: this document assumes familiarity with `01_spec.md`
sections 3 (bootstrap binary), 4 (installation workflow), 5
(lockfile), and 6 (code generation).

## Terminology

RFC 2119 keywords apply. The placeholder `<bootstrap>` is used in CLI
examples while repo issue 64 (APS vs APSS naming) is open. References
to `APSS.yaml` in `01_spec.md` are superseded by `APSS.yaml` per the
migration note attached to CF01 `01_spec.md`.

---

## 1. The Three-Concern Unification

Configuration, distribution, and installation are one system. DI01
owns the distribution edge (resolve, lockfile, registry,
publishing). CF01 owns the manifest edge (APSS.yaml, slug registry,
contribution schemas). The unified installer is the glue that reads
the CF01 manifest and drives both the DI01 resolve step and each
standard's install contract.

```
+------------------+  resolve  +----------------+  install  +------------------+
| APSS.yaml (CF01) | --------> | apss.lock      | --------> | per-standard     |
| slug registry    |  (DI01)   | composed bin   |  (per-std | install contracts|
| schemas (CF01)   |           | bootstrap CLI  |  contract)|                  |
+------------------+           +----------------+           +------------------+
```

This document specifies the seam between resolve and install, the
shape of per-standard install contracts that DI01 honors, and the
extensions to the bootstrap CLI that drive the unified flow.

---

## 2. APSS.yaml as the Resolve Input

### 2.1 What Resolution Reads

DI01 resolution reads the following from APSS.yaml:

- The declared set of standards (per CF01
  `06_unified_install_seam.md` §1).
- Each standard's optional `version:` semver requirement (universal
  key from `03_contribution_schema.md` §3.1).
- The cascade-merged `tool` and `workspace` sections (per
  `01_spec.md` §4 as rewritten in the APSS.yaml migration).

Resolution MUST NOT read any standard-owned keys; those are opaque
to DI01 and are passed through to the per-standard install contract
via the `InstallContext` (CF01 `06_unified_install_seam.md` §3.3).

### 2.2 What Resolution Writes

Resolution writes:

- `apss.lock` updates per `01_spec.md` §5.
- A resolved-config blob at `.apss/install/resolved.toml` whose schema
  mirrors APSS.yaml but with version pins replaced by resolved
  versions. This blob is consumed by per-standard install contracts.
  It is gitignored.

### 2.3 Locked Mode

`<bootstrap> install --locked` MUST fail at the resolve step if the
resolution would change `apss.lock`. The lockfile is the contract:
in locked mode, only previously-pinned versions are allowed. This
is unchanged from `01_spec.md` §4.2 but is repeated here because
locked mode is a critical safety property of the unified installer.

---

## 3. Per-Standard Install Contracts

DI01 honors the per-standard install contracts specified in CF01
`06_unified_install_seam.md` §3. This section specifies the DI01-side
expectations.

### 3.1 Invocation

The unified installer invokes each active standard's install contract
via the `Installable` trait (CF01 `06_unified_install_seam.md` §3.3).
The composed binary (per `01_spec.md` §6) is what holds the
`Installable` implementations; it MUST be regenerated whenever any
active standard's crate version changes so that the install
contract's behavior matches the resolved version.

DI01's code generation step (`01_spec.md` §6.1) MUST be extended to
include an `install_all()` entry point in the composed binary's
`src/main.rs`, generated from the resolved standards:

```rust
// Generated, do not edit.
pub fn install_all(ctx: &InstallContext) -> InstallReport {
    let mut report = InstallReport::default();
    report.merge(apss_v1_0001_code_topology::install(ctx));
    report.merge(apss_v1_0003_fitness_functions::install(ctx));
    report.merge(apss_v1_0004_docs::install(ctx));
    // ...
    report
}
```

This makes per-standard install contracts reachable through one
binary call rather than per-standard shell-outs.

### 3.2 Order

Per CF01 `06_unified_install_seam.md` §4, standards install in slug
ascending order. DI01's code generator MUST emit `install_all()`
calls in the same order so that the install transcript is
deterministic and reproducible.

### 3.3 Uninstall on Removal

When a standard is removed from APSS.yaml or set to `disable: true`,
the unified installer MUST call that standard's `uninstall()` using
the persisted `InstallReport` from the previous run (stored at
`.apss/install/<slug>.json`).

To support this when the standard is no longer in the composed
binary (because its crate is no longer linked), DI01 MUST:

1. Detect removed standards by diffing the new resolved manifest
   against the previous `apss.lock` and the persisted reports.
2. Either keep the previous composed binary on disk as
   `.apss/bin/<bootstrap>.prev` and invoke its `uninstall_<slug>`
   subcommand, OR fall back to "file-level uninstall" using the
   `InstallReport.outputs` list (delete files whose checksum
   matches the recorded checksum; warn for the rest).

The file-level fallback is REQUIRED; the previous-binary fast path
is OPTIONAL. The fallback is what guarantees the uninstall semantics
specified by CF01 even when the source crate is gone.

### 3.4 Update on Version Change

When a standard's resolved version changes, the unified installer
MUST call the new version's `update()` with the previous version and
the previous `InstallReport`. If `update()` is not implemented for
the standard, the installer falls back to `uninstall()` followed by
`install()` of the new version. Reports are merged so that the
persisted state reflects the new version's outputs.

---

## 4. Bootstrap CLI Extensions

### 4.1 New and Refined Commands

The bootstrap CLI surface from `01_spec.md` §3.2 is extended as
follows. Commands marked NEW are introduced by this document;
commands marked REFINED have unchanged names but tightened semantics
under the unified model.

| Command | Status | Description |
|---------|--------|-------------|
| `<bootstrap> init` | REFINED | Creates `APSS.yaml` and an initial `.gitignore` block for `.apss/`. |
| `<bootstrap> install` | REFINED | Drives the full resolve and install flow described in §3. |
| `<bootstrap> install --locked` | REFINED | Resolve must produce no lockfile change; per-standard install still runs. |
| `<bootstrap> install --offline` | REFINED | Resolve uses cached crates only; per-standard install still runs (per-standard contracts MUST be offline-safe). |
| `<bootstrap> install --dry-run` | NEW | Runs resolve and computes per-standard install diffs, but writes nothing. Prints the diff. |
| `<bootstrap> install --update <slug>` | REFINED | Updates one standard, runs only that standard's `update()`. |
| `<bootstrap> uninstall <slug>` | NEW | Runs one standard's `uninstall()`. Equivalent to removing the slug from APSS.yaml and re-running install, but does not require an edit. |
| `<bootstrap> status` | REFINED | Reports installed versions AND the persisted install reports per standard. |
| `<bootstrap> validate` | REFINED | Delegates to CF01 per the validation delegation protocol. |
| `<bootstrap> config show <slug>` | REFINED | Reads resolved config from `.apss/install/resolved.toml` so the output matches what the installer actually used. |
| `<bootstrap> config schema <slug>` | REFINED | Reads the per-slug contribution schema (`03_contribution_schema.md` §5.1). |

### 4.2 Removed Surface

`<bootstrap> install` no longer has the responsibility "make
`APSS.yaml` exist if missing"; that was `<bootstrap> init`'s job and
remains so. Likewise, `install` MUST NOT touch the slug registry
artifact directly; it consumes the artifact (regenerated by
`<bootstrap> v1 generate slug-registry` per
`02_slug_registry.md` §2.2.2).

These responsibility splits prevent the "install accidentally edited
my manifest" failure mode.

---

## 5. Lockfile Notes

The lockfile schema in `01_spec.md` §5.2 is unchanged. It is repeated
here only to record one addition: the lockfile MAY include a
`manifest_checksum` field whose value is the SHA-256 of the parsed
APSS.yaml (after cascade resolution). The installer MUST compare
this checksum against the current APSS.yaml on every run; a mismatch
in `--locked` mode is `DI_LOCKFILE_MANIFEST_DRIFT` and exits with
code 1.

| Code | Severity | Rule |
|------|----------|------|
| `DI_LOCKFILE_MANIFEST_DRIFT` | Error | APSS.yaml has changed since apss.lock was written; in `--locked` mode this is a hard fail. |
| `DI_INSTALL_REPORT_CORRUPT` | Error | `.apss/install/<slug>.json` cannot be read or fails schema validation. The installer falls back to a fresh install for that standard and emits this code as a warning at the second occurrence. |
| `DI_OUTPUT_DRIFT` | Warning | A previously installed output's on-disk checksum differs from the recorded checksum (user edited the file). Uninstall preserves the file per CF01 §2.5. |

These codes are additive to the table in `01_spec.md` §10.

---

## 6. Release Pipeline Implications

The release pipeline in `01_spec.md` §9 already validates version
bumps. Under the unified install model, two additional release-gate
checks MUST run:

1. **Install contract round-trip.** For every standard with an
   `Installable` impl, the release gate MUST execute install,
   uninstall, install on a clean fixture project and verify
   filesystem equality between the two installs. This catches
   non-idempotent install contracts before they ship.
2. **Manifest checksum migration.** For every standard whose major
   version changes, the release gate MUST verify that `update()`
   exists or that the standard explicitly declares "no update path"
   in its install contract. Missing `update()` on a major bump
   without that declaration MUST fail the gate with
   `DI_MAJOR_WITHOUT_UPDATE_PATH`.

| Code | Severity | Rule |
|------|----------|------|
| `DI_INSTALL_NOT_IDEMPOTENT` | Error | Release-gate install-roundtrip detected filesystem drift between repeated installs. |
| `DI_MAJOR_WITHOUT_UPDATE_PATH` | Error | Standard's major version is bumped, but neither `update()` is implemented nor "no update path" is declared in the install contract. |

These extend `01_spec.md` §9.2's required-checks list.

---

## 7. Filesystem Layout Under the Unified Model

```
project-root/
  APSS.yaml                       # manifest (CF01)
  apss.lock                       # resolution output (DI01 §5)
  .apss/
    bin/
      <bootstrap>                    # composed binary (DI01 §6)
      <bootstrap>.prev               # optional, for fast-path uninstall (§3.3)
    build/
      Cargo.toml
      src/main.rs                 # generated, includes install_all()
    install/
      resolved.toml               # resolved manifest blob (§2.2)
      <slug>.json                 # per-standard InstallReport
    cache/                        # offline-mode crate cache
```

`.apss/` is for generated artifacts only, per brief decision 1.
Nothing under `.apss/` is configuration; nothing under `.apss/` is
checked into version control. `APSS.yaml` and `apss.lock` ARE
checked in (per `01_spec.md` §7).

The unified installer MUST add `.apss/` to `.gitignore` on
`<bootstrap> init` if not already present.

---

## 8. The CF01 Seam, From DI01's Side

| Concern | Owner | Surface visible to DI01 |
|---------|-------|--------------------------|
| Which standards are declared | CF01 | resolved active set, computed per `06_unified_install_seam.md` (CF01) §1 |
| Per-standard configuration | CF01 | opaque blob passed via `InstallContext.resolved_config` |
| Version constraints | CF01 | `version:` semver per slug; DI01 resolves with the workspace cascade in mind |
| Lockfile semantics | DI01 | apss.lock format, integrity, manifest checksum |
| Composed binary | DI01 | `.apss/bin/<bootstrap>` and code generation |
| Install contract invocation | DI01 | calls into `Installable` impls in the composed binary |
| Install contract content | per-standard | `docs/02_install_contract.md` and `Installable` trait impl |

DI01 MUST NOT interpret per-standard configuration. CF01 MUST NOT
own version resolution. Per-standard install contracts MUST NOT
read APSS.yaml or apss.lock directly; they receive what they need
through `InstallContext`.

---

## 9. Binary Naming

Same caveat as CF01 `06_unified_install_seam.md` §7: repo issue 64
logs the APS vs APSS naming question. All CLI examples in this
document use `<bootstrap>` so that the resolution of issue 64 is a
mechanical substitution and not a spec rewrite.
