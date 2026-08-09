# Harness-Builtin Tool Vocabulary

This document is **normative**: it defines the closed-per-version set of
harness-builtin tool identifiers referenced by [01_spec.md section
4.3](./01_spec.md#43-the-harness-field) and the
portability rule it supports. It exists to make one specific claim checkable:
an agent manifest that omits `harness` (or, in a future revision, an agent
explicitly declared harness-agnostic) is asserting that it runs unmodified
under any harness this standard knows about, and therefore MUST NOT list a
harness-builtin tool name in its `tools` array. That claim can only be
verified against a concrete, versioned list of names - this is that list.

This standard is inspired by pi.recipes; it is not compatible with it (see
[04-rationale-and-prior-art.md](./04-rationale-and-prior-art.md)). The tool
identifiers below are specific to each harness's own CLI/SDK and carry no
relationship to pi.recipes' vocabulary.

## Normative Statement

The identifiers in this document are the harness-builtin tool names for this
version of the standard. A tool reference in an `AgentManifest.tools` entry
that is not in this document, and not resolvable under `tools/` (see
[01_spec.md section 5](./01_spec.md) for the resolution rule that section
defines for `skills`; `tools` entries carry no execution semantics and no
analogous bundled-directory resolution today - see section 1.3), is a
recipe-provided reference and MUST resolve per the harness-neutrality rule in
section 4.3: it is treated as opaque outside this standard's scope, not
matched against the tables below.

Concretely, for the validator this document backs (`is_harness_builtin`):

- If `harness: claude` and `tools` contains an identifier from the Claude Code
  table below, that entry references a harness-builtin tool.
- If `harness: codex` and `tools` contains an identifier from the Codex CLI
  table below, that entry references a harness-builtin tool.
- Any other string is a recipe-provided (non-builtin) tool reference.

## Growth and Compatibility

This table grows in MINOR versions of this standard as harnesses add
first-party tools or as additional harnesses are added to the `harness`
enumeration (section 4.3). Additions are backwards compatible: a recipe
written against an older version of this table remains valid, because adding
a new builtin identifier can only narrow which `tools` entries are
harness-builtin, never invalidate a previously-valid recipe. Removing or
renaming a row is a breaking change and MUST NOT happen within a MINOR
version.

---

## Claude Code

Identifiers are PascalCase. Verified by extracting literal tool-name string
constants from the installed Claude Code binary
(`/Users/neural/.local/share/claude/versions/2.1.226`, i.e. `claude
--version` reporting `2.1.226`) via `strings`, cross-checked against
`claude --help` (which documents the `--tools`/`--allowedTools` flags using
these same names, e.g. `--tools <tools...>` "specify tool names (e.g.
`Bash,Edit,Read`)").

| identifier | purpose | confirmed from |
|---|---|---|
| `Bash` | Run a shell command | claude binary strings (v2.1.226); `claude --help` example |
| `BashOutput` | Poll output of a backgrounded shell command | claude binary strings (v2.1.226) |
| `KillShell` | Terminate a backgrounded shell command | claude binary strings (v2.1.226) |
| `Read` | Read a file | claude binary strings (v2.1.226); `claude --help` example |
| `Edit` | Make a targeted string replacement in a file | claude binary strings (v2.1.226); `claude --help` example |
| `MultiEdit` | Apply multiple edits to one file in one call | claude binary strings (v2.1.226) |
| `Write` | Create or overwrite a file | claude binary strings (v2.1.226) |
| `Glob` | Find files by glob pattern | claude binary strings (v2.1.226) |
| `Grep` | Search file contents | claude binary strings (v2.1.226) |
| `NotebookEdit` | Edit a Jupyter notebook cell | claude binary strings (v2.1.226) |
| `WebFetch` | Fetch and summarize a URL | claude binary strings (v2.1.226) |
| `WebSearch` | Perform a web search | claude binary strings (v2.1.226) |
| `Task` | Launch a subagent | claude binary strings (v2.1.226) |
| `TodoWrite` | Maintain the session's structured todo list | claude binary strings (v2.1.226) |
| `ExitPlanMode` | Exit interactive plan mode | claude binary strings (v2.1.226) |
| `AskUserQuestion` | Ask the user a structured clarifying question | claude binary strings (v2.1.226) |

Not included: skill invocation, MCP-provided tools (`mcp__<server>__<tool>`),
and plugin/output-style machinery are not harness-builtin in the sense this
document defines - they are dynamically registered per-session, not part of
the fixed builtin set, so they are out of scope for this table.

## Codex CLI

Identifiers are snake_case. Verified against the installed Codex CLI binary
(`/opt/homebrew/bin/codex`, `codex --version` reporting `codex-cli 0.146.1`)
via `strings`, and against the `openai/codex` GitHub repository's
`codex-rs/core/src/tools/handlers/*_spec.rs` tool-definition source (each
`ToolSpec::Function { name: "...".to_string(), ... }` literal).

Important caveat found during verification: Codex's shell-execution tool is
**not** a single fixed identifier. The installed binary embeds a `shell_type`
configuration field whose value selects which literal tool name the model
sees for shell execution (`"shell_type": "shell_command"` was the default
observed), and separately exposes a session-based "unified exec" mode
(`exec_command` + `write_stdin`) as an alternative to a single request/response
shell call. All variants observed are recorded below rather than guessing
which one a given consumer's Codex configuration will present.

| identifier | purpose | confirmed from |
|---|---|---|
| `shell` | Run a shell command (classic, single request/response) | codex binary strings (v0.146.1, literal `"shell"`) |
| `shell_command` | Run a shell command (current default `shell_type` in the installed build) | codex binary strings (v0.146.1: `"shell_type": "shell_command"`, and `handlers/shell/shell_command.rs`); `openai/codex` source `codex-rs/core/src/tools/handlers/shell_spec.rs` (`name: "shell_command".to_string()`) |
| `exec_command` | Start/continue a PTY-backed shell session (unified exec mode) | codex binary strings (v0.146.1); `openai/codex` source `codex-rs/core/src/tools/handlers/shell_spec.rs` (`name: "exec_command".to_string()`) |
| `write_stdin` | Write to a running unified-exec session started by `exec_command` | codex binary strings (v0.146.1, literal `"write_stdin"`); `openai/codex` source `codex-rs/core/src/tools/handlers/shell_spec.rs` (`name: "write_stdin".to_string()`) |
| `apply_patch` | Edit files via a structured patch grammar | codex binary strings (v0.146.1, literal `"apply_patch"`); `openai/codex` source `codex-rs/core/src/tools/handlers/apply_patch_spec.rs` (`name: "apply_patch".to_string()`); platform.openai.com/docs/guides/tools-apply-patch |
| `update_plan` | Set/update the model's step-by-step plan | codex binary strings (v0.146.1, substring `update_plan` in config/tool tables); `openai/codex` source `codex-rs/core/src/tools/handlers/plan_spec.rs` (`name: "update_plan".to_string()`) |
| `view_image` | Load a local image file into context | codex binary strings (v0.146.1, substring `view_image` in tool-call/config strings); `openai/codex` source `codex-rs/protocol/src/models.rs` (`pub const VIEW_IMAGE_TOOL_NAME: &str = "view_image"`) |
| `web_search` | Perform a live web search | codex binary strings (v0.146.1, literal `"web_search"`) |

Not included: MCP-provided tools (`mcp__<server>__<tool>`), the
multi-agent/collaboration surface (`spawn_agent`, `send_message`,
`followup_task`, `wait_agent`, `interrupt_agent`, `list_agents`,
`tool_search`) observed in the installed build's `codex debug prompt-input`
output, and app/connector tools are dynamically registered per-session or
per-feature-flag rather than part of a fixed builtin set, so they are out of
scope for this table. They are recorded here only as a pointer for a future
table revision if any of them stabilize into an always-present builtin.

---

## See Also

- [01_spec.md section 4.3](./01_spec.md#43-the-harness-field) - the `harness`
  field and harness extensibility rule this table exists to support.
- [04-rationale-and-prior-art.md](./04-rationale-and-prior-art.md) - why this
  standard is inspired by, but not compatible with, pi.recipes, and why
  `tools` are references only, with no execution semantics defined here.
