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
