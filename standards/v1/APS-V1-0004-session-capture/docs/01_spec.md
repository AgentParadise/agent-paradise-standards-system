# APS-V1-0004 - Session Capture Standard

**Version**: 1.0.0
**Status**: Active (ratified)
**Category**: Technical
**Promoted from**: EXP-V1-0003 on 2026-08-06

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope and Authority

### 1.1 Purpose

This standard defines a single **capture contract** so that agent sessions from
any runtime back up uniformly and become searchable. The operator runs agents
across many runtimes (the Claude, Codex, and Cursor CLIs; a VPS; Docker
workspaces; syntropic137 workflows), each of which stores its transcript in a
different provider-native shape. This standard makes "back up my agent sessions"
work identically across all of them without requiring agreement on any
provider's internal transcript format.

The contract is a THIN **session envelope**: a small set of universal header
fields wrapped around the provider's original transcript, preserved verbatim in
a `raw` field. The envelope is designed to be:

1. **Uniform** - Same shape for every runtime and provider.
2. **Stable** - A new provider costs one new `source_format` value and a filled
   header; the standard never parses provider internals, so it cannot rot as
   providers churn.
3. **Attributable** - Every envelope records its origin.
4. **Idempotent** - Envelopes deduplicate on `(session_id, content_hash)`.
5. **Searchable** - Metadata is first-class for filtering and querying; content
   search is a server-derived concern over `raw`.

### 1.2 Scope

This standard covers:

- **The session envelope** - Its header fields and the opaque `raw` payload
  (section 4), machine-checkable against the shipped JSON Schema.
- **Transport** - The batch upload endpoint, its idempotency and auth (section
  5).
- **Conformance profiles** - Source (pull), Exporter (push), and Reconstitutor
  (restore), with crisp criteria for each (section 6).
- **Search expectations** - What a conformant store MUST and MAY support
  (section 7).
- **Reconstitution** - Restoring a captured session onto a different machine so
  the originating harness can resume it natively (section 6.4), backed by the
  shipped reconstitution registry.
- **Versioning and evolution** - SemVer of the envelope and additive-only
  evolution (section 8).

This standard does NOT cover:

- **Provider transcript internals** - The shape of `raw` is opaque to this
  standard. It is never parsed, reshaped, or validated by the contract.
- **Server-side normalization for search** - The parsers that build a searchable
  view over `raw` are a best-effort, swappable, server-side concern (section 7),
  not part of the client contract.
- **Storage engine, indexing technology, and query language** - Implementation
  detail of a conformant store.
- **Cross-harness reconstitution** - Restoring a Claude session into Codex, or
  any other harness-to-harness translation, is explicitly OUT of scope. It would
  require translating the derived projection (section 7.3) rather than restoring
  the captured bytes, which would make reconstitution lossy. Reconstitution is
  same-harness only (section 6.4).
- **Specific Source, Exporter, or Reconstitutor implementations** - Informative
  only (section 11). The Rust reference implementation lives in the SeshMagic
  repository.

### 1.3 Relationship to Other Standards

This standard is independent but designed to integrate with:

- **APSS meta (APS-V1-0000)** - Adopted as an APSS standard with SemVer schema
  versioning and additive-only evolution, matching the ecosystem's pattern.
- **Consumer adoption** - Adopted via an `apss.yaml` section. Consumers SHOULD
  take a versioned dependency on the `apss-v1-0004-session-capture` crate rather
  than reimplementing the envelope types, so that divergence from this standard
  is a build failure rather than an undetected drift.

---

## 2. Core Definitions

### 2.1 Session

A **session** is one agent's transcript for one continuous run. A multi-agent
run is NOT a single session: each agent contributes its own session, and the run
is grouped by a shared `metadata.workflow_id`.

### 2.2 Session Envelope

A **session envelope** (or "envelope") is the standardized object defined in
section 4: a thin set of universal header fields plus the opaque `raw` payload.
The envelope is the unit of capture, backup, dedup, and attribution.

### 2.3 Raw Transcript

The **raw transcript** (`raw`) is the provider-native session record, preserved
verbatim inside the envelope. It is opaque to this standard. It is always ground
truth.

### 2.4 Source

A **Source** is a conforming producer that operates in **pull** mode: it reads
provider transcripts from local disk, wraps each into an envelope, and hands
envelopes to an uploader. See section 6.1.

### 2.5 Exporter

An **Exporter** is a conforming producer that operates in **push** mode: it
constructs envelopes and POSTs them to the batch endpoint. See section 6.2.

### 2.6 Store

A **store** is a conforming consumer: it accepts envelopes at the batch endpoint,
sanitizes and persists them, and serves search over them (section 7).

### 2.7 Reconstitutor

A **Reconstitutor** is a conforming consumer-side restorer: it fetches a stored
envelope, writes the transcript back to the path the originating harness expects
on the machine it is running on, and hands off to that harness's native resume.
See section 6.4.

### 2.8 Origin

The **origin** is where an envelope came from: its `host` and `environment`. It
keeps a multi-source corpus attributable.

### 2.9 Sanitization

**Sanitization** is the store's mandatory, unconditional transformation of an
envelope before it is persisted: redaction of detected secrets, plus stripping of
NUL bytes (4.3.2). The output of sanitization is the **stored form**, which is
what the store persists, serves, and reconstitutes from. The stored form is the
store's ground truth; the pre-sanitization bytes are never persisted (5.4).

