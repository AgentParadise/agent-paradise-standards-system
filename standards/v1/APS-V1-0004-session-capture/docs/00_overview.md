# Session Capture Standard - Overview

## What is this?

**APS-V1-0004** defines one contract for backing up and searching agent
sessions from any runtime. The operator runs agents in many places (the Claude,
Codex, and Cursor CLIs on a laptop; a VPS; Docker workspaces; syntropic137
workflows), and each provider stores its transcript in a different native shape.
This standard makes "back up my agent sessions" work identically across all of
them, without forcing anyone to agree on a provider's internal message format.
It enables:

- **Uniform backup** - Every session, from every runtime, lands in one store in
  one shape.
- **Search across providers** - Filter and query sessions by metadata (repo,
  origin, agent, model) and search their content, without the store having to
  understand each provider's transcript format up front.
- **Attribution** - Every session carries where it came from, so a mixed corpus
  (this MacBook vs the VPS vs a specific workspace vs a workflow run) stays
  attributable.
- **Zero-churn provider onboarding** - A new provider is added by wrapping its
  raw transcript and filling the envelope header. No schema change.

## Why does it matter?

As operators run more agents across more runtimes, session history becomes a
high-value corpus: a searchable record of what was tried, where, and by which
agent. But providers change their transcript shapes and new ones appear
constantly. A standard that tried to normalize every provider into one canonical
message shape would be lossy and would rot the moment a provider changed. This
standard avoids that trap:

1. **The contract is thin and stable** - a universal envelope of a handful of
   fields, wrapped around the provider transcript preserved verbatim.
2. **Search is a server concern** - the normalized, searchable view is built by
   the server over the raw transcript, best-effort and swappable, never a
   client promise.
3. **It is versioned and adoptable** - published as an APSS standard with SemVer
   schema versioning and additive-only evolution.

## The core design decision

The load-bearing choice is an **envelope plus opaque raw payload**, NOT full
normalization.

- A THIN envelope carries the universal fields the store backs up, sorts,
  dedups, and attributes by: `scs_version`, `origin`, `agent`, `source_format`,
  `session_id`, `parent_session_id`, `started_at`, `last_activity_at`,
  `content_hash`, `metadata`, and `raw`.
- The provider's original transcript is preserved byte for byte in `raw`. The
  standard never parses or reshapes it.

Pattern precedent: CloudEvents (standard envelope, arbitrary `data`), email
(standard headers, arbitrary body), shipping containers (standard outside,
arbitrary contents). The consequence is that a new provider costs one
`source_format` value and a filled header; the standard cannot rot as providers
churn, because it never claimed to understand their internals.

## Quick example

A single Claude Code session becomes one envelope, shown here as a producer emits
it. `metadata` is freeform and powers search. Note two things a producer gets
right by omission and by shape:

- No `content_hash`. The store computes it at ingest (spec 4.2.3).
- `raw` is a string holding the transcript's exact bytes, not a parsed object.
  Only the string shape supports byte-exact resume (spec 4.3.1).

```jsonc
{
  "scs_version": "1.0",
  "origin":        { "host": "macbook-neural", "environment": "local" },
  "agent":         "ClaudeCode",
  "source_format": "claude-jsonl-v1",
  "session_id":    "019973e4-58a9-7b83-9f21-2b6c4d0a1e77",
  "parent_session_id": null,
  "started_at":    "2026-05-02T14:03:11Z",
  "last_activity_at": "2026-05-02T15:20:44Z",
  "metadata": {
    "repo": "AgentParadise/agent-paradise-standards-system",
    "cwd": "/Users/neural/Code/apss",
    "project": "apss",
    "model": "claude-opus-4-8",
    "tags": ["standards", "authoring"],
    "message_count": 42,
    "workflow_id": null,
    "source_path": "-Users-neural-Code-apss/019973e4-58a9-7b83-9f21-2b6c4d0a1e77.jsonl"
  },
  "raw": "{\"type\":\"user\",\"message\":{...}}\n{\"type\":\"assistant\",...}\n"
}
```

## Metadata is first-class for search

