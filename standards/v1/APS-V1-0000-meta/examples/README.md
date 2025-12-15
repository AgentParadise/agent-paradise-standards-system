# APS-V1-0000 Meta-Standard Examples

Examples demonstrating valid APS package structures and workflows.

## Available Examples

| Example | Description |
|---------|-------------|
| `minimal-standard/` | Minimal valid standard structure (reference) |

## Using Examples

Each example can be used as a reference when creating new standards:

```bash
# The meta-standard itself is the canonical example
ls standards/v1/APS-V1-0000-meta/

# Or create a new standard using the template
aps v1 create standard my-standard
```

## Example: Minimal Valid Standard

A minimal valid standard package requires:

```
APS-V1-XXXX-slug/
├── standard.toml       # Package metadata
├── Cargo.toml          # Rust crate manifest
├── src/
│   └── lib.rs          # Standard trait implementation
├── docs/
│   └── 01_spec.md      # Normative specification
├── examples/
│   └── README.md       # Examples index
├── tests/
│   └── README.md       # Test documentation
└── agents/
    └── skills/
        └── README.md   # Agent skill instructions
```

## CLI Workflow Example

```bash
# 1. Create a new experiment
aps v1 create experiment my-idea

# 2. Develop and iterate
# ... make changes ...

# 3. Validate
aps v1 validate experiment EXP-V1-0001

# 4. Promote to official standard
aps v1 promote EXP-V1-0001

# 5. Version management
aps v1 version bump APS-V1-0001 minor

# 6. Generate registry
aps v1 generate views
```
