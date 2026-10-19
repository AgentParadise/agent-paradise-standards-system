# EXP-V1-0007: Knowledge Store

**Version:** 0.1.0
**Status:** EXPERIMENTAL. A placeholder, not a standard. Nothing here is
enforced on any repository. Every clause may change.
**Category:** Design

## Terminology

The key words "MUST", "MUST NOT", "SHOULD", "SHOULD NOT" and "MAY" are
interpreted as described in RFC 2119. In this experiment they state the
intended shape, not a conformance obligation; there is no conformance suite.

- **Knowledge item:** the unit of knowledge, any file or note, identified by a
  random id that is neither its content hash nor its path.
- **Fact:** an immutable, past-tense event about one item, appended to the log.
- **Reference:** a pointer `(address, alg, key_scope, kind, length)` to bytes
  held outside the log. The `address` is opaque (see P9); it is never an
  unkeyed digest of plaintext.
- **Read model:** state rebuilt by folding the log; never authoritative.
- **Compartment:** a bounded area of the owner's life that items belong to;
  the unit of isolation.
- **Provenance:** where an item came from (source, path, title), held as
  private referenced content.

## 1. Purpose and boundary

This experiment states the principles of a personal knowledge store that an
agent can read and act on without being handed the owner's private content.
It standardizes the shape of the log, the storage split, the query surface
and the isolation unit. It does not standardize an event schema, a transport,
a storage engine, a query language, or any consumer's domain model.

## 2. Principles

### P1. Facts carry references, never content

A fact MUST NOT carry the owner's content: no note body, no title, no source
path, no free text derived from any of them. A fact MAY carry a reference to
content held in the object store. The reason is structural: an append-only
log cannot un-write a payload, so content in the log defeats deletion, access
control and safe agent access at once.

Metadata that does enter a fact SHOULD be allowlisted by shape (a count, a
media type, a fixed enumeration), never by trust in the caller. An unkeyed
hash of the owner's text is content for this purpose: it is a membership
oracle, and MUST NOT appear in a fact (see P9).

### P2. Provenance is private referenced content

Where an item came from (source corpus, path, title) MUST be held as a
referenced blob, not in a fact. That blob MUST carry a fresh random nonce so
its hash cannot be confirmed by hashing a guessed record. A re-ingest MAY use a
rebuildable index from provenance to item id to recognize material it has
already seen.

> **Rationale.**
> - Paths and titles are low-entropy: a guessed `(source, path, title)` record is easy to enumerate, so without a nonce even a keyed address (P9) is confirmable by anyone who later obtains the key.
> - Holding provenance as a blob means P3 deletion removes it too; in a fact it could never be erased.

### P3. Deletion is real

An implementation MUST be able to delete an item's bytes. Deleting a blob
MUST leave the log intact: the fact and its reference remain, the bytes are
gone, and any read of the reference reports "content erased" rather than an
integrity failure. A missing provenance record MUST read as "origin unknown",
never as an error that blocks other items. Deletion MUST NOT rewrite or remove
a fact, so that a hash chain over the log stays verifiable.

> **Rationale.**
> - Real deletion and an immutable log are compatible only because facts hold references (P1): erase the bytes, keep the fact.
> - Rewriting a fact would break the hash chain (P8) and every read model rebuilt from it (P4).

### P4. Read models are rebuildable

Every read model MUST be derivable by folding the log (plus, where needed,
the private blobs it references). A read model MUST NOT be the source of
truth. An implementation SHOULD prove this by rebuilding from the log plus
the referenced blobs still present, and comparing with live state. A blob
erased under P3 folds as "content erased" (or "origin unknown" for
provenance), never as a rebuild failure.

### P5. One stream per aggregate, and the item is the aggregate

Each aggregate instance MUST have its own event stream, keyed by the entity
id, and that stream MUST be the consistency boundary: a command decides
against one aggregate's state and appends to that one stream at an expected
position, so concurrent writers to the same aggregate get exactly one winner.

In a conforming knowledge store, the aggregate is the knowledge item, so each
item has its own stream. Its invariants (status, content reference, its own
relations) are the consistency boundary. Rules that span items, such as
uniqueness within a compartment or one blob shared by several items, MUST NOT
be solved by widening the aggregate. A compartment or a corpus MUST NOT be a
single stream that every item serializes through.

> **Rationale.**
> - The item is the aggregate: its invariants need only its own history; stream id = entity id (Dilger, *Understanding Eventsourcing*, pp. 17-18, 108; Vernon, *Implementing Domain-Driven Design*, pp. 355-362, 539, 562).
> - Not a compartment or corpus stream: every write would serialize through one expected position, and the stream would grow far past the 10-100 events a stream normally holds (Dilger, p. 121).
> - Cross-item rules use a Reservation aggregate keyed by the scarce value (Dilger, pp. 514-516) or a read-model check, never a bigger aggregate.
> - Streams are cheap, so many small ones are normal: a stream is a name over the append-only store (Vernon, p. 562), and one stream per product is the book's own inventory design (Dilger, p. 370).

### P6. The query surface is read models, never raw storage

Agents and tools MUST query read models (an API, a CLI) and MUST NOT be given
direct access to the log's storage or the object store. A read model returns
references; resolving a reference is a separate, authorizable hop through the
object store port. An implementation SHOULD ship agent-facing documentation
(an `AGENTS.md`) for the query surface.

