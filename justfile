# justfile - agent-paradise-standards-system task runner
# Cross-platform: Linux, macOS, Windows
#
# Usage: just <recipe>
# List recipes: just --list
# Recipe help: just --show <recipe>

# ═══════════════════════════════════════════════════════════════════════════════
# SETTINGS
# ═══════════════════════════════════════════════════════════════════════════════

set shell := ["bash", "-euc"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# ANSI colors
GREEN := '\033[0;32m'
YELLOW := '\033[0;33m'
RED := '\033[0;31m'
NORMAL := '\033[0m'

# ═══════════════════════════════════════════════════════════════════════════════
# HELP (default)
# ═══════════════════════════════════════════════════════════════════════════════

# Show available recipes
default:
    @just --list --unsorted

# ═══════════════════════════════════════════════════════════════════════════════
# QUALITY ASSURANCE
# ═══════════════════════════════════════════════════════════════════════════════

# Run all QA checks (format check, lint, test)
[group('qa')]
check: format lint test
    @echo '{{ GREEN }}════════════════════════════════════════{{ NORMAL }}'
    @echo '{{ GREEN }}✓ All checks passed!{{ NORMAL }}'
    @echo '{{ GREEN }}════════════════════════════════════════{{ NORMAL }}'

# Run QA with auto-fixes
[group('qa')]
check-fix: format-fix lint
    @echo '{{ GREEN }}✓ Auto-fixes applied{{ NORMAL }}'

# ═══════════════════════════════════════════════════════════════════════════════
# LINTING
# ═══════════════════════════════════════════════════════════════════════════════

# Check for lint errors
[group('lint')]
lint:
    @echo '{{ YELLOW }}Linting Rust code...{{ NORMAL }}'
    cargo clippy --workspace --all-targets -- -D warnings
    @echo '{{ GREEN }}✓ Lint check passed{{ NORMAL }}'

# ═══════════════════════════════════════════════════════════════════════════════
# FORMATTING
# ═══════════════════════════════════════════════════════════════════════════════

# Check formatting
[group('format')]
format:
    @echo '{{ YELLOW }}Checking Rust formatting...{{ NORMAL }}'
    cargo fmt --all --check
    @echo '{{ GREEN }}✓ Format check passed{{ NORMAL }}'

# Fix formatting
[group('format')]
format-fix:
    @echo '{{ YELLOW }}Formatting Rust code...{{ NORMAL }}'
    cargo fmt --all
    @echo '{{ GREEN }}✓ Formatting complete{{ NORMAL }}'

# ═══════════════════════════════════════════════════════════════════════════════
# TYPE CHECKING
# ═══════════════════════════════════════════════════════════════════════════════

# Run type checker (cargo check for Rust)
[group('typecheck')]
typecheck:
    @echo '{{ YELLOW }}Type checking Rust code...{{ NORMAL }}'
    cargo check --workspace --all-targets
    @echo '{{ GREEN }}✓ Type check passed{{ NORMAL }}'

# ═══════════════════════════════════════════════════════════════════════════════
# TESTING
# ═══════════════════════════════════════════════════════════════════════════════

# Run test suite
[group('test')]
test:
    @echo '{{ YELLOW }}Running tests...{{ NORMAL }}'
    cargo test --workspace
    @echo '{{ GREEN }}✓ Tests passed{{ NORMAL }}'

# Run tests in watch mode
[group('test')]
test-watch:
    @echo '{{ YELLOW }}Watching tests...{{ NORMAL }}'
    cargo watch -x test

# ═══════════════════════════════════════════════════════════════════════════════
# BUILDING
# ═══════════════════════════════════════════════════════════════════════════════

# Build project (debug)
[group('build')]
build:
    @echo '{{ YELLOW }}Building (debug)...{{ NORMAL }}'
    cargo build --workspace
    @echo '{{ GREEN }}✓ Build complete{{ NORMAL }}'

# Build project (release)
[group('build')]
build-release:
    @echo '{{ YELLOW }}Building (release)...{{ NORMAL }}'
    cargo build --workspace --release
    @echo '{{ GREEN }}✓ Release build complete{{ NORMAL }}'

# ═══════════════════════════════════════════════════════════════════════════════
# DEVELOPMENT
# ═══════════════════════════════════════════════════════════════════════════════

# Initialize development environment
[group('dev')]
[unix]
init:
    @echo '{{ GREEN }}Initializing development environment...{{ NORMAL }}'
    @command -v rustc >/dev/null 2>&1 || (echo '{{ RED }}Rust not found. Install from https://rustup.rs{{ NORMAL }}' && exit 1)
    @command -v just >/dev/null 2>&1 || (echo '{{ RED }}Just not found. Install: cargo install just{{ NORMAL }}' && exit 1)
    @echo '{{ GREEN }}Fetching dependencies...{{ NORMAL }}'
    cargo fetch
    @echo '{{ GREEN }}✓ Development environment ready!{{ NORMAL }}'

[group('dev')]
[windows]
init:
    Write-Host "Initializing development environment..." -ForegroundColor Green
    if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) { Write-Host "Rust not found. Install from https://rustup.rs" -ForegroundColor Red; exit 1 }
    if (-not (Get-Command just -ErrorAction SilentlyContinue)) { Write-Host "Just not found. Install: cargo install just" -ForegroundColor Red; exit 1 }
    Write-Host "Fetching dependencies..." -ForegroundColor Green
    cargo fetch
    Write-Host "✓ Development environment ready!" -ForegroundColor Green

