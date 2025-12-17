# Consumer SDK Examples

This directory contains examples demonstrating consumer SDK usage.

## Available Examples

| Example | Description |
|---------|-------------|
| `sample-manifest.toml` | Complete manifest with all configuration options |

## Usage

### Basic Setup

1. Copy `sample-manifest.toml` to your project as `.aps/manifest.toml`
2. Edit the project name and adopted standards
3. Run `aps sync` to generate artifacts

### Example Workflow

```bash
# In your project directory
mkdir -p .aps
cp /path/to/sample-manifest.toml .aps/manifest.toml

# Edit manifest
$EDITOR .aps/manifest.toml

# Sync artifacts
aps sync

# Check compliance
aps check
```

## Generated Files

After running `aps sync`, you'll have:

```
your-project/
├── .aps/
│   ├── manifest.toml    # Your configuration (committed)
│   └── index.json       # Auto-generated (commit recommended)
└── .topology/           # If code-topology adopted
    ├── manifest.toml
    ├── metrics/
    └── graphs/
```

## Dogfooding Example

For developing standards locally, use the submodule override:

```toml
[sources.overrides]
"github:AgentParadise/agent-paradise-standards-system" = { 
  type = "submodule",
  path = "lib/aps"
}
```

Then add APS as a git submodule:

```bash
git submodule add \
  git@github.com:AgentParadise/agent-paradise-standards-system.git \
  lib/aps
```
