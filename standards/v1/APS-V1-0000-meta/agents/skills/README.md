# APS-V1-0000 Agent Skills

These skills teach AI agents how to work with APS standards safely and effectively.

## Core Skills

### Standard Authoring

When creating or modifying APS standards:

1. **Follow directory contracts** — Every standard needs `docs/`, `examples/`, `tests/`, `agents/skills/`, `src/`
2. **Use the CLI** — Always use `aps v1 create standard <slug>` to scaffold new standards
3. **Validate before commit** — Run `aps v1 validate standard <id>` before committing changes

### Version Management

1. **Never break backward compatibility** in minor/patch releases
2. **Major bumps require evolution packs** at `evolution/major/<version>/`
3. **Substandard majors MUST align** with parent standard majors

### Breaking Changes

Before introducing a breaking change:

1. Check if it's truly necessary
2. Bump the major version
3. Create the evolution pack with:
   - `rationale.toml` — Why the change is needed
   - `compatibility.toml` — What breaks and what doesn't
   - `migration.md` — How consumers can migrate

### Experimental Standards

1. Create experiments with `aps v1 create experiment <slug>`
2. Experiments use `EXP-V1-XXXX` IDs
3. Experiments MUST pass the same validation as official standards
4. Promotion requires peer review and security audit

## Anti-Patterns to Avoid

- ❌ Editing `standard.toml` ID fields (IDs are immutable)
- ❌ Creating standards without using the CLI scaffold
- ❌ Skipping validation before commits
- ❌ Making breaking changes without major version bumps
- ❌ Treating registry views as authoritative (filesystem is truth)

## Quick Reference

```bash
# Create a new standard
aps v1 create standard my-standard

# Validate your work
aps v1 validate standard APS-V1-XXXX

# Create an experiment
aps v1 create experiment my-experiment

# Promote experiment to official
aps v1 promote EXP-V1-XXXX --to APS-V1-YYYY
```

