# EXP-V1-0003 - Agent Recipe Standard (Experimental Specification)

**Version**: 0.1.0
**Status**: Experimental
**Category**: technical

⚠️ **EXPERIMENTAL**: This standard is in incubation and may change significantly before promotion.

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope and Authority

### 1.1 Purpose

This standard defines a **declarative, harness-neutral schema for an agent recipe**: a description of *what agent to run*, independent of *where* or *how* it is executed. A recipe answers "which harness, which model, which skills, which system instructions" without any knowledge of workspace provisioning, task input, credentials, or observability wiring.

The schema is adopted from the `pi.recipes` shape used in earlier internal tooling and generalized so it is not tied to any single harness.

### 1.2 Scope

This standard covers:

- **The recipe document schema** - required and optional fields, their types, and their allowed values.
- **Harness-neutrality rules** - how the `agent` field is extended to support additional harnesses over time without breaking existing recipes.
- **Validation rules** - what makes a recipe document valid or invalid.
- **Serialization format** - YAML as the canonical on-disk format, with a JSON-compatible in-memory model.

This standard does NOT cover:

- **Workspace or executor behavior.** A recipe is a pure data document. The component that consumes a recipe and actually runs an agent (a "workspace" or equivalent executor) is out of scope for this standard. See section 1.4 for the consumer contract this standard exists to support.
- **Task input, input artifacts, credentials, observability, or execution limits.** These are sibling concerns that combine with a recipe to form a larger execution request (a `RunSpec`, informative only, see 1.4).
- **Skill content or system instruction authoring guidance.** This standard defines how skill references and system instructions are *represented* in a recipe, not how skills or instructions should be authored.
- **Per-harness configuration details** (for example, provider-specific API parameters). Those live in harness adapters that consume the recipe, not in the recipe itself.

### 1.3 Relationship to Other Standards

This standard is independent but is designed so that:

- **CI/CD substandards** can validate committed recipe files as part of a pipeline.
- **Consumer SDKs** (e.g. workspace executors in other repositories) can depend on this schema as their input contract without depending on any harness-specific code.

### 1.4 Informative: Consumer Contract

A recipe is designed to be the core of a larger execution request, informally:

```text
RunSpec = recipe + task + input_artifacts + credentials + observability + limits
```

A workspace (an executor living outside this repository) consumes a `RunSpec`, provisions an isolated environment, runs the harness named by `recipe.agent` configured per the recipe, and produces a `RunResult`. None of `task`, `input_artifacts`, `credentials`, `observability`, or `limits` are defined by this standard; they are noted here only so implementors understand where the recipe schema fits. This standard defines `recipe` alone.

---

## 2. Core Definitions

### 2.1 Recipe

An **agent recipe** (or **recipe**) is a declarative document that identifies which harness to run, which model and reasoning effort to configure it with, which skills to inject, and what system instructions to apply. A recipe MUST NOT contain task-specific input, credentials, or infrastructure configuration.

### 2.2 Harness

A **harness** is the underlying agent CLI or SDK that executes the recipe (for example, Claude Code or OpenAI Codex CLI). The recipe schema is harness-neutral: the `agent` field is a closed enumeration within any single version of this standard, but is version-extensible, so additional harnesses (e.g. `opencode`, `gemini`) MAY be added in future minor versions without breaking existing recipes.

### 2.3 Skill Reference

A **skill reference** is a harness-agnostic string identifier for a reusable capability to inject into the agent's context (for example a Claude Code skill name, or an equivalent construct in another harness). This standard treats skill references as opaque strings; resolving a skill reference to actual content is the consumer's responsibility.

### 2.4 System Instructions

**System instructions** are natural-language text to be applied to the agent's system/instructions channel, combined with the harness's own default system prompt according to a declared `mode`.

---

## 3. Recipe Schema

### 3.1 Canonical Format

The canonical on-disk format is YAML. A recipe document MUST be a single YAML mapping (object) at the document root, with the following shape:

