# Session Capture Standard

**ID:** `APS-V1-0004`
**Type:** Official standard
**Slug:** `session-capture`
**Version:** `2.0.0`
**Status:** Active (ratified, promoted from `EXP-V1-0003` on 2026-08-06)

One contract so "back up my agent sessions" works identically across every
runtime the operator uses (Claude / Codex / Cursor CLIs, a VPS, Docker
workspaces, syntropic137 workflows), without any of them agreeing on a
provider's internal transcript shape. Sessions become uniformly backed up,
searchable, and resumable on a different machine, through a thin envelope that
wraps each provider's raw transcript verbatim.

## Index

- [standard.toml](standard.toml)
- [Overview](docs/00_overview.md)
- [Specification](docs/01_spec.md)
- [JSON Schema](schemas/session-envelope.schema.json)
- [Reconstitution registry](registry/reconstitution.toml)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## At a glance

- **Envelope + opaque raw.** A thin, universal envelope carries the fields the
  store backs up, sorts, dedups, and attributes by; the provider transcript is
  preserved byte for byte in `raw`. The standard never claims to understand a
  provider's internals, so it cannot rot as providers churn.
- **Metadata is first-class for search.** Conformant stores can filter and query
  sessions by `metadata` fields (repo, origin, agent, model, project, tags),
  not only full-text over content. Lexical and metadata search is the baseline;
  semantic and embedding search is explicitly optional.
- **Three conformance profiles.** Source (pull: read local transcripts into
  envelopes), Exporter (push: POST envelopes to a batch endpoint with a scoped
  token), and Reconstitutor (restore: write a stored session back to disk on
  another machine and resume it natively).
- **Sanitization is a security floor.** Stores sanitize unconditionally and never
  persist pre-sanitization bytes. `content_hash` is computed at ingest over the
  captured content, so a session keeps one stable identity no matter how the
  sanitizer later evolves.
- **Resume is a fitness function, not a feature.** Reconstitution turns the
  "preserve `raw` verbatim" requirement into an executable round-trip test.
- **Server-derived search.** Normalization for cross-provider search is a
  server-side, best-effort, swappable concern over `raw`; the client contract
  commits to nothing provider-specific.

## Scope

This standard is DEFINITIONAL. It specifies the session envelope, the batch
transport, the three conformance profiles, reconstitution, and the versioning
rules. The behavioural reference implementation (Source adapters, the exporter,
the server-side sanitizer and parsers, the Reconstitutor client) lives in the
SeshMagic repository, not here.

The crate in this package provides the canonical envelope types, their
validation, and the reconstitution registry. Consumers should depend on it rather
than reimplement the envelope, so divergence from this standard is a build
failure rather than an undetected drift.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate package APS-V1-0004
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```

Run the crate's conformance tests with:

```bash
cargo test -p apss-v1-0004-session-capture
```
