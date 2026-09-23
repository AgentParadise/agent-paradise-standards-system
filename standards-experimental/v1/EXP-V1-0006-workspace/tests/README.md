# Agentic Workspace Tests

The tests validate the JSON Schema, Rust round trip, semantic invariants, and
negative security cases.

```bash
cargo test -p apss-v1-0006-workspace
```

Provider behavior is tested in the Agentic Workspace repository because APSS
owns the contract, not provider implementations.