# Clean build artifacts
[group('dev')]
[unix]
clean:
    @echo '{{ YELLOW }}Cleaning build artifacts...{{ NORMAL }}'
    cargo clean
    @echo '{{ GREEN }}✓ Clean complete{{ NORMAL }}'

[group('dev')]
[windows]
clean:
    Write-Host "Cleaning build artifacts..." -ForegroundColor Yellow
    cargo clean
    Write-Host "✓ Clean complete" -ForegroundColor Green

# ═══════════════════════════════════════════════════════════════════════════════
# CI
# ═══════════════════════════════════════════════════════════════════════════════

# CI pipeline (strict checks)
[group('ci')]
ci: format lint typecheck test build-release
    @echo '{{ GREEN }}════════════════════════════════════════{{ NORMAL }}'
    @echo '{{ GREEN }}✓ CI pipeline passed!{{ NORMAL }}'
    @echo '{{ GREEN }}════════════════════════════════════════{{ NORMAL }}'

# ═══════════════════════════════════════════════════════════════════════════════
# SUBPROJECTS
# ═══════════════════════════════════════════════════════════════════════════════

# Run agentic-primitives QA
[group('subprojects')]
ap-check:
    @echo '{{ YELLOW }}Running agentic-primitives QA...{{ NORMAL }}'
    cd lib/agentic-primitives && just check

# Run agentic-primitives QA with fixes
[group('subprojects')]
ap-check-fix:
    @echo '{{ YELLOW }}Running agentic-primitives QA with fixes...{{ NORMAL }}'
    cd lib/agentic-primitives && just check-fix

# ═══════════════════════════════════════════════════════════════════════════════
# UTILITIES
# ═══════════════════════════════════════════════════════════════════════════════

# Show version information
[group('utils')]
[unix]
version:
    @echo '{{ GREEN }}Rust version:{{ NORMAL }}'
    rustc --version
    cargo --version
    @echo ''
    @echo '{{ GREEN }}Just version:{{ NORMAL }}'
    just --version

[group('utils')]
[windows]
version:
    Write-Host "Rust version:" -ForegroundColor Green
    rustc --version
    cargo --version
    Write-Host ""
    Write-Host "Just version:" -ForegroundColor Green
    just --version

# Show TODO/FIXME comments
[group('utils')]
[unix]
todo:
    @echo '{{ YELLOW }}Scanning for TODO/FIXME comments...{{ NORMAL }}'
    rg -n "TODO|FIXME" --glob '*.rs' --glob '*.toml' --glob '*.md' . || echo '{{ GREEN }}No TODOs found!{{ NORMAL }}'

[group('utils')]
[windows]
todo:
    Write-Host "Scanning for TODO/FIXME comments..." -ForegroundColor Yellow
    rg -n "TODO|FIXME" --glob '*.rs' --glob '*.toml' --glob '*.md' .

# Run security audit
[group('utils')]
audit:
    @echo '{{ YELLOW }}Running security audit...{{ NORMAL }}'
    cargo audit || echo 'Install: cargo install cargo-audit'

# Check for outdated dependencies
[group('utils')]
deps-check:
    @echo '{{ YELLOW }}Checking dependencies...{{ NORMAL }}'
    cargo outdated || echo 'Install: cargo install cargo-outdated'

# Update dependencies
[group('utils')]
deps-update:
    @echo '{{ YELLOW }}Updating dependencies...{{ NORMAL }}'
    cargo update
    @echo '{{ GREEN }}✓ Dependencies updated{{ NORMAL }}'

