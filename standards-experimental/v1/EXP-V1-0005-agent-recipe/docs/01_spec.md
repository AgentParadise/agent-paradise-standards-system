# EXP-V1-0005 - Agent Recipe Directory Standard (Experimental Specification)

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
- **Harness-neutrality rules** - how the `harness` field is extended to support additional harnesses over time without breaking existing recipes.
- **Skill reference resolution** and **system-instruction merge semantics**.
- **The canonical loader contract** (`load_recipe_dir`) and its error codes.

This standard does NOT cover:

- **Workspace or executor behavior.** A recipe is a pure data artifact. The component that consumes a recipe and actually runs an agent (a "workspace" or equivalent executor) is out of scope for this standard. See section 1.5 for the consumer contract this standard exists to support.
- **Task input, input artifacts, credentials, observability, or execution limits.** These are sibling concerns that combine with a recipe to form a larger execution request (a `RunSpec`, informative only, see 1.5).
- **Skill content or system instruction authoring guidance.** This standard defines how skill references and system instructions are *represented*, not how skills or instructions should be authored.
- **Per-harness configuration details** (for example, provider-specific API parameters). Those live in harness adapters that consume the recipe, not in the recipe itself.
- **Tool execution.** `tools` entries are references only (names/identifiers); this standard defines an allowlist contract for them (section 4.6) but no execution semantics - how a named tool actually runs is a consumer/harness concern.

### 1.4 Relationship to Other Standards

This standard is independent but is designed so that:

- **CI/CD substandards** can validate committed recipe directories as part of a pipeline.
- **Consumer SDKs** (e.g. workspace executors in other repositories, notably `itmux`/Plan B) can depend on this crate's schema and loader as their input contract without depending on any harness-specific code or CLI-only dependencies.

### 1.5 Informative: Consumer Contract

A recipe is designed to be the core of a larger execution request, informally:

```text
RunSpec = recipe (a directory path) + task + input_artifacts + credentials + observability + limits
```

A workspace (an executor living outside this repository) consumes a `RunSpec`, provisions an isolated environment, runs the harness named by the resolved default agent's `harness` field configured per its manifest, and produces a `RunResult`. None of `task`, `input_artifacts`, `credentials`, `observability`, or `limits` are defined by this standard; they are noted here only so implementors understand where the recipe schema fits. This standard defines `recipe` alone.

---

## 2. Core Definitions

### 2.1 Recipe

An **agent recipe** (or **recipe**) is a directory that identifies one or more agents to run, each with a harness, model and reasoning effort, injected skills, and system instructions. A recipe MUST NOT contain task-specific input, credentials, or infrastructure configuration.

### 2.2 Harness

A **harness** is the underlying agent CLI or SDK that executes an agent (for example, Claude Code or OpenAI Codex CLI). The schema is harness-neutral: the `harness` field is a closed enumeration within any single version of this standard, but is version-extensible, so additional harnesses (e.g. `opencode`, `gemini`) MAY be added in future minor versions without breaking existing recipes.

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
harness: claude
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
| `harness` | enum string | NO | Which harness this agent REQUIRES. v1 values: `claude`, `codex`. Absent means harness-agnostic and constrains which tools may be referenced. See 4.3. |
| `model` | object | NO | Intended model selection; overridable per run. See 4.4 and 4.5. |
| `model.name` | string | NO | Provider-qualified model identifier (e.g. `anthropic/claude-opus-4-8`). MUST be non-empty when present. This standard does NOT validate that the named model exists; that is a harness/provider concern. |
| `model.effort` | enum string | NO | Reasoning effort: `low`, `medium`, or `high`. Defaults to `medium`. Maps to harness-specific concepts such as `thinking_level` (Gemini) or `reasoning_effort` (OpenAI). |
| `skills` | array[string] | NO | Harness-agnostic skill references to inject, in listed order. Defaults to an empty array when omitted. See section 5 for resolution. |
| `system_instructions` | object | NO | Per-agent system instruction override. See section 6. |
| `system_instructions.mode` | enum string | REQUIRED if `system_instructions` present | `append` or `replace`. |
| `system_instructions.content` | string | REQUIRED if `system_instructions` present | The instruction text. MUST be non-empty. |
| `tools` | array[string] | NO | Tool reference strings the agent is permitted to use. Absent means no restriction; present but empty means no tools are permitted. See 4.6 for the enforcement rule. |
| `subagents` | array[string] | NO | Names of other agents (files under `agents/`, without the `.yaml` extension) this agent may delegate to. Defaults to an empty array. |
| `from` | string | NO | Name of a sibling agent (another file under `agents/`) this agent inherits from. See section 4.7. |

