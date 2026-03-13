# Adopting CI01 in Your Project

## Prerequisites

1. A Rust project (or other language with an adapter)
2. GitHub Actions enabled
3. The `aps-cli` tool (or equivalent topology analyzer)

## Step 1: Install the Workflow

Copy the workflow template to your repository:

```bash
# From within your project root
mkdir -p .github/workflows

# Option A: Copy from this repo
cp path/to/CI01-github-actions/templates/topology-check.yml .github/workflows/

# Option B: Download directly
curl -o .github/workflows/topology-check.yml \
  https://raw.githubusercontent.com/AgentParadise/agent-paradise-standards-system/main/standards-experimental/v1/EXP-V1-0001-code-topology/substandards/CI01-github-actions/templates/topology-check.yml
```

## Step 2: Configure Thresholds

Create your threshold configuration:

```bash
mkdir -p .topology
cat > .topology/config.toml << 'EOF'
schema_version = "1.0.0"

[thresholds]
max_cyclomatic_warning = 10
max_cyclomatic_failure = 20
max_cognitive_warning = 15
max_cognitive_failure = 30
max_coupling_delta_warning = 0.10
max_coupling_delta_failure = 0.25
max_distance_warning = 0.5
max_distance_failure = 0.8

[behavior]
fail_on_warning = false
post_comment = true
include_diagrams = true

[ignore]
paths = ["tests/", "benches/"]
functions = ["*::main", "*::test_*"]
EOF
```

## Step 3: Baseline Your Project

Generate initial topology artifacts:

```bash
aps run topology analyze . --output .topology/

# Optionally commit the baseline
git add .topology/
git commit -m "chore: add topology baseline"
```

## Step 4: Test the Workflow

1. Create a branch with a complexity-increasing change
2. Open a PR
3. Watch the workflow run
4. Review the PR comment

## Customization

### Adjusting Thresholds

Different projects have different needs:

| Project Type | Recommended max_cyclomatic_failure |
|--------------|-----------------------------------|
| Library | 10-15 |
| Application | 15-25 |
| CLI Tool | 20-30 (entry points are complex) |
| Legacy Code | 30-50 (be pragmatic) |

### Ignoring Specific Code

```toml
[ignore]
# Ignore test files
paths = ["tests/", "benches/", "examples/"]

# Ignore specific functions
functions = [
    "*::main",           # CLI entry points
    "*::test_*",         # Test functions
    "*::benchmark_*",    # Benchmark functions
    "generated::*",      # Generated code
]

# Ignore specific modules
modules = [
    "vendor::*",         # Vendored dependencies
    "proto::*",          # Generated protobuf
]
```

### Warning-Only Mode

For gradual adoption, start in warning-only mode:

```toml
[behavior]
fail_on_warning = false
# Later, enable:
# fail_on_warning = true
```

### Custom Comment Template

Override the default comment template by creating `.topology/comment.md.hbs` (Handlebars template).

## Troubleshooting

### Workflow Fails to Analyze

1. Ensure `aps-cli` is installed correctly
2. Check that your project has a `Cargo.toml` (for Rust)
3. Verify the language adapter is available

### False Positives

If legitimate code is being flagged:

1. Add it to the ignore list
2. Adjust thresholds
3. Consider if the complexity is truly necessary

### Slow Analysis

For large projects:

```toml
[behavior]
# Only analyze changed files
incremental_only = true
```

## Integration with Branch Protection

Enable as a required status check:

1. Go to Settings → Branches → Branch protection rules
2. Add rule for `main` (or your default branch)
3. Enable "Require status checks to pass"
4. Select "Architecture Quality Gate"

## Gradual Rollout Strategy

1. **Week 1**: Warning-only mode, high thresholds
2. **Week 2-3**: Lower thresholds, still warning-only
3. **Week 4+**: Enable failure mode for new hotspots
4. **Ongoing**: Gradually lower thresholds as you refactor

