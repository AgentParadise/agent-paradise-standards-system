---
name: releasing-apss
description: >-
  Use when cutting, gating, tagging, or publishing a release of this APSS repo,
  or when explaining how releases work here. Trigger phrases: "cut a release",
  "release APSS", "publish to crates.io", "create a release PR", "main to
  release", "promote main to release", "bump versions for the release", "tag a
  release", "run the release gate", "how do releases work here", "what gets
  published", "ship a new version", "publish a new crate for the first time",
  "403 Forbidden publishing to crates.io", "this token does not have the
  required permissions", "CARGO_REGISTRY_TOKEN", "publish-new scope", "the
  publish job failed", "half the crates published". Covers the trunk-based
  main -> release branch flow, the Release Gate checks (source-branch,
  version-bump, changelog, QA, APS validation, security), release-create.yml
  tagging plus tiered crates.io publishing, and the one-time manual first
  publish a NEW crate name requires because the CI token is deliberately scoped
  publish-update only. Do NOT use for routine feature PRs into main (those need
  no release and no version bump), mid-feature version bumps, authoring
  standards, or consumer-side `apss install` (see the adopt/visualize runbooks).
---

# Releasing APSS

## Overview

APSS uses trunk-based development with a single `release` branch. Feature work
merges into `main` freely, with no version-bump pressure. A release is a
`main -> release` pull request that the **Release Gate** validates; merging that
PR into `release` triggers `release-create.yml`, which creates git tags, a
GitHub Release, and (once configured) publishes crates to crates.io.

The single most load-bearing idea: **version bumps are enforced once, at the
release gate, against the previous release. They are not required on merges to
main.** That is deliberate. Per-merge bump enforcement causes version churn and
merge conflicts on the version files when several PRs land between releases.
Bumping at release time lets changes accumulate on main and get versioned once.

## The release model

```
feature branch --PR--> main  (ci.yml: fmt, clippy, check, test, aps-validation)
                        |        no version-bump pressure here
                        |
                  main --PR--> release   (release-gate.yml: the hard gate)
                                  |
                          merge into release
                                  |
                        release-create.yml: tags + GitHub Release + crates.io
```

- **`main` is the trunk.** `ci.yml` runs on every push/PR to main and does NOT
  run the version-bump check. Merge features here without bumping.
- **`release` is the only branch releases come from.** A release PR's head must
  be `main` (the `source-branch-check` rejects anything else), so any version
  bumps must already be on `main` before you open the release PR.
- **Tags are per-unit.** The system tag is `v<workspace-version>`; each changed
  standard/substandard gets its own tag (for example `APS-V1-0001-v0.2.1`,
  `APS-V1-0000.CF01-v1.0.1`).

## Inputs and preconditions

- Changes already merged to `main` that you want to release.
- The `release` branch exists on origin, and `CARGO_REGISTRY_TOKEN` plus the
  `release-publish` environment are already configured (see Operational
  state for the gotchas learned bootstrapping this).

## Workflow: cutting a release

1. **Find what changed since the last release.** The gate diffs the release PR
   against the release base (the `release` branch tip, which equals the last
   release). Compute the same set locally:
   `git diff --name-only origin/release..main`. A package needs a bump if it has
   any changed file outside its `docs/` directory.

2. **Bump versions on main.** For each changed standard/substandard, bump the
   `version` in its `standard.toml` / `substandard.toml` AND the matching
   `Cargo.toml`. If any system crate (`apss-core`, `aps-cli`, `apss-bootstrap`)
   changed, bump the `[workspace.package] version` in the root `Cargo.toml`. Any
   semver level satisfies the gate; there is no level enforcement, so patch is
   fine for backwards-compatible changes. Land these via a normal PR into main.

3. **Open the release PR: `main` -> `release`.** The PR body MUST contain a
   `## Changelog` (or `## Release Notes` / `## What Changed` / `## Changes`)
   section; `changelog-check` fails without it, and `release-create` extracts
   that section into the GitHub Release notes.