```yaml
name: <recipe-name>              # identifier
agent: claude | codex            # which harness (v1: claude|codex; extensible)
model:
  name: <provider/model>         # e.g. anthropic/claude-opus-4-8
  effort: low | medium | high    # maps to thinking_level / reasoning effort
skills:                          # harness-agnostic skill refs to inject
  - <skill-ref>
system_instructions:
  mode: append | replace
  content: |
    <text>
```

### 3.2 Field Reference

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | YES | Identifier for the recipe. MUST be non-empty. RECOMMENDED to be kebab-case. |
| `agent` | enum string | YES | Which harness runs this recipe. v1 values: `claude`, `codex`. See 3.3 for extensibility. |
| `model` | object | YES | Model selection. See 3.4. |
| `model.name` | string | YES | Provider-qualified model identifier (e.g. `anthropic/claude-opus-4-8`). MUST be non-empty. This standard does NOT validate that the named model exists; that is a harness/provider concern. |
| `model.effort` | enum string | YES | Reasoning/thinking effort level: `low`, `medium`, or `high`. Maps to harness-specific concepts such as `thinking_level` (Claude) or reasoning effort (Codex). |
| `skills` | array[string] | NO | Harness-agnostic skill references to inject into the agent's context. Defaults to an empty array when omitted. Each entry MUST be a non-empty string. |
| `system_instructions` | object | NO | Additional system instructions. Omit entirely if the harness default system prompt is sufficient. |
| `system_instructions.mode` | enum string | REQUIRED if `system_instructions` present | `append` to add `content` after the harness's default system prompt, or `replace` to use `content` in place of it. |
| `system_instructions.content` | string | REQUIRED if `system_instructions` present | The instruction text. MUST be non-empty. |

### 3.3 The `agent` Field and Harness Extensibility

`agent` is a closed enumeration in this version of the standard, with values `claude` and `codex`. The standard is explicitly designed for this set to grow (for example `opencode`, `gemini`) in future MINOR versions of this experimental standard, without requiring existing recipes to change. Consumers MUST reject a recipe whose `agent` value is not recognized, using error code `AGENT_RECIPE_UNKNOWN_AGENT` (see section 5), rather than silently ignoring or coercing it.

### 3.4 The `model` Object

`model.name` is an opaque, provider-qualified string. This standard RECOMMENDS the `<provider>/<model>` shape (e.g. `anthropic/claude-opus-4-8`, `openai/gpt-5-codex`) but does not enforce a specific provider list, since new models and providers are added independently of this standard's release cycle.

`model.effort` MUST be one of `low`, `medium`, or `high`. These three levels are intentionally coarse so they map cleanly across harnesses that expose different granularities of reasoning effort (for example Claude's `thinking_level` and Codex's `reasoning effort`); a harness adapter is responsible for translating the coarse level into its own native parameter.

### 3.5 Skills

`skills` is an ordered list of skill references. Order MAY be meaningful to a consumer (for example, injection order) but this standard does not mandate any particular consumer behavior beyond preserving the order given. An empty or omitted `skills` list means no skills are injected.

### 3.6 System Instructions

When `system_instructions` is present:

- `mode: append` MUST cause the harness's own default system prompt (if any) to be used first, with `content` appended after it.
- `mode: replace` MUST cause `content` to be used in place of the harness's default system prompt.

When `system_instructions` is omitted, the harness's default system prompt MUST be used unmodified.

---

## 4. Serialization and Encoding

### 4.1 File Extension and Encoding

Recipe documents SHOULD be stored with a `.yaml` or `.yml` extension, UTF-8 encoded, and SHOULD end with a trailing newline.

### 4.2 Unknown Fields

A conforming validator MUST reject unrecognized top-level or nested fields with error code `AGENT_RECIPE_UNKNOWN_FIELD` rather than silently ignoring them, so that typos and drift from future schema versions are caught early. (See `examples/invalid/unknown-field.yaml`.)

### 4.3 Round-Tripping

