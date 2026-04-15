# Architecture Decision Records

This directory contains all ADRs governing the project's architecture.

## Referencing ADRs in code

Files that implement an ADR MUST reference it in a comment block at the top of the file.
This keeps agents and developers in context when making updates to the codebase.

**Format:** Use the ADR identifier (filename without `.md`) in a comment:

```rust
// Implements ADR-002-security
```

```python
# Implements ADR-001-initial-architecture
```

Use `grep ADR-002-security` to find all files implementing a given decision.

See [README.md](README.md) for the full index of ADRs.
