# APS-V1-0000.CF01: QA Conformance Checks (Normative)

**Version**: 1.0.0
**Status**: Active
**Parent**: APS-V1-0000.CF01 (Project Configuration)

Sibling normative spec to `01_spec.md` and the other CF01 sibling
specs. Equal precedence under APS-V1-0000 §1.1.

This document specifies how the CLI binary validates that every
standard in the APSS repository (and every consumer project's
APSS.yaml) conforms to the rules across the sibling specs. It is
the "how do we enforce all this" section.

## Terminology

RFC 2119 keywords apply. `<bootstrap>` is a placeholder pending repo
issue 64.

---

## 1. Scope of QA

CF01 conformance has two scopes:

1. **The APSS repository.** Every standard in `standards/v1/` and
   every experiment in `standards-experimental/v1/` MUST conform
   to the rules from the sibling specs. This is enforced by the
   meta-validator at `<bootstrap> v1 validate repo`.
2. **Consumer projects.** Any project whose root has an APSS.yaml
   gets validated by `<bootstrap> validate` (or the editor's LSP
   integration). This is the runtime-side validation.

QA in this document covers BOTH. The same rule engine runs in both
modes; the difference is which roots are scanned and which checks
are applicable.

---

## 2. The Six Conformance Dimensions

A standard or substandard conforms to the unified APSS.yaml model
when it satisfies six dimensions. The meta-validator MUST emit a
distinct error code per dimension so that failures are easy to
classify.

| Dimension | Rule |
|-----------|------|
| **Slug** | Metadata declares a slug; slug satisfies format and reserved rules; slug is unique (`02_slug_registry.md` §3, §5). |
| **Registry** | Standard appears in `generated/v1/slug_registry.json` with correct fields (`02_slug_registry.md` §2.2). |
| **Schema** | `config.schema.json` exists, round-trips from `StandardConfig::json_schema()`, and follows the dialect rules (`03_contribution_schema.md` §3, §5). |
| **Trait** | Standard implements `StandardConfig` and `ConfigContribution` (or `NoConfig` for trivial cases). The `slug()` returned by `ConfigContribution` equals the metadata slug (`03_contribution_schema.md` §2.1). |
| **Install** | Standard ships a `docs/02_install_contract.md` and an `Installable` trait impl, or explicitly marks "no install contract" (`06_unified_install_seam.md` §3.1). |
| **Substandards** | Substandard slugs are unique within parent; nesting rules from `05_substandard_nesting.md` hold; the slug registry's `substandards[]` array agrees with the on-disk substandards. |

Each dimension is independently checkable. The meta-validator MUST
NOT short-circuit: a slug failure does not skip schema checks. This
makes failures actionable rather than serialized.

---

## 3. Repo-Side Checks

### 3.1 The Single Entry Point

```
<bootstrap> v1 validate repo
```

runs all six dimensions across `standards/v1/` and
`standards-experimental/v1/`. This entry point already exists per
`01_spec.md` of the meta-standard; this document specifies the new
checks that MUST run inside it.

The release pipeline (DI01 §9.2) already requires `aps v1 validate
repo` to pass. Therefore all CF01 conformance is enforced on every
release candidate.

### 3.2 Required Checks

The meta-validator MUST run the following checks per standard. Each
maps to one or more error codes already defined in the sibling
specs; this table is the index.

| Check | When | Codes |
|-------|------|-------|
| Slug present and well-formed | every standard, every experiment | `CF_SLUG_MISSING`, `CF_SLUG_INVALID_FORMAT`, `CF_SLUG_RESERVED`, `CF_SLUG_TOO_GENERIC` |
| Slug uniqueness across repo | post-discovery, all standards | `CF_SLUG_COLLISION`, `CF_SUBSTANDARD_SLUG_COLLISION` |
| Generated registry is up to date | every run | `CF_SLUG_NOT_IN_REGISTRY`, `CF_REGISTRY_HAS_PHANTOM` |
| Contribution schema present and fresh | every standard with `StandardConfig` | `CF_SCHEMA_MISSING`, `CF_SCHEMA_STALE`, `CF_SCHEMA_INVALID_DIALECT`, `CF_SCHEMA_NON_LOCAL_REF`, `CF_SCHEMA_NO_DESCRIPTION`, `CF_SCHEMA_NO_DEFAULT`, `CF_SCHEMA_NO_NOCONFIG_MARKER` |
| `ConfigContribution` trait implemented | every standard | `CF_MISSING_CONTRIBUTION_TRAIT` |
| Substandard collisions and key shadow | every standard with substandards | `CF_SUBSTANDARD_KEY_SHADOW`, `CF_SUBSTANDARD_SLUG_COLLISION` |
| Install contract present | every standard | `CF_INSTALL_CONTRACT_MISSING`, `CF_INSTALL_CONTRACT_MALFORMED` |
| Install contract round-trip | release-gate only | `DI_INSTALL_NOT_IDEMPOTENT` (DI01) |

The new install-contract codes:

| Code | Severity | Rule |
|------|----------|------|
| `CF_INSTALL_CONTRACT_MISSING` | Error | Standard's package has no `docs/02_install_contract.md` and no `<!-- no install contract -->` marker. |
| `CF_INSTALL_CONTRACT_MALFORMED` | Error | Install contract exists but is missing one of the required sections (install, uninstall, update, inputs, outputs, failure mode) per `06_unified_install_seam.md` §3.2. |

### 3.3 Determinism Check

The meta-validator MUST produce identical output (modulo timestamps
in `generated_at` fields) on two consecutive runs against an
unchanged filesystem. CI MUST enforce this by running
`<bootstrap> v1 validate repo` twice and diffing the diagnostic
streams. A diff is `CF_VALIDATION_NONDETERMINISTIC` and fails CI.

