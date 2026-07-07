# EXP-V1-0004 - Agent Recipe Directory Standard (Experimental Specification)

**Version**: 0.2.0
**Status**: Experimental
**Category**: technical

⚠️ **EXPERIMENTAL**: This standard is in incubation and may change significantly before promotion.

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope and Authority

### 1.1 Purpose

This standard defines a **declarative, harness-neutral directory shape for an agent recipe**: a description of *what agent(s) to run*, independent of *where* or *how* it is executed. A recipe answers "which harness(es), which model(s), which skills, which system instructions" without any knowledge of workspace provisioning, task input, credentials, or observability wiring.

### 1.2 Relationship to the Prior (0.1.0) Single-File Shape

Version 0.1.0 of this experiment defined a recipe as a single YAML file. Version 0.2.0 (this document) supersedes that with a **directory** shape: a recipe is a directory containing a root manifest, one YAML file per agent, and optional shared assets. This is a breaking change within the experimental lifecycle (permitted per the meta-standard's rules for experiments); field names and enum values are preserved where they carry over (see section 4) for pi-compatibility.

### 1.3 Scope

This standard covers:

- **The recipe directory shape** - the marker file, the `agents/` directory, and the optional `skills/`/`SYSTEM.md` assets.
- **The root manifest schema** (`recipe.yaml`) and **per-agent manifest schema** (`agents/<name>.yaml`).
- **Harness-neutrality rules** - how the `agent` field is extended to support additional harnesses over time without breaking existing recipes.
- **Skill reference resolution** and **system-instruction merge semantics**.
- **The canonical loader contract** (`load_recipe_dir`) and its error codes.

This standard does NOT cover:

- **Workspace or executor behavior.** A recipe is a pure data artifact. The component that consumes a recipe and actually runs an agent (a "workspace" or equivalent executor) is out of scope for this standard. See section 1.5 for the consumer contract this standard exists to support.
- **Task input, input artifacts, credentials, observability, or execution limits.** These are sibling concerns that combine with a recipe to form a larger execution request (a `RunSpec`, informative only, see 1.5).
- **Skill content or system instruction authoring guidance.** This standard defines how skill references and system instructions are *represented*, not how skills or instructions should be authored.
- **Per-harness configuration details** (for example, provider-specific API parameters). Those live in harness adapters that consume the recipe, not in the recipe itself.
- **Tool execution.** `tools` entries are references only (names/identifiers); this standard defines no execution semantics for them.

### 1.4 Relationship to Other Standards

This standard is independent but is designed so that:

- **CI/CD substandards** can validate committed recipe directories as part of a pipeline.
- **Consumer SDKs** (e.g. workspace executors in other repositories, notably `itmux`/Plan B) can depend on this crate's schema and loader as their input contract without depending on any harness-specific code or CLI-only dependencies.

### 1.5 Informative: Consumer Contract

A recipe is designed to be the core of a larger execution request, informally:

```text
RunSpec = recipe (a directory path) + task + input_artifacts + credentials + observability + limits
```

A workspace (an executor living outside this repository) consumes a `RunSpec`, provisions an isolated environment, runs the harness named by the resolved default agent's `agent` field configured per its manifest, and produces a `RunResult`. None of `task`, `input_artifacts`, `credentials`, `observability`, or `limits` are defined by this standard; they are noted here only so implementors understand where the recipe schema fits. This standard defines `recipe` alone.

---

## 2. Core Definitions

### 2.1 Recipe

An **agent recipe** (or **recipe**) is a directory that identifies one or more agents to run, each with a harness, model and reasoning effort, injected skills, and system instructions. A recipe MUST NOT contain task-specific input, credentials, or infrastructure configuration.

### 2.2 Harness

A **harness** is the underlying agent CLI or SDK that executes an agent (for example, Claude Code or OpenAI Codex CLI). The schema is harness-neutral: the `agent` field is a closed enumeration within any single version of this standard, but is version-extensible, so additional harnesses (e.g. `opencode`, `gemini`) MAY be added in future minor versions without breaking existing recipes.

### 2.3 Skill Reference

A **skill reference** is a harness-agnostic string identifier for a reusable capability to inject into an agent's context. This standard treats skill references as strings that resolve to a plugin-dir path per section 5; resolving that path to actual content is the consumer's responsibility.

### 2.4 System Instructions

**System instructions** are natural-language text applied to an agent's system/instructions channel, combined with the recipe's shared `SYSTEM.md` (if present) according to a declared `mode` (section 6).

### 2.5 Agent Manifest

An **agent manifest** is one YAML file under `agents/` describing a single agent: its harness, model, skills, system instructions, tool references, and subagent references. Agent manifests are unified - there is no structural distinction between "the default agent" and "a subagent"; only `RecipeManifest.default_agent` and other agents' `subagents` lists establish role.

---

## 3. Directory Shape

A recipe MUST be represented as a directory:

```text
<recipe>/
  recipe.yaml            # RecipeManifest: name, version, default_agent
  agents/
    <name>.yaml           # AgentManifest (per-agent, unified agents + subagents)
  skills/                 # optional: bundled skill packages
  SYSTEM.md               # optional: shared base instructions
```

- The presence of `recipe.yaml` at the directory root MUST be treated as the marker denoting "this directory is a recipe". Its absence MUST be reported as `RECIPE_MISSING_MARKER` (section 8).
- `agents/` holds one YAML file per agent. Each file's stem (file name without the `.yaml`/`.yml` extension) is that agent's name for the purposes of `default_agent` and `subagents` resolution.
- `skills/` and `SYSTEM.md` are both OPTIONAL.

---

## 4. Recipe Schema

### 4.1 Root Manifest (`recipe.yaml`)

```yaml
name: pr-reviewer
version: 0.1.0
default_agent: main
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | YES | Identifier for the recipe. MUST be non-empty. RECOMMENDED to be kebab-case. |
| `version` | string | YES | SemVer-ish recipe version. |
| `default_agent` | string | YES | Name of the entry-point agent. MUST resolve to `agents/<default_agent>.yaml`. |

No other top-level fields are permitted; `RecipeManifest` uses `#[serde(deny_unknown_fields)]`.

### 4.2 Agent Manifest (`agents/<name>.yaml`)

```yaml
name: main
agent: claude
model:
  name: anthropic/claude-opus-4-8
  effort: high
skills:
  - code-review
system_instructions:
  mode: append
  content: |
    Focus exclusively on correctness and security issues.
tools:
  - shell
subagents:
  - reviewer
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | YES | Agent name. MUST be non-empty. SHOULD match the file stem. |
| `agent` | enum string | YES | Which harness runs this agent. v1 values: `claude`, `codex`. See 4.3 for extensibility. |
| `model` | object | YES | Model selection. See 4.4. |
| `model.name` | string | YES | Provider-qualified model identifier (e.g. `anthropic/claude-opus-4-8`). MUST be non-empty. This standard does NOT validate that the named model exists; that is a harness/provider concern. |
| `model.effort` | enum string | YES | Reasoning/thinking effort level: `low`, `medium`, or `high`. Maps to harness-specific concepts such as `thinking_level` (Claude) or reasoning effort (Codex). |
| `skills` | array[string] | NO | Harness-agnostic skill references to inject, in listed order. Defaults to an empty array when omitted. See section 5 for resolution. |
| `system_instructions` | object | NO | Per-agent system instruction override. See section 6. |
| `system_instructions.mode` | enum string | REQUIRED if `system_instructions` present | `append` or `replace`. |
| `system_instructions.content` | string | REQUIRED if `system_instructions` present | The instruction text. MUST be non-empty. |
| `tools` | array[string] | NO | Tool reference strings (names/identifiers only). Defaults to an empty array. No execution semantics are defined by this standard. |
| `subagents` | array[string] | NO | Names of other agents (files under `agents/`, without the `.yaml` extension) this agent may delegate to. Defaults to an empty array. |

No other fields are permitted at any nesting level - `AgentManifest`, `ModelSpec`, and `SystemInstructions` all use `#[serde(deny_unknown_fields)]`. A non-string mapping key is likewise rejected, since it can never match a known field name.

### 4.3 The `agent` Field and Harness Extensibility

`agent` is a closed enumeration in this version of the standard, with values `claude` and `codex`. The standard is explicitly designed for this set to grow (for example `opencode`, `gemini`) in future MINOR versions, without requiring existing recipes to change. An unrecognized `agent` value MUST fail to parse (reported as `RECIPE_MALFORMED_AGENT_YAML`, section 8) rather than being silently ignored or coerced.

### 4.4 The `model` Object

`model.name` is an opaque, provider-qualified string. This standard RECOMMENDS the `<provider>/<model>` shape (e.g. `anthropic/claude-opus-4-8`, `openai/gpt-5-codex`) but does not enforce a specific provider list, since new models and providers are added independently of this standard's release cycle.

`model.effort` MUST be one of `low`, `medium`, or `high`. These three levels are intentionally coarse so they map cleanly across harnesses that expose different granularities of reasoning effort; a harness adapter is responsible for translating the coarse level into its own native parameter.

---

## 5. Skill Reference Resolution

Each entry in an agent manifest's `skills` array is a skill reference resolving to a plugin-dir **path**, in the following order:

1. If `skills/<ref>/` exists inside the recipe directory, that subdirectory is the resolved path (a bundled skill).
2. Otherwise, the ref itself is used as-is (an external skill path or name).

Consumers (e.g. Plan B's `itmux run`, mapping `skills` to `claude_plugin_dirs`) MUST preserve the **listed order** of `skills` when resolving - resolution order is deterministic and load-bearing.

---

## 6. System Instruction Merge Semantics

A recipe MAY declare a shared base system prompt in `SYSTEM.md` at the recipe root. Each agent MAY additionally declare `system_instructions`. The final resolved system prompt for an agent is computed deterministically:

| `system_instructions` | `SYSTEM.md` present? | Resolved system prompt |
|---|---|---|
| `mode: append` | yes | `SYSTEM.md` + `"\n\n"` + `content` |
| `mode: append` | no | `content` |
| `mode: replace` | yes or no | `content` only (`SYSTEM.md` ignored) |
| absent | yes | `SYSTEM.md` verbatim |
| absent | no | no system prompt (`None`) |

This is implemented by `schema::resolved_system(agent, system_md)`.

---

## 7. Canonical Loader

`schema::load_recipe_dir(path: &Path) -> Result<Recipe, RecipeLoadError>` is the single source of truth for parsing a recipe directory into typed Rust values:

1. Read `recipe.yaml` (`RECIPE_MISSING_MARKER` if absent; `RECIPE_MALFORMED_MANIFEST` if present but unparsable).
2. Parse every `agents/*.yaml` file (`RECIPE_MALFORMED_AGENT_YAML` on the first file that fails to parse), keyed by file stem.
3. Resolve `default_agent` against the parsed agents (`RECIPE_DEFAULT_AGENT_UNRESOLVED` if it does not name a parsed agent).
4. Read the optional `SYSTEM.md`.
5. Gather the optional `skills/` directory's direct entries, sorted.

Any I/O failure along the way is reported as `RECIPE_IO_ERROR`.

`itmux` (Plan B) depends on this crate's `load_recipe_dir` directly (via a git-pinned Cargo dependency, per the directory-recipe rework plan's revision R2) rather than re-implementing the recipe shape.

A conforming implementation MUST be able to deserialize a valid recipe directory into `Recipe`/`RecipeManifest`/`AgentManifest` and re-serialize any of those types back to equivalent YAML without loss of information (field values MUST be preserved; field order and formatting MAY differ).

---

## 8. Error Codes

### 8.1 Loader Codes

These are emitted by `schema::load_recipe_dir` and surfaced verbatim by `validate::validate_recipe_dir`. They correspond one-to-one with `schema::RecipeLoadError` variants (`RecipeLoadError::code()` returns the matching constant from `schema::error_codes`).

| Code | Meaning |
|------|---------|
| `RECIPE_MISSING_MARKER` | `recipe.yaml` is absent from the candidate directory. |
| `RECIPE_MALFORMED_MANIFEST` | `recipe.yaml` exists but failed to parse as a `RecipeManifest` (missing/extra/invalid fields). |
| `RECIPE_MALFORMED_AGENT_YAML` | An `agents/*.yaml` file failed to parse as an `AgentManifest` (missing/extra/invalid fields, unrecognized `agent`/`effort`/`mode` value, or a non-string key). |
| `RECIPE_DEFAULT_AGENT_UNRESOLVED` | `default_agent` does not name any file actually present under `agents/`. |
| `RECIPE_IO_ERROR` | An I/O error occurred while reading the recipe directory (unreadable file, permission error, etc.). |

### 8.2 Validator Codes

`validate::validate_recipe_dir` is built on top of the loader (plan revision R1: loading and validation share one code path). On a failed load it surfaces exactly one loader code from §8.1. On a recipe that loads cleanly it runs the additional structural rules below, reporting *all* violations via `apss_core::Diagnostics` rather than failing on the first one. These codes live in `validate::error_codes`.

| Code | Meaning |
|------|---------|
| `RECIPE_SUBAGENT_UNRESOLVED` | A `subagents` entry names an agent with no matching `agents/<name>.yaml`. |
| `RECIPE_EMPTY_RECIPE_NAME` | `recipe.yaml`'s `name` is present but empty/whitespace. |
| `RECIPE_EMPTY_AGENT_NAME` | An agent manifest's `name` is present but empty/whitespace. |
| `RECIPE_EMPTY_MODEL_NAME` | An agent manifest's `model.name` is present but empty/whitespace. |
| `RECIPE_INVALID_SKILL_REF` | A `skills` entry is an empty string. |
| `RECIPE_INVALID_TOOL_REF` | A `tools` entry is an empty string. |

Field-shape rules (unknown fields, non-string keys, unrecognized `agent`/`effort`/`mode` enum values) are enforced by `#[serde(deny_unknown_fields)]` and the typed enums during load, so they surface as `RECIPE_MALFORMED_MANIFEST` / `RECIPE_MALFORMED_AGENT_YAML` on the offending file rather than as separate validator codes.

### 8.3 CLI

`validate_recipe_dir` is wired into the composed development CLI as a registered standard command:

```text
apss-dev run agent-recipe validate <recipe-dir>
```

(aliases: `recipe`, `exp-v1-0004`). Exit code 0 means no errors; 1 means one or more error diagnostics. This is the same `apss-dev run <slug> <command>` surface the official standards (`topology`, `architecture-fitness`, `documentation`) use to expose their validators. The separate `apss-dev v1 validate experiment EXP-V1-0004` command remains a purely structural meta-standard check of the crate's package layout and does not take a recipe-directory argument.

---

## 9. Compliance Checklist

A recipe directory is **compliant** with this standard if:

- [ ] `recipe.yaml` exists at the directory root and parses as a `RecipeManifest` with no unrecognized fields.
- [ ] `default_agent` resolves to a file under `agents/`.
- [ ] Every `agents/*.yaml` file parses as an `AgentManifest` with `name`, a recognized `agent`, and a valid `model` (`model.name` non-empty, `model.effort` one of `low`/`medium`/`high`).
- [ ] `skills`, `tools`, and `subagents`, if present, are arrays of strings.
- [ ] `system_instructions`, if present, has a valid `mode` and non-empty `content`.
- [ ] No unrecognized fields are present at any nesting level.

---

## 10. Generator

A conformant recipe directory can be scaffolded from the canonical template in `templates/recipe/skeleton/`:

```text
apss-dev run agent-recipe create <name> [--dir <parent>]
```

This writes `<parent>/<name>/` (parent defaults to the current directory) containing `recipe.yaml` (with `{{name}}` substituted), `agents/main.yaml` (a `claude` default agent plus a commented `codex` example), `SYSTEM.md`, and an empty `skills/` (kept with a `.gitkeep`). The generator refuses to overwrite an existing destination.

The library entry point is `generate::scaffold_recipe(name, dest)`. The template files are embedded into the crate via `include_str!`, so the generator is self-contained and works from any working directory, and its output can never drift from the reviewed on-disk skeleton.

**Round-trip guarantee (normative):** generator output MUST always pass `validate_recipe_dir` with zero errors. This is enforced by `tests/round_trip_test.rs`, which scaffolds into a temp directory and validates the result.

---

## 11. Future Extensions

Potential future additions, to be pursued only after this experiment gathers feedback:

- Additional `agent` values (`opencode`, `gemini`, others) as those harnesses gain first-class support.
- A substandard defining the full `RunSpec` envelope (`recipe` + `task` + `input_artifacts` + `credentials` + `observability` + `limits`) referenced informatively in 1.5.
- Recipe inheritance / composition (`extends: <recipe-name>`).
- Per-skill or per-tool configuration parameters, if injected skills/tools need arguments beyond a bare reference.
- JSON Schema artifact generation for editor tooling and IDE autocompletion.

---

## 12. Security Considerations

### 12.1 No Credentials in Recipes

Recipe directories MUST NOT contain credentials, tokens, or other secrets. Recipes are expected to be committed to version control; secret material belongs in the `credentials` component of a `RunSpec` (informative, see 1.5), not in the recipe.

### 12.2 System Instruction Content

`system_instructions.content` (and `SYSTEM.md`) is free-form text that becomes part of an agent's effective system prompt. Consumers SHOULD treat recipe sources with the same trust level as other executable configuration (for example, CI workflow files): a recipe from an untrusted source can materially change agent behavior via `mode: replace` or injected `skills`/`tools`.

---

## 13. References

- [RFC 2119: Key words for use in RFCs](https://datatracker.ietf.org/doc/html/rfc2119)
- [Semantic Versioning](https://semver.org/)

---

## Appendix A: Complete Example

```text
pr-reviewer/
  recipe.yaml
  agents/
    main.yaml
    reviewer.yaml
  skills/
    code-review/
      SKILL.md
  SYSTEM.md
```

`recipe.yaml`:

```yaml
name: pr-reviewer
version: 0.1.0
default_agent: main
```

`agents/main.yaml`:

```yaml
name: main
agent: claude
model:
  name: anthropic/claude-opus-4-8
  effort: high
skills:
  - code-review
system_instructions:
  mode: append
  content: |
    Focus exclusively on correctness and security issues.
tools:
  - shell
subagents:
  - reviewer
```

`agents/reviewer.yaml`:

```yaml
name: reviewer
agent: codex
model:
  name: openai/gpt-5-codex
  effort: medium
```

See `examples/valid/pr-reviewer/` for this example as a real directory.

## Appendix B: Minimal Example

```text
quick-fix/
  recipe.yaml
  agents/
    main.yaml
```

`recipe.yaml`:

```yaml
name: quick-fix
version: 0.1.0
default_agent: main
```

`agents/main.yaml`:

```yaml
name: main
agent: codex
model:
  name: openai/gpt-5-codex
  effort: low
```

---

*End of Specification*
