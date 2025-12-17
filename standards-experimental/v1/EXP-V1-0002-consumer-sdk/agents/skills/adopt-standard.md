# Skill: Adopt Standard

**Purpose**: Help a project adopt an APS standard by setting up the manifest and generating artifacts.

## When to Use

Use this skill when:
- A project wants to adopt a new APS standard
- Setting up `.aps/manifest.toml` for the first time
- Adding a standard like code-topology to an existing project

## Prerequisites

- APS CLI available (`aps` command)
- Project has a root directory

## Steps

### 1. Initialize Manifest (if needed)

If `.aps/manifest.toml` doesn't exist:

```bash
aps init --name <project-name>
```

This creates:
- `.aps/manifest.toml` - declares adopted standards
- `.aps/index.json` - agent discovery index

### 2. Add Standard

Add the desired standard to the manifest:

```bash
aps add code-topology@0.1.0
```

Available standards:
- `code-topology` - Code complexity and coupling analysis
- `consumer-sdk` - Consumer SDK (meta)

### 3. Configure Substandards (Optional)

Edit `.aps/manifest.toml` to enable substandards:

```toml
[substandards]
"code-topology.rust-adapter" = { enabled = true }
"code-topology.3d-force" = { enabled = true }
"code-topology.mermaid" = { enabled = true }
```

### 4. Sync Artifacts

Generate the standard's artifacts:

```bash
aps sync
```

This creates the artifact directories specified by each standard.

### 5. Verify

Check that everything is set up correctly:

```bash
aps check
```

## Example Workflow

```bash
# In your project root
cd /path/to/my-project

# Initialize APS
aps init --name my-project

# Add code-topology
aps add code-topology@0.1.0

# Generate artifacts
aps sync

# Verify compliance
aps check

# Commit
git add .aps/ .topology/
git commit -m "feat: adopt code-topology standard"
```

## Troubleshooting

### "Manifest not found"

Run `aps init` first to create `.aps/manifest.toml`.

### "Unknown standard slug"

Check available standards with the source documentation or use known slugs:
- `code-topology`
- `consumer-sdk`

### Artifacts not generated

The `aps sync` command generates the index but may require standard-specific tools for full artifact generation. Check the standard's documentation.
