# Agentic Workspace Tests

The tests validate the JSON Schemas, Rust round trip, semantic invariants, and
negative security cases. `schema_conformance.rs` also:

- validates and round-trips every `fixtures/valid/<schema>/` document;
- rejects every `fixtures/invalid/` document, each a base success fixture plus
  a minimal mutation;
- inserts credential value fields into every typed object of every success
  fixture and expects rejection (free-form string maps excepted, see spec
  Section 5.1.3);
- runs every launch/v1 document through the frozen 0.1.0 schema and the 0.2.0
  dispatching validator and requires identical verdicts;
- replays `fixtures/behaviour/` scenarios and checks idempotent start,
  conflict, sticky terminal states, and gap integrity.

```bash
cargo test -p apss-v1-0006-workspace
```

Provider behavior is tested in the Agentic Workspace repository because APSS
owns the contract, not provider implementations.
