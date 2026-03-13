#!/usr/bin/env bash
# APS-V1-0001 Code Topology — Pre-commit Hook
#
# Automatically regenerates .topology/ artifacts and stages them alongside
# your code changes, so topology data always stays in sync with the codebase.
#
# Install:
#   cp templates/pre-commit-topology.sh .git/hooks/pre-commit
#   chmod +x .git/hooks/pre-commit
#
# Or append to an existing pre-commit hook:
#   cat templates/pre-commit-topology.sh >> .git/hooks/pre-commit
#
# Or use with the pre-commit framework (https://pre-commit.com):
#   Add to .pre-commit-config.yaml:
#     - repo: local
#       hooks:
#         - id: topology
#           name: Regenerate topology artifacts
#           entry: bash -c 'scripts/topology-regen.sh'
#           language: system
#           pass_filenames: false

set -euo pipefail

# --- Configuration -----------------------------------------------------------
# Override these via environment variables if needed
APS_BIN="${APS_BIN:-aps}"                    # Path to aps CLI binary
TOPOLOGY_DIR="${TOPOLOGY_DIR:-.topology}"     # Output directory
TOPOLOGY_SEED="${TOPOLOGY_SEED:-42}"          # Deterministic seed

# --- Guard: only run if source files changed ---------------------------------
# Skip topology regeneration if only non-source files changed (docs, configs, etc.)
CHANGED_FILES=$(git diff --cached --name-only --diff-filter=ACMR)
if [ -z "$CHANGED_FILES" ]; then
    exit 0
fi

# Check if any source files changed (adjust extensions for your stack)
SOURCE_CHANGED=$(echo "$CHANGED_FILES" | grep -E '\.(py|rs|ts|tsx|js|jsx|go|java|kt|rb|c|cpp|h|hpp)$' || true)
if [ -z "$SOURCE_CHANGED" ]; then
    exit 0
fi

# --- Check: aps CLI available -----------------------------------------------
if ! command -v "$APS_BIN" &>/dev/null; then
    echo "⚠️  aps CLI not found (looked for: $APS_BIN)"
    echo "   Skipping topology regeneration."
    echo "   Install: cargo install aps-cli"
    echo "   Or set APS_BIN to the path of your aps binary."
    exit 0  # Don't block the commit — just warn
fi

# --- Regenerate topology artifacts -------------------------------------------
echo "🔍 Regenerating topology artifacts..."
"$APS_BIN" run topology analyze . --output "$TOPOLOGY_DIR" --seed "$TOPOLOGY_SEED" 2>/dev/null

if [ $? -eq 0 ]; then
    # Stage the updated topology artifacts
    git add "$TOPOLOGY_DIR/"
    echo "✅ Topology artifacts regenerated and staged"
else
    echo "⚠️  Topology analysis failed (non-blocking)"
fi