| Code | Severity | Rule |
|------|----------|------|
| `CF_VALIDATION_NONDETERMINISTIC` | Error | Two consecutive validator runs produced different diagnostic streams against the same source. |

This is the regression test against accidental ordering bugs in the
slug registry, the schema diff, or the install-report comparison.

---

## 4. Consumer-Side Checks

### 4.1 The Entry Point

```
<bootstrap> validate
<bootstrap> validate --project
<bootstrap> validate --slug <slug>
```

The semantics are specified in `04_validation_delegation.md` §6.
The QA harness re-states them here only to record that these
commands are the supported entry points for consumer projects and
editor LSPs; CI in consumer projects SHOULD wire them into their
existing test pipelines.

### 4.2 Pre-Install Validation

`<bootstrap> install` MUST run consumer-side validation BEFORE the
resolve step. A failed validation exits with code 1 and runs no
installer side effects. This is the "manifest is law" property:
the installer never tries to massage a broken APSS.yaml into
working state.

### 4.3 Editor Integration

The aggregate schema `generated/v1/apss.schema.json`
(`03_contribution_schema.md` §6.2) is the bridge to editor support.
A consumer project SHOULD register it in editor settings (e.g.
VS Code's `toml.schemas`) so that completion and hover work without
a custom language server. The unified installer SHOULD offer to
write this registration during `<bootstrap> init` when an editor
config is detected; this is OPTIONAL.

---

## 5. CI Integration in the APSS Repo

The APSS repository's CI MUST run, in this order:

```
1. just format
2. just lint
3. just typecheck
4. just test
5. just aps-validate   # this runs <bootstrap> v1 validate repo
6. determinism re-run  # second invocation of step 5 with diff check
```

Step 6 is the new requirement from §3.3. The implementation MAY be
folded into `aps-validate` directly (run twice internally) or into
a separate justfile recipe. The choice is implementation-level; the
behavior is normative.

### 5.1 Justfile Recipe (Recommended Shape)

```
[group('aps')]
aps-validate-deterministic:
    cargo run -p aps-cli -- v1 validate repo --json > /tmp/run-a.json
    cargo run -p aps-cli -- v1 validate repo --json > /tmp/run-b.json
    diff /tmp/run-a.json /tmp/run-b.json
```

The recipe is RECOMMENDED, not required, because justfile
ergonomics evolve and CF01 should not pin a specific runner shape.

### 5.2 Release Gate Integration

DI01 §9.2 already lists `aps v1 validate distribution` as a hard
gate. Under the unified model, that command's scope MUST include:

- the install-contract round-trip from
  `06_unified_install_seam.md` §6 (DI01 side),
- the schema-stale check from `03_contribution_schema.md` §5.3.

Other dimensions are already covered by `aps v1 validate repo`,
which is also a hard gate via `just ci`.

---

## 6. Failure Reporting Style

All CF01 diagnostics MUST follow the meta-standard's reporting rules
(APS-V1-0000 §16). Concretely:

- Diagnostics carry a unique error code from the tables in the
  sibling specs.
- Diagnostics include a span pointing into the source file where
  possible.
- Diagnostics include a one-line remediation hint when feasible.

The meta-validator MUST sort diagnostics by `(file_path, line,
column, code)` so that output is byte-stable across runs (per §3.3).

### 6.1 Diagnostic JSON Output

`<bootstrap> v1 validate repo --json` MUST emit a stable JSON shape:

```json
{
  "schema": "apss.diagnostics/v1",
  "exit_code": 1,
  "diagnostics": [
    {
      "code": "CF_SLUG_COLLISION",
      "severity": "error",
      "path": "standards/v1/APS-V1-0001-code-topology/standard.toml",
      "span": { "line": 5, "column": 1, "len": 10 },
      "message": "Slug 'topology' is also declared by EXP-V1-0001 at standards-experimental/v1/EXP-V1-0001-code-topology/experiment.toml",
      "hint": "Rename one of the slugs or, if these standards are intentionally aliased, complete the promotion of the experiment per APS-V1-0000 §14.3."
    }
  ]
}
```

This is the editor LSP's and the release gate's stable interface.
Adding fields is a backward-compatible change; removing or renaming
them is a major bump of CF01.

---

## 7. Cross-References

- Slug rules: `02_slug_registry.md`.
- Schema rules: `03_contribution_schema.md`.
- Validation delegation: `04_validation_delegation.md`.
- Substandard nesting: `05_substandard_nesting.md`.
- Unified install seam (CF01 side): `06_unified_install_seam.md`.
- Unified install seam (DI01 side):
  `../../DI01-distribution/docs/02_unified_install_seam.md`.
- Meta-standard validation rules: APS-V1-0000 §16.
- Release pipeline: DI01 `01_spec.md` §9.

---

## 8. Backward Compatibility

The QA harness specified here is additive. Existing CF01 v1
standards that pass `aps v1 validate repo` today will continue to
pass for everything except:

- the new `CF_MISSING_CONTRIBUTION_TRAIT` check, which fires for
  standards that have not yet adopted `ConfigContribution`,
- the new `CF_INSTALL_CONTRACT_MISSING` check, which fires for
  standards that have not yet authored their install contract.

Both errors have explicit opt-outs (`NoConfig` for the trait
extension; the one-line marker for the install contract) so the
migration is non-disruptive for standards that genuinely have no
config or no install-time side effects.

The migration window aligns with the APSS.yaml migration window
specified in the migration note attached to CF01 `01_spec.md`.
Inside the window, both checks SHOULD be downgraded to warnings;
outside it, they MUST be errors.
