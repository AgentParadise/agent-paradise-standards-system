# EXP-V1-0004: Documentation and Context Engineering

Generic frontmatter-driven index and progressive-disclosure mechanism for any docs directory, plus a pluggable doc-type registry layered on top. The standard is the substrate; the doc types (ADRs, the North Star, Retrospectives) are instances.

This is an experimental standard under EXP-V1-0004. The contract surface, doc type registry, and install hook are normative. The validator implementation is split across the parent and its substandards.

## Where the contract lives

- [`docs/00_overview.md`](docs/00_overview.md): what this standard provides and why.
- [`docs/01_spec.md`](docs/01_spec.md): the normative spec (frontmatter, indexing, validator contract, doc type registry, backlinking, diagnostic codes).
- [`docs/02_install_contract.md`](docs/02_install_contract.md): the normative install entry point, validator API, index generator API, the AGENTS.md and CLAUDE.md scaffolding contract (create-if-missing, never-overwrite), and the git pre-commit hook contract.

## Substandards (active doc types)

- [`substandards/ADR01-architecture-decision-records/`](substandards/ADR01-architecture-decision-records/): Architecture Decision Records under `docs/adrs/`. Ships templates (README, AGENTS.md, CLAUDE.md symlink, ADR-000 template) and the ADR reference accuracy validator (`ADR01-unknown-reference`).
- [`substandards/PV01-purpose-and-vision/`](substandards/PV01-purpose-and-vision/): the North Star (Mission, Vision, Position), a single document at `docs/north-star.md`. Slug `north-star`.
- [`substandards/RETRO01-retrospectives/`](substandards/RETRO01-retrospectives/): append-only retrospectives under `docs/retrospectives/`.

## Examples

- [`examples/`](examples/): minimal `APSS.yaml` docs section, example ADR template, and example readmes.

## Tests

- [`tests/`](tests/): integration tests covering frontmatter, indexing, README validation, root context, backlinking, and config parsing.
- Per-substandard tests live inside each substandard crate (`substandards/<ID>/tests/`).

## Install

```bash
aps run docs install [path]          # install the pre-commit hook (idempotent)
aps run docs uninstall [path]        # remove the hook (config preserved)
aps run docs validate [path]         # validate docs structure
aps run docs index [path]            # preview generated index tables (dry run)
aps run docs index [path] --write    # write index tables into README.md files
aps run docs hook --staged           # hook entry point used by pre-commit
```

## Configuration

This standard registers the slug `docs` and contributes the `docs:` section of `APSS.yaml`, owned by the meta-standard (APS-V1-0000.CF01). Zero-config works: absence of a key means the documented default applies (per Section 3.2 of the spec). A key is written only to opt out (`disable: true`) or to override a non-`disable` default.

## Status

Experimental. The contract surface is stable; validator implementations land iteratively per substandard.