> **Rationale.**
> - Raw storage access bypasses the one hop (resolving a reference) where authorization, audit and refusal live.
> - Read models are the stable contract; storage layout is not, and may become per-compartment and encrypted (P7, P9).

### P7. The Compartment is the isolation unit

An item MUST belong to exactly one compartment. A reader of one compartment
MUST learn nothing about another from the log it can see. Implementations
SHOULD keep per-compartment keys, per-compartment stores and per-compartment
network isolation possible by never letting an item, a stream, or a read
model span compartments, and by never deduplicating blobs across compartments
(see section 4).

### P8. The log is built to be hash-chained

An implementation SHOULD serialize facts deterministically, append at an
expected position, and derive idempotency from a command id rather than the
payload, so that a hash chain over the log (with the content covered through
the references' addresses) and externally anchored, signed chain heads can be
added without changing a fact. An implementation MAY sign such a chain; it is
the log's tamper evidence (see P9).

### P9. The content address is opaque

A reference's `address` MUST be opaque to anyone without the isolation unit's
key. A conforming store MUST NOT write an unkeyed digest of plaintext to the
log, in a reference or anywhere else. Two forms are allowed:

- **Keyed MAC (RECOMMENDED):** a MAC over the plaintext under a key scoped to
  one isolation unit (a compartment), e.g. HMAC-SHA-256.
- **Digest over ciphertext:** a plain digest of the encrypted bytes as stored.

A reference MUST name its algorithm (`alg`) and its key scope (`key_scope`),
and MUST NOT carry the key or anything derived from it other than the address
itself.

The object store MUST check integrity on read by recomputing the address (or
the ciphertext digest) and comparing. A hash chain over the facts MAY be
signed (P8); it, not a plaintext digest, is the log's tamper evidence.

> **Rationale.**
> - An unkeyed plaintext digest in an append-only log is a membership oracle: anyone holding the log and a candidate file can confirm the owner has it, and short or templated notes can be guessed outright.
> - The log cannot un-write it, so deleting the bytes (P3) would not remove the oracle.
> - Verifiability is kept: key holders recompute the address, the object store checks integrity on read, and the (optionally signed) hash chain gives tamper evidence (P8).

## 3. Normative artifact

There is none yet. The `examples/minimal/knowledge-fact.json` file shows one
fact that satisfies P1, P2 and P9: it carries two references (content and
provenance), a compartment id, and nothing derived from the owner's text. It
is an illustration, not a schema, but the reference fields it names are the
ones P9 requires:

| Field | Meaning |
|---|---|
| `address` | Opaque content address, prefixed by `alg` (e.g. `hmac-sha256:<hex>`). |
| `alg` | `hmac-sha256` (keyed MAC, RECOMMENDED) or `sha256-ciphertext`. |
| `key_scope` | The isolation unit whose key produced the address, e.g. `compartment`. Never the key. |
| `kind` | Media type of the content. |
| `length` | Byte length of the stored blob. |

## 4. Security note: no deduplication across isolation units

With per-unit keys (P9), identical bytes in two compartments get different
addresses and are stored twice. This is intended: deduplicating across units
would itself reveal that two compartments hold the same content, which P7
forbids. Deduplication within one unit is unaffected.

## 5. Open questions

These are open on the reference implementation too. A ruling on each is a
precondition for promotion.

1. **Timestamp precision.** Second-precision capture times in facts
   reconstruct the owner's timeline. Keep, coarsen, or move to provenance?
2. **What a deleted item leaves behind.** Counts, sizes, kinds and
   timestamps stay in the log after the bytes are gone. Is a second,
   metadata-stripping mechanism needed, and how does it coexist with P3's
   "never remove a fact"?
3. **Tenant keying.** One tenant per bounded context, one per compartment,
   or one per owner with a family registry? The reference implementation runs
   one tenant per context for its MVP and defers the rest to an experiment.
4. **Blob deduplication across compartments.** Ruled by P9: per-compartment
   keys mean no deduplication across compartments, from the start.
5. **Where placement lives.** Spatial or presentational placement of an item
   (which wall, which screen) is state of the space, not of the knowledge; the
   reference implementation moved it to a separate bounded context. Is that
   split part of this standard or an implementation choice?
6. **The content-address under encryption.** Ruled by P9: never an unkeyed
   plaintext digest; a keyed MAC per isolation unit (RECOMMENDED) or a digest
   over ciphertext. Still open: key rotation, and whether a rotated key
   re-addresses existing blobs or only new ones.
7. **Schema evolution.** The reference implementation uses one schema
   version per event family and additive variants, with no upcasters. Is an
   upcasting seam required before this is a standard?
8. **What a conformance suite would test.** A leak scan over every fact kind,
   a rebuild-equals-live proof, a deletion-leaves-chain-intact proof, and a
   one-stream-per-item concurrency proof are the candidates.

## 6. Reference implementation

DreamShip's Knowledge Core (`contexts/knowledge/` in `dream-ship_v0`), over the
Syntropic137 Event Sourcing Platform. Its ADR-0042 and its threat model
(`docs/design/knowledge-core-threat-model.md`) are the source of every
principle above, and its open questions are the ones listed here.
