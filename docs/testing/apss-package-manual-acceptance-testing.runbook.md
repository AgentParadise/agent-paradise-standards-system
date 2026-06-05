# APSS Package Manual Acceptance Testing Runbook

## Purpose

This runbook captures the manual end-to-end checks for APSS package installation, bundle installation, validation, and git hook behavior. Run it before publishing `apss-core` and `apss` to crates.io, and after any change to install, config, lockfile, hook, or bundle behavior.

## Preconditions

- Current working tree is on the APSS branch being validated.
- Rust and Cargo are installed.
- The example repo exists at `/Users/neural/Code/AgentParadise/apss-example-repo`.
- Temporary bundle output can be written under `/tmp`.

## 1. Run Repository QA

From the APSS repository:

```bash
just qa
```

Expected result:

- Formatting passes.
- Clippy passes with `-D warnings`.
- Typecheck passes.
- Workspace tests pass.
- Release build passes.
- V1 repo validation passes.
- DI01 distribution validation passes.

## 2. Install Local Bootstrap CLI

From the APSS repository:

```bash
cargo install --path crates/apss-bootstrap --force
apss --help
```

Expected result:

- `~/.cargo/bin/apss` is installed from the local branch.
- `apss --help` shows the bootstrap CLI.

## 3. Build Local Standard Bundle

From the APSS repository:

```bash
rm -rf /tmp/apss-e2e-bundles
mkdir -p /tmp/apss-e2e-bundles
cargo run -p aps-cli --bin apss-dev -- v1 bundle APS-V1-0001 --output /tmp/apss-e2e-bundles
find /tmp/apss-e2e-bundles -maxdepth 2 -type f -o -type d | sort
```

Expected result:

- A bundle directory exists at `/tmp/apss-e2e-bundles/APS-V1-0001-code-topology-0.1.0.apss`.
- The bundle contains `bundle.toml`, `Cargo.toml`, `crates/`, and `standards/`.

## 4. Prepare Example Repo Manifest

In `/Users/neural/Code/AgentParadise/apss-example-repo`, ensure the project manifest is named `APSS.yaml` and contains a Code Topology standard entry:

```yaml
schema: apss.project/v1

project:
  name: apss-example-repo
  apss_version: v1

tool:
  bin_dir: .apss/bin
  offline: true
  hooks:
    pre_commit: true

standards:
  code-topology:
    id: APS-V1-0001
    version: "0.1.0"
```

If the repo still has an old `apss.toml`, rename it to `APSS.yaml` and convert the content to YAML before continuing.

## 5. Install Bundle Into Example Repo

From `/Users/neural/Code/AgentParadise/apss-example-repo`:

```bash
apss install --bundle-dir /tmp/apss-e2e-bundles/APS-V1-0001-code-topology-0.1.0.apss
```

Expected result:

- `apss.lock` is updated.
- `.apss/build/Cargo.toml` and `.apss/build/src/main.rs` are generated.
- `.apss/bin/apss` is installed.
- `.apss/build/target` is removed after a successful build.
- `.git/hooks/pre-commit` is installed or refreshed.

## 6. Verify Validation And Dispatch

From `/Users/neural/Code/AgentParadise/apss-example-repo`:

```bash
apss validate
./.apss/bin/apss list
```

Expected result:

- `apss validate` prints `Validation passed.`
- `./.apss/bin/apss list` lists `code-topology` with `APS-V1-0001`.

## 7. Verify Git Hook Behavior

From `/Users/neural/Code/AgentParadise/apss-example-repo`:

```bash
.git/hooks/pre-commit
```

Expected result:

- The hook prints `APSS pre-commit: validating project`.
- The hook exits successfully.
- The hook prints `Validation passed.`

Also inspect the managed hook block:

```bash
sed -n '1,220p' .git/hooks/pre-commit
```

Expected result:

- The hook includes `# BEGIN APSS MANAGED PRE-COMMIT` and `# END APSS MANAGED PRE-COMMIT`.
- Consumer repos run `apss validate` through the bootstrap CLI when available.
- APSS standards repos run `just qa` when `standards/v1/APS-V1-0000-meta` is present.

## 8. Verify Hook Opt-Out Warning

Use a temporary copy so the example repo is not changed:

```bash
rm -rf /tmp/apss-example-hooks-off
cp -a /Users/neural/Code/AgentParadise/apss-example-repo /tmp/apss-example-hooks-off
perl -0pi -e 's/pre_commit: true/pre_commit: false/' /tmp/apss-example-hooks-off/APSS.yaml
cd /tmp/apss-example-hooks-off
apss install --bundle-dir /tmp/apss-e2e-bundles/APS-V1-0001-code-topology-0.1.0.apss 2>&1 | tee /tmp/apss-hooks-off.log
rg "Warning: APSS pre-commit hook installation is disabled" /tmp/apss-hooks-off.log
```

Expected result:

- Install succeeds.
- A clear warning says commit-time APSS validation is disabled.

## 9. Verify Fresh Init Defaults

Use a temporary repo:

```bash
rm -rf /tmp/apss-fresh-init
mkdir /tmp/apss-fresh-init
cd /tmp/apss-fresh-init
git init -q
apss init --standard code-topology@0.1.0
sed -n '1,160p' APSS.yaml
```

Expected result:

- `APSS.yaml` is created.
- The manifest includes `tool.hooks.pre_commit: true`.
- The generated standard entry includes a placeholder `APS-V1-XXXX` ID and the requested version.

## 10. Check Repo Size

From `/Users/neural/Code/AgentParadise/apss-example-repo`:

```bash
du -sh .apss
find .apss -maxdepth 4 -type d -name target -print
```

Expected result:

- `.apss` is small enough for local repo use, usually around 1 MiB after install.
- No `.apss/build/target` directory remains.

## 11. Publish Readiness Checks

From the APSS repository:

```bash
cargo publish --manifest-path crates/apss-core/Cargo.toml --dry-run --allow-dirty
cargo package --manifest-path crates/apss-bootstrap/Cargo.toml --allow-dirty
```

Expected result:

- `apss-core` dry-run succeeds.
- `apss` package succeeds only after `apss-core` exists in the crates.io index.

## Notes

- crates.io publishes are permanent. Run this runbook before publishing.
- Publish order is `apss-core` first, then `apss` after the crates.io index sees `apss-core`.
- Do not commit `/tmp` artifacts, `.apss/build/target`, or generated local test repos.
