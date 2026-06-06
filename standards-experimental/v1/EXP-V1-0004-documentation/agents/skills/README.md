---
name: "docs-validate-and-index"
description: "Validate and (re)generate APSS documentation structure for any repository. Use when checking ADRs, frontmatter, README indexes, root context files, or installing the pre-commit hook."
---

# docs-validate-and-index

A skill bundle for the **Agent Paradise Standards System (APSS)** documentation standard ([`EXP-V1-0004`](../../docs/00_overview.md)). It exposes the validator and the index generator that together enforce frontmatter-driven indexing, doc type registry rules, and backlinking.

The normative contract for everything below lives in [`docs/01_spec.md`](../../docs/01_spec.md) (validator: Section 9.2, index generator: Section 9.3, hook: Section 9.4, diagnostic codes: Section 10). This file is intentionally short; do not duplicate spec prose here.

## When to use

- The user asks to validate documentation structure, ADRs, frontmatter, or README indexes.
- The user asks to (re)generate index tables in `README.md` files.
- The user asks to install or remove the documentation pre-commit hook.
- An agent is about to commit doc changes and wants to pre-flight the validator.

Skip this skill when the user is editing prose only; the hook handles correctness on commit.

## Commands

```bash
aps run docs validate [path]         # Validate structure. Exit 0 only when no errors.
aps run docs index [path]            # Dry run: print indexes that would be written.
aps run docs index [path] --write    # Rewrite README.md indexes in place.
aps run docs install [path]          # Install hook + default config (idempotent).
aps run docs uninstall [path]        # Remove hook block; preserve config.
aps run docs hook --staged           # Hook entry point used by pre-commit (do not invoke by hand).
```

Every command emits diagnostics in the format defined in [Section 9.2 of the spec](../../docs/01_spec.md#92-validator-contract). Codes are human readable kebab strings (`index-stale`, `frontmatter-unclosed`, `ADR01-dir-not-found`).

## What "valid" means

The per-doc-type definition of valid structure is in [Section 9.4 of the spec](../../docs/01_spec.md#94-git-pre-commit-hook-contract). The configurable surface is in [Section 3](../../docs/01_spec.md#3-configuration); a full example is in [`examples/APSS.yaml`](../../examples/APSS.yaml). Configuration lives in a single root-level `APSS.yaml` owned by the meta-standard (CF01 owns the canonical filename via `apss_core::CONFIG_FILENAME`); the docs standard registers the `docs` slug and contributes the `docs:` block.

## Backlinking

If you add or modify code that implements an ADR (or any other governed doc), add the backlink near the top of the file (`// Implements ADR-001-security-architecture`). This rule applies to every doc type and is not optional. See [Section 7](../../docs/01_spec.md#7-backlinking-always-part-of-the-standard).
