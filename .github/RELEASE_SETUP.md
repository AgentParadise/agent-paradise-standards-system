# Release Pipeline Setup

One-time setup guide for the APSS release pipeline.

## 1. Create the `release` branch

```bash
git checkout main
git checkout -b release
git push origin release
```

## 2. Branch Protection Rules

### `release` branch

Go to **Settings > Branches > Add branch protection rule** for `release`:

| Setting | Value |
|---------|-------|
| Branch name pattern | `release` |
| Require a pull request before merging | Yes |
| Required approvals | 1+ (recommended) |
| Dismiss stale pull request approvals | Yes |
| Require status checks to pass | Yes |
| Required checks | `Release Gate` (the aggregator job) |
| Require branches to be up to date | Yes |
| Restrict who can push | Enabled (no direct pushes) |
| Do not allow bypassing the above settings | **Yes** (admins cannot bypass) |
| Allow force pushes | **No** |
| Allow deletions | **No** |

**Merge method**: Only allow **merge commits** for PRs into `release`. This preserves the full commit history from `main` and makes the merge commit a clear release boundary.

To enforce merge commits only on the `release` branch, use the repository-level merge settings:
- Go to **Settings > General > Pull Requests**
- Keep "Allow merge commits" enabled
- You may also keep squash/rebase enabled for other branches

Then in the branch protection rule for `release`:
- Require linear history: **No** (merge commits are not linear)

> **Note**: GitHub does not natively restrict merge strategies per-branch. To enforce merge-commit-only for `release`, add a CI check or use the `gh` CLI wrapper below.

### `main` branch (recommended)

| Setting | Value |
|---------|-------|
| Require a pull request before merging | Yes |
| Require status checks to pass | Yes |
| Required checks | `CI` (from ci.yml) |
| Allow force pushes | No |

## 3. GitHub Environment

Create a `release-publish` environment for the crates.io approval gate:

1. Go to **Settings > Environments > New environment**
2. Name: `release-publish`
3. Configure:
   - **Required reviewers**: Add 1+ maintainers who must approve before publishing
   - **Wait timer**: Optional (e.g., 5 minutes to allow last-minute cancellation)
   - **Deployment branches**: Limit to `release` only

## 4. Repository Secrets

Add these secrets at **Settings > Secrets and variables > Actions**:

| Secret | Purpose | How to get it |
|--------|---------|---------------|
| `CARGO_REGISTRY_TOKEN` | Publish crates to crates.io | [crates.io/settings/tokens](https://crates.io/settings/tokens) — create a token with `publish-update` scope |

`GITHUB_TOKEN` is provided automatically and has sufficient permissions for tagging and creating releases.

## 5. Merge Commit Enforcement (optional CLI helper)

To prevent accidental squash/rebase merges into `release`, you can add a status check:

```yaml
# .github/workflows/checks/merge-strategy-check.yml
name: Merge Strategy Check
on:
  pull_request:
    branches: [release]
    types: [opened, synchronize, reopened]

jobs:
  check-base:
    name: Verify source branch
    runs-on: ubuntu-latest
    steps:
      - name: Check PR is from main
        run: |
          if [ "${{ github.event.pull_request.head.ref }}" != "main" ]; then
            echo "::error::Release PRs must come from the main branch, not '${{ github.event.pull_request.head.ref }}'"
            exit 1
          fi
          echo "Source branch is main"
```

Add `Verify source branch` to the required status checks for the `release` branch.

## 6. Verify Setup

After completing the above:

```bash
# 1. Create a test PR from main to release
git checkout main
gh pr create --base release --title "Release v1.0.0" --body "## Changelog
- Initial release"

# 2. Verify these checks run:
#    - Release Gate (aggregator)
#    - Version Bump Check
#    - Changelog Check
#    - Full QA
#    - APS Standards Validation
#    - Cargo Audit
#    - Dependency Review

# 3. Verify you CANNOT:
#    - Push directly to release
#    - Merge without passing checks
#    - Bypass as admin

# 4. After merge, verify:
#    - Tags are created
#    - GitHub Release is created
#    - Publish job waits for approval in release-publish environment
```

## Release Workflow Summary

```
feature branch ──PR──> main (CI checks)
                         │
                         PR (merge commit only)
                         │
                         ▼
                      release ──> release-gate checks
                         │
                         │ on merge:
                         ├── detect version bumps
                         ├── create git tags
                         ├── create GitHub Release
                         └── publish to crates.io (after approval)
```
