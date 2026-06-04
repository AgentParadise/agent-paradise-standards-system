# TODO/FIXME Tracker — Overview

## What is this?

**EXP-V1-0002** defines a standard format for tracking TODO and FIXME comments in source code, with validation that they reference GitHub issues. It enables:

- **Action item tracking** — Discover all pending work across the codebase
- **Issue enforcement** — Ensure every TODO/FIXME is tracked in GitHub
- **AI agent visibility** — Structured artifacts for automated reasoning
- **Technical debt management** — Prevent orphaned comments

## Why does it matter?

As codebases scale (especially with AI agents contributing), maintaining visibility into pending work becomes critical. This standard provides:

1. **Committed artifacts** — Version-controlled TODO snapshots
2. **Language-agnostic format** — Same structure for Rust, TypeScript, Python
3. **AI-consumable data** — Structured JSON for agent reasoning
4. **Configurable enforcement** — Warn or error on untracked items

## Quick Example

**Source Code:**
```rust
// TODO(#123): Add integration tests with real repository
pub fn validate_token(token: &str) -> Result<(), Error> {
    // FIXME(#456): This breaks with empty input
    decode_jwt(token)
}
```

**Generated Artifacts:**
```
.todo-tracker/
├── manifest.toml              # Scan metadata
├── items.json                 # All TODO/FIXME items (core artifact)
└── summary.json               # Statistics
```

**items.json:**
```json
{
  "items": [
    {
      "tag": "TODO",
      "file": "src/auth.rs",
      "line": 1,
      "text": "TODO(#123): Add integration tests with real repository",
      "issue": {
        "type": "github",
        "number": 123
      }
    }
  ]
}
```

## Key Features

### 1. Format Enforcement

Required format: `TAG(#N): description`

```rust
✅ // TODO(#123): Add tests
❌ // TODO: Add tests (missing issue)
❌ // TODO #123 Add tests (missing colon)
```

### 2. Polyglot Support

Works across languages:
- **Rust:** `//`, `///`, `/*`
- **TypeScript/JavaScript:** `//`, `/*`
- **Python:** `#`
- **Go, C++, Java, Ruby, Shell:** Supported

### 3. GitHub Integration (Optional)

Validate issues exist via GitHub API:
- Check issue is open
- Cache results (24h TTL)
- Graceful degradation on failures

### 4. Configurable Enforcement

```toml
[enforcement]
missing_issue = "warn"  # warn | error | off
```

## Use Cases

### For Developers
- **Discover work** — See all pending TODOs at a glance
- **Track progress** — Monitor TODO count over time
- **Prevent orphans** — Ensure every TODO has an issue

### For AI Agents
- **Understand backlog** — Read `.todo-tracker/items.json` to see pending work
- **Prioritize tasks** — Group by issue number or file
- **Validate compliance** — Check all TODOs are tracked

### For Teams
- **Code reviews** — Catch untracked TODOs in PRs
- **CI/CD** — Fail builds on untracked items
- **Sprint planning** — Visualize technical debt

## Getting Started

### 1. Install Scanner

```bash
cargo build --release -p todo-tracker
```

### 2. Scan Your Codebase

```bash
aps todos scan
```

### 3. Review Artifacts

```bash
cat .todo-tracker/summary.json
```

### 4. Configure (Optional)

Create `.todo-tracker.toml`:

```toml
[tracker]
tags = ["TODO", "FIXME", "HACK"]

[enforcement]
missing_issue = "error"  # Fail on untracked items

[github]
enabled = true
repo = "owner/repo"
```

### 5. Add to CI

```yaml
- name: Check TODOs
  run: aps todos scan --fail-on-untracked
```

## Architecture

```
EXP-V1-0002 (This Standard)
├── Artifact Format ← The standard (JSON schema)
├── Comment Format ← Required syntax
├── Configuration ← How to customize
└── GitHub Integration ← Optional validation

Implementation:
├── Scanner (regex-based)
├── Validator (format checking)
├── GitHub API Client (optional)
└── Report Generator (human-readable)
```

## Configuration

### Minimal (Defaults)

No config needed! Defaults:
- Tags: `TODO`, `FIXME`
- Enforcement: `warn`
- GitHub: disabled

### Custom

```toml
[tracker]
tags = ["TODO", "FIXME", "HACK", "XXX"]
require_issue_tags = ["TODO", "FIXME"]

[enforcement]
missing_issue = "error"
invalid_format = "warn"

[github]
enabled = true
token_env = "GITHUB_TOKEN"

[scan]
include = ["src/**", "tests/**"]
exclude = ["target/**", "node_modules/**"]
```

## Artifacts Explained

### manifest.toml
Metadata about the scan:
- When it ran
- How many files scanned
- Configuration snapshot

### items.json (Core Artifact)
**This is the standard** — all TODO/FIXME items with:
- Location (file, line, column)
- Issue reference (if present)
- Description
- Validation status

### summary.json
Quick statistics:
- Total count
- By tag (TODO vs FIXME)
- By file
- By issue
- Tracked vs untracked

## Example Workflow

### Developer Workflow

```bash
# 1. Write code with TODOs
# TODO(#123): Add error handling

# 2. Scan before commit
aps todos scan

# 3. Review untracked items
⚠️  1 item missing issue reference

# 4. Create GitHub issue #124

# 5. Update comment
# TODO(#124): Add error handling

# 6. Scan again
✅ All items tracked

# 7. Commit
git add .todo-tracker/
git commit -m "feat: add error handling TODO"
```

### CI/CD Workflow

```yaml
name: TODO Check
on: [pull_request]
jobs:
  check-todos:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Scan TODOs
        run: aps todos scan --validate-github
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      - name: Enforce tracking
        run: aps todos validate --fail-on-untracked
```

## Status

**Experimental** — This standard is in incubation. Feedback welcome!

### What's Working
- ✅ Artifact schema defined
- ✅ Format specification complete
- 🚧 Scanner implementation (in progress)
- 🚧 GitHub API integration (in progress)
- ⏳ CLI commands (planned)
- ⏳ CI/CD templates (planned)

### Next Steps
1. Implement scanner
2. Add GitHub validation
3. Create CLI commands
4. Dogfood in this repository
5. Gather feedback
6. Iterate toward promotion

## Related Standards

- **EXP-V1-0001 (Code Topology)** — Can cross-reference TODOs with complexity metrics
- **Consumer SDK** — Can be adopted via `.aps/manifest.toml`
- **CI/CD Substandards** — GitHub Actions integration

## Learn More

- Read the [full specification](./01_spec.md)
- Check out [examples](../examples/)
- See [agent skills](../agents/skills/)

---

*This is an experimental standard. It may change significantly before promotion to official status.*
