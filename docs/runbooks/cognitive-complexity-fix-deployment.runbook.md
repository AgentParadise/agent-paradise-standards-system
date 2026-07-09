# Deployment: cognitive-complexity nesting fix

This runbook covers shipping the cognitive-complexity correctness fix
(`fix/cognitive-complexity-nesting-off-by-one`) from this repo to
crates.io and on to the `harness-app-template` consumer.

## What changed and why it matters for consumers

The TS/JS cognitive-complexity metric was inflated by an off-by-one
nesting error: the measured function's own tree-sitter node counted as a
nesting level, charging every construct in the body one extra nesting
penalty. Additionally `switch`/`match` were charged per case/arm instead
of once (SonarSource charges the structure a single time).

Effect on emitted values: **cognitive scores drop for real code** (e.g.
4 flat `if`s went 8 → 4; 3 nested `if`s went 9 → 6). This is a
behavior-changing correctness fix: any consumer that has pinned a
baseline against the old (inflated) numbers will see its readings move.

## 1. Version bump

`apss-v1-0001-code-topology` is at `0.2.0` (0.x). Under SemVer, a 0.x
minor bump signals a breaking change. Because emitted metric values
shift, which breaks consumers pinning baselines; this is breaking in
practice even though it is a bug fix.

Recommended bumps:

| Crate | Package | From | To | Why |
|---|---|---|---|---|
| Analyzer | `apss-v1-0001-code-topology` | 0.2.0 | **0.3.0** | Output values change → breaking for 0.x |
| CLI | `aps-cli` (bin `apss-dev`) | 1.1.0 | **1.2.0** | Pins the analyzer; user-visible output changes |
| Core | `apss-core` | 1.1.0 | unchanged* | No code change |

\* The workspace pins `aps-cli`/`apss-core`/meta crates via
`version.workspace = true` (currently `1.1.0`). Bumping the workspace
version to publish the CLI will also roll the version stamp of every
workspace-versioned crate. If you prefer to avoid dragging `apss-core`'s
number, give `aps-cli` an explicit `version = "1.2.0"` in its own
`[package]` instead of inheriting the workspace version. Either way,
only the analyzer crate has a code change; the CLI must be republished
so its pinned dependency (`code-topology = "0.3.0"`) resolves.

Also update the pin in `crates/aps-cli/Cargo.toml`:
`code-topology = { package = "apss-v1-0001-code-topology", version = "0.3.0", ... }`.

## 2. Publish order (crates.io)

Publish leaf-first along the dependency graph
(`apss-core` ← `apss-v1-0001-code-topology` ← `aps-cli`):

```sh
# apss-core only if its version actually changed:
cargo publish -p apss-core

# analyzer crate (the real change):
cargo publish -p apss-v1-0001-code-topology

# CLI (after the analyzer 0.3.0 is live on crates.io):
cargo publish -p aps-cli
```

Wait for each publish to be indexed before publishing the next dependent
crate. Verify with `cargo search apss-v1-0001-code-topology`.

## 3. Consumer update (harness-app-template)

1. Bump the APSS pin in the consumer's `apss.lock` / `apss.yaml` to the
   fixed analyzer/CLI version.
2. **Re-derive the sensors baseline.** MT01 `max-cognitive` (and the
   cognitive-derived floors) were ratcheted against the old inflated
   numbers. After upgrading, run the sensors pipeline and re-baseline:
   `just sensors gate --update-baseline` so
   `harness/sensors/baseline.json` reflects the corrected (lower)
   cognitive values as a reviewable, audit-trailed edit.
3. If APSS was disabled as a cognitive source in
   `harness/sensors/gate.mjs`, re-enable it now that its cognitive
   numbers are SonarSource-correct.

## 4. Interim (until the new version ships)

Until `0.3.0` is published and adopted, the consumer should source
cognitive complexity from its ts-morph `complexity.mjs` implementation,
which is already SonarSource-correct. Do not baseline against the old
APSS cognitive numbers; they are the inflated values this fix corrects.
