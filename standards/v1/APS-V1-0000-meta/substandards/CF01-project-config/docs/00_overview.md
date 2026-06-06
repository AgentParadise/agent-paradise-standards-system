# APS-V1-0000.CF01 Project Configuration

## Overview

CF01 defines the single, unified configuration mechanism that every APSS
project uses to declare which standards it adopts, configure them, and drive
the installer that materialises them on disk. The mechanism follows the
VS Code settings model: ONE shared file owned by the meta-standard, into
which each standard registers a namespaced section, with validation delegated
to each standard's own validator.

## Problem

Before CF01, project state was split across two files with different
serialisation formats:

- Project-level activation lived in `APSS.yaml` at the root.
- Per-standard configuration sometimes lived under `.apss/config.toml`
  (notably for EXP-V1-0004 documentation).

That split made it impossible to read "what is this project actually doing"
from one place, and it forced every standard to invent its own file format
and discovery story. It also blurred the line between configuration (which
the operator authors) and generated artifacts (which the installer writes).

## Solution

CF01 specifies a single manifest, `APSS.yaml`, at the project root and gives
it three roles at once.

1. **Project configuration.** Core sections owned by CF01 capture project
   identity, the workspace shape, and tooling preferences.
2. **Standard activation.** The `standards` mapping is the project's
   dependency declaration: which standards (and which substandards) are
   active, at what version range.
3. **Installation manifest.** The unified installer reads `APSS.yaml`,
   resolves it through DI01, then invokes each active standard's install
   contract to produce on-disk state. Removing an entry and re-running
   uninstalls cleanly.

The npm analogy is the binding model: `APSS.yaml` is to APSS what
`package.json` is to npm. One file, one install command, one source of
truth for what the project considers active.

Each standard contributes a top-level section keyed by its registered slug
(for example `[standards.code-topology]`). The slug registry is owned
by CF01; the section's content is owned by the standard. CF01 itself never
needs to know the shape of those sections, because each standard ships a
contribution schema describing its keys and a validator that owns the
section's semantics.

The `.apss/` dotdir is reserved for GENERATED artifacts (resolved indexes,
build outputs, composed binaries). Configuration MUST NOT live there.

## Related

- **APS-V1-0000.DI01** Distribution and Installation. CF01 owns the manifest;
  DI01 owns resolution, packaging, and the lockfile. The unified installer
  reads the manifest (CF01), asks DI01 to resolve declared standards, and
  then drives each standard's install contract.
- **APS-V1-0000.CL01** CLI Contract. Each standard's CLI dispatch and
  `StandardCli` trait is the in-process API the unified installer uses to
  reach a standard's install contract.
- **APS-V1-0000.SS01** Substandard Structure. Substandards nest under the
  parent slug in `APSS.yaml` rather than receiving their own top-level slug.
- **Meta-standard section 8.3** `StandardConfig` trait specification.
