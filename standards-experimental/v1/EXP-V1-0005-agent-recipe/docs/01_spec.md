# EXP-V1-0005 - Agent Recipe Directory Standard (Experimental Specification)

**Version**: 0.3.0
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

Version 0.1.0 of this experiment defined a recipe as a single YAML file. Version 0.2.0 superseded that with a **directory** shape, and version 0.3.0 (this document) keeps that shape while renaming `agent` to `harness`, making `tools` an enforced allowlist, and restricting `from:` inheritance and `subagents` to narrowing only: a recipe is a directory containing a root manifest, one YAML file per agent, and optional shared assets. This is a breaking change within the experimental lifecycle (permitted per the meta-standard's rules for experiments); field names and enum values are preserved where they carry over from the pi.recipes-inspired single-agent shape (see section 4 and [04-rationale-and-prior-art.md](./04-rationale-and-prior-art.md)), even though this standard is not compatible with pi.recipes itself.

### 1.3 Scope

This standard covers:

- **The recipe directory shape** - the marker file, the `agents/` directory, and the optional `skills/`/`tools/`/`evals/`/`judges/`/`prompts/`/`SYSTEM.md` assets.
- **The root manifest schema** (`recipe.yaml`) and **per-agent manifest schema** (`agents/<name>.yaml`).
- **Harness-neutrality rules** - how the `harness` field is extended to support additional harnesses over time without breaking existing recipes.
- **Skill reference resolution** and **system-instruction merge semantics**.
- **MCP server policy** - the package-tier and agent-tier `mcp` allowlists and the narrowing rule between them (section 7).
- **The declaration of eval cases and judges** (`evals/`, `judges/`, `prompts/`) - the recipe's bar for "good", kept separate from any run's evaluation results (section 9).
- **The canonical loader contract** (`load_recipe_dir`) and its error codes.

This standard does NOT cover:

- **Workspace or executor behavior.** A recipe is a pure data artifact. The component that consumes a recipe and actually runs an agent (a "workspace" or equivalent executor) is out of scope for this standard. See section 1.5 for the consumer contract this standard exists to support.
- **Task input, input artifacts, credentials, observability, or execution limits.** These are sibling concerns that combine with a recipe to form a larger execution request (a `RunSpec`, informative only, see 1.5).
- **Skill content or system instruction authoring guidance.** This standard defines how skill references and system instructions are *represented*, not how skills or instructions should be authored.
- **Per-harness configuration details** (for example, provider-specific API parameters). Those live in harness adapters that consume the recipe, not in the recipe itself.
- **Tool execution.** `tools` entries are references only (names/identifiers); this standard defines an allowlist contract for them (section 4.6) but no execution semantics - how a named tool actually runs is a consumer/harness concern.
- **Eval execution and scoring.** `evals/` and `judges/` (section 9) declare eval cases and judges; this standard defines no scoring model, passing threshold, or execution semantics for how a judge is run or a result recorded - those are consumer concerns. Evaluation *results* are out of scope entirely: a result belongs to a run, not to the recipe (section 4.5).

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

This is enforced as a **closed allowlist of recipe-root entries**, not as a denylist of suspicious filenames: a denylist is only ever a sieve, since the next credential filename nobody enumerated still validates clean. A conforming loader MUST reject any root entry that is neither one of the kinds this specification defines (`recipe.yaml`, `agents/`, `skills/`, `tools/`, `evals/`, `judges/`, `prompts/`, `SYSTEM.md`, `README.md`) nor named literally in the manifest's OPTIONAL `extra_paths` list, reporting `RECIPE_UNDECLARED_ROOT_ENTRY` (section 10.1). `extra_paths` is the declared escape hatch for anything a recipe legitimately ships, such as a `LICENSE`; declaring an entry is an assertion by the author that it is not task input, a credential, or infrastructure configuration. The check also reaches inside the directories whose contents this specification fixes: `agents/` and `judges/` hold `<name>.yaml` (or `.yml`) files, `prompts/` holds `<name>.md` files, and each `evals/<case>/` holds only `input.json` and `expected.md`. Anything else in those directories, including a nested subdirectory, is reported the same way, because `agents/.env` is exactly the content this section prohibits and appears at no root path. `tools/` and `skills/` are **exempt**: they carry vendored package payloads of arbitrary shape, and policing them would reject legitimate recipes rather than catch credentials. A recipe that ships a secret inside a tool or skill package is outside what this check can establish.

### 2.2 Harness

A **harness** is the underlying agent CLI or SDK that executes an agent (for example, Claude Code or OpenAI Codex CLI). The schema is harness-neutral: the `harness` field is a closed enumeration within any single version of this standard, but is version-extensible, so additional harnesses (e.g. `opencode`, `gemini`) MAY be added in future minor versions without breaking existing recipes.

### 2.3 Skill Reference

A **skill reference** is a harness-agnostic identifier for a reusable capability to inject into an agent's context, written either as a bare string or a pinned object (`ref` plus optional `source_url`/`version`/`resolved_sha`). This standard treats a skill reference as resolving to a plugin-dir path per section 5.1; resolving that path to actual content is the consumer's responsibility.

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
  tools/                  # optional: recipe-provided tools (section 5.2)
    <ref>/
      tool.yaml
  evals/                  # optional: eval cases (section 9.1)
    <name>/
      input.json
      expected.md
  judges/                 # optional: judge manifests (section 9.2)
    <name>.yaml
  prompts/                # optional: prompt text referenced by judges (section 9.3)
    <name>.md
  SYSTEM.md               # optional: shared base instructions
```

- The presence of `recipe.yaml` at the directory root MUST be treated as the marker denoting "this directory is a recipe". Its absence MUST be reported as `RECIPE_MISSING_MARKER` (section 10).
- `agents/` holds one YAML file per agent. Each file's stem (file name without the `.yaml`/`.yml` extension) is that agent's name for the purposes of `default_agent` and `subagents` resolution.
- `skills/`, `tools/`, `evals/`, `judges/`, `prompts/`, and `SYSTEM.md` are all OPTIONAL.

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
| `mcp` | object | NO | Package-tier MCP server policy: the ceiling every agent's own `mcp` is checked against. Absent means no MCP server is permitted for any agent. See section 7. |
| `extra_paths` | array of string | NO | Recipe-root entries this recipe ships beyond the kinds this specification defines. Each is matched literally against a root entry name, never as a glob or a prefix. Declaring an entry asserts it is not task input, a credential, or infrastructure configuration. See section 2.1. |

No other top-level fields are permitted; `RecipeManifest` uses `#[serde(deny_unknown_fields)]`.

### 4.2 Agent Manifest (`agents/<name>.yaml`)

```yaml
name: main
description: Reviews pull requests for correctness and security issues.
harness: claude
model:
  name: anthropic/claude-opus-4-8
  effort: high
  max_tokens: 16000
  temperature: 0.2
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
allow_delegation: false
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | YES | Agent name. MUST be non-empty. SHOULD match the file stem. |
| `description` | string | NO | Human-readable description of this agent. Informative only; this standard does not interpret its contents. |
| `harness` | enum string | NO | Which harness this agent REQUIRES. v1 values: `claude`, `codex`. Absent means harness-agnostic and constrains which tools may be referenced. See 4.3. |
| `model` | object | NO | Intended model selection; overridable per run. See 4.4 and 4.5. |
| `model.name` | string | NO | Provider-qualified model identifier (e.g. `anthropic/claude-opus-4-8`). MUST be non-empty when present. This standard does NOT validate that the named model exists; that is a harness/provider concern. |
| `model.effort` | enum string | NO | Reasoning effort: `low`, `medium`, or `high`. Defaults to `medium`. Maps to harness-specific concepts such as `thinking_level` (Gemini) or `reasoning_effort` (OpenAI). |
| `model.max_tokens` | integer | NO | Declared ceiling on output tokens. See 4.5 for the override rule this field follows. |
| `model.temperature` | float | NO | Declared sampling temperature. This standard does NOT validate it against a numeric range. See 4.4. |
| `skills` | array[`SkillRef`] | NO | Harness-agnostic skill references to inject, in listed order. Each entry is either a bare string or a pinned object; both resolve identically. Defaults to an empty array when omitted. See section 5.1 for the two forms and resolution. |
| `system_instructions` | object | NO | Per-agent system instruction override. See section 6. |
| `system_instructions.mode` | enum string | REQUIRED if `system_instructions` present | `append` or `replace`. |
| `system_instructions.content` | string | REQUIRED if `system_instructions` present | The instruction text. MUST be non-empty. |
| `system_instructions.harness_prompt` | enum string | NO | `append` or `replace`. Defaults to `append`. Governs whether the resolved system prompt (6.1) is added alongside, or replaces, the harness's own built-in system prompt. Independent of `mode`. See section 6.2. |
| `tools` | array[string] | NO | Tool reference strings the agent is permitted to use. Absent means no restriction; present but empty means no tools are permitted. See 4.6 for the enforcement rule. |
| `mcp` | object | NO | Agent-tier MCP server policy, narrowing the package's `mcp`. Absent means no restriction of its own (the agent's effective policy is exactly the package's). See section 7. |
| `subagents` | array[string] | NO | Names of other agents (files under `agents/`, without the `.yaml` extension) this agent may delegate to. Defaults to an empty array. Not the same concept as `allow_delegation` - see 4.4a. |
| `allow_delegation` | boolean | NO | Whether this agent may delegate to a peer harness outside this recipe. Defaults to `false`. Not the same concept as `subagents` - see 4.4a. |
| `from` | string | NO | Name of a sibling agent (another file under `agents/`) this agent inherits from. See section 4.7. |

No other fields are permitted at any nesting level - `AgentManifest`, `ModelSpec`, and `SystemInstructions` all use `#[serde(deny_unknown_fields)]`. A non-string mapping key is likewise rejected, since it can never match a known field name.

### 4.3 The `harness` Field

`harness` names the agent harness an agent requires. It is a closed enumeration in this version of the standard, with values `claude` and `codex`. The standard is explicitly designed for this set to grow (for example `opencode`, `gemini`) in future MINOR versions, without requiring existing recipes to change. An unrecognized `harness` value MUST fail to parse (reported as `RECIPE_MALFORMED_HARNESS_YAML`, section 10) rather than being silently ignored or coerced.

`harness` is OPTIONAL, and its absence is meaningful rather than merely permissive:

- **Absent** asserts that the agent is **harness-agnostic**: it MUST run correctly under any conforming harness.
- **Present** asserts a **dependency**: the agent references capabilities that only the named harness provides.

An agent's harness dependence is therefore not a stylistic choice but a consequence of what it actually references. An agent that omits `harness` MUST NOT reference harness-builtin tool names; it may reference only recipe-provided tools, which the recipe itself carries. Note that portability of a recipe-provided tool is the author's responsibility and is NOT machine-checked (section 5.2): what a validator mechanically holds an agnostic agent to is that it names no harness-builtin tool, not that the tools it ships are genuinely portable. A validator MUST report an agent that omits `harness` while referencing a harness-builtin tool.

This makes the *builtin-reference* part of portability checkable rather than aspirational: an agent claiming to be harness-agnostic is mechanically held to naming no harness-builtin tool. The portability of the tools it ships is not checkable; see section 5.2.

### 4.4 The `model` Object

`model` and each of its fields are OPTIONAL. An absent `model.name` asserts no opinion about which model to use, and the consumer supplies its own default; it does not assert that the choice is unimportant. A recipe intended for production SHOULD declare the model it is meant to run, because the model is part of what the recipe asserts about the agent's quality (see section 4.5).

`model.name` is an opaque, provider-qualified string. This standard RECOMMENDS the `<provider>/<model...>` shape (e.g. `anthropic/claude-opus-4-8`, `openai/gpt-5-codex`) but does not enforce a specific provider list, since new models and providers are added independently of this standard's release cycle. Where the value contains a `/`, the first segment is the provider and **the remainder is opaque and MAY itself contain further separators**, so gateway-qualified identifiers such as `openrouter/anthropic/claude-opus-4-8` are well-formed.

`model.effort` MUST be one of `low`, `medium`, or `high`, and defaults to `medium` when omitted. These three levels are intentionally coarse so they map cleanly across harnesses that expose different granularities of reasoning effort; a harness adapter is responsible for translating the coarse level into its own native parameter. The name and value space align with the cross-provider `reasoning_effort` convention rather than any single vendor's spelling.

`model.temperature` is an OPTIONAL declared sampling temperature. This standard does NOT validate it against a numeric range. Providers disagree on what range is valid (some accept `0..=2`, others `0..=1`), and this standard already declines to validate that `model.name` names a real model, for the identical reason: range-checking one provider's convention would make a recipe that is valid for one provider invalid for another. A harness adapter that requires a bounded value is responsible for its own validation against its own provider's range.

### 4.4a `allow_delegation` vs. `subagents`

`allow_delegation` and `subagents` answer different questions and MUST NOT be conflated.

- **`subagents`** names sibling agents WITHIN this recipe that an agent may delegate to. Each entry is checked to resolve to a real `agents/<name>.yaml` (`RECIPE_SUBAGENT_UNRESOLVED`, section 10.2), and, as of 0.3.0, to stay within the delegating agent's own permissions (see below).
- **`allow_delegation`** is permission to hand work to the OTHER harness as a peer. It names no sibling at all, resolves nothing, and is not validated against `agents/`.

An agent MAY declare either, both, or neither; the two fields vary independently, and one's presence or value has no bearing on the other's meaning or validity. `allow_delegation` defaults to `false` because a capability that lets an agent reach outside the recipe boundary SHOULD be opt-in.

`allow_delegation` is nonetheless a **permission**, not merely a capability flag, because it is the same boundary-crossing shape `tools` and `mcp` narrowing exists to police: it grants an agent the ability to reach outside the recipe. Section 4.7 states that permission narrows monotonically for the fields it governs (`tools`, `mcp`, `allow_delegation`), and `allow_delegation` is bound by that principle exactly like `tools` and `mcp` are - see the `from:` merge rule in section 4.7.

**`subagents` is a permission boundary, as of 0.3.0.** A `subagents` entry is checked to *name a real agent* (`agents/<name>.yaml` exists, `RECIPE_SUBAGENT_UNRESOLVED`, section 10.2), and the named sibling's **resolved** `tools` and `mcp` MUST be within the delegating agent's own resolved values:

- If the delegator's resolved `tools` is absent, it is unrestricted (section 4.6) and bounds nothing, so no check applies.
- If the delegator's resolved `tools` is present, the subagent's resolved `tools` MUST be present and a subset of it. An **absent** `tools` on the subagent is a widening, not a neutral omission, because absent means unrestricted: a bounded delegator cannot confer unrestricted access. Either violation is `RECIPE_SUBAGENT_WIDENS_TOOLS`.
- The subagent's resolved `mcp` MUST be within the delegator's resolved `mcp`, computed with the same `mcp_policy_widenings` subset rule every other tier uses (section 7). A violation is `RECIPE_SUBAGENT_WIDENS_MCP`.

Both sides are compared **resolved**, so a subagent cannot acquire a wider permission set through its own `from:` chain either.

Prior versions checked resolution only, which left delegation as an escape hatch from an agent's declared ceiling and made section 4.6's "enforced allowlist" and section 4.7's monotonic-narrowing claim false in the one case that mattered most. Closing it is a breaking change: a recipe where a restricted agent delegates to an unrestricted sibling validated cleanly under 0.2.0 and is rejected under 0.3.0. The fix is to give the subagent an explicit `tools` allowlist within its delegator's.

### 4.5 Declared Intent and Run-Time Override

A recipe states an agent's **intended** setup. It does not state an immutable binding. This distinction is what allows one recipe to be both a production definition and an experimental subject.

A consumer (see the run specification in [03-syntropic137-mapping.md](./03-syntropic137-mapping.md)) MAY override a declared `harness` or `model` for a given run. When it does:

1. The consumer MUST record the **effective** values actually used, distinctly from the values the recipe declared.
2. Any evaluation result, judgement, or experiment record produced by that run MUST be attributed to the effective values, never to the declared ones.
3. The recipe itself MUST NOT be rewritten to reflect an override. Overrides belong to the run, not to the artifact.

The two fields differ in how freely they may be overridden, because they differ in kind:

- **`model` overrides are unconstrained.** A model is interchangeable by design; substituting one is the ordinary case, which is precisely what makes a fixed recipe evaluable across many models.
- **`harness` overrides MUST satisfy the agent's references.** A harness is a capability dependency, closer to an ABI than a preference. A consumer MUST NOT substitute a harness that does not provide every harness-builtin tool the agent references.
- **`model.max_tokens` overrides are constrained the same way `tools` and `mcp` are.** `model.max_tokens` is a declared ceiling, not a fixed value: a run MAY narrow it further, but MUST NOT raise it above the recipe's declared value. This is the same monotonic-narrowing principle sections 4.7 and 7 establish for `tools` and `mcp` - permission (here, the ceiling on output length) narrows as you descend from recipe to run, and never widens - applied to a fourth tier this standard did not previously cover. As with `tools` and `mcp`, this is a normative statement about consumers: this crate has no `RunSpec` type and therefore does not, and cannot, validate it as a fixture-backed rule. `model.temperature` carries no such constraint; a run MAY set it to any value regardless of what the recipe declared, exactly like an unconstrained `model` override.

The intended consequence is that a single recipe carries one definition of good (its `evals/` and `judges/`) while the model under test varies per run. Holding the bar fixed and varying the subject is only sound if the bar lives with the agent and the subject does not.

### 4.6 Tool References and Enforcement

`tools` is an allowlist, not a set of hints. A conforming consumer MUST NOT grant an agent a tool its `tools` list does not permit. An absent `tools` field places no restriction; a present but empty list permits no tools. These are distinct states and a consumer MUST NOT treat them alike.

Because those two states differ in permission, `tools` MUST be written as either an absent key or an array; **an explicit `tools: null` is malformed and MUST be rejected**, not silently read as absent. Many YAML and JSON deserializers collapse a missing key and an explicit null into the same "unset" value, which on a permission-bearing field means malformed input fails *open*, granting everything. The same rule applies to agent-tier `mcp` (section 7) for the same reason.

A tool reference is either **harness-builtin** (provided natively by the harness named in `harness`, per `05-harness-tool-vocabulary.md`) or **recipe-provided** (resolving under `tools/`, section 5.2). Recipe-provided tools are carried by the recipe itself; whether they are genuinely portable is the author's responsibility and is not machine-checked (section 5.2). When a name matches both, recipe-provided wins (section 5.2).

An agent that omits `harness` is harness-agnostic (section 4.3) and MUST NOT reference a name that is harness-builtin under *any* harness this standard knows about, because an agnostic agent declares no single vocabulary to check its `tools` entries against. A validator MUST report such a reference as `RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL` (section 10).

### 4.7 Agent Inheritance (`from:`)

An agent manifest MAY declare `from: <name>`, naming another agent (a sibling file under `agents/`) it inherits from. Resolution is a field-wise merge, computed by `schema::resolve_inherited(recipe, name) -> Result<AgentManifest, RecipeLoadError>`: the parent is resolved first (its own `from`, if any, resolved transitively), then the child's declared fields are merged on top of it. A chain longer than two agents is legal; the nearest declaration wins for any field the child does not itself declare.

**Inheritance narrows. A child agent MUST NOT grant itself a tool its parent does not permit. For the fields this principle currently governs - `tools`, `mcp`, and `allow_delegation` - permission narrows monotonically at every tier of this standard: package policy, agent, inheriting agent, and run.** As of 0.3.0 this principle also governs `subagents`: a named subagent's *resolved* `tools` and `mcp` MUST be within the delegating agent's own, else `RECIPE_SUBAGENT_WIDENS_TOOLS` / `RECIPE_SUBAGENT_WIDENS_MCP` (section 4.4a). Delegation is therefore not an escape hatch from an agent's declared ceiling.

The merge is not uniform across fields; each field's own semantics decide how it composes:

| Field | Merge rule |
|---|---|
| `harness`, `description` | Scalar: the child's value wins when present; the parent's is inherited when the child omits it. |
| `model` (and `model.name`, `model.max_tokens`, `model.temperature` within it) | Scalar, applied at the `model` level and then at each of these fields within it: the child's `model` block, when present, wins field-by-field over the parent's - a child that declares only `model.effort` still inherits `model.name`/`model.max_tokens`/`model.temperature` from the parent. A child that omits `model` entirely inherits the parent's whole `model`. Neither `max_tokens` nor `temperature` is narrowing-checked at this tier: they are not permission fields the way `tools`/`mcp`/`allow_delegation` are, and `max_tokens`' ceiling-narrowing rule (section 4.5) is a run-tier constraint this crate has no `RunSpec` to check. |
| `tools` | Narrowing only. When both the child's and the resolved parent's `tools` are present, the child's list MUST be a subset of the parent's, else `RECIPE_FROM_WIDENS_TOOLS`. When the child omits `tools`, it inherits the parent's value unchanged (whatever that is) rather than defaulting to unrestricted - an omission MUST NOT be read as a widening back to `None`. A parent of `None` (unrestricted) with a child that declares its own `Some([...])` is a narrowing and is always allowed. |
| `mcp` | Narrowing only, exactly like `tools`. When both the child's and the resolved parent's `mcp` are present, the child's policy MUST NOT permit a server or method the parent's does not, else `RECIPE_MCP_FROM_WIDENS_POLICY`. When the child omits `mcp`, it inherits the parent's resolved value unchanged. When the parent has no `mcp` of its own (nothing in its chain declared one), a child that declares its own is always a narrowing relative to "unset" and needs no check. This per-link check runs during `resolve_inherited` itself, the same place `tools`' check runs; a separate, additional check against the package's `mcp` (section 7.3, `RECIPE_MCP_AGENT_WIDENS_POLICY`) runs once in `validate_recipe_dir` against this fully resolved value, which `tools` has no equivalent of because `tools` has no package tier. |
| `allow_delegation` | Narrowing only, exactly like `tools`/`mcp`, despite being a plain `bool` rather than a collection. The child's own declared value (or serde's `false` default, if omitted) is always the resolved value - there is no `Option<bool>` "unset" state to inherit through - but the child MUST NOT resolve to `true` when the resolved parent's is `false`, else `RECIPE_FROM_WIDENS_DELEGATION`. A child tightening `true` -> `false`, or matching the parent's value, is always allowed. See section 4.4a. |
| `subagents` | The child's own value always stands, in full; it is never merged with the parent's. `subagents: []` on the child therefore deliberately clears an inherited list rather than being treated as "not specified". |
| `system_instructions` | When the child omits it, the parent's (already-resolved) value is inherited unchanged. When the child declares its own with `mode: append`, its `content` is appended to the parent's *resolved* content (not the parent's raw, un-inherited YAML) with a blank line between them; `mode: replace` discards the parent's `system_instructions` entirely. This governs only the agent-tier `content` field and is independent of `SYSTEM.md` composition, which `resolved_system` (section 6) handles separately. |
| `skills` | Not merged: the child's own `skills` (possibly empty) is used as declared, with no inheritance from the parent. |
| `name`, `from` | Never inherited: they are the agent's own identity and its own next link in the chain, not state to merge. |

A `from:` chain is validated for two additional failure modes, both surfaced with the file the offending agent was loaded from:

- **`RECIPE_FROM_CYCLE`**: resolving an agent's `from:` chain revisits an agent already seen in that resolution, including an agent that names itself. Detected by tracking visited agent names in a set as the chain is walked, rather than recursing until the call stack overflows.
- **`RECIPE_FROM_UNRESOLVED`**: a `from:` value does not name any parsed entry under `agents/` (no `agents/<name>.yaml` or `.yml`).

`validate_recipe_dir` runs `resolve_inherited` for every agent that declares `from:` and reports any of the codes above (`RECIPE_FROM_CYCLE`, `RECIPE_FROM_UNRESOLVED`, `RECIPE_FROM_WIDENS_TOOLS`, `RECIPE_MCP_FROM_WIDENS_POLICY`, `RECIPE_FROM_WIDENS_DELEGATION`) as a diagnostic, alongside the structural checks in section 10.2. `load_recipe_dir` itself does not perform `from:` resolution: it parses each agent manifest as authored, so a caller can inspect both the as-authored form (`Recipe::agents`) and, on demand, the resolved form (`resolve_inherited`). Resolving unconditionally inside the loader would make the authored form unobservable, which would harm diagnostics more than it would simplify callers.

---

## 5. Skill and Tool Reference Resolution

### 5.1 Skill Reference Resolution

Each entry in an agent manifest's `skills` array is a `SkillRef`, and takes **either** of two forms:

```yaml
skills:
  - code-review                              # bare form
  - ref: security                            # pinned form
    source_url: https://example.com/security.git
    version: 1.2.0
    resolved_sha: abc123
```

| Form | Shape | Fields |
|---|---|---|
| **Bare** | a plain string | the ref itself, e.g. `code-review` |
| **Pinned** | an object | `ref` (YES), `source_url` (NO), `version` (NO), `resolved_sha` (NO) |

**Both forms are accepted, deliberately.** Every recipe authored before pinning existed uses the bare form, and this standard MUST continue to accept it unchanged - accepting both is what keeps skill pinning an *additive* change rather than a breaking one. A recipe author who has no reproducibility concern for a given skill (or no pinning tooling yet) may keep writing `skills: [code-review]` exactly as before.

**Both forms resolve identically.** Regardless of which form an entry uses, resolution consumes exactly one value - the ref - in the following order:

1. If `skills/<ref>/` exists inside the recipe directory, that subdirectory is the resolved path (a bundled skill).
2. Otherwise, the ref itself is used as-is (an external skill path or name).

For the bare form, the ref is the string itself. For the pinned form, the ref is the `ref` field. A bare entry and a pinned entry naming the same `ref` therefore resolve to the same path; `source_url`, `version`, and `resolved_sha` play no part in resolution itself. This is not a subtle distinction to be inferred - the two forms are the same reference with optional provenance attached, not two different kinds of thing. Consumers (e.g. Plan B's `itmux run`, mapping `skills` to `claude_plugin_dirs`) MUST preserve the **listed order** of `skills` when resolving - resolution order is deterministic and load-bearing.

**Why pin at all.** A recipe carries its own definition of good in `evals/` and `judges/` (section 9). That definition is only meaningful if the recipe's inputs are reproducible: a skill resolved as `@latest` means two runs of the same recipe are not necessarily the same agent, so a comparison between the runs proves nothing and neither run's result can be attributed back to a fixed recipe. The pinned form lets a recipe author record exactly which version (and, once a resolver exists, which exact content) a skill was built against, so a run and its eval result mean something specific and repeatable.

**What the pinned fields mean.**

| Field | Type | Required | Description |
|---|---|---|---|
| `ref` | string | YES | The skill reference, resolved exactly as the bare-string form is. MUST be non-empty. |
| `source_url` | string | NO | Where the skill was fetched from (e.g. a git URL). Informative only. |
| `version` | string | NO | The pinned version. MUST NOT be `latest` or `@latest` (case-insensitively), and MUST NOT be empty or all-whitespace, when present - see `RECIPE_SKILL_UNPINNED` below. Absent asserts no version opinion, which is not the same as an unpinned reference and is not rejected. |
| `resolved_sha` | string | NO | A resolved content hash, if a resolver has already computed one - the strongest reproducibility guarantee this standard can express. |

`resolved_sha` is deliberately NOT required. It is the strongest guarantee available, but a recipe author may reasonably pin by `version` before any resolver exists to produce a sha; requiring all four pinned fields up front would make the pinned form unusable until such tooling exists. Only the specific unpinned-`latest` case is rejected, not the absence of `source_url` or `resolved_sha`.

**What this standard does not do.** This standard records what a recipe declares. It does NOT resolve a skill reference over the network, fetch its content, or compute a `resolved_sha` itself - that belongs to a consumer (e.g. a resolver tool that fills in `resolved_sha` after fetching), not to this standard's schema or validator.

A validator MUST reject a pinned entry whose `version` is `latest` or `@latest` (case-insensitively) or is empty/whitespace, reported as `RECIPE_SKILL_UNPINNED` (section 10.2). An absent `version` is not rejected: it is a different assertion (no opinion) than a `version` that names a moving target.

### 5.2 Tool Reference Resolution (`tools/`)

Section 4.3 lets an agent omit `harness` and assert harness-agnosticism, and section 4.6 forbids such an agent from referencing a harness-builtin tool name. Taken together those two rules would make an agnostic agent's `tools` list vacuous - it could reference nothing at all - unless a recipe has a way to carry its own tool implementations. `tools/` is that way.

```text
<recipe>/
  tools/
    <ref>/
      tool.yaml           # ToolManifest: name, description, command, args, protocol
```

`tools/` is OPTIONAL. Each subdirectory directly under it is a **tool package**, named by `<ref>` - the same string an agent's `tools` entry uses to reference it - and MUST contain a `tool.yaml` conforming to `ToolManifest`:

| Field | Type | Required | Description |
|-------|------|----------|--------------|
| `name` | string | YES | Tool name. MUST be non-empty. Conventionally matches `<ref>`, though resolution keys off the directory name, not this field. |
| `description` | string | NO | Human-readable description. |
| `command` | string | YES | The executable to invoke. MUST be non-empty. |
| `args` | array[string] | NO | Fixed leading arguments passed to `command` on every invocation. Defaults to an empty array. |
| `protocol` | enum string | NO | `mcp-stdio` or `subprocess`. Defaults to `mcp-stdio`. |

No other fields are permitted; `ToolManifest` uses `#[serde(deny_unknown_fields)]`.

**Every direct child of `tools/` MUST contain a `tool.yaml`.** A subdirectory without one is reported as `RECIPE_MISSING_TOOL_MANIFEST` (section 10.1), never skipped: skipping it would certify a recipe whose tool package is structurally incomplete.

**Resolution.** A `tools` entry `<ref>` is **recipe-provided** when `tools/<ref>/tool.yaml` exists inside the recipe directory and parses as a `ToolManifest`; `Recipe::resolve_tool(ref)` is the single function every consumer of this standard MUST use to answer that question. An entry that does not resolve this way is either harness-builtin (checked against `05-harness-tool-vocabulary.md`, per harness) or simply unresolvable, neither of which this section governs.

**The portability rule.** A recipe-provided tool MUST NOT link a harness API. It MUST be invocable as a subprocess by any conforming consumer. This is precisely what distinguishes `tools/` from an `extensions/`-style directory that imports the harness's own runtime API: an extension of that kind cannot cross harnesses, because its implementation is written against one harness's process. A self-contained script, a compiled binary, or an MCP stdio server can all satisfy this rule, because none of them require the invoking process to *be* a particular harness - they only require a process to invoke them, which every harness has.

**This rule is author-enforced, not machine-checked.** A `tool.yaml` `command` is an opaque string, so a validator cannot decide whether the process it names links a harness API: `command: claude` and `command: ./bin/extract` are indistinguishable to a static check, and any denylist of harness binary names would both miss wrappers and reject legitimate ones. A conforming validator therefore MUST NOT be read as certifying portability of a recipe-provided tool; it certifies only that the manifest is well-formed (section 10.2). Recipe authors are responsible for this rule, and a consumer that needs a stronger guarantee MUST establish it out of band. Note the consequence for section 4.6: because recipe-provided wins on a name collision, an agnostic agent CAN reference a harness-builtin name by shipping a `tools/<name>/tool.yaml`, and `RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL` will not fire. That is the intended precedence, and it is sound only to the degree the author honors this rule.

**Protocol semantics.**

- **`mcp-stdio`** (the default): the tool is an MCP server spoken to over stdio. This supplies schema and invocation semantics for free, and is already cross-harness by design, so it needs no further contract from this standard beyond "launch `command` (with `args`) as an MCP stdio server".
- **`subprocess`**: the escape hatch for a one-file script where a full MCP server is overkill. The contract is argv in, JSON on stdout, and a non-zero exit code means failure. This standard does not further specify the JSON shape; that is a concern for the tool's own `description` and its caller's expectations, not for this standard.

**Precedence when a name is ambiguous.** A `tools` entry MAY simultaneously match a harness-builtin name (section 4.6, `05-harness-tool-vocabulary.md`) and a `tools/<ref>/` directory in the same recipe. This is resolved in favor of the recipe: **recipe-provided wins**, because the recipe ships the implementation and therefore knows what the name means. Concretely, this means the harness-agnostic-agent-uses-builtin-tool check (section 4.6, `RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL`) MUST treat a `tools` entry that resolves under `tools/` as recipe-provided even if the same name is also harness-builtin, and MUST NOT report it as an error on that basis. A validator that checked builtin-ness first and never consulted `tools/` would over-reject; the check MUST consult `tools/` resolution before concluding a name is builtin-only.

**What this standard does not do.** This standard defines a tool's declaration and invocation contract. It does NOT execute tools, and this crate carries no process-spawning dependency as a consequence. A `tools/<ref>/tool.yaml` that fails to parse at all (not readable YAML, or missing/extra/invalid fields) is reported as `RECIPE_MALFORMED_TOOL_MANIFEST` (section 10.1); a `tools/<ref>/tool.yaml` that parses successfully but has an empty `name` or `command` is reported as `RECIPE_INVALID_TOOL_MANIFEST` (section 10.2). This standard does NOT validate that `command` exists on disk or is executable, because a recipe is a portable artifact that MAY be validated on a machine that will never run it.

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

## 7. MCP Server Policy (`mcp`)

`mcp` declares which MCP (Model Context Protocol) servers, and which methods on each, may be used. It is declared at two tiers: once per package (`recipe.yaml`), and optionally narrowed per agent (`agents/<name>.yaml`). Both use the same shape:

```yaml
mcp:
  servers:
    warehouse:
      include:
        - run_query
      exclude: []
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `mcp.servers` | map[string, object] | NO | Per-server method policy, keyed by server id. Defaults to an empty map when `mcp` is omitted entirely. |
| `mcp.servers.<id>.include` | array[string] | NO | Method names permitted for this server. Defaults to an empty array. |
| `mcp.servers.<id>.exclude` | array[string] | NO | Method names withdrawn from `include`. Defaults to an empty array. |

`RecipeManifest.mcp` and `AgentManifest.mcp` both use `#[serde(deny_unknown_fields)]`, as do `McpPolicy` and `McpServerPolicy`.

### 7.1 The Effective Method Set

For a single server's policy, the **effective method set** is `include` minus `exclude`, both compared as literal strings:

```text
effective(server_policy) = server_policy.include \ server_policy.exclude
```

An empty `include` permits **no** methods for that server - it is not equivalent to omitting the server entirely, and it mirrors `tools: Some(vec![])` (section 4.6): present-but-empty is a deliberate assertion of zero permission, not an oversight. `McpServerPolicy::effective_methods` is the single implementation of this computation; nothing else in this crate re-derives it.

This version of the standard does **not** support a wildcard `include` value (for example `"*"`). An `include`/`exclude` entry is always a literal method name. This is a deliberate simplification: a wildcard that must remain narrowable by a child tier without ever becoming widenable by one is materially harder to reason about correctly than an explicit list, and a smaller correct rule beats a larger ambiguous one. A future minor version MAY add wildcard support if a concrete need arises.

A server that a tier's `mcp.servers` map does not mention at all has **no** implicit permission - there is no "everything not listed is allowed" fallback. This is why `RecipeManifest.mcp` and `AgentManifest.mcp` are restrictive by default (absent means no MCP access at all), deliberately unlike `tools`' `None`, which means unrestricted (section 4.6). The two fields answer different questions: `tools`' `None` says "this agent asserts no opinion, so nothing is restricted here"; `mcp`'s absence says "nothing has been granted here." An MCP server is a live, potentially stateful, potentially destructive external dependency in a way a harness-builtin tool reference is not, so this standard does not extend the same permissive default to it.

### 7.2 Two Tiers, One Ceiling

**`mcp` is declared once per package and narrowed per agent. Permission narrows monotonically as you descend: package policy, agent, inheriting agent (`from:`), and run. Each tier may restrict what the tier above it permitted. None may widen it.** This is the same governing principle section 4.7 states for `tools` via `from:`, applied here across an additional tier that sits above the agent altogether. Section 4.7 lists four rungs on this ladder - "package policy, agent, inheriting agent, and run" - and the inheriting agent is its own rung: narrowing relative to the package is not sufficient by itself, because it says nothing about whether an agent narrowed relative to *its own* `from:` parent.

- The **package tier** (`RecipeManifest.mcp`) is the ceiling: the maximum `mcp` policy any agent in the recipe may have, for any server.
- The **agent tier** (`AgentManifest.mcp`) is `Option<McpPolicy>`. `None` (the field omitted) declares no restriction of its own: the agent's effective policy is exactly the package's, which is by construction never a widening of itself. `Some(policy)` narrows to the named servers and their effective method sets, checked against both the package tier and (if the agent declares `from:`) its parent, as described in 7.3.
- The **inheriting agent** (`from:`) tier resolves through `schema::resolve_inherited` exactly as `tools` does (section 4.7): the child's own `mcp`, when declared, replaces the parent's in full, but only after passing the per-link narrowing check in 7.3; when the child omits `mcp`, it inherits the parent's resolved value unchanged, which needs no check.
- The **run** tier (a consumer's runtime restriction of a resolved agent) is out of scope for this standard's schema, exactly as run-time `tools` restriction is (section 4.5); it is noted here only to complete the ladder.

### 7.3 The Narrowing Rule

**An agent's `mcp` policy MUST NOT permit a server or method that is not permitted both by the package policy and by its resolved `from:` parent (if any). A validator MUST report a package-tier violation as `RECIPE_MCP_AGENT_WIDENS_POLICY`, and `schema::resolve_inherited` MUST report a `from:`-link violation as `RECIPE_MCP_FROM_WIDENS_POLICY`.**

Both checks are computed by the same function, `schema::mcp_policy_widenings(ceiling, candidate) -> Vec<String>`, the single shared implementation every narrowing check in this crate MUST use rather than re-deriving the include/exclude comparison independently. `ceiling` is whichever policy `candidate` must not exceed - the package's `mcp` for the package-tier check, or a resolved parent's `mcp` for the `from:`-link check. For each server `candidate`'s policy names:

1. **If `ceiling` does not mention that server at all**, `candidate`'s reference to it is a widening, regardless of what `candidate`'s own policy for that server says - even an empty `include` still names a server `ceiling` never authorized. This is deliberate: the naive check "compare only servers present in both policies" silently passes this case, because it never looks at servers `candidate` introduces unilaterally.
2. **If `ceiling` also mentions that server**, `candidate`'s effective method set (7.1) MUST be a subset of `ceiling`'s effective method set for that server. `include` and `exclude` interact through the effective-set computation alone: `candidate` may always add its own `exclude` entries (narrowing further is always legal), but removing an `exclude` `ceiling` declared - even while keeping the same `include` - widens the effective set back open and is rejected on that basis, without needing a rule specific to `exclude` removal.

These two checks are **not** the same enforcement mechanism `tools` uses, and this standard does not claim they are - stating that plainly matters more than a tidy-sounding equivalence, because an implementor who assumed otherwise would build the wrong thing:

- **`tools` has no package tier at all.** Its narrowing rule (section 4.7, `RECIPE_FROM_WIDENS_TOOLS`) exists solely at each `from:` link, comparing a child directly against its resolved parent. There is nothing above the top of a `tools` chain to check against; an agent with no `from:` is unconstrained by `tools` narrowing entirely.
- **`mcp` has two checks that compose.** `schema::resolve_inherited` enforces the `from:`-link check (`RECIPE_MCP_FROM_WIDENS_POLICY`) at every link while resolving a chain, exactly the way `tools` enforces its one check - this half genuinely does match `tools`, link for link. Separately, and in addition, `validate::validate_recipe_dir` enforces the package-tier check (`RECIPE_MCP_AGENT_WIDENS_POLICY`) once per agent, against that agent's fully resolved `mcp`, regardless of whether the agent declares `from:` at all. An agent with no `from:` is still bound by this check; a `tools`-only agent with no `from:` is not bound by anything.

Together the two checks close the same gap from both directions: the `from:`-link check catches a widening at the step where it happens, no matter how deep in the chain; the package-tier check catches a widening laundered through the chain even if some intermediate widening were, hypothetically, allowed to slip past a single link (it is not, per the previous paragraph, but the package check does not depend on that for correctness - it re-derives the answer independently, from the fully resolved value, rather than trusting that every link upstream was checked correctly).

---

## 8. Canonical Loader

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

## 9. Evals, Judges, and Prompts (`evals/`, `judges/`, `prompts/`)

A recipe holds one definition of good. `evals/` and `judges/` are that definition, and they belong IN the recipe because they MUST travel with the agent they judge: an eval case and the judge that scores it are meaningless detached from the agent they were written against.

**Evaluation *results* do not belong in the recipe.** A result belongs to a run, and MUST be attributed to that run's effective model and harness per section 4.5, never to the values the recipe declared. This is the split that makes a model sweep sound: hold the recipe fixed, vary the model per run via the override mechanism in 4.5, and run the same `evals/` and `judges/` against each. Same bar, many subjects, comparable results. If the bar lived outside the recipe, nothing would guarantee every model was measured against the *same* bar; if results lived inside the recipe, the recipe would stop being a statement of intent and become a record of the last experiment - precisely what section 4.5 forbids by requiring overrides to be recorded on the run, not written back into the artifact.

```text
<recipe>/
  evals/
    <name>/
      input.json          # the eval case's input
      expected.md          # the bar this case's output is judged against
  judges/
    <name>.yaml            # JudgeManifest: name, prompt (or prompt_file)
  prompts/
    <name>.md               # prompt text referenced by judges via prompt_file
```

`evals/`, `judges/`, and `prompts/` are all OPTIONAL. A recipe with no evals is valid; most recipes will start that way and grow an eval as the agent matures. Their absence MUST NOT be treated as an error.

**This standard defines the declaration of an eval case and a judge. It does NOT define execution semantics: how a judge scores an eval case's output, what a passing threshold is, how the judge's own model is chosen, or how a result is recorded.** Those are consumer concerns, belonging to whatever runs the eval, not to this standard. Standardizing the declaration keeps these artifacts discoverable and portable across harnesses and consumers without this standard taking on a scoring model it would then have to keep stable forever.

### 9.1 Eval Cases (`evals/<name>/`)

Each subdirectory directly under `evals/` is an eval case, named by `<name>` - the eval case's own identity, with no other manifest naming it. A case directory MUST contain both:

| File | Required | Description |
|---|---|---|
| `input.json` | YES | The case's input, in whatever shape the consumer that runs the eval expects. This standard does not define or validate its contents. |
| `expected.md` | YES | The bar this case's output is judged against, in prose. This standard does not define or validate its contents. |

An `evals/<name>/` directory missing either file is malformed and MUST be reported as `RECIPE_MALFORMED_EVAL_CASE` (section 10), never silently skipped. Both MUST be **regular files**: a conforming loader checks file type, not mere existence, so that a directory named `input.json` does not satisfy the requirement. A path that exists but cannot be read as a file is exactly the silently-broken bar this rule exists to prevent. Silently skipping a broken eval case is the worst failure mode available here: the bar quietly shrinks by one case while the suite continues to report green, and nothing signals that it happened.

This is discovered by `schema::load_recipe_dir` into `Recipe::evals: Vec<EvalCase>`, each carrying `name`, `input_path`, and `expected_path`, sorted by case name.

### 9.2 Judges (`judges/<name>.yaml`)

Each `judges/*.yaml` file conforms to `JudgeManifest`:

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | YES | Judge name. MUST be non-empty. |
| `description` | string | NO | Human-readable description. |
| `prompt` | string | NO* | The judge's prompt, given inline. |
| `prompt_file` | string | NO* | Reference to a prompt file, conventionally resolving to `prompts/<prompt_file>`, used instead of an inline `prompt`. |

\* At least one of `prompt` / `prompt_file` MUST be present. A judge declaring neither is malformed and MUST be reported as `RECIPE_INVALID_JUDGE_MANIFEST` (section 10), alongside an empty `name`. This standard does NOT require `prompt_file` to resolve to an existing file under `prompts/`: doing so would mean validating a runtime lookup this standard otherwise declines to specify, and a recipe is a portable artifact that MAY be validated on a machine that never runs its judges.

No other fields are permitted; `JudgeManifest` uses `#[serde(deny_unknown_fields)]`, kept deliberately small so the shape can grow additively in a future MINOR version once a real consumer's needs are known, rather than this standard guessing at a scoring model in advance.

This is discovered by `schema::load_recipe_dir` into `Recipe::judges: Vec<JudgeManifest>`, sorted by source file path.

### 9.3 Prompts (`prompts/<name>.md`)

`prompts/` holds prompt text as plain Markdown files, one per file, referenced by a judge's `prompt_file` (or by any other future consumer of prompt text this standard does not yet name). It exists so a judge's prompt can be authored and reviewed as its own file rather than folded into YAML string literals, and so multiple judges can share one prompt by reference. `schema::load_recipe_dir` gathers every `prompts/*.md` file into `Recipe::prompts: Vec<PathBuf>`, sorted; this standard does not otherwise interpret their contents.

---

## 10. Error Codes

### 10.1 Loader Codes

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
| `RECIPE_MCP_FROM_WIDENS_POLICY` | A child agent's `mcp` is not a subset of its resolved parent's `mcp`, checked at this one `from:` link (section 7.3). Emitted by `schema::resolve_inherited`. Distinct from `RECIPE_MCP_AGENT_WIDENS_POLICY` (section 10.2), which checks the fully resolved agent against the package tier. |
| `RECIPE_FROM_WIDENS_DELEGATION` | A child agent declares `allow_delegation: true` while its resolved parent declares `allow_delegation: false` (section 4.4a, section 4.7). Emitted by `schema::resolve_inherited`. |
| `RECIPE_MALFORMED_TOOL_MANIFEST` | A `tools/*/tool.yaml` file failed to parse as a `ToolManifest` (missing/extra/invalid fields, or a non-string key). See section 5.2. |
| `RECIPE_MALFORMED_EVAL_CASE` | An `evals/<name>/` directory is missing `input.json` or `expected.md`. Reported rather than silently skipped, so an incomplete eval case cannot quietly shrink the bar while the suite still reports green. See section 9.1. |
| `RECIPE_MALFORMED_JUDGE_MANIFEST` | A `judges/*.yaml` file failed to parse as a `JudgeManifest` (missing/extra/invalid fields, or a non-string key). See section 9.2. |
| `RECIPE_MISSING_TOOL_MANIFEST` | A direct child of `tools/` has no `tool.yaml`. Reported rather than silently skipped, for the same reason as `RECIPE_MALFORMED_EVAL_CASE`: skipping certifies a structurally incomplete tool package. See section 5.2. |
| `RECIPE_SUBAGENT_WIDENS_TOOLS` | A named subagent's resolved `tools` is not within the delegating agent's, including the case where the subagent declares none at all while the delegator is bounded. See section 4.4a. |
| `RECIPE_SUBAGENT_WIDENS_MCP` | A named subagent's resolved `mcp` policy is not within the delegating agent's. See section 4.4a. |
| `RECIPE_UNDECLARED_ROOT_ENTRY` | The recipe root holds an entry that is neither a kind this specification defines nor named in `extra_paths`. This is how section 2.1's prohibition on task input, credentials, and infrastructure configuration is enforced. See section 2.1. |

### 10.2 Validator Codes

`validate::validate_recipe_dir` is built on top of the loader (plan revision R1: loading and validation share one code path). On a failed load it surfaces exactly one loader code from §10.1. On a recipe that loads cleanly it runs the additional structural rules below, reporting *all* violations via `apss_core::Diagnostics` rather than failing on the first one. These codes live in `validate::error_codes`.

| Code | Meaning |
|------|---------|
| `RECIPE_SUBAGENT_UNRESOLVED` | A `subagents` entry names an agent with no matching `agents/<name>.yaml`. |
| `RECIPE_EMPTY_RECIPE_NAME` | `recipe.yaml`'s `name` is present but empty/whitespace. |
| `RECIPE_EMPTY_AGENT_NAME` | An agent manifest's `name` is present but empty/whitespace. |
| `RECIPE_EMPTY_MODEL_NAME` | An agent manifest's `model.name` is present but empty/whitespace. |
| `RECIPE_INVALID_SKILL_REF` | A `skills` entry is an empty string (bare form) or has an empty `ref` (pinned form). |
| `RECIPE_SKILL_UNPINNED` | A pinned `skills` entry's `version` is `latest`/`@latest` (case-insensitively) or is empty/whitespace. See section 5.1. |
| `RECIPE_INVALID_TOOL_REF` | A `tools` entry is an empty string. |
| `RECIPE_EMPTY_INSTRUCTIONS_CONTENT` | An agent's `system_instructions.content` is present but empty/whitespace. |
| `RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL` | An agent that omits `harness` lists a `tools` entry that is harness-builtin under some harness and does not resolve as recipe-provided. See section 4.6. A name that resolves under `tools/` MUST NOT trigger this, even if it is also harness-builtin under some harness - recipe-provided wins (section 5.2). |
| `RECIPE_MCP_AGENT_WIDENS_POLICY` | An agent's fully `from:`-resolved `mcp` policy names a server the package's `mcp` does not permit, or permits a method for a shared server the package does not. See section 7. |
| `RECIPE_INVALID_TOOL_MANIFEST` | A `tools/<ref>/tool.yaml` parsed successfully but has an empty `name` or an empty `command`. See section 5.2. |
| `RECIPE_INVALID_JUDGE_MANIFEST` | A `judges/*.yaml` parsed successfully but has an empty `name`, or declares neither `prompt` nor `prompt_file`. See section 9.2. |

Field-shape rules (unknown fields, non-string keys, unrecognized `harness`/`effort`/`mode` enum values) are enforced by `#[serde(deny_unknown_fields)]` and the typed enums during load, so they surface as `RECIPE_MALFORMED_MANIFEST` / `RECIPE_MALFORMED_HARNESS_YAML` on the offending file rather than as separate validator codes.

### 10.3 Evaluation Point: As-Authored vs. Resolved

Every validator rule runs against exactly one of two forms of an agent manifest: the **as-authored** form (`Recipe::agents`, exactly what `agents/<name>.yaml` declares, before any `from:` merge) or the **resolved** form (`schema::resolve_inherited(recipe, name)`, the as-authored value merged field-wise with its `from:` chain per section 4.7). Section 7.3 states this explicitly for the `mcp` package check; this subsection states it for every other rule, since a rule's evaluation point is not otherwise obvious from its error code alone and gets it wrong silently otherwise (Fix 1 of the final review wave moved `RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL` from as-authored to resolved for exactly this reason - see below).

| Rule / code | Evaluated against | Why |
|---|---|---|
| `RECIPE_MCP_AGENT_WIDENS_POLICY` (package tier, section 7.3) | Resolved | The package ceiling must bind the agent's actual effective policy, not a value a `from:` parent might override; checking the authored form would let a widening be laundered through inheritance. |
| `RECIPE_MCP_FROM_WIDENS_POLICY`, `RECIPE_FROM_WIDENS_TOOLS`, `RECIPE_FROM_WIDENS_DELEGATION`, `RECIPE_FROM_CYCLE`, `RECIPE_FROM_UNRESOLVED` | Both, by construction | These are per-link checks computed *during* resolution itself (inside `resolve_inherited`); "as-authored" and "resolved" are the two ends of the one link being checked at each step, not a choice between two independent views of the whole chain. |
| `RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL` (section 4.3/4.6) | Resolved | An as-authored `harness: None` is not proof of harness-agnosticism: a `from:` child that omits `harness` while inheriting one from its parent is not agnostic, and rejecting it for narrowing a builtin toolset it received from that parent would make narrowing impossible without redeclaring `harness` on every child. Checked against `resolve_inherited(recipe, name).harness` and `.tools`, consistent with the `mcp` package check above. |
| `RECIPE_INVALID_TOOL_REF` (empty `tools` entry) | As-authored | This is a syntax check (is the string non-empty), not a permission check; there is nothing for `from:` resolution to change about whether a literal string is empty. |
| `RECIPE_SKILL_UNPINNED`, `RECIPE_INVALID_SKILL_REF` | As-authored | `skills` is explicitly NOT merged through `from:` (section 4.7's merge table): the child's own `skills` is used as declared, so there is no resolved form distinct from the authored one to check against. |
| `RECIPE_SUBAGENT_UNRESOLVED` | As-authored | `subagents` is likewise not merged through `from:` - the child's own value always stands (section 4.7) - so the authored value already is the value being checked. |
| `RECIPE_EMPTY_MODEL_NAME`, `RECIPE_EMPTY_AGENT_NAME`, `RECIPE_EMPTY_RECIPE_NAME`, `RECIPE_EMPTY_INSTRUCTIONS_CONTENT` | As-authored | These reject an explicitly-declared-but-empty string; the loader's `Option` typing already ensures "declared" and "empty" are distinguishable per field. |

**Known latent inconsistency (not fixed by this table):** `RECIPE_EMPTY_MODEL_NAME` runs against the as-authored `model.name`, which means an agent that authors `model: {name: ""}` is rejected even in a case where a `from:` parent would have supplied a non-empty `model.name` had the child simply omitted `model.name` instead of authoring it empty. This is inconsistent with treating `model.name` as inheritable (section 4.7's `model` merge rule), but is a narrow, low-impact edge case - an author who wants to inherit `model.name` already has the correct spelling for it (omit the field, not set it to `""`) - and is left as a follow-up rather than fixed in this pass.

### 10.4 CLI

`validate_recipe_dir` is wired into the composed development CLI as a registered standard command:

```text
apss-dev run agent-recipe validate <recipe-dir>
```

(aliases: `recipe`, `exp-v1-0005`). Exit code 0 means no errors; 1 means one or more error diagnostics. This is the same `apss-dev run <slug> <command>` surface the official standards (`topology`, `architecture-fitness`, `documentation`) use to expose their validators. The separate `apss-dev v1 validate experiment EXP-V1-0005` command remains a purely structural meta-standard check of the crate's package layout and does not take a recipe-directory argument.

---

## 11. Compliance Checklist

A recipe directory is **compliant** with this standard if:

- [ ] `recipe.yaml` exists at the directory root and parses as a `RecipeManifest` with no unrecognized fields.
- [ ] `default_agent` resolves to a file under `agents/`.
- [ ] Every `agents/*.yaml` file parses as an `AgentManifest` with a non-empty `name`. `harness` and `model` are both OPTIONAL (section 4.3, 4.4); when `model` is present, `model.name`, when present, is non-empty, and `model.effort`, when present, is one of `low`/`medium`/`high`.
- [ ] `skills`, if present, is an array of `SkillRef` entries - either a bare string or a pinned object (`ref`, plus optional `source_url`/`version`/`resolved_sha`) - not a plain array of strings (section 5.1). A pinned entry's `version`, when present, MUST NOT be `latest`/`@latest` (case-insensitively) or empty/whitespace (`RECIPE_SKILL_UNPINNED`).
- [ ] `tools` and `subagents`, if present, are arrays of strings.
- [ ] `system_instructions`, if present, has a valid `mode` and non-empty `content`.
- [ ] An agent that omits `harness` (harness-agnostic, after `from:` resolution) does not reference a `tools` entry that is harness-builtin under any harness this standard knows about, unless that entry also resolves as recipe-provided under `tools/` (`RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL`, section 4.3/4.6, checked against the resolved manifest per section 10.3).
- [ ] A `from:`-child's `tools`, when both it and its resolved parent's `tools` are present, is a subset of the resolved parent's `tools` (`RECIPE_FROM_WIDENS_TOOLS`, section 4.7).
- [ ] A `from:`-child does not resolve `allow_delegation: true` when its resolved parent's is `false` (`RECIPE_FROM_WIDENS_DELEGATION`, section 4.4a, 4.7).
- [ ] Every agent's fully `from:`-resolved `mcp` policy, if present, is a subset of the package's `mcp` policy (section 7), and a `from:`-child's `mcp` is a subset of its resolved parent's `mcp` at each link (`RECIPE_MCP_FROM_WIDENS_POLICY`, section 7.3).
- [ ] Every `subagents` entry names a real `agents/<name>.yaml` (`RECIPE_SUBAGENT_UNRESOLVED`, section 4.4a).
- [ ] Every named subagent's resolved `tools` and `mcp` are within the delegating agent's (`RECIPE_SUBAGENT_WIDENS_TOOLS`, `RECIPE_SUBAGENT_WIDENS_MCP`, section 4.4a).
- [ ] Every `evals/<name>/` directory, if `evals/` is present, contains both `input.json` and `expected.md` (section 9.1).
- [ ] Every `judges/*.yaml` file, if `judges/` is present, has a non-empty `name` and at least one of `prompt`/`prompt_file` (section 9.2).
- [ ] No unrecognized fields are present at any nesting level.

---

## 12. Generator

A conformant recipe directory can be scaffolded from the canonical template in `templates/recipe/skeleton/`:

```text
apss-dev run agent-recipe create <name> [--dir <parent>]
```

This writes `<parent>/<name>/` (parent defaults to the current directory) containing `recipe.yaml` (with `{{name}}` substituted), `agents/main.yaml` (a `claude` default agent plus a commented `codex` example), `SYSTEM.md`, and an empty `skills/` (kept with a `.gitkeep`). The generator refuses to overwrite an existing destination.

The library entry point is `generate::scaffold_recipe(name, dest)`. The template files are embedded into the crate via `include_str!`, so the generator is self-contained and works from any working directory, and its output can never drift from the reviewed on-disk skeleton.

**Round-trip guarantee (normative):** generator output MUST always pass `validate_recipe_dir` with zero errors. This is enforced by `tests/round_trip_test.rs`, which scaffolds into a temp directory and validates the result.

---

## 13. Future Extensions

Potential future additions, to be pursued only after this experiment gathers feedback:

- Additional `harness` values (`opencode`, `gemini`, others) as those harnesses gain first-class support.
- A substandard defining the full `RunSpec` envelope (`recipe` + `task` + `input_artifacts` + `credentials` + `observability` + `limits`) referenced informatively in 1.5.
- Recipe inheritance / composition (`extends: <recipe-name>`).
- Per-skill or per-tool configuration parameters, if injected skills/tools need arguments beyond a bare reference.
- JSON Schema artifact generation for editor tooling and IDE autocompletion.

---

## 14. Security Considerations

### 14.1 No Credentials in Recipes

Recipe directories MUST NOT contain credentials, tokens, or other secrets. Recipes are expected to be committed to version control; secret material belongs in the `credentials` component of a `RunSpec` (informative, see 1.5), not in the recipe.

### 14.2 System Instruction Content

`system_instructions.content` (and `SYSTEM.md`) is free-form text that becomes part of an agent's effective system prompt. Consumers SHOULD treat recipe sources with the same trust level as other executable configuration (for example, CI workflow files): a recipe from an untrusted source can materially change agent behavior via `mode: replace` or injected `skills`/`tools`.

---

## 15. References

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