4. **Let the Release Gate run and pass.** It runs source-branch-check,
   version-bump-check, changelog-check, full QA (fmt, clippy `-D warnings`,
   check, test, release build), APS validation (`v1 validate repo` +
   `distribution` + self-validation/backwards-compat/CF01/DI01 tests),
   cargo-audit, and dependency-review. The `release-gate-success` job aggregates
   them and is the required status check.

5. **Merge into `release`.** This fires `release-create.yml`, which tags every
   changed unit, pushes tags, creates the GitHub Release from the changelog, and
   runs the publish job.

6. **Verify the publish against crates.io, not against the job's exit status.**
   The publish job tolerates an already-published version, so a re-run after a
   partial publish resumes rather than dying. That was NOT true before
   2026-08-07: the job pre-checked with `cargo search`, whose eventually
   consistent search endpoint kept reporting a just-published crate as missing,
   making every re-run fail on the first crate that had already gone out. If you
   are working on a release whose workflow predates that fix, expect re-runs to
   fail and publish the remainder by hand (see the new-crate section below).

   Check the actual registry rather than trusting a green job:

   ```bash
   for c in apss-core apss <changed-standard-crates>; do
     curl -s "https://crates.io/api/v1/crates/$c" | jq -r '"\(.crate.name) \(.crate.max_version)"'
   done
   ```

## What publishes and what does not

`release-create.yml` publishes in dependency-tier order, only for changed units:

- **Tier 1:** `apss-core` (only when a system crate changed).
- **Tier 2/3:** changed **official** standard crates under
  `standards/v1/APS-V1-XXXX-<slug>` where `XXXX != 0000`, in dependency order.
- **Last:** `apss` (the bootstrap binary).

**Never published:** the meta-standard (`APS-V1-0000`) and its internal
substandards, and everything under `standards-experimental/`. These still get
git tags and release notes, but no `cargo publish`. Treat that exclusion as
load-bearing: the publish job filters them out by directory prefix.

## Releasing a crate that has NEVER been published

**The first release of any new crate name needs a one-time manual `cargo
publish`. CI cannot do it, by design.**

`CARGO_REGISTRY_TOKEN` is deliberately scoped `publish-update` on crates `apss`
and `apss-*`. It can push new *versions* of crates the project already owns, and
nothing else. It cannot create a new crate name, because a CI token that could
would let anyone who leaked it squat or publish arbitrary `apss-*` packages. The
narrower scope is the security posture, not an oversight: do not "fix" this by
adding `publish-new` to the CI token.

The symptom, if you forget: the publish job runs, tier 1 succeeds, and the new
crate fails with

```
error: failed to publish <crate> to registry at https://crates.io
Caused by:
  the remote server responded with an error (status 403 Forbidden):
  this token does not have the required permissions to perform this action
```

That message names the token, not the crate, so it is easy to misread as a
workflow bug. It is not. The workflow handles new packages correctly: detection
is a `git diff` against the last release tag (a directory that did not exist
reads as changed), the crate name is derived from the directory name rather than
from history, and the idempotency check fails open when `cargo search` finds
nothing.

### The flow

1. Let the release merge as normal. Tags and the GitHub Release are created, and
   the publish job fails at the new crate. **This is expected and recoverable.**
   Anything published before the failure (typically `apss-core`) stays published.
2. Publish the new crate by hand, **from the release tag**, using a personal
   credential that has `publish-new`. Publishing from a working tree instead of
   the tag would put bytes on crates.io that do not match what the tag claims:

   ```bash
   git checkout <TAG>              # e.g. APS-V1-0004-v1.0.0
   cargo publish -p <crate-name>
   ```

   Run `cargo publish --dry-run -p <crate-name>` first. It packages the crate and
   compiles it from the packaged tarball, which catches the real risk: files
   pulled in by `include_str!` from outside `src/` are only present if they sit
   inside the crate directory.
