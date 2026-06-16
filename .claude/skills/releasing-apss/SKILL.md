---
name: releasing-apss
description: >-
  Use when cutting, gating, tagging, or publishing a release of this APSS repo,
  or when explaining how releases work here. Trigger phrases: "cut a release",
  "release APSS", "publish to crates.io", "create a release PR", "main to
  release", "promote main to release", "bump versions for the release", "tag a
  release", "run the release gate", "how do releases work here", "what gets
  published", "ship a new version". Covers the trunk-based main -> release
  branch flow, the Release Gate checks (source-branch, version-bump, changelog,
  QA, APS validation, security), and release-create.yml tagging plus tiered
  crates.io publishing. Do NOT use for routine feature PRs into main (those need
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
- A `release` branch exists on origin (see Operational state: it may not yet).
- For publishing: the `CARGO_REGISTRY_TOKEN` secret and the `release-publish`
  environment must be configured (see Operational state).

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

6. **Verify the publish.** The publish job is idempotent (it skips crates whose
   version is already on crates.io), so a re-run is safe.

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

## Operational state (as of 2026-06-15)

This section is the high-churn layer. The workflow code above is stable; the
items below describe what is wired versus what still needs one-time setup, and
will change once setup happens.

- **No `release` branch and no `v1.*` tags exist yet.** The gate and
  release-create have never run. The first release must bootstrap the `release`
  branch (for example, branch it from the last commit that matches what is
  currently published, then open the first `main -> release` PR). With no prior
  tag, `release-create` treats every crate as new.
- **Publishing is coded but not operational.** The `publish` job runs real
  `cargo publish`, but it depends on the `CARGO_REGISTRY_TOKEN` secret and the
  `release-publish` GitHub Environment (an approval gate). Neither is configured
  yet, so the publish step would fail at the token until they are added.
- **Crates were published manually before this automation.** Confirm current
  crates.io versions before a first automated publish; publishing a version that
  already exists is rejected by crates.io (the idempotency check guards against
  this, but verify rather than assume).

To make publishing live: add `CARGO_REGISTRY_TOKEN` as a repo/org secret, create
the `release-publish` environment (with required reviewers if you want a manual
approval gate), create the `release` branch, then run the workflow once on a
small change to confirm the path end to end.

## Recommended practices (as of 2026-06-15)

- **Prep without publishing first.** For an unproven release path, do the bumps,
  open the `main -> release` PR, and let the gate pass, then stop before merging.
  The merge into `release` is the irreversible step (tags + crates.io). Review
  the gate result before pulling that trigger.
- **Keep rename/sweep changes out of source comments when avoidable.** The
  version-bump check counts any non-`docs/` change (including a comment-only edit
  to a `.rs` file) as bump-requiring, so a broad sweep forces patch bumps across
  many units at the next release. Harmless, but it widens the bump set.
- **Match the changed set exactly.** Bump only units that changed since the last
  release. Over-bumping unchanged standards creates tags and (if published)
  releases for units that did not change.

## Workflow file map

- `.github/workflows/ci.yml`: main trunk CI (no version-bump gate).
- `.github/workflows/release-gate.yml`: the `main -> release` PR gate.
- `.github/workflows/checks/source-branch-check.yml`: head must be `main`.
- `.github/workflows/checks/version-bump-check.yml`: changed units must bump
  (docs-only exempt; any level).
- `.github/workflows/checks/changelog-check.yml`: PR body needs a changelog
  section.
- `.github/workflows/release-create.yml`: on merge to `release`, tags + GitHub
  Release + tiered crates.io publish.