No other fields are permitted at any nesting level - `AgentManifest`, `ModelSpec`, and `SystemInstructions` all use `#[serde(deny_unknown_fields)]`. A non-string mapping key is likewise rejected, since it can never match a known field name.

### 4.3 The `harness` Field

`harness` names the agent harness an agent requires. It is a closed enumeration in this version of the standard, with values `claude` and `codex`. The standard is explicitly designed for this set to grow (for example `opencode`, `gemini`) in future MINOR versions, without requiring existing recipes to change. An unrecognized `harness` value MUST fail to parse (reported as `RECIPE_MALFORMED_HARNESS_YAML`, section 8) rather than being silently ignored or coerced.

`harness` is OPTIONAL, and its absence is meaningful rather than merely permissive:

- **Absent** asserts that the agent is **harness-agnostic**: it MUST run correctly under any conforming harness.
- **Present** asserts a **dependency**: the agent references capabilities that only the named harness provides.

An agent's harness dependence is therefore not a stylistic choice but a consequence of what it actually references. An agent that omits `harness` MUST NOT reference harness-builtin tool names; it may reference only recipe-provided tools, which the recipe itself carries and which are portable by construction. A validator MUST report an agent that omits `harness` while referencing a harness-builtin tool.

This makes portability checkable rather than aspirational: an agent claiming to be harness-agnostic is mechanically held to that claim.

### 4.4 The `model` Object

`model` and each of its fields are OPTIONAL. An absent `model.name` asserts no opinion about which model to use, and the consumer supplies its own default; it does not assert that the choice is unimportant. A recipe intended for production SHOULD declare the model it is meant to run, because the model is part of what the recipe asserts about the agent's quality (see section 4.5).

`model.name` is an opaque, provider-qualified string. This standard RECOMMENDS the `<provider>/<model...>` shape (e.g. `anthropic/claude-opus-4-8`, `openai/gpt-5-codex`) but does not enforce a specific provider list, since new models and providers are added independently of this standard's release cycle. Where the value contains a `/`, the first segment is the provider and **the remainder is opaque and MAY itself contain further separators**, so gateway-qualified identifiers such as `openrouter/anthropic/claude-opus-4-8` are well-formed.

`model.effort` MUST be one of `low`, `medium`, or `high`, and defaults to `medium` when omitted. These three levels are intentionally coarse so they map cleanly across harnesses that expose different granularities of reasoning effort; a harness adapter is responsible for translating the coarse level into its own native parameter. The name and value space align with the cross-provider `reasoning_effort` convention rather than any single vendor's spelling.

### 4.5 Declared Intent and Run-Time Override

A recipe states an agent's **intended** setup. It does not state an immutable binding. This distinction is what allows one recipe to be both a production definition and an experimental subject.

A consumer (see the run specification in [03-syntropic137-mapping.md](./03-syntropic137-mapping.md)) MAY override a declared `harness` or `model` for a given run. When it does:

1. The consumer MUST record the **effective** values actually used, distinctly from the values the recipe declared.
2. Any evaluation result, judgement, or experiment record produced by that run MUST be attributed to the effective values, never to the declared ones.
3. The recipe itself MUST NOT be rewritten to reflect an override. Overrides belong to the run, not to the artifact.

The two fields differ in how freely they may be overridden, because they differ in kind:

