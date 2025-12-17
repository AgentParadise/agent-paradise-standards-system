# Consumer SDK Agent Skills

This directory contains agent skills for adopting and managing APS standards.

## Available Skills

| Skill | Description |
|-------|-------------|
| [adopt-standard](./adopt-standard.md) | Set up a project to use APS standards |
| [check-compliance](./check-compliance.md) | Verify compliance with adopted standards |

## Quick Reference

### Adopt a Standard

```bash
aps init --name my-project
aps add code-topology@0.1.0
aps sync
```

### Check Compliance

```bash
aps check
```

### List Adopted Standards

```bash
aps list
```

## Purpose

These skills enable AI agents to:
1. Help projects adopt APS standards
2. Maintain compliance with adopted standards
3. Discover available capabilities via the index
