# Runbook: Visualize Your Codebase with the Topology Standard

## Purpose

Install APSS in a target repository, run the Code Topology standard (APS-V1-0001) against its source code, and generate interactive visualizations: a 3D coupling graph, CodeCity, cluster map, VSA matrix, and a combined dashboard. Optionally wire validation into a git pre-commit hook.

This runbook is written to be handed to a coding agent (for example Claude Code) verbatim. Every step has a command and an expected result. Run steps in order.

## TL;DR (copy-paste)

From the root of a repo that contains Rust or Python source:

```bash
cargo install apss                                  # one-time, needs apss 1.1.0+
apss init --standard code-topology                  # creates apss.yaml
# edit apss.yaml: set the code-topology id to APS-V1-0001 (see step 2)
apss install                                        # resolves the standard from crates.io
apss run code-topology analyze .                    # writes .topology/ artifacts
apss run code-topology viz .topology --type all     # opens the dashboard
```

That is the whole flow. The sections below explain each step, the expected output, the heavy-artifact caveat, and the optional pre-commit hook. The single manual touch today is setting the standard id in step 2 (a slug-to-id catalog is tracked in issue #76).

## Audience and Prerequisites

- A target repository containing Rust (`.rs`), Python (`.py`/`.pyi`), or TypeScript (`.ts`/`.tsx`) source files. JavaScript (`.js`/`.jsx`) is not yet supported by the analyzer.
- Rust toolchain and Cargo installed (`cargo --version` succeeds).
- Network access to crates.io. The standard is resolved and built from crates.io; no APSS repository checkout is required. If you are offline or air-gapped, see the appendix at the end.

## 1. Install the APSS CLI

```bash
cargo install apss
apss --version
```

Expected: `apss 1.1.0` or newer on PATH. Version 1.1.0 is the first release that resolves standards from crates.io; on 1.0.0, `apss install` requires a local bundle (see the offline appendix). If you already have an older `apss`, `cargo install apss --force` upgrades it.

## 2. Initialize the Target Repository

From the target repository root:

```bash
apss init --standard code-topology
```

Expected:

- `apss.yaml` created (user-owned, edit freely) with a `code-topology` standard entry.

The generated entry scaffolds the id as a placeholder:

```yaml
standards:
  code-topology:
    id: APS-V1-XXXX  # FIXME: replace with the real ID
    version: ">=0.1.0"
```

Replace the placeholder id with the Code Topology standard's real ID, `APS-V1-0001`:

```yaml
standards:
  code-topology:
    id: APS-V1-0001
    version: ">=0.1.0"
```

## 3. Install the Standard

From the target repository root:

```bash
apss install
```

Expected:

- The Code Topology standard is resolved from crates.io and pinned in `apss.lock`.
- A composed binary compiled to `.apss/bin/apss`.
- `Installed Git hook: .git/hooks/pre-commit` (see step 7).

Commit `apss.yaml` and `apss.lock`. Do not commit `.apss/`; it is generated output.

> [!IMPORTANT]
> Confirm the composed binary at `.apss/bin/apss` exists before relying on topology anywhere. That binary runs the native tree-sitter analyzer, which is fast (topology is ~0.1s on a small repo). If `apss install` has not run, or the binary is missing (`No apss.yaml found` / `Composed binary not found`), the standalone commands below fail outright, and any harness integration that wraps this standard (the architectural fitness gate) **silently falls back to a much slower external collector** (dependency-cruiser): measured at ~136s versus ~0.1s on the same repository. A misconfigured install does not error in the gate; it just gets quietly slow. See the harness-integration note in [adopt-apss-standards.runbook.md](adopt-apss-standards.runbook.md).

## 4. Validate the Installation

```bash
apss validate
apss status
```

Expected: `Validation passed.` and a status listing `code-topology  APS-V1-0001 <version> [all]` with the binary marked installed.

## 5. Run the Topology Analysis

From the target repository root:

```bash
apss run code-topology analyze .
```

Expected output shape:

```text
Found N source file(s):
  rust: N files
✓ Analyzed M functions (0 warnings)
✓ Wrote artifacts to .topology
```

Artifacts produced: `.topology/manifest.toml`, `metrics/modules.json`, `metrics/functions.json`, `metrics/slices.json`, `graphs/dependency-graph.json`, `graphs/coupling-matrix.json`.

> **Warning: topology JSON can get heavy.** Artifact size grows with codebase size, and `functions.json` dominates: a 14-file demo produces ~176 KB of artifacts, while a medium Rust workspace already produces a 400+ KB `functions.json`. Large monorepos can reach many megabytes. Do not load these files into an agent context wholesale; query specific fields instead. Consider adding `.topology/` to `.gitignore` and regenerating in CI rather than committing artifacts. Artifact slimming/sharding is a known future optimization.

## 6. Generate Visualizations

3D coupling graph only:

```bash
apss run code-topology viz .topology --type 3d
```

Full dashboard (recommended):

```bash
apss run code-topology viz .topology --type all
```

Expected: HTML files under `.topology/viz/` (`index.html`, `topology-3d.html`, `codecity.html`, `clusters.html`, `vsa.html`); the dashboard opens in the default browser. The viz command takes the `.topology` directory path, not the repo root (issue #70 tracks this inconsistency).

Visualization types: `3d` (force-directed coupling), `codecity` (buildings = modules), `clusters` (2D package graph), `vsa` (vertical slice matrix), `all`.

## 7. Git Pre-Commit Hook (Optional but Recommended)

`apss install` already installed a pre-commit hook that runs `apss validate` on every commit. Control it via `apss.yaml`:

```yaml
tool:
  hooks:
    pre_commit: true   # set false to disable, then rerun: apss install
```

Verify it works:

```bash
git commit --allow-empty -m "test: hook check"
```

Expected: validation output before the commit completes. Hook failures block the commit.

## Troubleshooting

| Symptom | Cause and Fix |
|---|---|
| `No supported source files found` | Target repo has no `.rs`/`.py`/`.ts`/`.tsx` files at or below the analyzed path. Point `analyze` at the source root. |
| `Standard 'topology' not found in apss.yaml` | The consumer config uses the slug `code-topology`; `topology` is a dev-CLI alias only (issue #70). |
| `No modules.json found at ./metrics/modules.json` | `viz` was given the repo root. Pass the `.topology` directory (issue #70). |
| `apss install` cannot reach crates.io | You are offline or behind a proxy. Use the offline install in the appendix. |
| `cargo install apss` fails on publish metadata | Update Rust; the workspace requires Rust 1.85+. |

## Related

- Consumer flow overview: root `README.md`, section "Using APSS in Your Project"
- Distribution model: ADR-0002 (`standards/v1/APS-V1-0000-meta/docs/adrs/0002-crates-io-distribution.md`) and the [DI01 distribution spec](../../standards/v1/APS-V1-0000-meta/substandards/DI01-distribution/docs/01_spec.md)
- Distribution lifecycle: `standards/v1/APS-V1-0000-meta/substandards/DI01-distribution/docs/03_package_manager_lifecycle.md`
- Release acceptance testing: `docs/testing/apss-package-manual-acceptance-testing.runbook.md`
- Tracking: #67 (runbooks), #70 (CLI UX); #68 closed by ADR-0002 Phase C (consumer binary runs all commands). ADR-0002 Phase D enables registry install (`apss install` from crates.io with no checkout).

## Appendix: Offline / Air-Gapped Install

The default flow in steps 1 to 3 resolves the standard from crates.io. When the consumer machine cannot reach crates.io (air-gapped or restricted networks), install from a locally built bundle instead. This path requires a one-time checkout of the APSS repository on a machine that can build it.

### A.1 Build the Topology Bundle

On a machine with the APSS repository checked out (`git clone https://github.com/AgentParadise/agent-paradise-standards-system`, set `APSS_REPO` to its path):

```bash
cd "$APSS_REPO"
mkdir -p /tmp/apss-bundles
cargo run -p aps-cli --bin apss-dev -- v1 bundle APS-V1-0001 --output /tmp/apss-bundles
```

Expected: `Created APSS bundle: /tmp/apss-bundles/APS-V1-0001-code-topology-<version>.apss`. The bundle is a self-contained source workspace: `apss-core`, the standard crate, and all substandard modules. Nothing else is downloaded. Transfer this bundle directory to the target machine if it differs from the build machine.

### A.2 Install From the Bundle

From the target repository root, after `apss init --standard code-topology` (step 2), install with `--bundle-dir` instead of the plain `apss install`:

```bash
apss install --bundle-dir /tmp/apss-bundles/APS-V1-0001-code-topology-<version>.apss
```

Expected: the same outcome as step 3 (composed binary at `.apss/bin/apss`, `apss.lock` written, pre-commit hook installed), but the standard source comes from the local bundle rather than crates.io. Steps 4 onward are identical.

If `.apss/bin/apss` reports `No composed CLI commands are registered`, the bundle was built from a pre-0.2.0 checkout. Rebuild the bundle from a current checkout and rerun `apss install --bundle-dir`.
