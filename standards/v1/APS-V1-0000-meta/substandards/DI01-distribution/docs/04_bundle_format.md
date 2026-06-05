# APSS Bundle Format

## Purpose

An APSS bundle is the distribution unit for a standard or substandard. It is
what a registry stores, what `apss install` resolves, and what a consumer
repository can install without cloning the full APSS standards repository.

Tooling package managers install APSS tools. APSS bundles install standards.

## Bundle Directory

The source form of a bundle is a directory named:

```text
<id>-<slug>-<version>.apss/
```

Example:

```text
APS-V1-0001-code-topology-1.0.0.apss/
```

A registry MAY store this directory as an archive, but the archive transport
format is separate from the bundle content format.

## Required Files

Every bundle MUST include:

- `bundle.toml`
- The package metadata file named by `bundle.toml.metadata_file`
- All source files needed to build or execute the standard implementation
- All standard documentation that defines install, validation, and usage
  behavior

Bundles MUST NOT include generated build output such as `target/`,
`.apss/build/`, or project-specific lockfiles.

## Managed Files

Files generated from an APSS bundle into a consumer repository MUST include a
clear managed-file notice when they are validator, hook, scaffold, generated
standard, or generated runtime files.

The notice MUST say:

```text
Managed by APSS. Do not edit this generated file directly.
```

Operator-owned configuration files, including `APSS.yaml`, MUST NOT use that
notice because they are expected to be manually edited.

## Manifest

`bundle.toml` is the bundle manifest.

```toml
schema = "apss.bundle/v1"
id = "APS-V1-0001"
name = "Code Topology"
slug = "code-topology"
version = "1.0.0"
kind = "standard"
metadata_file = "standard.toml"

[source]
package_path = "standards/v1/APS-V1-0001-code-topology"
repository = "https://github.com/AgentParadise/agent-paradise-standards-system"

[payload]
metadata = "standard.toml"
docs = "docs"
implementation = "."
```

### Fields

- `schema` MUST be `apss.bundle/v1`.
- `id` MUST match the package metadata ID.
- `slug` MUST match the package metadata slug.
- `version` MUST match the package metadata version.
- `kind` MUST be `standard`, `substandard`, or `experiment`.
- `metadata_file` MUST name the included metadata file.
- `source.package_path` SHOULD record the source repository path used to
  produce the bundle.
- `source.repository` SHOULD record the source repository URL when known.
- `payload.metadata` MUST point to the included metadata file.
- `payload.docs` SHOULD point to included documentation when present.
- `payload.implementation` SHOULD point to the implementation payload root.

## Local Installation

For local testing, `apss install --bundle-dir <path>` MAY consume a bundle
directory directly. This path MUST behave like installing the same standard
from a registry-resolved bundle, except that checksum and registry resolution
may be skipped while the package manager is still incomplete.

## Registry Installation

Registry installation MUST eventually resolve:

- The selected bundle version
- A content checksum
- A registry source descriptor
- The implementation crate source inside the bundle

Until registry resolution exists, unresolved registry lockfiles MUST be
refused unless a local repository or local bundle path is supplied.
