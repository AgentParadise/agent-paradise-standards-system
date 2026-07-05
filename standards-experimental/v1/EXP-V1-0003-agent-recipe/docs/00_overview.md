# Agent Recipe Standard - Overview

## What is this?

**EXP-V1-0003** defines a declarative, harness-neutral schema for an **agent recipe**: a YAML document describing *what agent to run* (which harness, which model, which skills, which system instructions) without any knowledge of *where* or *how* it executes.

## Why does it matter?

As agent orchestration spreads across multiple harnesses (Claude Code, Codex, and others to come), tooling needs a stable, harness-neutral way to say "run this agent, configured this way" that does not hard-code any one harness's flags or SDK shape. This standard provides:

1. **A single schema** - the same recipe document works whether the consumer targets Claude Code or Codex.
2. **Forward compatibility** - the `agent` enum is designed to grow (`opencode`, `gemini`, ...) without breaking existing recipes.
3. **Separation of concerns** - a recipe never contains task input, credentials, or infrastructure details, so it is safe to commit and diff.

## Quick Example

```yaml
name: pr-reviewer
agent: claude
model:
  name: anthropic/claude-opus-4-8
  effort: high
skills:
  - code-review
  - security-review
system_instructions:
  mode: append
  content: |
    Focus exclusively on correctness and security issues.
```

## Key Fields

| Field | Purpose |
|-------|---------|
| `name` | Identifier for the recipe |
| `agent` | Which harness runs it (`claude` \| `codex` in v1) |
| `model.name` | Provider-qualified model id |
| `model.effort` | Coarse reasoning effort: `low` \| `medium` \| `high` |
| `skills` | Harness-agnostic skill references to inject |
| `system_instructions` | Optional append/replace system prompt text |

## Where a Recipe Fits

A recipe is the core of a larger execution request (informative, not part of this standard):

```
RunSpec = recipe + task + input_artifacts + credentials + observability + limits
```

A workspace (an executor living in another repository) consumes a `RunSpec` and produces a `RunResult`. This standard defines only `recipe`.

## Status

**Experimental** - This standard is in incubation. Feedback welcome!

### What's Working

- Schema defined (`docs/01_spec.md`)
- Rust reference types with serde (de)serialization
- Validation producing structured `Diagnostics` with stable error codes
- Example recipes (valid and invalid) for conformance testing

### Next Steps

1. Gather feedback from consumers in other repositories (e.g. workspace executors)
2. Add JSON Schema artifact generation for editor tooling
3. Define the full `RunSpec` envelope as a follow-on standard once a real consumer exists
4. Iterate toward promotion

## Learn More

- Read the [full specification](./01_spec.md)
- Check out [examples](../examples/)
- See [agent skills](../agents/skills/)

---

*This is an experimental standard. It may change significantly before promotion to official status.*
