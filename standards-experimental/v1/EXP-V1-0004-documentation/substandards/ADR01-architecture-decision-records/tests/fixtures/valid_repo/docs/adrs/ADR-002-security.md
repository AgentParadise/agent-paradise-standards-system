---
name: "Security Architecture"
description: "Authentication, authorization, and data protection patterns"
status: accepted
---

# ADR-002: Security Architecture

**Status:** Accepted
**Date:** 2026-02-01

## Context

The system handles sensitive user data and needs robust security.

## Decision

Use OAuth 2.0 + RBAC with encrypted-at-rest storage.

## Consequences

- All endpoints require authentication
- Role-based access control governs resource visibility
