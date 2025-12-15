# APS-V1-0000 — Meta-Standard Overview

## What is APS?

The **Agent Paradise Standards System (APS)** is an executable, evolvable standards framework designed for agentic engineering at scale.

APS standards are not static documents. Each standard is a **versioned Rust crate** that includes:

- Normative specification (docs)
- Executable validation rules (Rust code)
- Protobuf contracts (for technical standards)
- Examples and tests
- Agent skills (for AI integration)
- Templates (for scaffolding)

## What is APS-V1-0000?

APS-V1-0000 is the **Meta-Standard** — the "standard of standards" for the V1 ecosystem.

It defines:

- **Repository layout** — Where official and experimental standards live
- **Package structure** — Required directories and files for every standard
- **Metadata schemas** — `standard.toml`, `substandard.toml`, `experiment.toml`
- **Validation rules** — What the `aps` CLI checks
- **Versioning rules** — SemVer, evolution packs, backward compatibility
- **Substandard conformance** — How substandards relate to parent standards
- **Experimental lifecycle** — Incubation and promotion to official

## How to Use This Standard

### For Standard Authors

1. Use `aps v1 create standard <slug>` to scaffold a new standard
2. Implement the `Standard` trait in your crate
3. Add examples and tests
4. Run `aps v1 validate standard <id>` to check compliance

### For Standard Consumers

1. Check if the standard you need exists: `aps v1 list`
2. Review the standard's `docs/01_spec.md` for requirements
3. Use the standard's templates for adoption assets

### For Agents

Agent skills in `agents/skills/` teach AI how to:

- Author new standards safely
- Validate packages against standards
- Never introduce breaking changes without proper evolution packs

## Key Principle: Code is the Standard

The Rust crate + protobuf contracts are the **source of truth**. Documentation is supporting material for humans, not the authoritative definition.

If there's ever a conflict:

1. Enforced tooling/validators (highest authority)
2. Protobuf-defined contracts
3. This spec (`docs/01_spec.md`)

