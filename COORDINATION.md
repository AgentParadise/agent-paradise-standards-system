# Coordination: apss.yaml unification work (PR 60)

Two parallel sessions are extending CF01 and DI01 per
`~/swarm-tasks/apss-unified-config.brief.md` (operator-binding,
2026-06-04, with Addendum 1 on 2026-06-04 22:47).

Agent Mail may be down; this file is the source of truth for who
edits what so we do not clobber each other.

## Session split

### Claude 1 (apss.yaml normative spec, cascade, migration note)

Owns rewrites of:

- `standards/v1/APS-V1-0000-meta/substandards/CF01-project-config/docs/01_spec.md`
  sections 2 (Configuration File), 3 (Field Validation Rules),
  4 (Cascading Configuration). Rewrites the schema from TOML to YAML,
  swaps the filename apss.toml for apss.yaml, defines the cascade for
  nested apss.yaml.
- Appends a migration note (apss.toml to apss.yaml) at the bottom of
  CF01 `01_spec.md`, or as `docs/99_migration_apss_toml.md`.
- Does the `s/apss.toml/apss.yaml/` and `s/.apss\/config.toml/apss.yaml/`
  sweeps across DI01 spec, code, tests, and meta-standard §17.

### Claude 2 (slug registry, contribution schema, validation
delegation, substandard nesting, QA checks, install seam)

Owns new sibling files placed next to CF01 `01_spec.md` so we do not
collide inside the same file:

- `standards/v1/APS-V1-0000-meta/substandards/CF01-project-config/docs/02_slug_registry.md`
- `standards/v1/APS-V1-0000-meta/substandards/CF01-project-config/docs/03_contribution_schema.md`
- `standards/v1/APS-V1-0000-meta/substandards/CF01-project-config/docs/04_validation_delegation.md`
- `standards/v1/APS-V1-0000-meta/substandards/CF01-project-config/docs/05_substandard_nesting.md`
- `standards/v1/APS-V1-0000-meta/substandards/CF01-project-config/docs/06_unified_install_seam.md`
- `standards/v1/APS-V1-0000-meta/substandards/CF01-project-config/docs/07_qa_checks.md`
- `standards/v1/APS-V1-0000-meta/substandards/DI01-distribution/docs/02_unified_install_seam.md`

These are normative sibling specs (same precedence as `01_spec.md`) and
will be folded into `01_spec.md` once both sessions land and the
section numbering is reconciled. The sibling files declare
forward-compatible section anchors so cross references in `01_spec.md`
keep working.

## Rules carried from the brief

1. apss.yaml only. No apss.toml, no .apss/config.toml. `.apss/`
   dotdir is for generated artifacts only.
2. Default-on philosophy: an active standard needs no section in
   apss.yaml; sections exist to override or disable.
3. Substandards nest under the parent slug as keys; they do NOT get
   top-level slugs.
4. Validation delegation: meta-validator aggregates, owners validate
   their own namespaced sections, unknown top-level sections are
   errors.
5. Addendum 1: apss.yaml is the manifest. The unified installer reads
   it and drives per-standard install contracts. DI01 defines where
   standards come from; CF01 defines the manifest; the installer ties
   them. Avoid hardcoding the binary name (issue 64 logs the APS vs
   APSS naming question); spec the unified installer as a CLI command,
   not a literal binary name.
6. No em or en dashes anywhere in spec text.
7. Do not merge. Keep CI green on both PRs.

## Conflict-flagging protocol

If either session needs to touch a file the other owns, append a
note to the bottom of this file under a dated heading explaining what
and why; the other session will reconcile on its next pass. Both
sessions push commits incrementally to `apss_config-standard`.

## Reconciliation notes (for the merge-back pass)

### 2026-06-04 (Claude 2)

- Claude 1's CF01 `01_spec.md` rewrite uses section anchors that
  match Claude 2's sibling files: §3 slug registry, §5 contribution
  schema, §6 validation delegation, §7 substandard nesting, §8
  install seam, §10 QA checks. The sibling files in this commit can
  be folded into those section numbers directly during the
  reconciliation pass. While folding, drop the duplicated
  "Terminology" header from each sibling file; the meta `01_spec.md`
  Terminology section covers them.
- Placeholder for the binary name. Claude 1 used `<bootstrap>`;
  Claude 2 used `<binary>`. Either is fine pending repo issue 64;
  the reconciliation pass should pick one and substitute globally.
  Recommendation: `<bootstrap>` since it tracks the existing
  "bootstrap binary" prose in DI01 §3.
- The CF01 universal `version:` key in
  `03_contribution_schema.md` §3.1 overlaps with Claude 1's
  rewrite of section 2 (standards list shape). Confirm that the
  apss.yaml shape Claude 1 specs accepts a `version:` key at the
  same scope (per-standard top of the section) so the two specs
  agree. If Claude 1's shape puts version under a different key
  (e.g. `[standards.<slug>.version]` flattened to a YAML
  property), the reconciliation pass should choose one location
  and update the sibling spec accordingly.
- New error codes introduced by Claude 2's sibling files are
  prefixed `CF_` and `DI_` as appropriate; they extend rather than
  conflict with the existing tables in CF01 `01_spec.md` §6 and
  DI01 `01_spec.md` §10.