### 2.10 Captured Content

The **captured content** is the session as received from a producer, before
sanitization. It is what `content_hash` is computed over (4.2.3). It is never
persisted: the store hashes it in memory at ingest and then sanitizes.

### 2.11 Sanitizer Version

The **sanitizer version** identifies the sanitization ruleset a store applied to
produce a given stored form. A store records it so that a ruleset change can be
followed by re-sanitization of the affected stored forms (5.5). It does not affect
`content_hash`, which describes captured content rather than the stored form.

---

## 3. Design Rationale (Informative)

This section records the load-bearing decisions and what each one buys. It is
informative; the normative rules are in sections 4 through 8.

### 3.1 Envelope plus opaque raw, NOT full normalization

Providers have genuinely different shapes: Claude content-blocks, Codex turns,
Cursor bubbles, Gemini parts. Flattening them into one canonical message is
overfitting: it is lossy, and it breaks the moment a provider changes or a new
one appears. Instead the standard is a thin envelope of universal fields wrapped
around the provider's original transcript preserved byte for byte in `raw`.
Pattern precedent: CloudEvents (standard envelope, arbitrary `data`), email
(standard headers, arbitrary body), shipping containers (standard outside,
arbitrary contents). Consequence: a new provider equals wrapping its raw and
filling the header fields, with zero schema change.

### 3.2 Search normalization is server-side and derived, never in the contract

Cross-provider search still matters. But the normalized, searchable view is
built by the SERVER running best-effort, per-provider parsers over `raw`. That
view is allowed to be lossy because `raw` is always ground truth. Consequence:
parsers improve or get added without touching this standard and without
re-uploading anything. Normalization is a swappable read-side concern, not a
write-side promise. This is the direct answer to whether the standard is
overfitting: the client contract commits to nothing provider-specific.

### 3.3 Three conformance profiles: Source, Exporter, Reconstitutor

Different runtimes surface sessions differently, so the standard offers two ways
to *produce* rather than forcing one. Runtimes that write transcripts to disk use
a Source (pull); ephemeral or remote runtimes push envelopes themselves via an
Exporter. Consequence: local CLIs, containers, and workflows all conform without
changing how they natively store transcripts.

A third profile runs the other direction. A Reconstitutor (6.4) restores a stored
session onto a machine so its harness can resume it, which is the exact inverse of
a Source. Splitting production from restoration keeps each profile independently
implementable and independently testable.

### 3.4 Metadata is first-class for search

The highest-value operator query is not full-text over content; it is "show me
sessions matching this facet": a repo, an origin, an agent, a model. Metadata is
therefore promoted to a first-class, queryable surface (section 7). Lexical and
metadata search is the required baseline; semantic search is optional and
cost-sensitive.

### 3.5 Sanitization is server-enforced (mandatory), client best-effort

Push producers (containers, workflows) are effectively untrusted, and their raw
transcripts often contain pasted secrets. The server ALWAYS runs sanitization
before persisting, regardless of source. Clients SHOULD pre-redact but cannot be
relied on. Consequence: a compromised or naive exporter cannot leak secrets into
the shared store.

### 3.6 Idempotency by (session_id, content_hash)

Re-syncs, retries, and overlapping Source-plus-Exporter capture of the same
session must not duplicate. The batch endpoint dedups on
`(session_id, content_hash)`. Consequence: capture can be at-least-once and
aggressive; the store converges.

### 3.7 Origin is first-class

Every envelope carries where it came from, so a multi-source corpus stays
attributable: this MacBook vs the VPS vs a specific workspace vs a workflow run.

### 3.8 Reconstitution is the payoff of verbatim `raw`

Preserving `raw` verbatim (3.1) was chosen to stop the contract rotting as
providers churn. It turns out to buy a second, larger capability for free: if the
captured bytes are the harness's own session file, they can be written back and
the harness can resume from them. Capture on one machine, resume on another.

This requires exactly one thing the envelope does not already carry. `source_format`
tells a *reader* how to parse `raw`; nothing tells a *writer* where to put it
back. Section 6.4 adds that as a third profile, and the shipped reconstitution
registry supplies the per-`source_format` knowledge.

Reconstitution also converts a prose promise into an executable one. Section 4.3
says `raw` MUST be preserved verbatim, but nothing could previously prove a
producer obeyed. A round trip can: capture a session, reconstitute it, and compare
(6.4.4). The MUST becomes a test.

### 3.9 `content_hash` is over the captured content, not the stored form

Sanitization is unconditional (3.5), so what a store persists is by definition not
what the producer captured. The hash must therefore describe one or the other, and
the choice is load-bearing.

This standard hashes the **captured content**, before sanitization, because the
hash's only job is idempotent dedup (3.6) and dedup needs a *stable* identity.
Hashing the sanitized form would make the sanitizer ruleset part of session
identity: every improvement to secret detection would change the digest of
sessions whose content never changed, so re-uploading an untouched session would
read as new. Worse, no migration could repair it, because reconstructing what a
fresh ingest under the new ruleset would produce requires the original bytes, and
3.5 requires those be discarded. The rule would quietly contradict itself.

Hashing before sanitization avoids all of that while giving up nothing: the store
still never persists unsanitized bytes (5.4), it merely hashes them in memory on
the way past. Section 5.5 then becomes a short, honest statement instead of an
impossible migration.

