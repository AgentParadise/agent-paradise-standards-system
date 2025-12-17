# ADR 0001: GitHub Package Distribution

**Status:** Accepted  
**Date:** 2025-12-17  
**Context:** EXP-V1-0002 Consumer SDK

## Context

The Consumer SDK needs a way to distribute standards to downstream projects. Currently, consumers must either:
1. Clone/submodule the entire APS monorepo
2. Manually copy standard files

Neither approach is ideal for composability or security.

## Decision

Use **GitHub Releases as a package registry** where each standard is published as an individual tarball with checksum verification.

### Key Components

1. **System Cache** (`~/.aps/cache/`) — Shared across projects, verified on extraction
2. **Lock File** (`.aps/manifest.lock`) — Pins exact versions and checksums
3. **Checksum Verification** — SHA-256 sidecar files for integrity
4. **Self-Validation** — Standards validate their own structure on install
5. **Registry** (`registry.json`) — Published with releases, lists available standards

## Architecture

```
Consumer Repo                    System Cache                 GitHub Releases
─────────────                    ────────────                 ───────────────
.aps/manifest.toml  ──────────►  ~/.aps/cache/  ◄──────────  *.tar.gz
.aps/manifest.lock               ├── code-topology/0.1.0/    *.tar.gz.sha256
.aps/index.json                  └── consumer-sdk/0.1.0/     registry.json
```

## Schemas

### manifest.lock

```toml
schema = "aps.lock/v1"
generated_at = "2025-12-17T12:00:00Z"

[[package]]
slug = "code-topology"
id = "EXP-V1-0001"
version = "0.1.0"
source = "github:AgentParadise/agent-paradise-standards-system"
checksum = "sha256:a1b2c3d4e5f6..."
resolved_url = "https://github.com/.../aps-code-topology-0.1.0.tar.gz"
```

### registry.json

```json
{
  "schema_version": "1.0.0",
  "standards": [
    {
      "slug": "code-topology",
      "id": "EXP-V1-0001", 
      "versions": [{"version": "0.1.0", "checksum": "sha256:..."}],
      "latest": "0.1.0"
    }
  ]
}
```

## Security Model

1. **Checksum verification** — Every download verified against SHA-256
2. **Lock file pinning** — Exact URLs and checksums prevent MITM
3. **Self-validation** — Packages validate their structure post-extraction
4. **Cache permissions** — Read-only after verification

## Alternatives Considered

### 1. Git Submodules
- **Pro:** Simple, uses existing Git infrastructure
- **Con:** Pulls entire repo, complex version management

### 2. Cargo-style Registry
- **Pro:** Industry standard
- **Con:** Requires separate infrastructure, overkill for documents

### 3. npm/PyPI Publishing
- **Pro:** Familiar tooling
- **Con:** Wrong ecosystem, adds dependencies

## Consequences

### Positive
- Composable: only pull standards you need
- Secure: checksums and validation
- Reproducible: lock files ensure consistency
- Familiar: similar to Cargo/npm patterns

### Negative
- More infrastructure: GitHub Actions workflow needed
- Cache management: users need to understand cache
- Offline: requires internet for first fetch (cache helps after)

## Implementation

See milestones in the Consumer SDK experiment:
- M1: Lock file support
- M2: System cache
- M3: Checksum verification
- M4: Self-validation
- M5: GitHub release integration