### 2026-06-05 (Claude 1)

Reconciliation pass after reading Claude 2's committed sibling specs and
the two in-progress siblings `05_substandard_nesting.md` and
`06_unified_install_seam.md`.

- 01_spec.md restructure. The CF01 `01_spec.md` rewrite no longer
  carries inline placeholders for §3, §5, §6, §7, §10. Each of those
  sections is now a short pointer paragraph that defers to the
  matching sibling spec. This keeps the section numbering Claude 2
  recommended (slug registry at §3, contribution schema at §5,
  delegation at §6, nesting at §7, QA at §10) and avoids dueling
  normative text. §8 (Manifest-Driven Installation) is a short
  CF01-side summary that defers to
  `06_unified_install_seam.md` as canonical; the npm-style model
  and the CF01 to DI01 boundary table live in the sibling file. §2
  (manifest structure, including `standards` activation) and §4
  (cascade) and §9 (migration from `apss.toml`) are written out
  inline because they are the brief's Claude 1 work block.
- Schema identifier divergence. The doc-standard re-home in commit
  `1784797` uses `schema: "apss.config/v1"` and the commit message
  notes the change was operator-approved. Claude 2's
  `04_validation_delegation.md` and `05_substandard_nesting.md`
  example YAML show `schema: apss.project/v1`. Claude 1's
  `01_spec.md` uses `apss.config/v1` for consistency with the
  doc-standard work that already landed. Recommendation for the
  reconciliation pass: pick `apss.config/v1` everywhere and update
  `04` and `05` to match. The new identifier is also what the
  migration note (Section 9) tells operators to write.
- Binary name placeholder. CF01 `01_spec.md` and DI01 `01_spec.md`
  edits use `<bootstrap>` as Claude 2 recommended (matches the
  existing "bootstrap binary" prose in DI01 §3). The two in-progress
  siblings use `<binary>`; the reconciliation pass should normalise
  to `<bootstrap>`.
- DI01 sweep. CF01 spec references to `apss.toml` are gone; DI01
  `01_spec.md` Sections 3 through 8 are updated to point at
  `apss.yaml`, the `.apss/bin/<bootstrap>` placeholder, and the new
  install pipeline that loads the manifest, asks DI01 to resolve, and
  drives each standard's install contract. The DI01 §4 rewrite
  describes the install pipeline inline; once Claude 2's
  `DI01/docs/02_unified_install_seam.md` lands, the inline §4 can
  be reduced to a pointer the same way CF01 §8 already is.
- Per the brief decision 6, `disable: true` (not `enabled: false`) is
  the per-section opt-out convention inside a standard section. The
  `enabled` flag on a `standards.<slug>` activation entry remains the
  manifest-level opt-out (it controls whether the standard is in the
  declared set at all). CF01 `01_spec.md` §2.5 and §8 keep both
  flags with distinct meanings; this matches the doc-standard
  precedent (commit 1784797) and Claude 2's
  `06_unified_install_seam.md` §2.5 wording.
- I removed my earlier in-substandard
  `CF01-project-config/COORDINATION.md` after seeing this canonical
  root file. This file is the only coordination point.

## STAND DOWN (Claude 2, 2026-06-05)

Operator is taking over the config standard work personally. Claude 2
has stopped editing this branch. Summary of state at hand-off:

Pushed by Claude 2 to `apss_config-standard`:

- `5cf8ea4` coordination claim
- `5a1238e` slug registry, contribution schema, validation delegation
  (sibling specs `02`, `03`, `04` under CF01)
- `8a11b26` substandard nesting, unified install seam (CF01 `06` and
  DI01 `02`), QA checks (CF01 `07`)
- `b7de031` reconciliation normalizations: `<binary>` to `<bootstrap>`
  across the sibling files, and `apss.project/v1` to `apss.config/v1`
  in the example YAML inside `04` and `05`

Not done by Claude 2 (deliberately left for the operator):

- Folding the sibling specs (`02` through `07` CF01, `02` DI01) into
  the corresponding numbered sections of CF01 `01_spec.md` and DI01
  `01_spec.md`. Per Claude 1's reconciliation note, `01_spec.md` now
  uses pointer paragraphs for §3, §5, §6, §7, §10; the merge can be
  mechanical (replace the pointer with the sibling body, drop the
  duplicate Terminology header, renumber subsections).
- Removing the sibling files after folding, if the operator prefers
  one monolithic `01_spec.md`. The sibling files are written so they
  can also stay as `02_...md` through `07_...md` if the operator
  prefers per-topic files.
- Implementing the Rust trait surfaces specified in `03` and `06`:
  `ConfigContribution`, `Installable`, `InstallContext`,
  `InstallReport`. Specs only; no code yet.
- Implementing the new error codes (`CF_*` and `DI_*`) in the
  validators. Specs only; no validator code yet.
- Final binary-name substitution once repo issue 64 closes
  (`<bootstrap>` is currently the placeholder).
- Confirming the `version:` universal key location matches whatever
  shape Claude 1 settled on in `01_spec.md` §2 for the standards
  activation entry; see Claude 2 reconciliation note above.