Metadata is the operator's highest-value point. Conformant stores SHOULD support
querying and filtering sessions by metadata fields, not only full-text over
content. Queries such as "all sessions from repo X", "everything from
origin=vps", or "everything by agent Codex" are the baseline search surface.

- **Lexical and metadata search is the baseline** - required of a conformant
  store.
- **Semantic and embedding search is optional** - explicitly cost-sensitive, and
  never required by this standard.

## Search is server-derived, not a client contract

Cross-provider search still matters, but the normalized, searchable view is
built by the SERVER running best-effort, per-provider parsers over `raw`. That
view is allowed to be lossy, because `raw` is always ground truth. Parsers
improve or get added without touching this standard and without re-uploading
anything. This is the direct answer to "are we overfitting?": the client
contract commits to nothing provider-specific.

## Three conformance profiles

Different runtimes surface sessions differently, so the standard offers two ways
to produce envelopes rather than forcing one, plus a third profile that restores
them.

- **Source (pull)** - the runtime writes transcripts to disk (Claude, Codex,
  Cursor). A Source reads them, wraps `raw`, fills the header, and hands
  envelopes to an uploader.
- **Exporter (push)** - the runtime is ephemeral or remote (a Docker workspace,
  a syntropic137 workflow). It POSTs envelopes itself to the batch endpoint with
  a scoped token.
- **Reconstitutor (restore)** - the inverse of a Source. It fetches a stored
  session, writes the transcript back to the path the harness expects on
  *this* machine, and hands off to the harness's native resume. Same-harness
  only; capture on the laptop, resume on the VPS.

Local CLIs, containers, and workflows all conform without changing how they
natively store transcripts.

## Why resume comes almost free

Preserving `raw` verbatim was chosen so the contract could not rot as providers
change their transcript formats. It turns out to buy something larger: if the
stored bytes are the harness's own session file, they can be written back and the
harness can resume from them.

That also makes the standard's central promise testable. "Preserve `raw`
verbatim" was previously a rule nothing could check. A round trip checks it:
capture a session, reconstitute it, compare. The requirement becomes a test rather
than an assertion.

## Transport

`POST /v1/sessions/batch` with a `{ "envelopes": [...] }` body, idempotent by
`(session_id, content_hash)`, authenticated with a scoped bearer token
(`sessions:write`). The server ALWAYS sanitizes envelopes before persisting,
regardless of source, so a compromised or naive exporter cannot leak secrets into
the shared store.

Because sanitization changes the bytes, `content_hash` is defined over the
captured content, before sanitization, and is computed by the store rather than
the producer. Dedup is therefore resolved server-side, and stays stable across
sanitizer changes.

## Versioning and evolution

`scs_version` is SemVer. Within a major, changes are additive only (new optional
fields, new `source_format` values, new `environment` values). New providers
never require a version bump: they add a `source_format` value and, optionally, a
server-side parser.

## Adopters

- **SeshMagic** - Source implementations for Claude, Codex, and Cursor, the
  reference exporter, the store (sanitizer and parsers), and the Reconstitutor
  client. This is the home of the behavioural implementation.
- **syntropic137 workflows** - Exporter profile, emitting one envelope per agent
  from a workflow-completion hook, grouped by a shared `metadata.workflow_id`.
- **agentic-primitives `providers/workspaces`** - Exporter profile, embedded in
  the workspace image so capture is turnkey for every spawned workspace.

## Status

**Active (ratified)** - Promoted from `EXP-V1-0003` on 2026-08-06 at version
1.0.0. The envelope contract is proven by a corpus of roughly 3,000 real sessions
captured across Claude, Codex, and Cursor, which is what qualified it for
ratification: consumers can now take a stable, versioned dependency on it.

Within a major version, changes are additive only (specification section 8).
The current major is 2. It was spent not on a wire change - the envelope stayed
additive - but on Rust source compatibility: `Origin` became `#[non_exhaustive]`
with constructors, so that every FUTURE optional field is additive in Rust as
well as on the wire.

## Learn more

- Read the [full specification](./01_spec.md).
- Inspect the [JSON Schema](../schemas/session-envelope.schema.json).
- Inspect the [reconstitution registry](../registry/reconstitution.toml).
- Browse [examples](../examples/).
- See [agent skills](../agents/skills/).
