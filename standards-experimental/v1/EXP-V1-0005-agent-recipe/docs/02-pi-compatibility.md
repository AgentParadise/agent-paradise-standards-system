# pi.recipes Compatibility

The Agent Recipe directory shape is adopted from the `pi.recipes` convention (a
recipe as a directory of agent manifests plus shared assets) and generalized so
it is not tied to any single harness or runtime. This document records the
deltas: where EXP-V1-0005 deliberately diverges from `pi.recipes`, and why.

## Summary of Deltas

| Concern | pi.recipes | EXP-V1-0005 |
|---------|------------|-------------|
| Extensions / tools | TypeScript `extensions/` with executable code | No `extensions/`; `tools` are references (names) only, no code |
| Harness selection | Runtime-level | Per agent (`agent: claude \| codex`) - one recipe can mix harnesses |
| Reasoning effort | `thinking_level` (harness-specific) | `model.effort: low \| medium \| high` (harness-neutral) |
| Agents vs subagents | Separate concepts | Unified in one `agents/` dir; a subagent is just an agent referenced by another |

## 1. No TypeScript `extensions/` - `tools` are references only

`pi.recipes` allows an `extensions/` directory of executable (TypeScript)
extension code that the runtime loads. EXP-V1-0005 deliberately does **not**
carry executable code inside a recipe:

- There is no `extensions/` directory in the directory shape.
- An agent's `tools:` field is a list of **references** (opaque
  string identifiers such as `shell` or a tool name), not code. This standard
  defines no execution semantics for `tools`; resolving and enforcing them is
  entirely a consumer/harness concern.

This keeps a recipe a pure, safe-to-commit data artifact: cloning or diffing a
recipe never means pulling in executable code. It also keeps the schema crate
free of any runtime/language dependency, so downstream consumers can depend on
`load_recipe_dir` as a plain library.

## 2. Harness is per-agent

In EXP-V1-0005 the harness is chosen **per agent**, via the required `agent`
field on each `agents/<name>.yaml`:

```yaml
# agents/main.yaml
agent: claude
# agents/reviewer.yaml
agent: codex
```

Because each agent names its own harness, a single recipe can mix harnesses -
for example a `claude` default agent that delegates to a `codex` subagent. The
`agent` enum is closed in v1 (`claude`, `codex`) but version-extensible;
unrecognized values fail to parse rather than being silently coerced.

## 3. `effort` instead of `thinking_level`

`pi.recipes` (Claude-oriented) uses `thinking_level`. EXP-V1-0005 uses a
harness-neutral `model.effort` with three coarse levels:

```yaml
model:
  name: anthropic/claude-opus-4-8
  effort: low | medium | high
```

The three levels are intentionally coarse so they map cleanly across harnesses
that expose different granularities of reasoning effort (Claude's
`thinking_level`, Codex's reasoning effort, etc.). Translating the coarse level
into a harness's native parameter is the adapter's job, not the recipe's.

## 4. Agents and subagents unified in one `agents/` directory

`pi.recipes` treats agents and subagents as related but distinct concepts.
EXP-V1-0005 **unifies** them:

- Every agent - default agent and subagent alike - is one YAML file under
  `agents/`.
- A "subagent" is simply an agent that another agent references by name in its
  `subagents:` list. There is no separate directory, schema, or file marker for
  subagents.
- Which agent is the entry point is decided solely by
  `recipe.yaml: default_agent`; which agents are subagents is decided solely by
  other agents' `subagents:` lists.

**v1 scope:** subagent references are **validated only, not executed**. The
validator resolves each `subagents:` entry to a sibling `agents/<name>.yaml`
(emitting `RECIPE_SUBAGENT_UNRESOLVED` if it dangles), but this standard does
not yet define delegation/execution semantics between an agent and its
subagents. Executing subagents (true multi-agent orchestration) is a planned
follow-on once a real consumer exists.

## What Carries Over

Field names and enum values are kept identical to the single-agent
`pi.recipes`-shaped manifest where they overlap, for compatibility:
`name`, `agent`, `model.name`, `model.effort`, `skills`, and
`system_instructions` (`mode: append | replace` + `content`). The directory
shape adds `tools` and `subagents` on top of that carried-over agent shape.
