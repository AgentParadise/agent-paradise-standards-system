# Runbook: Integrate APS-V1-0004 Session Capture Into a Backend Service

## Purpose

Take a service that already stores agent session transcripts, or one you are
about to build, and make it conform to the Session Capture Standard by depending
on the standard's crate instead of hand-rolling the envelope. Written to be
handed to a coding agent or a developer verbatim: every step has a command or a
code change and an expected result.

The point of the dependency is not convenience. It is that a divergence from the
standard becomes a build or test failure rather than a discrepancy someone finds
months later.

## What you are depending on (read this first)

**There is no trait to implement.** This is the most common wrong assumption, and
designing around it produces exactly the drift the standard exists to prevent.

The crate ships *concrete* types and functions:

| Item | What it is |
|---|---|
| `SessionEnvelope`, `Origin`, `Metadata` | The envelope. Use these types directly; do not define your own. |
| `envelope.validate()` | Structural conformance for an envelope in flight. |
| `envelope.validate_stored()` | The above, plus requiring `content_hash`. |
| `envelope.idempotency_key()` | `(session_id, content_hash)`, `None` while in flight. |
| `content_hash::parse_ijson(text)` | Parse request bytes under I-JSON rules. **Not optional.** See step 3. |
| `content_hash_for(&envelope)` | The canonical `content_hash`. |
| `SESSION_ENVELOPE_SCHEMA` / `session_envelope_schema()` | The embedded JSON Schema. |
| `reconstitution::Registry` | Per-harness restore knowledge, for resume. |
| `reconstitution::resolve_within_root()` | Containment-checked path resolution for restore. |

A trait would let you implement a "conforming" envelope of your own shape and
drift again while still compiling. Shared concrete types make that impossible.
The standard deliberately leaves storage engine, query language, and API shape
out of scope (spec section 1.2), so there is nothing a `SessionStore`-style trait
would usefully constrain.

## Prerequisites

- Rust toolchain and Cargo.
- The spec open alongside you:
  [`standards/v1/APS-V1-0004-session-capture/docs/01_spec.md`](../../standards/v1/APS-V1-0004-session-capture/docs/01_spec.md).
  Section numbers below refer to it.

## 1. Add the dependency

Once the crate is published:

```toml
[dependencies]
apss-v1-0004-session-capture = "1.0"
```

Before it is published, pin the git revision so the build is reproducible:

```toml
[dependencies]
apss-v1-0004-session-capture = { git = "https://github.com/AgentParadise/agent-paradise-standards-system", rev = "<commit-sha>" }
```

The library name is `session_capture`, so imports read
`use session_capture::SessionEnvelope;`.

**Expected result:** `cargo build` succeeds.

## 2. Delete your envelope types

Find every local definition of a session envelope and remove it, replacing
references with the crate's types.

```bash
rg -n "struct SessionEnvelope|SCS_VERSION|scs_version" --type rust
```

Do not keep a local copy "for convenience" or wrap the crate's type in your own
struct that re-declares the same fields. Either reintroduces the drift surface.
A newtype that *contains* `SessionEnvelope` is fine; one that mirrors its fields
is not.

**Expected result:** exactly one definition of the envelope in your dependency
graph, and it comes from the crate.

## 3. Fix the ingest path (this is where conformance is won or lost)

Order matters, and two steps are easy to get wrong.

```rust
use session_capture::{SessionEnvelope, content_hash_for, content_hash::parse_ijson};

// 3a. Parse the ORIGINAL request bytes under I-JSON rules.
//     This cannot be deferred: a normal JSON parser silently keeps the last of a
//     duplicate member name, so `{"a":1,"a":2}` and `{"a":2}` become
//     indistinguishable and would receive the same content_hash. By the time you
//     hold a serde_json::Value the evidence is gone (spec 4.2.3).
let value = parse_ijson(&request_body)?;      // reject -> 400 / `rejected` per 5.3

// 3b. Deserialize and validate.
let envelope: SessionEnvelope = serde_json::from_value(value)?;
envelope.validate()?;                          // reject -> `rejected` with a reason

// 3c. Compute the hash BEFORE sanitizing, over the captured content.
//     Never honour a producer-supplied value: overwrite it (spec 4.2.3).
let hash = content_hash_for(&envelope)?;

// 3d. Dedup on (session_id, hash). Only now sanitize, then persist.
//     The pre-sanitization bytes are never written to disk (spec 5.4).
let stored = sanitize(envelope);               // your redaction + NUL strip
persist(stored, &hash, sanitizer_version)?;
```

The two failure modes worth naming:

- **Hashing after sanitizing.** This makes your sanitizer ruleset part of session
  identity, so every secret-detection improvement changes the digest of content
  that never changed, and re-uploads stop deduplicating. Nothing detects it until
  your corpus is full of duplicates.
- **Trusting a producer's `content_hash`.** You cannot know how it was derived.
  Recompute and overwrite, always.

**Expected result:** a captured envelope round-trips through ingest and
`validate_stored()` passes on what you read back.

## 4. Accept the batch shape the standard defines

`POST /v1/sessions/batch` takes a wrapper, not a bare array (spec 5.1):