The cost is that `content_hash` is an identity, not an integrity check: it does
not describe the stored bytes and MUST NOT be used to verify a retrieved envelope
was returned unmodified (4.2.3). That is the correct trade, because integrity of
storage is the store's internal concern while dedup is part of the wire contract.

A second consequence: producers do not compute the authoritative hash. Section
4.2.3 fully specifies the input, so a producer *could* compute the same digest,
but the store's value is authoritative and idempotency is resolved store-side.

---

## 4. The Session Envelope

### 4.1 Structure

A session envelope is a JSON object. The header fields (4.2) and `raw` (4.3) are
REQUIRED, with one exception: `content_hash` is populated by the store rather
than the producer, so it is absent in an envelope in flight and present once
stored (4.2.3). `parent_session_id` and `metadata` are OPTIONAL.

The machine-checkable form is
[`schemas/session-envelope.schema.json`](../schemas/session-envelope.schema.json).
It validates both envelope states, so it does not mark `content_hash` `required`.

```jsonc
{
  "scs_version": "1.0",
  "origin":        { "host": "macbook-neural", "environment": "local" },
  "agent":         "ClaudeCode",
  "source_format": "claude-jsonl-v1",
  "session_id":    "019973e4-58a9-7b83-...",
  "parent_session_id": null,
  "started_at":    "2026-05-02T14:03:11Z",
  "last_activity_at": "2026-05-02T15:20:44Z",
  "content_hash":  "sha256:...",
  "metadata": {
    "repo": "owner/name", "git_remote": "https://github.com/owner/name.git",
    "cwd": "/path", "project": "name", "model": "...",
    "tags": [], "message_count": 42, "workflow_id": null
  },
  "raw": { "...": "provider-native transcript, preserved verbatim" }
}
```

### 4.2 Header Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `scs_version` | string (SemVer, `MAJOR.MINOR`) | YES | Version of the envelope schema. Additive-only within a major (section 8). |
| `origin` | object | YES | Where the envelope came from. See 4.2.1. |
| `agent` | string | YES | The producing runtime, for example `ClaudeCode`, `Codex`, `Cursor`, `Gemini`. Any provider-specific string is permitted. |
| `source_format` | string | YES | Provenance tag for `raw`: how a server-side parser should read it, for example `claude-jsonl-v1`. New providers add a value here with zero schema change. |
| `session_id` | string | YES | Identifier for the session, stable per source. Half of the idempotency key (section 5.3). |
| `parent_session_id` | string or null | NO | For a subagent session, the parent session's id. Reserved in v1; derivation is a later phase (section 9). |
| `started_at` | string (RFC3339) | YES | Real session start time, derived from the transcript. See 4.2.2. |
| `last_activity_at` | string (RFC3339) | YES | Real time of the last activity in the session. See 4.2.2. |
| `content_hash` | string (`algo:hexdigest`) | STORE | Hash over the **captured content**, computed by the store at ingest. Absent in flight, present once stored. Never trusted from a producer. The other half of the idempotency key. See 4.2.3. |
| `metadata` | object | NO | Freeform, non-load-bearing, first-class for search (section 7). See 4.4. |
| `raw` | object, array, or string | YES | The provider-native transcript, preserved verbatim. See 4.3. |

#### 4.2.1 Origin

`origin` is REQUIRED and MUST carry:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `host` | string | YES | Human-meaningful host identity, for example `macbook-neural` or the VPS hostname. |
| `environment` | enum | YES | One of `local`, `vps`, `container`, `workflow`. |

New `environment` values MAY be added in a later minor version (section 8);
consumers MUST tolerate unknown values.

#### 4.2.2 Timestamps

`started_at` and `last_activity_at` MUST be real session times in RFC3339,
derived from the transcript. They MUST NOT be the capture or upload time. A
producer that cannot derive real timestamps from a transcript MUST NOT
substitute the current time silently.

#### 4.2.3 Content Hash

An envelope exists in two legitimate states, and they differ in exactly this
field:

| State | When | `content_hash` |
|-------|------|----------------|
| **In flight** | Produced by a Source or Exporter, being uploaded | Absent or null. The producer cannot compute it. |
| **Stored** | Persisted by a store, and what a query or reconstitution returns | REQUIRED and populated by the store. |

The shipped JSON Schema validates both, so it does not mark `content_hash`
required. A store MUST reject a *stored* envelope lacking it, and MUST populate it
during ingest. Consumers reading from a store may rely on its presence.

##### The hash input

`content_hash` identifies the **captured content**: the session as received,
before sanitization. It is computed by the store, at ingest, over exactly this
object:

```json
{
  "raw":           "<the raw value as received>",
  "session_format": "<source_format>",
  "session_id":    "<session_id>"
}
```

Precisely:

- The hash input MUST contain exactly the three members `raw`, `session_format`,
  and `session_id`, taking `session_format` from the envelope's `source_format`.
- The hash input MUST NOT contain `content_hash`. Hashing a structure that
  contains its own digest is not computable, so the field is excluded by
  construction rather than nulled.
- The hash input MUST NOT contain timestamps, `origin`, `agent`, or `metadata`.
  Those describe the capture event rather than the session content, and can differ
  between two captures of the same session. Including them would defeat dedup
  (5.3).
- The hash input MUST be serialized using JSON Canonicalization Scheme
  ([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)) before hashing, so that
  independent stores agree byte for byte. Without a canonical serialization,
  member ordering and number and string escaping vary by implementation and two
  conformant stores would disagree on the digest for identical content.
