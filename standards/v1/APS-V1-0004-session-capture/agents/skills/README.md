# Agent Skills for the Session Capture Standard

This directory documents agent skills for working with APS-V1-0004 (the Session
Capture Standard). The standard is definitional; these skills describe how an
agent reasons about capturing and searching sessions, and where the reference
implementation lives.

## Available Skills

| Skill | Purpose |
|-------|---------|
| `build-envelope.md` | How to wrap a provider transcript into a valid session envelope (Source or Exporter). |
| `search-sessions.md` | How to query a conformant store by metadata and content. |
| `check-conformance.md` | How to check that a producer, store, or Reconstitutor meets the conformance criteria. |
| `reconstitute-session.md` | How to restore a stored session onto this machine and resume it natively (Reconstitutor). |

The skill bodies are the spec sections they point at; until the individual skill
files are authored, treat [`../../docs/01_spec.md`](../../docs/01_spec.md) as the
agent context.

## What an agent should know

- **Never flatten `raw`.** The provider transcript is preserved verbatim. An
  agent building an envelope wraps the transcript in `raw` and fills the header
  fields; it does not reshape provider messages.
- **Metadata is the search surface.** To find prior sessions, filter by
  `metadata.repo`, `origin.environment`, `agent`, `model`, or `tags` first;
  full-text over content is a fallback. Semantic search may not be available.
- **Idempotency is free.** Re-emitting the same session is safe: the batch
  endpoint dedups on `(session_id, content_hash)`. An agent building an envelope
  does NOT compute `content_hash`; the store derives it over the sanitized stored
  form (spec 4.2.3). Omit the field.
- **The server sanitizes.** An agent should still pre-redact obvious secrets,
  but the store always sanitizes before persisting.
- **`raw` must be captured verbatim, as bytes.** For a line-delimited transcript,
  capture the file's exact contents. Parsing it into an array of objects and
  storing that is lossy: re-serializing does not reproduce the original bytes, so
  it forfeits reconstitution (spec 6.4.4).
- **Never trust an envelope when restoring.** `session_id` and
  `metadata.source_path` come from producers, which may be untrusted. Validate
  both before interpolating into a command or a write path (spec 6.4.5); the
  crate's `reconstitution` module does this for you.

## Where the implementation lives

The behavioural implementation (Source adapters, the exporter and CLI, the
server-side sanitizer and parsers, the Reconstitutor client) lives in the
SeshMagic repository, not here.

This package carries the spec, the JSON Schema, the canonical envelope types and
their validation, and the reconstitution registry. Consumers depend on the
`apss-v1-0004-session-capture` crate rather than reimplementing the envelope, so
divergence from the standard is a build failure instead of an undetected drift.
