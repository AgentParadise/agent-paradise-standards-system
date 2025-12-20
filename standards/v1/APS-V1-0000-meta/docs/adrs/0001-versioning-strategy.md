# ADR 0001: Hybrid Versioning Strategy

**Status:** Accepted  
**Date:** 2025-12-17  
**Context:** APS-V1-0000 Meta Standard

## Context

APS standards need a versioning strategy that supports:
- Independent evolution of each standard
- Reproducible builds for consumers
- Clear upgrade paths
- Validation of version correctness

The Consumer SDK (EXP-V1-0002) introduces package distribution via GitHub releases, making versioning semantics critical for reliability.

## Decision

Adopt a **hybrid versioning model** with four layers:

### 1. Standard Version (Semver)

Each standard maintains its own semantic version:

```toml
# standard.toml
[standard]
version = "1.2.3"
backwards_compat = true

# experiment.toml
[experiment]
version = "0.1.0"
backwards_compat = true
```

**Rules:**
- Format: `MAJOR.MINOR.PATCH` (e.g., `1.2.3`)
- MAJOR: Breaking changes to schema, artifacts, or behavior
- MINOR: New features, backward-compatible additions
- PATCH: Bug fixes, documentation updates
- `backwards_compat: false` MUST accompany a MAJOR version increment

### 2. APS Major Version (Directory)

The `v1/`, `v2/` directory structure indicates APS schema version:

```
standards/v1/APS-V1-0000-meta/      # APS v1 schema
standards/v2/APS-V2-0001-future/    # Future APS v2 schema
```

**Rules:**
- v1 → v2 only for fundamental schema changes to APS itself
- All v1 standards share compatible metadata format
- Migration guides required for major version transitions
- Standard IDs include version: `APS-V1-xxxx`, `EXP-V1-xxxx`

### 3. Bundle Release (GitHub Tag)

GitHub releases tag a snapshot of all standards:

```
Release v0.1.0:
  ├── aps-meta-1.0.0.tar.gz
  ├── aps-code-topology-0.1.0.tar.gz
  ├── aps-consumer-sdk-0.1.0.tar.gz
  └── registry.json (lists all versions)
```

**Rules:**
- Bundle tag uses semver for the registry itself
- Individual standards versioned independently within bundle
- `registry.json` maps slugs to available versions and checksums

### 4. Lock File Pinning

Consumers pin exact versions in `.aps/manifest.lock`:

```toml
[[package]]
slug = "code-topology"
version = "0.1.0"
checksum = "sha256:abc123..."
resolved_url = "https://github.com/.../aps-code-topology-0.1.0.tar.gz"
```

This ensures reproducible builds regardless of bundle release updates.

## Alternatives Considered

### 1. Single Monorepo Version
- All standards share one version number
- **Pro:** Simple
- **Con:** Forced coordination, unnecessary bumps

### 2. Independent Releases per Standard
- Each standard has its own GitHub release tag
- **Pro:** Maximum independence
- **Con:** Complex release management, harder discovery

### 3. Hybrid (Chosen)
- Bundle for discovery, lock files for reproducibility
- **Pro:** Best of both worlds
- **Con:** Slightly more complex model

## Consequences

### Positive
- Standards evolve independently without coordination
- Lock files ensure reproducible builds
- Clear semantics for breaking changes
- Bundle releases provide convenient snapshots
- Consumers can mix versions as needed

### Negative
- More complex than single-version approach
- Requires discipline in version bumping
- Need tooling to validate version rules

## Validation Rules

The CLI enforces:

1. **Format:** Version must be valid semver (`X.Y.Z`)
2. **Breaking changes:** `backwards_compat: false` requires MAJOR > 0 or explicit bump from previous
3. **Experiments:** May use `0.x.x` versions freely (pre-stable)
4. **Substandards:** Inherit parent's APS major version in ID

## Examples

### Bumping Versions

```bash
# Patch: Bug fix
aps v1 version bump patch   # 1.2.3 → 1.2.4

# Minor: New feature
aps v1 version bump minor   # 1.2.3 → 1.3.0

# Major: Breaking change
aps v1 version bump major   # 1.2.3 → 2.0.0
```

### Consumer Lock File

```toml
# Pins exact versions for reproducibility
[[package]]
slug = "code-topology"
version = "0.1.0"
checksum = "sha256:a1b2c3..."

[[package]]
slug = "consumer-sdk"
version = "0.2.0"
checksum = "sha256:d4e5f6..."
```

## Related

- EXP-V1-0002: Consumer SDK (package distribution)
- ADR 0001 (Consumer SDK): GitHub Package Distribution
