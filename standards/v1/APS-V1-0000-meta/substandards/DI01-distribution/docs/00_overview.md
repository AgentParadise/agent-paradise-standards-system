# APS-V1-0000.DI01 Distribution & Installation

## Overview

DI01 defines how APSS standards are packaged as APSS bundles, distributed,
resolved against a registry, installed into a project, and composed into a
project-local CLI binary.

DI01 owns the right half of the unified manifest model: given the
`APSS.yaml` declared by the project (CF01), turn each `standards.<slug>`
entry into a pinned, checksummed, on-disk install by resolving versions,
invoking each standard's install contract, and producing a composed binary.

## Problem

Without a distribution mechanism:

- Users must clone the entire APSS repo to use any standard.
- There is no way to install just the standards a project needs.
- No version resolution, lockfiles, or reproducible installs.
- The CLI has hardcoded standard routing instead of dynamic composition.
- The line between configuration (operator-authored) and installation
  (mechanically derived from configuration) is blurry.

## Solution

DI01 specifies:

1. **Standard bundle publishing.** Each standard is published as an
   APSS-native bundle. Rust crates are an implementation detail inside a
   bundle, not the user-facing package format.
2. **Bootstrap binary.** A lightweight, globally-installable CLI for `init`,
   `install`, `status`, and `run` dispatch. The canonical binary name is
   resolved separately in repo issue 64; this spec refers to it generically
   where the name can be avoided.
3. **Composed binary.** A project-local binary generated from the manifest,
   containing only the standards the project declared as active.
4. **Lockfile (`apss.lock`).** Pins exact bundle versions and checksums for
   reproducible builds.
5. **Code generation.** A `.apss/build/` Rust crate generated from the
   resolved manifest.
6. **CF01 to DI01 seam.** A small, explicit interface where CF01 hands DI01
   an ordered list of resolved tuples and DI01 returns `ResolvedStandard`
   values that the unified installer then feeds into each per-standard
   install contract.

## User Workflow

```bash
cargo install <bootstrap>           # one-time global install
cd my-project
<bootstrap> init --standard code-topology    # creates APSS.yaml
<bootstrap> install                     # reads APSS.yaml, resolves, installs, builds composed binary
<bootstrap> run code-topology analyze .      # forwards to composed binary
```

A single `install` command materialises everything declared in the
manifest: per-standard git hooks, validators, scaffolds, and the composed
binary. Removing an entry from `APSS.yaml` and re-running `install` cleanly
uninstalls that standard.

## Related

- **APS-V1-0000.CF01** Project Configuration. Owns the `APSS.yaml` manifest
  that DI01 consumes. The unified installer crosses the CF01/DI01 boundary
  via the `ResolvedStandard` seam described in the spec.
- **APS-V1-0000.CL01** CLI Contract. Defines the `StandardCli` trait the
  installer uses to reach each standard's install contract.
- **Per-standard install contracts.** Each standard ships
  `docs/02_install_contract.md` describing its `install`, `uninstall`, and
  `plan` entry points. The unified installer invokes these.
