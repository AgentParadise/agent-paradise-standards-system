# APS-V1-0000.DP01 — Dependency Policy

## Overview

Shippable APS standards default to **zero external dependencies**. Every third-party crate or package pulled into the workspace is a deliberate, reviewed exception recorded in a root-level `approved-deps.toml` file.

## Why zero-deps by default

- **Supply-chain surface**: each direct dep brings a transitive tree of crates maintained by strangers. Fewer entry points = fewer places a compromise can enter.
- **CVE blast radius**: unused crates still ship vulnerabilities. Not pulling them is the cheapest hardening available.
- **Build-cache size and compile time**: stdlib-only standards compile in seconds against fresh caches; dep-heavy standards trade correctness for convenience.
- **Reviewability**: when every dep requires a named justification plus a dated transitive audit, reviewers have a consistent bar rather than arguing case-by-case.

## What this substandard provides

1. Normative rules for the dependency policy — see `docs/01_spec.md`.
2. The canonical TOML schema for `approved-deps.toml` — one entry per approved crate with name, justification, category, allowed-for list, and transitive-audit date.
3. A manifest scanner (Rust/Python/Node) that reports unapproved dependencies as diagnostics.
4. A template fitness rule (`[[rules.dependency_manifest]]`) plugged into dimension LG01 so violations surface in `aps run fitness validate`.

## Relationship to LG01

LG01 (Legal/Governance) is the fitness dimension that owns license, dependency, and compliance concerns. DEP01 supplies the rule *template*; LG01's runtime evaluator (in `architecture-fitness`) calls into DEP01 to get violations and aggregates them into the dimension score.

## Out of scope

- Transitive audit automation. DEP01 enforces only direct deps. The `transitive_audit_date` field is a human-review artifact; a future tier may automate it.
- Lock-file scanning. `Cargo.lock`, `package-lock.json`, `poetry.lock` capture transitive state but are not the gate.
- Go / Ruby / other manifest formats. The roadmap scopes to Cargo, Python (`pyproject.toml`), and Node (`package.json`).