- **`model` overrides are unconstrained.** A model is interchangeable by design; substituting one is the ordinary case, which is precisely what makes a fixed recipe evaluable across many models.
- **`harness` overrides MUST satisfy the agent's references.** A harness is a capability dependency, closer to an ABI than a preference. A consumer MUST NOT substitute a harness that does not provide every harness-builtin tool the agent references.

The intended consequence is that a single recipe carries one definition of good (its `evals/` and `judges/`) while the model under test varies per run. Holding the bar fixed and varying the subject is only sound if the bar lives with the agent and the subject does not.

### 4.6 Tool References and Enforcement

`tools` is an allowlist, not a set of hints. A conforming consumer MUST NOT grant an agent a tool its `tools` list does not permit. An absent `tools` field places no restriction; a present but empty list permits no tools. These are distinct states and a consumer MUST NOT treat them alike.

A tool reference is either **harness-builtin** (provided natively by the harness named in `harness`, per `05-harness-tool-vocabulary.md`) or **recipe-provided** (resolving under `tools/`, section 5). Recipe-provided tools are portable by construction because the recipe carries them.

An agent that omits `harness` is harness-agnostic (section 4.3) and MUST NOT reference a name that is harness-builtin under *any* harness this standard knows about, because an agnostic agent declares no single vocabulary to check its `tools` entries against. A validator MUST report such a reference as `RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL` (section 8).

### 4.7 Agent Inheritance (`from:`)

An agent manifest MAY declare `from: <name>`, naming another agent (a sibling file under `agents/`) it inherits from. Resolution is a field-wise merge, computed by `schema::resolve_inherited(recipe, name) -> Result<AgentManifest, RecipeLoadError>`: the parent is resolved first (its own `from`, if any, resolved transitively), then the child's declared fields are merged on top of it. A chain longer than two agents is legal; the nearest declaration wins for any field the child does not itself declare.

**Inheritance narrows. A child agent MUST NOT grant itself a tool its parent does not permit. Permission narrows monotonically at every tier of this standard: package policy, agent, inheriting agent, and run.**

The merge is not uniform across fields; each field's own semantics decide how it composes:

