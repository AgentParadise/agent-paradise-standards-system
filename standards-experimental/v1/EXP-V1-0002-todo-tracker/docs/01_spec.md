# EXP-V1-0002 — TODO/FIXME Tracker and Issue Validator

**Version**: 0.1.0
**Status**: Experimental
**Category**: Technical

⚠️ **EXPERIMENTAL**: This standard is in incubation and may change significantly before promotion.

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope and Authority

### 1.1 Purpose

This standard defines a **language-agnostic artifact format** for tracking TODO and FIXME comments in source code, with validation that they reference issue tracker items. The artifacts are designed to be:

1. **Committable** — Stored in version control alongside code
2. **Deterministic** — Same codebase → same artifacts
3. **Machine-readable** — Consumable by AI agents and tooling
4. **Human-reviewable** — Diffable and inspectable
5. **Actionable** — Enables enforcement and visualization

### 1.2 Scope

This standard covers:

- **Artifact format specification** — Directory structure and file schemas
- **Comment format specification** — Required syntax for TODO/FIXME items
- **Validation rules** — Format compliance and issue reference validation
- **Configuration schema** — How to configure the tracker
- **GitHub integration** — Optional API validation

This standard does NOT cover:

- Specific scanning implementations (informative only)
- IDE integrations
- Issue tracker integrations beyond GitHub (reserved for future)
- Automatic issue creation

### 1.3 Relationship to Other Standards

This standard is independent but designed to integrate with:

- **EXP-V1-0001 (Code Topology)** — Can reference TODO items in complexity reports
- **Consumer SDK** — Can be adopted via `.aps/manifest.toml`
- **CI/CD Substandards** — Can be enforced in GitHub Actions

---

## 2. Core Definitions

### 2.1 TODO Item

A **TODO item** is a source code comment indicating work to be done, using one of the recognized tags (TODO, FIXME, etc.).

### 2.2 Tracked Item

A **tracked item** is a TODO item that includes a reference to an issue tracker item (e.g., GitHub issue).

### 2.3 Untracked Item

An **untracked item** is a TODO item without an issue reference.

### 2.4 Artifact

An **artifact** is the persisted, committable output of TODO/FIXME scanning. Artifacts are stored in a `.todo-tracker/` directory at the repository root (or configurable location).

---

## 3. Comment Format Specification

### 3.1 Required Format

TODO and FIXME comments MUST follow this format:

```
TAG(#ISSUE_NUMBER): DESCRIPTION
```

Where:
- `TAG` is one of: `TODO`, `FIXME` (extensible via configuration)
- `#ISSUE_NUMBER` is a GitHub issue number (e.g., `#123`)
- `DESCRIPTION` is a human-readable description of the work

### 3.2 Language-Specific Examples

**Rust:**
```rust
// TODO(#123): Add integration tests with real repository
fn validate_token() {
    // FIXME(#456): This breaks with empty input
}
```

**TypeScript/JavaScript:**
```typescript
// TODO(#789): Implement retry logic
function fetchData() {
    /* FIXME(#101): Handle network errors */
}
```

**Python:**
```python
# TODO(#202): Add type hints
def process_data():
    # FIXME(#303): Fix memory leak
    pass
```

### 3.3 Invalid Formats

The following formats are NOT compliant:

```rust
// TODO: Add tests (missing issue reference)
// TODO #123 Add tests (missing colon)
// TODO(123): Add tests (missing # symbol)
// TODO(#123) Add tests (missing colon)
```

### 3.4 Multi-line Comments

For multi-line TODO items, only the first line MUST contain the issue reference:

```rust
// TODO(#123): Refactor authentication system
// This includes:
// - Token validation
// - Session management
// - Permission checks
```

---

## 4. Artifact Format Specification

### 4.1 Directory Structure

The tracker MUST generate artifacts in the following structure:

```
.todo-tracker/
├── manifest.toml           # Scan metadata (REQUIRED)
├── items.json              # All TODO/FIXME items (REQUIRED)
├── summary.json            # Statistics (REQUIRED)
└── validation/
    └── cache.json          # GitHub API cache (OPTIONAL, gitignored)
```

### 4.2 Manifest Schema

**File:** `.todo-tracker/manifest.toml`

