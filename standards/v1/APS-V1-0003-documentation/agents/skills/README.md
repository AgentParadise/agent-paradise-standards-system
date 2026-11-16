---
name: "docs-validate-and-index"
description: "Validate and (re)generate APSS documentation structure for any repository. Use when checking ADRs, frontmatter, README indexes, root context files, or installing the pre-commit hook."
---

# docs-validate-and-index

A skill bundle for the **Agent Paradise Standards System (APSS)** documentation standard ([`APS-V1-0003`](../../docs/00_overview.md)). It exposes the validator and the index generator that together enforce frontmatter-driven indexing, doc type registry rules, and backlinking.

The normative contract for everything below lives in [`docs/01_spec.md`](../../docs/01_spec.md) (validator: Section 9.2, index generator: Section 9.3, hook: Section 9.4, diagnostic codes: Section 10). This file is intentionally short; do not duplicate spec prose here.

## When to use

- The user asks to validate documentation structure, ADRs, frontmatter, or README indexes.
- The user asks to (re)generate index tables in `README.md` files.
- The user asks to install or remove the documentation pre-commit hook.
- An agent is about to commit doc changes and wants to pre-flight the validator.

Skip this skill when the user is editing prose only; the hook handles correctness on commit.

## Commands

Implemented and runnable today. Use the canonical slug `documentation` (it must match the `apss.yaml` key); `docs` and `doc` are accepted only by the development CLI `apss-dev`, not by the composed project binary:

```bash
apss run documentation validate [path]         # Validate structure. Exit 0 only when no errors.
apss run documentation index [path]            # Dry run: print indexes that would be written.
apss run documentation index [path] --write    # Rewrite README.md indexes in place.
```

The `install`, `uninstall`, and `hook` subcommands are planned and NOT yet implemented by the handler (it currently provides `validate` and `index` only). Their contract is specified in [`docs/02_install_contract.md`](../../docs/02_install_contract.md).

Every command emits diagnostics in the format defined in [Section 9.2 of the spec](../../docs/01_spec.md#92-validator-contract). Codes are human readable kebab strings (`index-stale`, `frontmatter-unclosed`, `ADR01-dir-not-found`).

## What "valid" means

The per-doc-type definition of valid structure is in [Section 9.4 of the spec](../../docs/01_spec.md#94-git-pre-commit-hook-contract). The configurable surface is in [Section 3](../../docs/01_spec.md#3-configuration); a full example is in [`examples/apss.yaml`](../../examples/apss.yaml). Configuration lives in a single root-level `apss.yaml` owned by the meta-standard (CF01 owns the canonical filename via `apss_core::CONFIG_FILENAME`); the docs standard registers the `docs` slug and contributes the `docs:` block.

## Requirements and the requirements gate (RQ01)

If the system you are about to build or change has a requirements document (default `docs/requirements.md`, or a path listed in `docs.requirements.locations`), read it and the North Star it `serves` first. Then:

- Use the requirements you touch as acceptance criteria: their measure and named check.
- **Requirements gate:** if the feature is not covered by an `accepted` requirement (items under `## Later and Out of Scope` do not count), do not build it. Raise it with the document's `owner`, propose the amendment (id, requirement, measure, check), and build after the owner amends the document.
- When you land a check for a requirement, update its check status, name the check, and add a PATCH change log entry.

Bug fixes that restore an existing requirement and behaviour-free changes pass the gate. The contract is [Section 9 of the RQ01 spec](../../substandards/RQ01-requirements/docs/01_spec.md#9-agent-obligations-read-first-then-the-requirements-gate).

## Backlinking

If you add or modify code that implements an ADR (or any other governed doc), add the backlink near the top of the file (`// Implements ADR-001-security-architecture`). This rule applies to every doc type and is not optional. See [Section 7](../../docs/01_spec.md#7-backlinking-always-part-of-the-standard).
