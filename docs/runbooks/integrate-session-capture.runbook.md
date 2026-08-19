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
| `content_hash::parse_ijson(text)` | Parse the request body text (`&str`) under I-JSON rules. **Not optional.** See step 3. |
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
apss-v1-0004-session-capture = "2.0"
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

The normative order, which the code below follows:

1. Authenticate; the token must be scoped `sessions:write` (5.2).
2. Decode the body as UTF-8 text.
3. `parse_ijson` on that text, before anything else parses it.
4. Require a non-empty `{ "envelopes": [...] }` wrapper (5.1).
5. Per envelope: schema-validate, deserialize, `validate()`, compute the hash
   *before* sanitizing, dedup, sanitize, **assign the computed hash**, record the
   sanitizer version, `validate_stored()`, persist.
6. Return a per-item `accepted` / `duplicate` / `rejected` outcome, with a reason
   for each rejection (5.3).

```rust
use session_capture::{
    SessionEnvelope, content_hash_for, content_hash::parse_ijson, session_envelope_schema,
};

// `parse_ijson` takes &str, so decode first and reject invalid UTF-8 explicitly
// rather than lossily. `body` here is your framework's byte buffer.
let text = std::str::from_utf8(&body).map_err(|_| reject("body is not valid UTF-8"))?;

// 3a. I-JSON on the ORIGINAL text. This cannot be deferred: a normal JSON parser
//     silently keeps the last of a duplicate member name, so `{"a":1,"a":2}` and
//     `{"a":2}` become indistinguishable and would receive the same
//     content_hash. Once you hold a serde_json::Value the evidence is gone
//     (spec 4.2.3).
let body_value = parse_ijson(text).map_err(|e| reject(&e.to_string()))?;

// 3b. The batch wrapper. A bare array and an empty array are both invalid (5.1).
let envelopes = body_value
    .get("envelopes")
    .and_then(|v| v.as_array())
    .filter(|a| !a.is_empty())
    .ok_or_else(|| reject("body must be a non-empty { \"envelopes\": [...] } (5.1)"))?;

let validator = jsonschema::validator_for(&session_envelope_schema())?;
let mut outcomes = Vec::new();

for value in envelopes {
    // 3c. Schema validation is a store obligation in its own right (6.3.2).
    if validator.validate(value).is_err() {
        outcomes.push(Outcome::rejected("does not satisfy the session-envelope schema"));
        continue;
    }

    let envelope: SessionEnvelope = match serde_json::from_value(value.clone()) {
        Ok(e) => e,
        Err(e) => {
            outcomes.push(Outcome::rejected(&e.to_string()));
            continue;
        }
    };
    if let Err(e) = envelope.validate() {
        outcomes.push(Outcome::rejected(&e.to_string()));
        continue;
    }

    // 3d. Hash BEFORE sanitizing, over the captured content. Any value the
    //     producer sent is ignored by content_hash_for and overwritten below.
    let hash = match content_hash_for(&envelope) {
        Ok(h) => h,
        Err(e) => {
            outcomes.push(Outcome::rejected(&e.to_string()));
            continue;
        }
    };

    // 3e. Dedup on the computed hash, never a supplied one (5.3).
    if store.contains(&envelope.session_id, &hash)? {
        outcomes.push(Outcome::Duplicate);
        continue;
    }

    // 3f. Sanitize, then ASSIGN the computed hash. content_hash_for returns a
    //     String and does not mutate the envelope, so skipping this assignment
    //     leaves a producer-supplied hash in the stored row.
    let mut stored = sanitize(envelope);       // your redaction + NUL strip
    stored.content_hash = Some(hash);
    stored.validate_stored()?;                 // a stored envelope must carry it
    store.persist(&stored, sanitizer_version)?;
    outcomes.push(Outcome::Accepted);
}
```

Note that each fallible call returns a *different* error type (`ContentHashError`,
`serde_json::Error`, `ValidationError`). They all implement `std::error::Error`,
so `?` works under `anyhow` or `Box<dyn Error>`, but a handler with its own error
type needs `From` impls. The loop above deliberately matches instead of using `?`,
because 5.3 requires a per-item outcome rather than failing the whole batch.

The failure modes worth naming, because none of them surfaces as an error:

- **Hashing after sanitizing.** Makes your sanitizer ruleset part of session
  identity, so every secret-detection improvement changes the digest of content
  that never changed and re-uploads stop deduplicating.
- **Computing the hash but not assigning it.** The stored row keeps whatever the
  producer sent. Dedup then compares against a value you did not derive.
- **Trusting a producer's `content_hash`.** You cannot know how it was derived.

**Expected result:** a captured envelope round-trips through ingest, and
`validate_stored()` passes on what you read back.

## 4. Reject the shapes the standard forbids

`POST /v1/sessions/batch` takes a wrapper, not a bare array (spec 5.1):

```json
{ "envelopes": [ /* one or more */ ] }
```

Step 3b already enforces this. If your service currently accepts a bare array,
change the route and the exporter in the same change so nothing is left pointing
at the old shape.

**Expected result:** the wrapper is accepted; a bare array and an empty
`envelopes` array both return an error naming section 5.1.

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
   different field set, every stored digest is wrong under the ratified
   definition, and old and new rows will not deduplicate against each other.

   **You cannot repair this by re-hashing what you hold.** A conforming store
   discarded the pre-sanitization bytes (spec 5.4), so running `content_hash_for`
   over a stored row yields another *post-sanitization* identity, which is
   exactly what the standard rejects. It would look like a fix and silently
   entrench the wrong digest.

   The only correct repair is **re-ingestion from the original transcripts**,
   running them through the step 3 path so the hash is computed over captured
   content. Where those originals still exist on disk (`~/.claude/projects`,
   `~/.codex/sessions`), re-running capture is the migration. Where they do not,
   those rows cannot be brought into conformance: keep them, mark them as
   legacy-hash, and accept that they will not deduplicate against re-captures.
3. **`raw` shape.** If a reader parses a line-delimited transcript into an array
   of objects, it has already forfeited byte-exact resume: re-serializing does
   not reproduce the original bytes. Capture the file's exact bytes as a JSON
   string instead (spec 4.3.1). Check this before building anything on resume.

## Verifying without writing code

The standard ships a CLI for checking envelopes and comparing hash
implementations. Note that `apss-dev` is the APSS repo's own development binary:
depending on the crate does not install it. Run these from a checkout of this
repository, or call the library functions directly from a test in your service.

```bash
cargo run -p aps-cli --bin apss-dev -- run session-capture validate path/to/envelope-or-batch.json
cargo run -p aps-cli --bin apss-dev -- run session-capture hash path/to/envelope.json
```

`hash` prints one tab-separated line per envelope, `session_id<TAB>digest`, so
compare the second field against what your service computes rather than diffing
the whole line. A mismatch means your store will not deduplicate against any
other conformant store, which is otherwise a silent failure.

## Common mistakes

| Mistake | Consequence |
|---|---|
| Implementing "the trait" | There isn't one. You have written a parallel envelope that will drift. |
| Validating after parsing instead of `parse_ijson` on the original text | Duplicate member names go undetected; two different sessions collide on one hash. |
| Hashing after sanitizing | Sanitizer updates silently break dedup, unrecoverably. |
| Computing the hash but never assigning it to the stored envelope | The producer's value survives into storage; dedup compares a digest you did not derive. |
| Re-hashing already-stored rows to "fix" a legacy corpus | Produces another post-sanitization identity. Looks like a fix, entrenches the wrong digest. Re-ingest instead. |
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