```toml
schema = "aps.todo-tracker/v1"
generated_at = "2026-01-21T10:30:00Z"
scanner_version = "0.1.0"

[scan]
root_path = "."
files_scanned = 245
lines_scanned = 15432
items_found = 18

[config]
tags = ["TODO", "FIXME"]
require_issue = true
enforcement = "warn"  # warn | error | off

[github]
enabled = false
repo = "AgentParadise/agent-paradise-standards-system"
```

**Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema` | string | YES | Must be `"aps.todo-tracker/v1"` |
| `generated_at` | ISO 8601 | YES | Timestamp of scan |
| `scanner_version` | semver | YES | Version of scanner used |
| `scan.root_path` | string | YES | Root directory scanned |
| `scan.files_scanned` | integer | YES | Number of files scanned |
| `scan.lines_scanned` | integer | YES | Total lines scanned |
| `scan.items_found` | integer | YES | Total items found |
| `config.tags` | array[string] | YES | Tags scanned for |
| `config.require_issue` | boolean | YES | Whether issues are required |
| `config.enforcement` | enum | YES | `"warn"`, `"error"`, or `"off"` |
| `github.enabled` | boolean | NO | Whether API validation was used |
| `github.repo` | string | NO | Repository validated against |

### 4.3 Items Schema

**File:** `.todo-tracker/items.json`

This is the **core artifact** that represents all TODO/FIXME items found.

```json
{
  "schema_version": "1.0.0",
  "generated_at": "2026-01-21T10:30:00Z",
  "items": [
    {
      "id": "a1b2c3d4e5f6...",
      "tag": "TODO",
      "file": "src/auth.rs",
      "line": 45,
      "column": 5,
      "text": "TODO(#123): Add integration tests with real repository",
      "description": "Add integration tests with real repository",
      "issue": {
        "type": "github",
        "number": 123,
        "repo": "AgentParadise/agent-paradise-standards-system",
        "url": "https://github.com/AgentParadise/agent-paradise-standards-system/issues/123",
        "validated": true,
        "state": "open",
        "title": "Add integration tests",
        "validated_at": "2026-01-21T10:30:00Z"
      },
      "context": {
        "function": "validate_token",
        "module": "auth",
        "before": "pub fn validate_token(token: &str) -> Result<(), Error> {",
        "after": "    let decoded = decode_jwt(token)?;"
      }
    }
  ]
}
```

**Item Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | YES | SHA256 hash of file+line+text |
| `tag` | string | YES | Tag type (TODO, FIXME, etc.) |
| `file` | string | YES | File path relative to repo root |
| `line` | integer | YES | Line number (1-indexed) |
| `column` | integer | YES | Column number (1-indexed) |
| `text` | string | YES | Full comment text |
| `description` | string | YES | Extracted description |
| `issue` | object\|null | YES | Issue reference or null |
| `context` | object\|null | NO | Code context (optional) |

**Issue Reference Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | YES | Issue tracker type (`"github"`) |
| `number` | integer | YES | Issue number |
| `repo` | string | NO | Repository (if validated) |
| `url` | string | NO | Full issue URL |
| `validated` | boolean | YES | Whether API validation occurred |
| `state` | string | NO | Issue state (`"open"`, `"closed"`) |
| `title` | string | NO | Issue title (from API) |
| `validated_at` | ISO 8601 | NO | When validation occurred |

**Context Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `function` | string | NO | Function/method name |
| `module` | string | NO | Module/class name |
| `before` | string | NO | Line before comment |
| `after` | string | NO | Line after comment |

### 4.4 Summary Schema

**File:** `.todo-tracker/summary.json`

```json
{
  "schema_version": "1.0.0",
  "generated_at": "2026-01-21T10:30:00Z",
  "totals": {
    "items": 18,
    "files": 8,
    "tracked": 12,
    "untracked": 6
  },
  "by_tag": {
    "TODO": {
      "total": 15,
      "tracked": 10,
      "untracked": 5
    },
    "FIXME": {
      "total": 3,
      "tracked": 2,
      "untracked": 1
    }
  },
  "by_file": {
    "src/auth.rs": 5,
    "src/api.rs": 3,
    "tests/integration.rs": 2
  },
  "by_issue": {
    "123": 5,
    "456": 2,
    "null": 6
  },
  "validation": {
    "github_api_enabled": true,
    "validated_count": 12,
    "failed_count": 0,
    "open_issues": 12,
    "closed_issues": 0
  }
}
```

---

## 5. Configuration Schema

### 5.1 Configuration File

**File:** `.todo-tracker.toml` (optional, at repository root)

```toml
schema = "aps.todo-tracker-config/v1"

