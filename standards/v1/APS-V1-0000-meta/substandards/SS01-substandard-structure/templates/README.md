# Templates — APS-V1-0000.SS01

## Substandard Templates

Substandards reuse the template infrastructure from the parent meta-standard.

### Available Templates

The parent standard at `standards/v1/APS-V1-0000-meta/templates/substandard/` provides:

```
templates/substandard/
  skeleton/
    substandard.toml
    Cargo.toml
    src/lib.rs
    docs/
      00_overview.md
      01_spec.md
    examples/README.md
    tests/README.md
    agents/skills/README.md
  template.toml           # Template metadata
```

### Template Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `{{id}}` | Full substandard ID | `APS-V1-0001.GH01` |
| `{{name}}` | Human-readable name | `GitHub Profile` |
| `{{slug}}` | Kebab-case slug | `github` |
| `{{version}}` | Initial version | `1.0.0` |
| `{{parent_id}}` | Parent standard ID | `APS-V1-0001` |
| `{{parent_major}}` | Parent major version | `1` |
| `{{maintainers}}` | Maintainer list | `["AgentParadise"]` |

### Usage

```bash
# Create via CLI
aps v1 create substandard APS-V1-0001 --profile GH01 --name "GitHub Profile"

# Or programmatically
let context = SubstandardContext::new(
    "APS-V1-0001.GH01",
    "GitHub Profile",
    "github",
    "APS-V1-0001"
);
engine.render_skeleton(&skeleton_dir, &output_dir, &context)?;
```

### Customizing Templates

Substandards MAY provide their own templates for further specialization:

```
substandards/SS01-substandard-structure/
  templates/
    language-binding/      # Template for language bindings
    platform-profile/      # Template for platform integrations
```

