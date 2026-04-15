# APS-V1-0000.DI01 — Distribution & Installation

## Overview

DI01 defines how APS standards are packaged, distributed, installed, and composed into project-local CLI binaries.

## Problem

Without a distribution mechanism:
- Users must clone the entire APS repo to use any standard
- There's no way to install just the standards a project needs
- No version resolution, lockfiles, or reproducible installs
- The CLI has hardcoded standard routing instead of dynamic composition

## Solution

DI01 specifies:

1. **Standard crate publishing** — each standard is an independent Rust crate on crates.io
2. **Bootstrap binary (`apss`)** — lightweight global CLI for `init`, `install`, `status`
3. **Composed binary** — project-local binary generated from `apss.toml` with only needed standards
4. **Lockfile (`apss.lock`)** — pins exact versions for reproducible builds
5. **Code generation** — `.apss/build/` Rust crate generated from resolved config

## User Workflow

```bash
cargo install apss              # one-time global bootstrap install
cd my-project
apss init --standard topology   # creates apss.toml
apss install                    # resolves, generates, builds .apss/bin/apss
apss run topology analyze .     # forwards to composed binary
```

## Related

- **APS-V1-0000.CF01** — Project Configuration (defines `apss.toml` that DI01 consumes)
- **APS-V1-0000.CL01** — CLI Contract (defines `StandardCli` trait for dispatch)