| Field | Merge rule |
|---|---|
| `harness` | Scalar: the child's value wins when present; the parent's is inherited when the child omits it. |
| `model` (and `model.name` within it) | Scalar, applied at the `model` level and then at `model.name` within it: the child's `model` block, when present, wins field-by-field over the parent's - a child that declares only `model.effort` still inherits `model.name` from the parent. A child that omits `model` entirely inherits the parent's whole `model`. |
| `tools` | Narrowing only. When both the child's and the resolved parent's `tools` are present, the child's list MUST be a subset of the parent's, else `RECIPE_FROM_WIDENS_TOOLS`. When the child omits `tools`, it inherits the parent's value unchanged (whatever that is) rather than defaulting to unrestricted - an omission MUST NOT be read as a widening back to `None`. A parent of `None` (unrestricted) with a child that declares its own `Some([...])` is a narrowing and is always allowed. |
| `subagents` | The child's own value always stands, in full; it is never merged with the parent's. `subagents: []` on the child therefore deliberately clears an inherited list rather than being treated as "not specified". |
| `system_instructions` | When the child omits it, the parent's (already-resolved) value is inherited unchanged. When the child declares its own with `mode: append`, its `content` is appended to the parent's *resolved* content (not the parent's raw, un-inherited YAML) with a blank line between them; `mode: replace` discards the parent's `system_instructions` entirely. This governs only the agent-tier `content` field and is independent of `SYSTEM.md` composition, which `resolved_system` (section 6) handles separately. |
| `skills` | Not merged: the child's own `skills` (possibly empty) is used as declared, with no inheritance from the parent. |
| `name`, `from` | Never inherited: they are the agent's own identity and its own next link in the chain, not state to merge. |

A `from:` chain is validated for two additional failure modes, both surfaced with the file the offending agent was loaded from:

- **`RECIPE_FROM_CYCLE`**: resolving an agent's `from:` chain revisits an agent already seen in that resolution, including an agent that names itself. Detected by tracking visited agent names in a set as the chain is walked, rather than recursing until the call stack overflows.
- **`RECIPE_FROM_UNRESOLVED`**: a `from:` value does not name any parsed entry under `agents/` (no `agents/<name>.yaml` or `.yml`).

`validate_recipe_dir` runs `resolve_inherited` for every agent that declares `from:` and reports any of the three codes above (`RECIPE_FROM_CYCLE`, `RECIPE_FROM_UNRESOLVED`, `RECIPE_FROM_WIDENS_TOOLS`) as a diagnostic, alongside the structural checks in section 8.2. `load_recipe_dir` itself does not perform `from:` resolution: it parses each agent manifest as authored, so a caller can inspect both the as-authored form (`Recipe::agents`) and, on demand, the resolved form (`resolve_inherited`). Resolving unconditionally inside the loader would make the authored form unobservable, which would harm diagnostics more than it would simplify callers.

---

## 5. Skill Reference Resolution

Each entry in an agent manifest's `skills` array is a skill reference resolving to a plugin-dir **path**, in the following order:

1. If `skills/<ref>/` exists inside the recipe directory, that subdirectory is the resolved path (a bundled skill).
2. Otherwise, the ref itself is used as-is (an external skill path or name).

Consumers (e.g. Plan B's `itmux run`, mapping `skills` to `claude_plugin_dirs`) MUST preserve the **listed order** of `skills` when resolving - resolution order is deterministic and load-bearing.

---

## 6. System Instruction Merge Semantics

This standard separates two independent axes that govern an agent's final system prompt: `system_instructions.mode`, which controls composition with the recipe's shared `SYSTEM.md`, and `system_instructions.harness_prompt`, which controls whether the resolved result appends to or replaces the harness's own built-in system prompt. A recipe author sets each axis independently; neither implies a value for the other.

### 6.1 `mode`: composition with `SYSTEM.md`

A recipe MAY declare a shared base system prompt in `SYSTEM.md` at the recipe root. Each agent MAY additionally declare `system_instructions`. The final resolved system prompt for an agent is computed deterministically:

| `system_instructions` | `SYSTEM.md` present? | Resolved system prompt |
|---|---|---|
| `mode: append` | yes | `SYSTEM.md` + `"\n\n"` + `content` |
| `mode: append` | no | `content` |
| `mode: replace` | yes or no | `content` only (`SYSTEM.md` ignored) |
| absent | yes | `SYSTEM.md` verbatim |
| absent | no | no system prompt (`None`) |

This is implemented by `schema::resolved_system(agent, system_md)`. `mode` governs this composition step only; it has no bearing on the harness's own built-in prompt.

### 6.2 `harness_prompt`: relationship to the harness's built-in prompt

`system_instructions.harness_prompt` decides whether the system prompt resolved in 6.1 is appended to the harness's own default/built-in system prompt, or replaces it outright. It takes one of two values:

- `append` (the default): the resolved prompt is added alongside the harness's built-in prompt. This maps to a harness invocation flag such as Claude's `--append-system-prompt`.
- `replace`: the resolved prompt stands in for the harness's built-in prompt entirely. This maps to a harness invocation flag such as Claude's `--system-prompt`.

`harness_prompt` is independent of `mode`. In particular, `mode: replace` (which discards `SYSTEM.md`) and `harness_prompt: replace` (which discards the harness's built-in prompt) address different prompts and MAY be set to different values in any combination; setting one does not set or imply the other. `harness_prompt` is consumed by a harness adapter downstream of this crate's resolution logic, not by `schema::resolved_system` itself.

An agent manifest with no `system_instructions` block has no per-agent override to resolve, so `harness_prompt` does not apply: the agent simply receives the harness's default prompt unchanged, which is what `append` with no content already means.

A consumer whose target harness cannot suppress its built-in system prompt (no equivalent of a "replace" flag) MUST fail loudly when it encounters `harness_prompt: replace`, rather than silently falling back to appending. Silent degradation here would leave an agent's instructions weaker than the recipe declares, with no signal to the operator; a clear failure is the correct behavior.

---

## 7. Canonical Loader

`schema::load_recipe_dir(path: &Path) -> Result<Recipe, RecipeLoadError>` is the single source of truth for parsing a recipe directory into typed Rust values:

1. Read `recipe.yaml` (`RECIPE_MISSING_MARKER` if absent; `RECIPE_MALFORMED_MANIFEST` if present but unparsable).
2. Parse every `agents/*.yaml` file (`RECIPE_MALFORMED_HARNESS_YAML` on the first file that fails to parse), keyed by file stem.
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
| `RECIPE_MALFORMED_HARNESS_YAML` | An `agents/*.yaml` file failed to parse as an `AgentManifest` (missing/extra/invalid fields, unrecognized `harness`/`effort`/`mode`/`harness_prompt` value, or a non-string key). |
| `RECIPE_DUPLICATE_AGENT` | Two agent files resolve to the same stem (e.g. `main.yaml` and `main.yml`), which would collide in the recipe. |
| `RECIPE_DEFAULT_AGENT_UNRESOLVED` | `default_agent` does not name any file actually present under `agents/`. |
| `RECIPE_IO_ERROR` | An I/O error occurred while reading the recipe directory (unreadable file, permission error, etc.). |
| `RECIPE_FROM_CYCLE` | Resolving an agent's `from:` chain (section 4.7) revisits an agent already seen, including an agent naming itself. Emitted by `schema::resolve_inherited`, not `load_recipe_dir`. |
| `RECIPE_FROM_UNRESOLVED` | A `from:` value does not name any parsed entry under `agents/`. Emitted by `schema::resolve_inherited`. |
| `RECIPE_FROM_WIDENS_TOOLS` | A child agent's `tools` is not a subset of its resolved parent's `tools` (section 4.7). Emitted by `schema::resolve_inherited`. |

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
| `RECIPE_EMPTY_INSTRUCTIONS_CONTENT` | An agent's `system_instructions.content` is present but empty/whitespace. |
| `RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL` | An agent that omits `harness` lists a `tools` entry that is harness-builtin under some harness and does not resolve as recipe-provided. See section 4.6. |

Field-shape rules (unknown fields, non-string keys, unrecognized `harness`/`effort`/`mode` enum values) are enforced by `#[serde(deny_unknown_fields)]` and the typed enums during load, so they surface as `RECIPE_MALFORMED_MANIFEST` / `RECIPE_MALFORMED_HARNESS_YAML` on the offending file rather than as separate validator codes.

### 8.3 CLI

`validate_recipe_dir` is wired into the composed development CLI as a registered standard command:

```text
apss-dev run agent-recipe validate <recipe-dir>
```

(aliases: `recipe`, `exp-v1-0005`). Exit code 0 means no errors; 1 means one or more error diagnostics. This is the same `apss-dev run <slug> <command>` surface the official standards (`topology`, `architecture-fitness`, `documentation`) use to expose their validators. The separate `apss-dev v1 validate experiment EXP-V1-0005` command remains a purely structural meta-standard check of the crate's package layout and does not take a recipe-directory argument.

---

## 9. Compliance Checklist

A recipe directory is **compliant** with this standard if:

- [ ] `recipe.yaml` exists at the directory root and parses as a `RecipeManifest` with no unrecognized fields.
- [ ] `default_agent` resolves to a file under `agents/`.
- [ ] Every `agents/*.yaml` file parses as an `AgentManifest` with `name`, a recognized `harness`, and a valid `model` (`model.name` non-empty, `model.effort` one of `low`/`medium`/`high`).
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

- Additional `harness` values (`opencode`, `gemini`, others) as those harnesses gain first-class support.
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
harness: claude
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
from: main
model:
  effort: high
```

`reviewer` inherits `harness: claude` and `model.name: anthropic/claude-opus-4-8` from `main` (section 4.7); it declares its own `model.effort` explicitly, which wins over the parent's because a child's own value always wins when present.

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
harness: codex
model:
  name: openai/gpt-5-codex
  effort: low
```

---

*End of Specification*