[tracker]
# Tags to scan for
tags = ["TODO", "FIXME", "HACK", "XXX"]

# Which tags require issue references
require_issue_tags = ["TODO", "FIXME"]

[enforcement]
# How to handle missing issue references: off | warn | error
missing_issue = "warn"

# How to handle malformed format: off | warn | error
invalid_format = "warn"

# How to handle closed issues: off | warn | error (requires GitHub validation)
closed_issue = "warn"

[github]
# Enable GitHub API validation
enabled = false

# Repository (auto-detected from .git/config if not specified)
repo = "AgentParadise/agent-paradise-standards-system"

# Environment variable for token (default: GITHUB_TOKEN)
token_env = "GITHUB_TOKEN"

# Cache validation results (hours)
cache_ttl = 24

[scan]
# Paths to include (glob patterns)
include = ["src/**", "crates/**", "tests/**"]

# Paths to exclude (glob patterns)
exclude = ["target/**", "node_modules/**", ".git/**", "dist/**"]

# File extensions to scan (empty = all text files)
extensions = [".rs", ".ts", ".tsx", ".js", ".jsx", ".py"]

# Maximum file size to scan (MB)
max_file_size = 10
```

### 5.2 Default Configuration

If no `.todo-tracker.toml` exists, the following defaults apply:

```toml
[tracker]
tags = ["TODO", "FIXME"]
require_issue_tags = ["TODO", "FIXME"]

[enforcement]
missing_issue = "warn"
invalid_format = "warn"
closed_issue = "off"

[github]
enabled = false
token_env = "GITHUB_TOKEN"
cache_ttl = 24

