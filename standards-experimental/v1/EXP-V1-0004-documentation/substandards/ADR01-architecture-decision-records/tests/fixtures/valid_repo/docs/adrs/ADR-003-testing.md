---
name: "Testing Strategy"
description: "Test pyramid, coverage requirements, and CI integration"
---

# ADR-003: Testing Strategy

**Status:** Accepted
**Date:** 2026-02-10

## Context

Need a consistent testing approach across all modules.

## Decision

Follow the test pyramid: unit > integration > e2e. Minimum 80% line coverage.

## Consequences

- CI blocks merges below coverage threshold
- Integration tests hit real databases (no mocks for data layer)
