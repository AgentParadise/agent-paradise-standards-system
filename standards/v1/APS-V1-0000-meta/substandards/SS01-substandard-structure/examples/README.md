# Examples — APS-V1-0000.SS01

## Substandard Structure Examples

### Example 1: Minimal Substandard

A minimal conformant substandard requires:

```
MY-PROFILE-01-example/
  substandard.toml
  Cargo.toml
  src/lib.rs
  docs/01_spec.md
  examples/README.md
  tests/README.md
  agents/skills/README.md
```

**substandard.toml**:
```toml
schema = "aps.substandard/v1"

[substandard]
id = "APS-V1-0001.EX01"
name = "Example Profile"
slug = "example"
version = "1.0.0"
parent_id = "APS-V1-0001"
parent_major = "1"

[ownership]
maintainers = ["YourTeam"]
```

### Example 2: Language Binding

A Python binding for a standard:

```
PY01-python/
  substandard.toml
  Cargo.toml           # Rust crate that may wrap Python
  src/lib.rs
  python/              # Python source
    __init__.py
    bindings.py
  docs/01_spec.md
  examples/
    basic_usage.py
    README.md
  tests/
    test_bindings.py
    README.md
  agents/skills/README.md
```

### Example 3: Platform Profile

A GitHub Actions integration:

```
GH01-github/
  substandard.toml
  Cargo.toml
  src/lib.rs
  .github/
    workflows/
      standard-check.yml
  docs/01_spec.md
  examples/
    workflow-example.yml
    README.md
  tests/README.md
  agents/skills/README.md
```