- The algorithm MUST be SHA-256 in `scs_version` 1.x. It is not merely
  recommended: two stores choosing different algorithms would produce different
  identities for identical content, which is the exact failure canonicalization
  exists to prevent. A future major version MAY introduce another algorithm, and
  the `algo:` prefix exists so that transition is unambiguous.
- The digest MUST be rendered as `sha256:` followed by **lowercase** hexadecimal.
  Casing is normative: `sha256:AB…` and `sha256:ab…` are the same digest but
  different strings, and dedup compares strings.

##### `raw` values that cannot be hashed

RFC 8785 canonicalizes I-JSON ([RFC 7493](https://www.rfc-editor.org/rfc/rfc7493))
input. A `raw` value that is not I-JSON compatible therefore has no canonical
form and no conforming digest. In practice this means:

- Object member names within `raw` MUST be unique. Duplicate names have no
  defined canonical ordering.
- Numbers within `raw` MUST be representable as IEEE-754 doubles. An
  arbitrary-precision literal such as `1e400` has no canonical representation.

A store MUST reject an envelope whose `raw` cannot be canonicalized, reporting it
as `rejected` with a reason (5.3). It MUST NOT coerce the value and hash the
result, which would silently assign an identity that another store would not
reproduce.

This constraint does not apply to a string `raw`, which is the shape required for
any reconstitutable `source_format` (4.3.1) and is always canonicalizable.

##### Why before sanitization

This is the load-bearing choice, and it is the opposite of what it might seem the
security rules imply. Sanitization is still absolute: the store MUST NOT persist
the pre-sanitization bytes (5.4). They are hashed in memory during ingest and then
discarded.

Hashing *after* sanitization would make the sanitizer ruleset part of session
identity. Every sanitizer update would change the digest of sessions whose content
never changed, so re-uploading an untouched session would read as new rather than
duplicate, and no migration could repair it: reconstructing what a fresh ingest
under the new ruleset would have produced requires the original bytes, which
§5.4 has already required be discarded. Hashing the captured content instead makes
the digest permanently stable, and dedup survives every future sanitizer change
(5.5).

##### Store obligations

- The store MUST compute `content_hash` itself, at ingest, before sanitizing.
- The store MUST NOT accept a producer-supplied `content_hash` as authoritative.
  A store that prefers a producer-supplied value cannot know how that value was
  derived, so dedup (5.3) silently degrades. If a producer supplies the field, the
  store MUST recompute and overwrite it.
- Producers SHOULD omit `content_hash`. A producer MAY compute the same digest
  locally for its own bookkeeping, since the input is fully specified above, but
  the store's value is authoritative.

`content_hash` is an identity for deduplication, NOT an integrity check over
stored bytes. It deliberately does not describe the stored form, because the
stored form is sanitized and may be re-sanitized later (5.5). A consumer MUST NOT
use `content_hash` to verify that a retrieved envelope was returned unmodified.

Because producers cannot rely on the store's hash before upload, idempotency is
resolved at the store. Producers MAY re-upload content the store already holds;
the batch endpoint reports `duplicate` per item (5.3), and capture is designed to
be at-least-once (3.6).

### 4.3 The Raw Payload

- `raw` is REQUIRED and MUST preserve the provider-native transcript verbatim. A
  producer MUST NOT flatten, reshape, reorder, or normalize provider message
  structures into `raw`.
- `raw` MAY be inlined for small sessions, or carried as an object-store
  reference for large ones. In both cases the header fields (4.2) are always
  present in the envelope. The inline-versus-reference threshold is an
  implementation choice.
- The standard assigns no meaning to the contents of `raw`. Any parsing of it is
  a server-side concern (section 7).

#### 4.3.1 String `raw` and Reconstitutability

`raw` MAY be a JSON string, object, or array, but the choice has a consequence
that producers MUST understand.

A **string** `raw` carries the transcript's exact bytes. This is the only shape
that supports byte-exact reconstitution (6.4.4), and it is REQUIRED for any
`source_format` that appears in the reconstitution registry (6.4.2).

An **object** or **array** `raw` carries a parsed structure. Parsing is
irreversible with respect to bytes: whitespace, member ordering, and number and
string formatting are not recoverable, so re-serializing does not reproduce the
original file. An envelope with an object or array `raw` is valid, and remains
fully usable for backup, search, and attribution, but it is NOT reconstitutable
and MUST NOT be claimed as conformant to the Reconstitutor profile.

Accordingly:

- A Source for a harness that writes a text transcript (JSON Lines, plain text)
  MUST capture `raw` as a string containing the file's exact bytes.
- A producer MUST NOT parse a line-delimited transcript into an array of objects
  and present that as verbatim `raw`. Doing so silently forfeits resume while
  appearing conformant, which is why 6.4.4 exists as an executable check.
- Object or array `raw` is appropriate only where the provider's native session
  record genuinely is a structured object rather than a file.

#### 4.3.2 The Only Permitted Transformations

"Verbatim" binds producers absolutely: a producer MUST NOT alter `raw` at all.
A store, which MUST sanitize before persisting (5.4), is permitted exactly two
transformations and no others:

1. **Secret redaction.** The store MAY replace detected secret spans with inert
   placeholders. Redaction MUST NOT change the surrounding structure of `raw`.
2. **NUL stripping.** The store MAY strip NUL (`U+0000`) from `raw`. This is
   permitted because common storage engines cannot hold it: PostgreSQL `text` and
   `jsonb` reject NUL outright, and an unstripped NUL surfaces as
   `ERROR: unsupported Unicode escape sequence` at write time, which risks losing
   the session entirely rather than losing one byte of it.

Beyond these two, a store MUST NOT flatten, reshape, reorder, re-encode, or
re-serialize `raw`. Outside of redacted spans and stripped NULs, the stored form
MUST be byte-identical to what the producer sent, and MUST remain retrievable in
that form (6.3.5).

Both transformations produce the stored form (2.9), and both happen AFTER
hashing. `content_hash` is computed over the captured content as received
(4.2.3), so neither redaction nor NUL stripping can move a session's identity.

### 4.4 Metadata

`metadata` is OPTIONAL, freeform, and non-load-bearing: the standard makes no
promise about its contents beyond `message_count` being a hint. It is, however,
the primary search surface (section 7). Producers SHOULD populate the following
well-known fields when available:

| Field | Type | Description |
|-------|------|-------------|
| `repo` | string | Repository slug the session worked in, for example `owner/name`. |
| `git_remote` | string | Git remote URL for the working tree. |
| `cwd` | string | Working directory of the session. |
| `project` | string | Project name the session belongs to. |
| `model` | string | Model identifier used. |
| `tags` | array of string | Free-form operator tags. |
| `message_count` | integer | Hint for the number of messages. |
| `source_path` | string | The transcript's original path on the capturing machine, relative to the harness's session root. A Source SHOULD record it. A Reconstitutor uses it in preference to deriving a path from the registry template (6.4.2), which removes any dependence on re-deriving filename components the harness never documented. |
| `workflow_id` | string or null | For an Exporter session that is part of a multi-agent run, the shared id grouping the run's per-agent envelopes. |

Additional keys are permitted. Consumers MUST tolerate unknown metadata keys.

---

## 5. Transport

### 5.1 Endpoint

Envelopes are uploaded in batches to:

```
POST /v1/sessions/batch
```

The request body MUST be:

```json
{ "envelopes": [ /* one or more session envelopes */ ] }
```

The wrapper object is REQUIRED. A store MUST NOT accept a bare top-level JSON
array as a batch body. The wrapper exists so that batch-level fields can be added
additively (section 8.1); a bare top-level array can never gain one without a
breaking change, and at the standard level that cost is paid by every consumer at
once rather than by one application.

### 5.2 Authentication

- The request MUST carry `Authorization: Bearer <token>`.
- The token MUST be scoped `sessions:write`.
- Tokens SHOULD be per-satellite principals, so `origin` is attributable to a
  credential.

### 5.3 Idempotency

- The endpoint MUST deduplicate on the pair `(session_id, content_hash)`.
- The `content_hash` used for dedup MUST be the store's own, computed at ingest
  over the captured content (4.2.3). A store MUST NOT dedup on a
  producer-supplied hash.
- Re-submitting an envelope with a `(session_id, content_hash)` already present
  MUST NOT create a duplicate.
- The response MUST report a per-item outcome of `accepted`, `duplicate`, or
  `rejected`, and MUST include a reason for `rejected`.

### 5.4 Server-Enforced Sanitization

- The store MUST sanitize every envelope before persisting, regardless of
  source (section 3.5).
- Sanitization MUST run even when the producer claims to have pre-redacted.
- The store MUST NOT persist the pre-sanitization bytes. Sanitization is a
  security floor, not a view: once blobs replicate to object storage and offsite
  backup, a retained unsanitized copy is an unbounded secret-exposure surface.
- Sanitization MUST be confined to the two transformations of 4.3.2.

A future minor version MAY define an opt-in raw-retention profile for stores with
a stronger security posture. Until such a profile exists, retaining unsanitized
bytes is non-conformant.

### 5.5 Sanitizer Evolution

A store's sanitization ruleset will change over time, because secret-detection
patterns improve. This section defines what that costs.

Because `content_hash` is computed over the captured content before sanitization
(4.2.3), a sanitizer change does NOT change any session's digest. Dedup is
therefore unaffected by sanitizer evolution, and re-uploading an unchanged session
still resolves as `duplicate` no matter how many times the ruleset has moved.

A conforming store MUST:

1. Record which sanitizer version produced each stored form, so the set needing
   re-sanitization is identifiable.
2. Re-sanitize stored forms when the ruleset changes, so secrets the previous
   ruleset failed to detect are redacted in already-stored sessions.
3. Keep `content_hash` unchanged when re-sanitizing. The digest describes captured
   content, which re-sanitization does not alter.

Re-sanitization is strictly additive in effect: it can redact spans the old
ruleset missed, but it cannot restore spans the old ruleset already redacted,
because the original bytes were discarded at ingest (5.4). This is the intended
trade. A store therefore MUST NOT treat re-sanitization as reproducing what a
fresh ingest under the new ruleset would have produced; those two results
legitimately differ, and nothing in this standard depends on them being equal.

The dedup key stays `(session_id, content_hash)`. The sanitizer version MUST NOT
be added to it, which would let the same session persist once per ruleset
revision.

---

## 6. Conformance Profiles

A conforming producer implements at least one of the following profiles.

### 6.1 Source (Pull)

A **Source** reads provider transcripts from local disk and produces envelopes.
A conforming Source MUST:

1. Enumerate the provider's local transcripts.
2. For each, build a schema-valid envelope (section 4): wrap the transcript in
   `raw` verbatim (4.3), and fill every REQUIRED header field.
3. Derive real `started_at` and `last_activity_at` from the transcript (4.2.2).
4. Omit `content_hash`, or send it as null. A Source MUST NOT send a hash
   computed over its pre-sanitization bytes and expect it to be honoured; the
   store computes the authoritative hash over the captured content (4.2.3).
5. Hand envelopes to an uploader targeting the batch endpoint (section 5).

A conforming Source SHOULD populate the well-known `metadata` fields (4.4) when
the information is available, and SHOULD pre-redact obvious secrets before
upload (3.5), while relying on the server for authoritative sanitization.

### 6.2 Exporter (Push)

An **Exporter** constructs envelopes and pushes them to the batch endpoint,
for runtimes that are ephemeral or remote. A conforming Exporter MUST:

1. Build schema-valid envelopes (section 4), preserving `raw` verbatim (4.3) and
   filling every REQUIRED header field.
2. Stamp a correct `origin` (4.2.1), including the environment class
   (`container` or `workflow` as appropriate).
3. Derive real timestamps (4.2.2). As with a Source, an Exporter MUST NOT expect
   a self-computed `content_hash` to be honoured (4.2.3).
4. Authenticate with a `sessions:write`-scoped token and POST to
   `/v1/sessions/batch` (section 5), using the `{ "envelopes": [...] }` wrapper
   (5.1).
5. SHOULD pre-redact obvious secrets before push (3.5), while relying on the
   server for authoritative sanitization.

For a multi-agent run, an Exporter MUST emit one envelope PER AGENT and MUST set
the same `metadata.workflow_id` on every envelope from that run.

### 6.3 Store (Consumer)

A conforming store MUST:

1. Accept envelopes at `/v1/sessions/batch` (section 5), enforcing the scoped
   token.
2. Reject envelopes that fail schema validation (section 4), reporting a reason
   (5.3).
3. Deduplicate on `(session_id, content_hash)` (5.3).
4. Sanitize before persisting, unconditionally (5.4).
5. Persist `raw` such that it remains retrievable verbatim.
6. Serve the search baseline of section 7.

### 6.4 Reconstitutor (Restore)

A **Reconstitutor** restores a stored session onto a machine so the originating
harness can resume it natively. It is the inverse of a Source: a Source reads the
harness's session file from disk into an envelope, a Reconstitutor writes it back.

Reconstitution is **same-harness only**. A session captured from Claude Code is
restored for Claude Code. Cross-harness restore is out of scope (1.2).

#### 6.4.1 Requirements

A conforming Reconstitutor MUST:

1. Fetch the stored form of an envelope, unmodified (6.3.5).
2. Resolve the target path, preferring `metadata.source_path` (4.4) when present
   and otherwise deriving it from the reconstitution registry template (6.4.2)
   keyed on the envelope's `source_format`. In both cases the relocation rule
   (6.4.3) applies to the harness-root portion of the path.
3. Write `raw` to that path byte-for-byte as stored. It MUST NOT reshape,
   re-encode, or re-serialize the transcript, and MUST NOT rewrite paths or
   identifiers *inside* the transcript. Only the container path is remapped.
4. Hand off to the harness's native resume, constructed from the registry's resume
   descriptor (6.4.2).

A conforming Reconstitutor MUST NOT execute any command string carried in an
envelope. See 6.4.5.

#### 6.4.2 The Reconstitution Registry

Per-harness restore knowledge lives in a registry shipped with this standard
(`registry/reconstitution.toml`), keyed by `source_format`. Each entry declares:

| Field | Description |
|-------|-------------|
| `path_template` | Where the harness expects the session file, as a function of the working directory and session id, for example `~/.claude/projects/{slug(cwd)}/{session_id}.jsonl`. |
| `slug_rule` | How the harness derives its directory slug from a working directory path, since this is harness-specific and not guessable. |
| `resume` | A **structured descriptor** of how to resume: the program and its argument template, for example program `claude` with args `["--resume", "{session_id}"]`. Never a concatenated shell string. |
| `serialization` | How `raw` is written back to disk, for example `jsonl` or `json`. |

The registry is **informative and separately versioned**. It is not normative spec
text: providers change their on-disk layout on their own schedule, and a registry
entry going stale MUST NOT be treated as this standard changing. Correcting or
adding an entry is not a version bump of the envelope (8.2).

Registry entries describe *where a harness keeps its file*, not what is inside it.
This standard still never parses `raw` (1.2).

#### 6.4.3 The Relocation Rule

The stored `path_template` is a function of the *capturing* machine's working
directory. On a different machine the same repository usually lives somewhere
else, so a Reconstitutor MUST recompute the target path against the local
location of the repository rather than reusing the captured path.

- The Reconstitutor MUST locate the repository locally, using `metadata.repo` and
  `metadata.git_remote` as hints.
- If the repository is absent locally, the Reconstitutor SHOULD offer to clone it
  from `metadata.git_remote`.
- If the target path cannot be resolved, the Reconstitutor MUST fail loudly. It
  MUST NOT write the transcript to a guessed or fallback location, which would
  produce a session the harness cannot find.
- Absolute paths *inside* the transcript MUST be left alone (6.4.1.3). They may be
  wrong on the new machine; that is the harness's business, and rewriting them
  would break byte-fidelity of the stored transcript.

#### 6.4.4 Round-Trip Fitness Function

Reconstitution makes the verbatim-`raw` requirement (4.3) testable rather than
merely asserted. Two round trips are defined.

**Producer round trip (byte-exact, REQUIRED for a Source).** For a Source paired
with a Reconstitutor for the same `source_format`, reconstituting a captured
session MUST reproduce the original session file byte-for-byte:

```
reconstitute(capture(session)) == session
```

This runs entirely producer-side, before the store and therefore before
sanitization, so it is exactly byte-equal. A Source that fails it is reshaping
`raw` and is non-conformant.

**Cross-machine round trip (REQUIRED for a Reconstitutor).** Capture on machine A,
reconstitute on machine B at a different local path, and the harness's native
resume MUST succeed with session context intact.

Equality here is against the **stored form**, not the original bytes: redacted
spans and stripped NULs (4.3.2) are expected and MUST NOT be treated as failures.
A resumed session carrying inert redaction placeholders is a conformant resume; a
resumed agent re-acquires live credentials from its own environment. "Lossless
resume" in this standard means lossless relative to the stored form (2.9).

#### 6.4.5 Security

Resume descriptors come from the standard's registry (6.4.2), never from the
envelope. This is deliberate. Push producers are untrusted (10.3), and an envelope
field carrying a resume command would let a producer specify a command that a
Reconstitutor then executes on the restoring machine, which is remote code
execution by design.

Accordingly:

- A Reconstitutor MUST source resume descriptors only from the registry.
- A Reconstitutor MUST ignore any envelope field purporting to carry a path
  template, resume command, or executable string.
- Resume descriptors MUST be structured (program plus argument list) and MUST be
  invoked without a shell, so that values interpolated from envelope data cannot
  inject additional commands.
- `session_id` is interpolated into resume arguments and into path templates, and
  originates from an untrusted producer. A Reconstitutor MUST validate it against
  the registry entry's `session_id_pattern` before use.

`metadata.source_path` (4.4) is likewise untrusted producer input that is used to
choose a filesystem write location. A Reconstitutor MUST therefore:

- Treat `source_path` as relative to the harness session root, and reject any
  value that is absolute or that escapes that root after normalization. A value
  containing `..` segments, a leading `/`, a drive prefix, or a symlink that
  resolves outside the root MUST be rejected.
- Reject rather than sanitize. A `source_path` that does not resolve safely
  indicates a malformed or hostile envelope, and silently rewriting it to
  something "safe" would restore a session to a location the harness will not
  find (6.4.3).
- Fall back to the registry template when `source_path` is rejected, and record
  that it did.

Without this, a single malicious envelope would let any push producer write
arbitrary file content to an arbitrary path on any machine that reconstitutes it.

### 6.5 Conformance Checklist

An implementation is **conformant** with this standard if it satisfies every item
for each profile it claims.

**Producer (Source or Exporter)**

- [ ] Every emitted envelope is schema-valid (section 4).
- [ ] `raw` is preserved verbatim, never flattened or normalized (4.3).
- [ ] `started_at` and `last_activity_at` are real session times (4.2.2).
- [ ] `origin` is correct and complete (4.2.1).
- [ ] No self-computed `content_hash` is relied upon (4.2.3).
- [ ] Batches use the `{ "envelopes": [...] }` wrapper (5.1).
- [ ] For a Source paired with a Reconstitutor: the byte-exact producer round trip
      passes (6.4.4).

**Store**

- [ ] The batch endpoint enforces the scoped token, requires the wrapper, and
      dedups on its own hash (section 5).
- [ ] The store sanitizes before persisting, unconditionally, and persists no
      pre-sanitization bytes (5.4).
- [ ] Sanitization is confined to redaction and NUL stripping (4.3.2).
- [ ] `content_hash` is computed at ingest over the captured content, using the
      canonical hash input, and producer-supplied hashes are overwritten (4.2.3).
- [ ] Sanitizer version is recorded per stored form, and a ruleset change
      triggers re-sanitization without changing `content_hash` (5.5).
- [ ] `raw` type and reconstitutability are consistent: a `source_format` in the
      reconstitution registry carries a string `raw` (4.3.1).
- [ ] `raw` remains retrievable in its stored form (6.3.5).
- [ ] The store supports the metadata and lexical search baseline (section 7).

**Reconstitutor**

- [ ] Target paths resolve through the registry and the relocation rule
      (6.4.2, 6.4.3).
- [ ] The transcript is written byte-for-byte as stored; nothing inside it is
      rewritten (6.4.1).
- [ ] Unresolvable target paths fail loudly rather than falling back (6.4.3).
- [ ] Resume descriptors come only from the registry, are structured, and are
      invoked without a shell (6.4.5).
- [ ] `session_id` is validated against the registry pattern, and
      `metadata.source_path` is rejected if absolute or escaping the harness root
      (6.4.5).
- [ ] The cross-machine round trip passes (6.4.4).

---

## 7. Search Expectations

### 7.1 Metadata and Lexical Search (Baseline, REQUIRED)

A conforming store MUST support querying and filtering sessions by `metadata`
fields, not only full-text over content. At minimum, a store MUST support
filtering by `origin.host`, `origin.environment`, `agent`, and the well-known
metadata fields it received (for example `repo`, `project`, `model`, `tags`).
Representative queries a store MUST be able to answer include:

- "All sessions from repo X" (`metadata.repo`).
- "Everything from `origin.environment = vps`."
- "Everything by `agent = Codex`."

A conforming store MUST also support lexical (full-text) search over session
content. Because `raw` is opaque to the standard, this search operates over a
server-derived view (7.3), which is allowed to be lossy.

### 7.2 Semantic Search (OPTIONAL)

Semantic and embedding search over sessions is explicitly OPTIONAL and
cost-sensitive. A store MAY offer it. This standard never requires it. Absence of
semantic search MUST NOT be treated as non-conformance.

### 7.3 Server-Derived Normalization

Any normalized, searchable view over session content is built by the store,
best-effort, using per-provider parsers keyed on `source_format` (4.2). This
view:

- MAY be lossy, because `raw` (4.3) is always ground truth.
- MAY be rebuilt, improved, or extended with new parsers at any time, without
  changing this standard and without producers re-uploading anything.
- Is NOT part of the client contract. Producers commit to nothing
  provider-specific; they only promise a valid envelope and a verbatim `raw`.

---

## 8. Versioning and Evolution

### 8.1 Envelope Version

`scs_version` is SemVer. Within a major version, changes MUST be additive only:

- New OPTIONAL header or metadata fields.
- New `source_format` values.
- New `origin.environment` values.

A change that removes a field, renames a field, or narrows a type MUST bump the
major version, and MUST be negotiated at the batch endpoint.

### 8.2 New Providers Never Bump the Version

Adding support for a new provider MUST NOT require a version bump. A new provider
is onboarded by:

1. Choosing a new `source_format` value.
2. Wrapping the provider's transcript in `raw` verbatim.
3. Optionally adding a server-side parser (section 7.3) keyed on that
   `source_format`.
4. Optionally adding a reconstitution registry entry (6.4.2) keyed on that
   `source_format`, if the provider's sessions should be resumable.

Adding, correcting, or removing a reconstitution registry entry is likewise NOT a
version bump of the envelope. The registry is versioned independently (6.4.2).

### 8.3 Consumer Tolerance

Consumers MUST tolerate unknown `agent` strings, unknown `source_format` values,
unknown `origin.environment` values, and unknown `metadata` keys, so that
additive evolution does not break existing stores.

---

## 9. Reserved and Deferred (Informative)

- **`parent_session_id` derivation** - The field is reserved in v1. Deriving the
  parent for a subagent session is a later phase.
- **Large-`raw` referencing** - The inline-versus-object-store threshold for
  `raw` (4.3) is deferred to implementations.
- **Raw-retention profile** - An opt-in profile permitting stores with a stronger
  security posture to retain pre-sanitization bytes (5.4). Not defined here; until
  it exists, retention is non-conformant.
- **Cross-harness reconstitution** - Out of scope by decision, not deferral
  (1.2).
- **Derived-view schema** - A normalized projection over `raw` (turns, tool
  calls, errors, token counts) would let analysis tooling be written once and run
  against any conformant store. It is a candidate for a future substandard, and it
  MUST remain explicitly lossy and non-load-bearing so the envelope contract stays
  as section 7.3 defines it.

---

## 10. Security Considerations

### 10.1 Secrets in Transcripts

Raw transcripts often contain pasted secrets. Producers SHOULD pre-redact, but
the store MUST sanitize unconditionally before persisting (section 5.4). Client
redaction MUST NOT be relied upon as the only defense.

### 10.2 Token Scope

Upload tokens MUST be scoped `sessions:write` and SHOULD be per-satellite
principals so uploads are attributable to a credential (section 5.2). Tokens MUST
NOT be committed to version control.

### 10.3 Untrusted Producers

Push producers (containers, workflows) are effectively untrusted. The store MUST
treat every envelope as untrusted input: validate against the schema, sanitize,
and never execute or interpret `raw` beyond best-effort parsing for search.

---

## 11. Informative: Reference Implementation

The canonical envelope types, their validation, and the reconstitution registry
ship in THIS package as the `apss-v1-0004-session-capture` crate. Consumers depend
on that crate rather than reimplementing the envelope, so drift from this standard
surfaces as a build or test failure (1.3).

The behavioural reference implementation lives in the SeshMagic repository, NOT
here. It comprises:

- Source implementations for Claude, Codex, and Cursor (the pull half).
- A reference exporter and CLI for the push half: batching, retry, auth, and
  origin-stamping.
- The server-side sanitizer and the per-provider parsers that build the
  searchable view (section 7.3).
- The Reconstitutor client (section 6.4): path resolution, relocation, and native
  resume handoff.

Target adopters are SeshMagic (Sources, the reference exporter, the store, and the
Reconstitutor), syntropic137 workflows (Exporter, one envelope per agent grouped
by `workflow_id`), and agentic-primitives `providers/workspaces` (Exporter,
embedded in the workspace image).

---

## 12. References

- [RFC 2119: Key words for use in RFCs](https://datatracker.ietf.org/doc/html/rfc2119)
- [RFC 3339: Date and Time on the Internet: Timestamps](https://datatracker.ietf.org/doc/html/rfc3339)
- [Semantic Versioning](https://semver.org/)
- [CloudEvents Specification](https://cloudevents.io/)

---

*End of Specification*
