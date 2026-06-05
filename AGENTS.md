# AGENTS.md

Guidance for coding agents working in the Agent Paradise Standards System repository.

## Project Overview

Agent Paradise Standards System (APSS) is a Rust workspace for executable, versioned engineering standards. Standards are implemented as Rust crates, validated by the APS CLI, and documented as agent-readable specifications.

## Repository Layout

- `crates/aps-core/` - core library for diagnostics, discovery, metadata, config, lockfiles, registry, and standard resolution.
- `crates/aps-cli/` - `aps` CLI for running standards and managing V1 standard lifecycle commands.
- `crates/apss-bootstrap/` - `apss` bootstrap CLI for consumer project initialization, installation, status, and dispatch.
- `standards/v1/APS-V1-0000-meta/` - meta-standard defining structure, validation, config, CLI, distribution, experiment lifecycle, and promotion rules.
- `standards/v1/` - official standards governed by the meta-standard.
- `standards-experimental/v1/` - incubating experimental standards governed by the meta-standard.
- `.github/workflows/` - CI, release gate, and release creation workflows.

## Setup Commands

- Install Rust with `rustup`; the workspace declares Rust `1.85` or newer.
- Install Just with `cargo install just`.
- Initialize local development with `just init`.
- Fetch dependencies with `cargo fetch` when needed.

## Build And Test Commands

- Run the standard local check suite: `just check`.
- Auto-format and then lint: `just check-fix`.
- Check formatting only: `cargo fmt --all --check`.
- Format Rust code: `cargo fmt --all`.
- Run clippy strictly: `cargo clippy --workspace --all-targets -- -D warnings`.
- Type-check all targets: `cargo check --workspace --all-targets`.
- Run tests: `cargo test --workspace`.
- Build all crates: `cargo build --workspace`.
- Build release artifacts: `cargo build --workspace --release`.
- Run the full CI-equivalent recipe: `just ci`.

## APS Validation Commands

- Validate all V1 standards: `just aps-validate`.
- Validate all V1 standards directly: `cargo run -p aps-cli --bin apss-dev -- v1 validate repo`.
- List discovered V1 packages: `just aps-list`.
- Validate a specific package: `just aps-validate-pkg <APS-ID>`.
- Create standards and experiments through the `aps-cli` recipes in `justfile`; do not hand-copy scaffold structures unless the CLI path is unsuitable.

## Code Style

- Keep Rust formatted with `cargo fmt`.
- Treat clippy warnings as failures; CI runs `-D warnings`.
- Prefer small, focused changes that preserve existing crate boundaries.
- Put shared engine behavior in `aps-core`, CLI user interactions in `aps-cli` or `apss-bootstrap`, and standard-specific behavior in the relevant standard crate.
- Avoid adding new workspace dependencies unless they are clearly justified and compatible with the release/distribution rules.
- Use `TODO` for intentional future improvements and `FIXME` only for known broken behavior.
- No em dashes are allowed anywhere in the repo. Restructure sentences or use colons or commas instead.

## Testing Guidelines

- Add or update tests for behavior changes.
- Prefer targeted tests first, then run broader workspace checks when the change is complete.
- For config or validation logic, cover success and diagnostic/error cases.
- For generated files such as schemas, include freshness or round-trip tests when practical.
- If touching standards, run `cargo run -p aps-cli --bin apss-dev -- v1 validate repo` before finishing.

## Standards Structure

- Treat the meta-standard as the source of truth for how this repository works. It governs official standards, experimental standards, substandards, versioning, backwards compatibility, artifact definitions, validation criteria, promotion criteria, and automation expectations.
- Standards are versioned so they can evolve over time. Major versions must remain backwards compatible within the guarantees defined by the meta-standard.
- Standards may define substandards. Substandards are first-class governed units and must follow the structure, metadata, validation, and lifecycle rules defined by the meta-standard.
- Standards should define artifacts when practical. Not every standard must produce artifacts, but artifact definitions are important because validators, queries, reports, and downstream automation can be built against them.
- Standards should include executable code where it makes the standard enforceable or useful. Typical code includes validators, generators, artifact producers, query helpers, scaffolds, and automation that can run in git hooks, QA pipelines, and CI/CD pipelines.
- Treat the meta-standard as the source of truth for experiment lifecycle rules, including experimental version bumps, validation criteria, promotion criteria, and removal from `standards-experimental/v1/` after promotion.
- Official standards live under `standards/v1/APS-V1-XXXX-<slug>/`.
- Substandards live under `standards/v1/APS-V1-XXXX-<slug>/substandards/<PROFILE>-<slug>/`.
- Experimental standards live under `standards-experimental/v1/EXP-V1-XXXX-<slug>/`.
- Experimental standards should be held to the same validation bar as official standards unless the meta-standard explicitly defines a narrower exception.
- Fast validation that the meta-standard enforces should run in QA automation, such as CI checks or git hooks, when practical.
- Standards and substandards should keep metadata files, `Cargo.toml`, docs, and implementation aligned.
- When changing non-documentation files in a standard or substandard, check whether `standard.toml`, `substandard.toml`, and `Cargo.toml` versions need to be bumped.

## Pull Request Guidelines

- Use conventional commit messages: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, or `chore:`.
- Before opening or updating a PR, run `just check`; if standards changed, also run `just aps-validate`.
- Keep PR descriptions specific about changed standards, crates, tests, and validation commands.
- Release PRs target `release` from `main` and need a `## Changelog` or `## Release Notes` section.

## Agent Operating Notes

- Read nearby files before editing; preserve established naming, module layout, and diagnostic conventions.
- Do not commit changes unless the user explicitly asks.
- Do not modify generated or build-output directories such as `target/`.
- Do not rewrite unrelated files while addressing a focused issue.
- If local changes already exist, treat them as user work and avoid overwriting them without confirmation.