```json
{ "envelopes": [ /* one or more */ ] }
```

A bare top-level array MUST be rejected. If your service currently accepts one,
change the route and the exporter in the same change so nothing is left pointing
at the old shape. An empty `envelopes` array is also invalid.

**Expected result:** the wrapper is accepted, a bare array returns an error that
names section 5.1.

## 5. Record the sanitizer version

Store which sanitizer ruleset produced each stored row (spec 5.5). When you later
improve secret detection you re-sanitize the affected rows; `content_hash` does
not change, because it describes captured content. Without the recorded version
you cannot target the migration.

**Expected result:** every stored row carries a sanitizer version.

## 6. Add the conformance test that makes drift a build failure

This is the step that gives the whole exercise its value. A test that runs a
*real* stored envelope through the standard's own validator:

```rust
#[test]
fn stored_envelopes_conform_to_aps_v1_0004() {
    let envelope: session_capture::SessionEnvelope =
        serde_json::from_str(&fixture_from_real_corpus()).unwrap();
    envelope
        .validate_stored()
        .expect("stored envelopes must satisfy APS-V1-0004");
}
```

Wire it into CI. When the standard tightens a rule in a later minor, your build
tells you.

**Expected result:** CI fails if your service emits an envelope the standard
rejects.

## 7. Optional: reconstitution (cross-machine resume)

Only if you want resume. Same-harness only; cross-harness is out of scope.

```rust
use session_capture::reconstitution::{Registry, resolve_within_root};

let registry = Registry::shipped();
let entry = registry.entry(&envelope.source_format)?;   // unknown -> fail loudly
entry.validate_session_id(&envelope.session_id)?;       // untrusted input

// metadata.source_path is untrusted and chooses a WRITE LOCATION.
let target = resolve_within_root(harness_root, source_path)?;
```

Non-negotiable, because push producers are untrusted (spec 6.4.5, 10.3):

- Resume descriptors come from the registry, **never** from envelope fields. An
  envelope-carried command executed on the restoring machine is remote code
  execution by design.
- Invoke the resume descriptor as program-plus-args, **without a shell**.
- Resolve `metadata.source_path` with `resolve_within_root`, never by joining it
  onto a root yourself. The lexical check alone cannot see symlinks.
- An unknown `source_format` MUST fail loudly rather than guess a path. Cursor is
  absent from the registry on purpose: it stores sessions in SQLite, so it is
  captured and searchable but not reconstitutable.

**Expected result:** capture on machine A, restore on machine B at a different
local path, and the harness's native resume continues the session.

## Migrating an existing corpus

If you already store envelopes under a pre-ratification shape:

1. **`scs_version`.** Emit `"1.0"`. If existing rows carry another form such as
   `"scs/1"`, add a tolerant reader and backfill, or the new conformance test
   goes red against your own corpus on day one.
2. **`content_hash`.** If you previously hashed post-sanitize, or over a
   different field set, recompute for the whole corpus using `content_hash_for`.
   Until you do, old and new rows will not deduplicate against each other.
3. **`raw` shape.** If a reader parses a line-delimited transcript into an array
   of objects, it has already forfeited byte-exact resume: re-serializing does
   not reproduce the original bytes. Capture the file's exact bytes as a JSON
   string instead (spec 4.3.1). Check this before building anything on resume.

## Verifying without writing code

The standard ships a CLI for checking envelopes and comparing hash
implementations:

```bash
apss-dev run session-capture validate path/to/envelope-or-batch.json
apss-dev run session-capture hash path/to/envelope.json
```

`hash` prints the canonical digest, so you can diff it against what your service
computes. A mismatch means your store will not deduplicate against any other
conformant store, which is otherwise a silent failure.

## Common mistakes

| Mistake | Consequence |
|---|---|
| Implementing "the trait" | There isn't one. You have written a parallel envelope that will drift. |
| Validating after parsing instead of `parse_ijson` on the raw bytes | Duplicate member names go undetected; two different sessions collide on one hash. |
| Hashing after sanitizing | Sanitizer updates silently break dedup, unrecoverably. |
| Keeping a local `SessionEnvelope` alongside the crate's | Reintroduces the exact drift the dependency exists to prevent. |
| Accepting a bare JSON array batch | Non-conformant, and unextendable later without a breaking change. |
| Joining `metadata.source_path` onto a root by hand | Arbitrary file write from an untrusted envelope. |

## References

- Specification: [`docs/01_spec.md`](../../standards/v1/APS-V1-0004-session-capture/docs/01_spec.md)
- Overview: [`docs/00_overview.md`](../../standards/v1/APS-V1-0004-session-capture/docs/00_overview.md)
- JSON Schema: [`schemas/session-envelope.schema.json`](../../standards/v1/APS-V1-0004-session-capture/schemas/session-envelope.schema.json)
- Reconstitution registry: [`registry/reconstitution.toml`](../../standards/v1/APS-V1-0004-session-capture/registry/reconstitution.toml)
- Worked examples: [`examples/`](../../standards/v1/APS-V1-0004-session-capture/examples/)
- Adopting APSS standards generally: [`adopt-apss-standards.runbook.md`](./adopt-apss-standards.runbook.md)