A conforming implementation MUST be able to deserialize a valid recipe document and re-serialize it to an equivalent YAML document without loss of information (field values MUST be preserved; field order and formatting MAY differ).

---

## 5. Validation Rules and Error Codes

A recipe document is **valid** if and only if all of the following hold. Each rule references the machine-readable error code a conforming validator MUST emit on violation.

| Rule | Error Code | Severity |
|------|-----------|----------|
| `name` is present and non-empty | `AGENT_RECIPE_MISSING_NAME` | error |
| `agent` is present | `AGENT_RECIPE_MISSING_AGENT` | error |
| `agent` is one of the recognized values (3.3) | `AGENT_RECIPE_UNKNOWN_AGENT` | error |
| `model` is present | `AGENT_RECIPE_MISSING_MODEL` | error |
| `model.name` is present and non-empty | `AGENT_RECIPE_MISSING_MODEL_NAME` | error |
| `model.effort` is present and is one of `low`/`medium`/`high` | `AGENT_RECIPE_INVALID_MODEL_EFFORT` | error |
| Each entry in `skills` (if present) is a non-empty string | `AGENT_RECIPE_INVALID_SKILL_REF` | error |
| If `system_instructions` is present, `mode` is one of `append`/`replace` | `AGENT_RECIPE_INVALID_INSTRUCTIONS_MODE` | error |
| If `system_instructions` is present, `content` is non-empty | `AGENT_RECIPE_EMPTY_INSTRUCTIONS_CONTENT` | error |
| No unrecognized fields are present anywhere in the document | `AGENT_RECIPE_UNKNOWN_FIELD` | error |

A validator MUST collect and report all applicable violations rather than stopping at the first one, matching this repository's `Diagnostics` convention.

---

## 6. Compliance Checklist

A recipe document is **compliant** with this standard if:

- [ ] It parses as a single YAML mapping at the document root.
- [ ] All required fields (`name`, `agent`, `model.name`, `model.effort`) are present.
- [ ] `agent` is a recognized harness value.
- [ ] `model.effort` is one of `low`, `medium`, `high`.
- [ ] `skills`, if present, contains only non-empty string entries.
- [ ] `system_instructions`, if present, has a valid `mode` and non-empty `content`.
- [ ] No unrecognized fields are present.

---

## 7. Future Extensions

Potential future additions (not in v0.1.0), to be pursued only after this experiment gathers feedback:

- Additional `agent` values (`opencode`, `gemini`, others) as those harnesses gain first-class support.
- A substandard defining the full `RunSpec` envelope (`recipe` + `task` + `input_artifacts` + `credentials` + `observability` + `limits`) referenced informatively in 1.4.
- Recipe inheritance / composition (`extends: <recipe-name>`).
- Per-skill configuration parameters, if injected skills need arguments beyond a bare reference.
- JSON Schema artifact generation for editor tooling and IDE autocompletion.

---

## 8. Security Considerations

### 8.1 No Credentials in Recipes

Recipe documents MUST NOT contain credentials, tokens, or other secrets. Recipes are expected to be committed to version control; secret material belongs in the `credentials` component of a `RunSpec` (informative, see 1.4), not in the recipe.

### 8.2 System Instruction Content

`system_instructions.content` is free-form text that becomes part of the agent's effective system prompt. Consumers SHOULD treat recipe sources with the same trust level as other executable configuration (for example, CI workflow files): a recipe from an untrusted source can materially change agent behavior via `mode: replace` or injected `skills`.

---

## 9. References

- [RFC 2119: Key words for use in RFCs](https://datatracker.ietf.org/doc/html/rfc2119)
- [Semantic Versioning](https://semver.org/)

---

## Appendix A: Complete Example

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
    Do not comment on style unless it affects readability of a security-relevant path.
```

## Appendix B: Minimal Example

```yaml
name: quick-fix
agent: codex
model:
  name: openai/gpt-5-codex
  effort: low
```

---

*End of Specification*