[scan]
include = ["**/*"]
exclude = ["target/**", "node_modules/**", ".git/**", "dist/**", "build/**"]
extensions = []  # All text files
max_file_size = 10
```

---

## 6. Validation Rules

### 6.1 Format Validation

A TODO/FIXME comment is **valid** if:

1. It matches the pattern: `TAG(#N): description`
2. `TAG` is in the configured `tags` list
3. `N` is a positive integer
4. `description` is non-empty

### 6.2 Issue Reference Validation

If `require_issue = true` and `TAG` is in `require_issue_tags`:

- **MUST** have an issue reference in format `(#N)`
- **MAY** trigger GitHub API validation if enabled
- **MAY** warn/error based on `enforcement.missing_issue` setting

### 6.3 GitHub API Validation

If `github.enabled = true`:

1. Scanner MUST attempt to validate issue exists
2. Scanner MUST cache results for `cache_ttl` hours
3. Scanner MUST gracefully degrade on API failures
4. Scanner MAY warn on closed issues if configured

### 6.4 Enforcement Levels

| Level | Behavior |
|-------|----------|
| `off` | No validation, informational only |
| `warn` | Report violations, exit code 0 |
| `error` | Report violations, exit code 1 |

---

## 7. Language Support

### 7.1 Comment Pattern Detection

The scanner MUST support the following comment patterns:

| Language | Line Comments | Block Comments |
|----------|---------------|----------------|
| Rust | `//`, `///`, `//!` | `/*`, `/*!` |
| TypeScript/JavaScript | `//` | `/*` |
| Python | `#` | - |
| Go | `//` | `/*` |
| C/C++ | `//` | `/*` |
| Java | `//` | `/*` |
| Ruby | `#` | - |
| Shell | `#` | - |

### 7.2 Extensibility

Additional languages MAY be supported by:

1. Detecting file extension
2. Applying appropriate comment patterns
3. Using generic fallback for unknown languages

---

## 8. GitHub Integration

### 8.1 Repository Detection

The scanner SHOULD auto-detect the GitHub repository from:

1. `.git/config` remote URL
2. Explicit configuration in `.todo-tracker.toml`

### 8.2 Authentication

GitHub API validation REQUIRES:

1. Personal Access Token (PAT) in environment variable
2. Default variable name: `GITHUB_TOKEN`
3. Configurable via `github.token_env`

### 8.3 Rate Limiting

The scanner MUST:

1. Respect GitHub API rate limits
2. Cache validation results
3. Gracefully degrade on rate limit errors

### 8.4 API Endpoints

The scanner SHOULD use:

- `GET /repos/{owner}/{repo}/issues/{issue_number}` for validation

---

## 9. Compliance Checklist

A repository is **compliant** with this standard if:

- [ ] All TODO/FIXME comments follow the required format
- [ ] Required tags have issue references
- [ ] `.todo-tracker/` artifacts are generated
- [ ] `manifest.toml` conforms to schema
- [ ] `items.json` conforms to schema
- [ ] `summary.json` conforms to schema
- [ ] Artifacts are committable and deterministic

---

## 10. Informative: Scanner Implementation

### 10.1 Scanning Algorithm

A reference implementation SHOULD:

1. Discover files (respecting `.gitignore` and exclusions)
2. For each file:
   - Detect language from extension
   - Read line-by-line
   - Match comment patterns
   - Extract TODO/FIXME items
   - Parse issue references
3. Generate artifacts
4. Optionally validate via GitHub API

### 10.2 ID Generation

Item IDs SHOULD be generated as:

```
SHA256(file_path + ":" + line_number + ":" + text)
```

This ensures:
- Deterministic IDs
- Uniqueness per location
- Stability across scans

---

## 11. Security Considerations

### 11.1 Token Storage

GitHub tokens MUST:

- Be stored in environment variables (not config files)
- Never be committed to version control
- Have minimal required permissions (read:issues)

### 11.2 API Validation

GitHub API validation:

- Is OPTIONAL by default
- MUST gracefully handle failures
- SHOULD NOT block on network errors

---

## 12. Future Extensions

Potential future additions (not in v0.1.0):

- Historical tracking and trend analysis
- Support for other issue trackers (GitLab, Jira, Linear)
- Auto-creation of GitHub issues
- IDE integrations
- Custom tag definitions
- Priority levels
- Assignee tracking

---

## 13. References

- [RFC 2119: Key words for use in RFCs](https://datatracker.ietf.org/doc/html/rfc2119)
- [GitHub REST API: Issues](https://docs.github.com/en/rest/issues/issues)
- [Semantic Versioning](https://semver.org/)

---

## Appendix A: Complete Example

**Source File:** `src/auth.rs`

```rust
pub mod auth {
    // TODO(#123): Add integration tests with real repository
    pub fn validate_token(token: &str) -> Result<(), Error> {
        // FIXME(#456): This breaks with empty input
        let decoded = decode_jwt(token)?;
        Ok(())
    }
    
    // TODO: Add rate limiting (INVALID - missing issue)
}
```

**Generated Artifacts:**

`.todo-tracker/manifest.toml`:
```toml
schema = "aps.todo-tracker/v1"
generated_at = "2026-01-21T10:30:00Z"
scanner_version = "0.1.0"

[scan]
root_path = "."
files_scanned = 1
lines_scanned = 12
items_found = 3

[config]
tags = ["TODO", "FIXME"]
require_issue = true
enforcement = "warn"
```

`.todo-tracker/items.json`:
```json
{
  "schema_version": "1.0.0",
  "generated_at": "2026-01-21T10:30:00Z",
  "items": [
    {
      "id": "abc123...",
      "tag": "TODO",
      "file": "src/auth.rs",
      "line": 2,
      "column": 5,
      "text": "TODO(#123): Add integration tests with real repository",
      "description": "Add integration tests with real repository",
      "issue": {
        "type": "github",
        "number": 123,
        "validated": false
      }
    },
    {
      "id": "def456...",
      "tag": "FIXME",
      "file": "src/auth.rs",
      "line": 4,
      "column": 9,
      "text": "FIXME(#456): This breaks with empty input",
      "description": "This breaks with empty input",
      "issue": {
        "type": "github",
        "number": 456,
        "validated": false
      }
    },
    {
      "id": "ghi789...",
      "tag": "TODO",
      "file": "src/auth.rs",
      "line": 9,
      "column": 5,
      "text": "TODO: Add rate limiting",
      "description": "Add rate limiting",
      "issue": null
    }
  ]
}
```

**Validation Output:**
```
⚠️  1 item missing issue reference:
  src/auth.rs:9 - TODO: Add rate limiting
```

---*End of Specification*
