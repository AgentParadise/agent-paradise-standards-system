# CLI01 — CLI Contract Specification

**Version**: 1.0.0  
**Status**: Active  
**Parent**: APS-V1-0000 (Meta Standard)

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope

This specification defines:

1. The command-line interface structure for running APS standards
2. Command naming patterns and argument conventions
3. Output format requirements for automation
4. Exit code semantics
5. The `StandardCli` Rust trait for integration

---

## 2. Command Hierarchy

### 2.1 Root Structure

The APS CLI MUST follow this hierarchy:

```
aps
├── run <slug> <command> [args]   # Run standard CLI
│   └── --list                    # List available standards
│
├── v1                            # v1 authoring/meta commands
│   ├── validate                  # Validate repo structure
│   ├── create                    # Create packages
│   ├── list                      # List packages
│   ├── promote                   # Promote experiments
│   └── version                   # Version management
│
└── v2                            # Future: v2 authoring
```

### 2.2 Standard Dispatch

The `aps run` command dispatches to standard-specific CLIs:

```bash
aps run <slug> <command> [args...]
```

Where:
- `<slug>` is a standard slug or ID (e.g., "topology", "EXP-V1-0001")
- `<command>` is a standard-specific command (e.g., "analyze", "validate")
- `[args...]` are command-specific arguments

### 2.3 Slug Resolution

The CLI MUST resolve slugs to standard IDs using a registry:

| Input | Resolves To |
|-------|-------------|
| `topology` | EXP-V1-0001 |
| `topo` | EXP-V1-0001 |
| `code-topology` | EXP-V1-0001 |
| `EXP-V1-0001` | EXP-V1-0001 |

Resolution MUST be case-insensitive for slugs.

---

## 3. Standard Commands

### 3.1 Required Commands

Standards that produce artifacts MUST implement:

| Command | Description | Exit Codes |
|---------|-------------|------------|
| `validate <path>` | Validate artifacts at path | 0=pass, 1=fail, 2=warn |

### 3.2 Recommended Commands

Standards SHOULD implement:

| Command | Description | Arguments |
|---------|-------------|-----------|
| `analyze <path>` | Generate artifacts from codebase | `--output <dir>` |
| `check <path>` | Check repo compliance with standard | — |
| `diff <a> <b>` | Compare two artifact sets | `--format <fmt>` |

### 3.3 Optional Commands

Standards MAY implement:

| Command | Description |
|---------|-------------|
| `init` | Initialize standard in a repo |
| `report` | Generate human-readable report |
| `export` | Export to external format |

---

## 4. Argument Conventions

### 4.1 Path Arguments

Path arguments MUST support:

```bash
aps run topology analyze .                    # Current directory
aps run topology analyze /absolute/path       # Absolute path
aps run topology analyze ./relative/path      # Relative path
```

### 4.2 Common Flags

All standard commands SHOULD support:

| Flag | Description |
|------|-------------|
| `--help` | Show command help |
| `--json` | Output in JSON format |
| `--quiet` | Suppress non-error output |
| `--verbose` | Enable debug output |

### 4.3 Output Flags

Commands that produce output SHOULD support:

| Flag | Description |
|------|-------------|
| `--output <path>` | Output directory |
| `--format <fmt>` | Output format (json, toml, md) |

---

## 5. Output Formats

### 5.1 JSON Output

When `--json` is specified, output MUST conform to:

```json
{
  "status": "success|warning|error",
  "command": "topology analyze",
  "version": "0.1.0",
  "timestamp": "2025-12-17T12:00:00Z",
  "data": { },
  "diagnostics": [
    {
      "severity": "error|warning|info",
      "code": "MISSING_ARTIFACT",
      "message": "Missing .topology/manifest.toml",
      "location": {
        "file": ".",
        "line": null
      }
    }
  ]
}
```

### 5.2 Text Output

Default text output SHOULD be human-readable with:
- Clear section headers
- Colored output when terminal supports it
- Summary line at end

---

## 6. Exit Codes

### 6.1 Required Codes

All commands MUST use these exit codes:

| Code | Meaning | When |
|------|---------|------|
| 0 | Success | No errors or warnings |
| 1 | Error | Blocking errors found |
| 2 | Warning | Warnings only, no errors |

### 6.2 Optional Codes

Commands MAY use:

| Code | Meaning |
|------|---------|
| 3 | Invalid arguments |
| 4 | IO/system error |
| 5 | Not implemented |

---

## 7. StandardCli Trait

### 7.1 Trait Definition

Standards MUST implement the `StandardCli` trait:

```rust
/// Trait for standards that expose CLI commands.
pub trait StandardCli: Send + Sync {
    /// Standard slug for command dispatch.
    fn slug(&self) -> &str;
    
    /// Standard ID (e.g., "EXP-V1-0001").
    fn id(&self) -> &str;
    
    /// APS version this standard uses.
    fn aps_version(&self) -> &str;
    
    /// List supported commands.
    fn commands(&self) -> Vec<CliCommandInfo>;
    
    /// Execute a command.
    fn execute(&self, command: &str, args: &[String]) -> CliResult;
}
```

### 7.2 Command Info

```rust
/// Information about a CLI command.
pub struct CliCommandInfo {
    /// Command name (e.g., "analyze").
    pub name: String,
    /// Short description.
    pub description: String,
    /// Usage pattern.
    pub usage: String,
    /// Whether this command is required.
    pub required: bool,
}
```

### 7.3 Result Type

```rust
/// Result of a CLI command execution.
pub struct CliResult {
    /// Exit code.
    pub exit_code: i32,
    /// Status for JSON output.
    pub status: CliStatus,
    /// Structured output data.
    pub data: Option<serde_json::Value>,
    /// Diagnostic messages.
    pub diagnostics: Vec<Diagnostic>,
}

/// CLI execution status.
pub enum CliStatus {
    Success,
    Warning,
    Error,
}
```

---

## 8. Registration

### 8.1 Static Registry

For compiled-in standards, registration is static:

```rust
fn get_standard_cli(slug: &str) -> Option<Box<dyn StandardCli>> {
    match slug.to_lowercase().as_str() {
        "topology" | "topo" | "code-topology" | "exp-v1-0001" => {
            Some(Box::new(TopologyCli::new()))
        }
        _ => None,
    }
}
```

### 8.2 Discovery Command

`aps run --list` MUST output available standards:

```
Available Standards:

  topology (EXP-V1-0001) v0.1.0
    Code Topology - architectural metrics and visualization
    Commands: analyze, validate, diff, report

  (more standards...)
```

---

## 9. Examples

### 9.1 Code Topology

```bash
# Analyze a Rust project
aps run topology analyze . --output .topology/

# Validate artifacts
aps run topology validate .topology/

# Compare branches
aps run topology diff .topology-base/ .topology-pr/ --format json

# Generate report
aps run topology report .topology/ --format md
```

### 9.2 CI Integration

```yaml
- name: Check Topology
  run: |
    aps run topology analyze --output .topology-pr/
    aps run topology diff .topology-base/ .topology-pr/ --format json > diff.json
    
    if [ $(jq -r '.status' diff.json) = "error" ]; then
      exit 1
    fi
```

---

## Appendix A: Error Codes

Standard error codes for diagnostics:

| Code | Description |
|------|-------------|
| `MISSING_ARTIFACT` | Required artifact not found |
| `INVALID_SCHEMA` | Artifact doesn't match schema |
| `THRESHOLD_EXCEEDED` | Metric exceeds configured threshold |
| `PARSE_ERROR` | Failed to parse source file |
| `IO_ERROR` | File system error |
