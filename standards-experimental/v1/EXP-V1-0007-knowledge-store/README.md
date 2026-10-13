# Knowledge Store (EXPERIMENTAL)

**ID:** `EXP-V1-0007`
**Type:** Experiment
**Slug:** `knowledge-store`
**Version:** `0.1.0`
**Status:** EXPERIMENTAL PLACEHOLDER. Not a standard. Not enforced anywhere.

This experiment holds the principles of an event-sourced, privacy-first
personal knowledge store: an append-only log of facts that carry references,
never content, over a content-addressed object store, with rebuildable read
models as the only query surface and a Compartment as the unit of isolation.

It exists so the boundary can be written down and iterated on while one real
system (DreamShip's Knowledge Core, over the Syntropic137 Event Sourcing
Platform) uses it. It becomes a real standard only after that usage has
proven the principles. Until then every clause here may change, and the
"Open questions" section of the spec is as important as the principles.

## Artifacts

- [Overview](docs/00_overview.md)
- [Normative specification (principles and open questions)](docs/01_spec.md)
- [A reference-only fact, as an example](examples/minimal/knowledge-fact.json)
- Rust: the principle list as data, and a `principles` command (`src/lib.rs`)

## Validation

```bash
cargo test -p apss-v1-0007-knowledge-store
cargo run -p aps-cli --bin apss-dev -- v1 validate package EXP-V1-0007
```

## Promotion gate

This experiment is not eligible for promotion until a second implementation
(not DreamShip) has been built against the principles, and until every open
question in the spec has a ruling.