3. Finish the remaining tiers. The publish job now tolerates already-published
   versions, so re-running it resumes from where it died and tier 3 (`apss`)
   goes out automatically. No need to re-tag or re-release.

   **If the re-run still fails on a crate that is already published**, the run
   is using a workflow from before the 2026-08-07 fix (a re-run executes the
   workflow file from the commit that triggered it, so fixing it on `main` does
   not help an in-flight run). Do not fight it. Publish the remaining crates by
   hand from their tags, exactly as in step 2:

   ```bash
   git checkout v<system-version>     # e.g. v1.5.0
   cargo publish -p apss
   ```

   The red job is then cosmetic. What matters is the registry.
4. Every subsequent release of that crate is fully automated: `publish-update`
   covers it from then on. `publish-new` is needed once per crate name, not once
   per release.

### Consider Trusted Publishing afterward

crates.io supports GitHub Actions OIDC, which removes the long-lived token from
CI entirely. It has the same chicken-and-egg (trust cannot be configured for a
crate that does not exist yet), so it is a follow-up to step 2 rather than a
replacement for it. Worth doing if the shared token's blast radius matters.

## Operational state (as of 2026-07-10)

The pipeline is now **live and proven end to end**: the `release` branch
exists, `CARGO_REGISTRY_TOKEN` and the `release-publish` environment (required
reviewer: repo owner) are configured, and the first real release
(`v1.2.0`, PR #102) went through the gate clean and published all five
publishable crates to crates.io successfully. Treat the workflow-level
description above as trustworthy; this section now tracks known gotchas
instead of "not wired yet" caveats.

- **What v1.5.0 actually cost (2026-08-07), as a worked example.** The release
  that first published `apss-v1-0004-session-capture` needed three manual
  interventions, and every one of them is now covered above. The publish job
  403'd on the new crate (CI token is `publish-update` only, by design). The
  operator's local token turned out to be dead, a *different* 403 reading
  `authentication failed` rather than `does not have the required permissions`
  (read that message carefully; the two mean different things). And both re-runs
  then died on `apss-core@1.5.0 already exists`, because the old `cargo search`
  pre-check never saw it. Final state was reached by publishing two of the three
  crates by hand from their tags. The git tags and GitHub Release were correct
  throughout; only the publish job was red. **Verify against crates.io, not
  against the job.**
- **Never push directly to `release`.** It is a real branch with a real
  `push` trigger: any push, including a one-off debugging fix, fires
  `release-create.yml` for real (tags + a GitHub Release get created
  immediately, no gate). This happened by accident while bootstrapping the
  pipeline; the `release-publish` approval gate is what prevented an
  accidental crates.io publish, but the stray tags and a garbage GitHub
  Release still had to be deleted by hand. Always land changes to `release`
  through a gated `main -> release` PR, never `git push origin ...:release`.
- **Local reusable workflows cannot live in a subdirectory of
  `.github/workflows/`.** `uses: ./.github/workflows/checks/foo.yml` is
  silently rejected by GitHub Actions ("workflows must be defined at the top
  level of the .github/workflows/ directory"). This isn't caught by
  `actionlint` and doesn't show up as a normal failed run with useful
  logs, it manifests as the workflow's `pull_request` trigger never firing
  at all, forever, with GitHub's push-time workflow validation quietly
  logging a 0-job "failure" on every unrelated push in the repo. If a
  workflow that calls local reusable workflows never seems to trigger, check
  this first. All check workflows now live flat in `.github/workflows/`
  (`source-branch-check.yml`, `version-bump-check.yml`,
  `changelog-check.yml`), not in a `checks/` subdirectory.
- **`cargo test`'s positional filter matches the full test *name*, not the
  file name.** `cargo test -p foo -- self_validation` silently matches zero
  tests (and reports success) if no test function's name contains that exact
  substring. Use `cargo test -p foo --test self_validation_test` (selects by
  binary/file name, runs every test in it) for gate/CI test-selection steps,
  never a bare positional substring filter you haven't verified matches
  something.
- **The changelog in the GitHub Release can come out thin.** `create-release`
  extracts release notes via `git log -1 --pretty=%B` on the merge commit, but
  a standard GitHub PR-merge commit only contains a short "Merge pull request
  #N from ..." summary, not the full PR body, even though the PR body is what
  `changelog-check` validated. The version table still comes through
  correctly; the prose changelog does not. Known gap, not yet fixed in the
  workflow; expect to edit the GitHub Release notes by hand after a merge if
  you want the full PR body preserved there.
- **`release-create.yml`'s crate-name and version derivation are regex/grep
  based and have had real bugs**, not just cosmetic ones: extracting a
  standard's numeric ID from its directory name via `sed 's/-.*//'` cut at
  the first hyphen (`APS-V1-0000-meta` -> `APS`, not `0000`), and a
  workspace-inherited crate's version was read via `grep '^version'` on its
  own `Cargo.toml`, which matched the literal string `version.workspace =
  true` when that crate has no direct version line. Both are fixed, but if a
  future refactor changes directory-naming or `Cargo.toml` shape, re-verify
  this script by extracting it and running it locally against real repo
  history (`git log`, redirecting `$GITHUB_OUTPUT` to a file) before trusting
  a real run, the failure mode is silent corruption, not an error.
- **Crates were published manually before this automation existed.** The
  idempotency check (skip if already on crates.io) protects against
  re-publishing, but if crate layout or naming ever changes, confirm current
  crates.io versions before trusting a first automated publish of a
  previously-manual crate.

## Recommended practices (as of 2026-07-10)

- **Prep without publishing first** for anything unproven: do the bumps, open
  the `main -> release` PR, and let the gate pass, then stop before merging.
  The merge into `release` is the irreversible step (tags + a GitHub Release
  happen immediately and automatically; crates.io publish pauses for manual
  approval on `release-publish`, but the tags/release do not). Review the
  gate result before pulling that trigger.
- **If the release gate's `pull_request` trigger doesn't fire on a PR at
  all** (no check appears, not even a red X, just nothing), check
  `mergeable`/`mergeStateStatus` on the PR first
  (`gh pr view N --json mergeable,mergeStateStatus`). GitHub cannot evaluate
  `pull_request` triggers against a PR it can't compute a merge for; a
  `CONFLICTING` PR silently never gets checks, which looks identical to a
  broken workflow registration from the outside. Resolve the conflict, then
  it fires immediately.
- **Keep rename/sweep changes out of source comments when avoidable.** The
  version-bump check counts any non-`docs/` change (including a comment-only edit
  to a `.rs` file) as bump-requiring, so a broad sweep forces patch bumps across
  many units at the next release. Harmless, but it widens the bump set.
- **Match the changed set exactly.** Bump only units that changed since the last
  release. Over-bumping unchanged standards creates tags and (if published)
  releases for units that did not change.

## Workflow file map

- `.github/workflows/ci.yml`: main trunk CI (no version-bump gate).
- `.github/workflows/main-to-release-gate.yml`: the `main -> release` PR gate
  (`name: Release Gate`; the file is not named `release-gate.yml` because
  that path had a permanently stuck workflow registration and had to be
  renamed to force GitHub to re-register it, see Operational state above).
- `.github/workflows/source-branch-check.yml`: head must be `main`.
- `.github/workflows/version-bump-check.yml`: changed units must bump
  (docs-only exempt; any level).
- `.github/workflows/changelog-check.yml`: PR body needs a changelog
  section.
- `.github/workflows/release-create.yml`: on push to `release`, tags + GitHub
  Release + tiered crates.io publish (the `publish` job pauses for manual
  approval on the `release-publish` environment).
