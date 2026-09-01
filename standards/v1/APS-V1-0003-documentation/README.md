# APS-V1-0003: Documentation and Context Engineering

Generic frontmatter-driven index and progressive-disclosure mechanism for any docs directory, plus a pluggable doc-type registry layered on top. The standard is the substrate; the doc types (ADRs, the North Star, Retrospectives) are instances.

This is an official, active standard under APS-V1-0003 (promoted from EXP-V1-0004). The contract surface, doc type registry, and install hook are normative. The validator implementation is split across the parent and its substandards.

## Where the contract lives

- [`docs/00_overview.md`](docs/00_overview.md): what this standard provides and why.
- [`docs/01_spec.md`](docs/01_spec.md): the normative spec (frontmatter, indexing, validator contract, doc type registry, backlinking, diagnostic codes).
- [`evolution/major/1.0.0/`](evolution/major/1.0.0/): the 1.0.0 evolution pack - why `CLAUDE.md` stopped being a symlink, what breaks, and how to migrate.
- [`docs/02_install_contract.md`](docs/02_install_contract.md): the normative install entry point, validator API, index generator API, the AGENTS.md and CLAUDE.md scaffolding contract (create-if-missing, never-overwrite), and the git pre-commit hook contract.

## Substandards (active doc types)

- [`substandards/AD01-architecture-decision-records/`](substandards/AD01-architecture-decision-records/): Architecture Decision Records under `docs/adrs/`. Ships templates (README, AGENTS.md, CLAUDE.md byte-identical copy, ADR-000 template) and the ADR reference accuracy validator (`ADR01-unknown-reference`).
- [`substandards/PV01-purpose-and-vision/`](substandards/PV01-purpose-and-vision/): the North Star (Mission, Vision, Position), a single document at `docs/north-star.md`. Slug `north-star`.
- [`substandards/RT01-retrospectives/`](substandards/RT01-retrospectives/): append-only retrospectives under `docs/retrospectives/`.

## Examples

- [`examples/`](examples/): minimal `apss.yaml` docs section, example ADR template, and example readmes.

## Tests

- [`tests/`](tests/): integration tests covering frontmatter, indexing, README validation, root context, backlinking, and config parsing.
- Per-substandard tests live inside each substandard crate (`substandards/<ID>/tests/`).

## Commands

```bash
apss run documentation validate [path]       # validate docs structure and ADRs
apss run documentation index [path]          # preview generated index tables (dry run)
apss run documentation index [path] --write  # write index tables into README.md files
```

In `apss.yaml` and `apss run`, use the canonical slug `documentation` (it must
match the standard key in `apss.yaml`). The `docs` and `doc` spellings are
accepted only by the development CLI `apss-dev`, not by the composed project
binary, so `apss run docs ...` fails in a consumer project.

`apss install` installs a pre-commit hook that runs `apss validate` (project
config and standard-structure validation), not `apss run documentation validate`.
Run the documentation standard's own validation manually or in CI with
`apss run documentation validate`. Dedicated `install`/`uninstall`/`hook`
subcommands are planned for a follow-up and are not yet implemented.

## Configuration

This standard's canonical slug is `documentation` (the `docs` and `doc` spellings are dev-CLI aliases only). It contributes the `docs:` section of `apss.yaml`, owned by the meta-standard (APS-V1-0000.CF01). Zero-config works: absence of a key means the documented default applies (per Section 3.2 of the spec). A key is written only to opt out (`disable: true`) or to override a non-`disable` default.

## Status

Official and active (APS-V1-0003, promoted from EXP-V1-0004). The contract surface is stable; validator implementations land iteratively per substandard.
