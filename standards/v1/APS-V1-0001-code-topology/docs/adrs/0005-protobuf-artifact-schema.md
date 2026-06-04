# ADR-0005: Protobuf as Artifact Schema Source of Truth

## Status

**Accepted**

## Context

The Code Topology standard defines artifacts stored in `.topology/` that serve as the contract between:
- **Producers** (language adapters) — Generate artifacts from source code
- **Consumers** (projectors) — Read artifacts to create visualizations

The artifact interface is the most important part of this standard. We need to decide how to formally define these schemas.

### Options Considered

1. **JSON Examples Only** — Define schemas via example JSON in spec
2. **JSON Schema** — Use JSON Schema for validation
3. **Rust Structs** — Define in Rust with serde, derive JSON
4. **Protobuf** — Define in proto3, generate for multiple languages

## Decision

**Use Protobuf as the canonical schema definition.**

### Rationale

1. **Meta-standard alignment** — APS-V1-0000 §9 specifies "Protobuf definitions are the canonical machine contract"

2. **Language-agnostic** — Proto can generate bindings for Rust, Python, TypeScript, Go, C++, matching our polyglot focus

3. **Self-documenting** — Field documentation lives with the schema, not scattered across files

4. **Versionable** — Proto evolution rules (field numbers, `optional`) ensure backward compatibility

5. **Validatable** — Tools can validate artifacts against compiled proto descriptors

6. **Compact option** — Proto binary format available for large topologies (100K+ functions)

### File Structure

```
proto/
├── manifest.proto    # .topology/manifest.toml schema
├── metrics.proto     # metrics/*.json schemas  
├── graphs.proto      # graphs/*.json schemas (including coupling matrix)
└── topology.proto    # Aggregate + adapter types
```

### Serialization Strategy

| On-Disk Format | Proto Purpose |
|----------------|---------------|
| TOML (manifest) | Human-readable config; proto defines structure |
| JSON (metrics, graphs) | Human-readable, git-diffable; proto defines structure |
| Proto binary (optional) | Large repos, performance-critical |

### Rust Integration

Rust structs in `lib.rs` are hand-written to match proto definitions:
- Use `#[derive(Serialize, Deserialize)]` for JSON compat
- Field names align with proto `json_name` attributes
- Can optionally generate with `prost` in future

## Consequences

### Positive

- Clear, versionable contract between producers and consumers
- Multiple language implementations possible
- Validation tooling available (protoc, buf)
- Aligns with meta-standard requirements for technical standards

### Negative

- Developers must keep Rust structs in sync with proto (until code generation)
- Adds proto files that some contributors may be unfamiliar with
- Slightly more complex build if we add prost generation

### Mitigation

- Document Rust ↔ Proto alignment requirements
- Provide validation scripts to check alignment
- Consider prost generation as future enhancement

## References

- APS-V1-0000 §9: Protobuf Contracts (Technical Standards)
- [Protocol Buffers Language Guide](https://protobuf.dev/programming-guides/proto3/)
- [prost - Rust Protocol Buffers](https://github.com/tokio-rs/prost)

