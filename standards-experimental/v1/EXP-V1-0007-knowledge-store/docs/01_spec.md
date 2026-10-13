# EXP-V1-0007: Knowledge Store

**Version:** 0.1.0
**Status:** EXPERIMENTAL. A placeholder, not a standard. Nothing here is
enforced on any repository. Every clause may change.
**Category:** Technical

## Terminology

The key words "MUST", "MUST NOT", "SHOULD", "SHOULD NOT" and "MAY" are
interpreted as described in RFC 2119. In this experiment they state the
intended shape, not a conformance obligation; there is no conformance suite.

- **Knowledge item:** the unit of knowledge, any file or note, identified by a
  random id that is neither its content hash nor its path.
- **Fact:** an immutable, past-tense event about one item, appended to the log.
- **Reference:** a content-addressed pointer `(hash, kind, length)` to bytes
  held outside the log.
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
oracle, and MUST NOT appear in a fact.

### P2. Provenance is private referenced content

Where an item came from (source corpus, path, title) MUST be held as a
referenced blob, not in a fact. That blob MUST carry a fresh random nonce so
its hash cannot be confirmed by hashing a guessed record. A re-ingest MAY use a
rebuildable index from provenance to item id to recognize material it has
already seen.

### P3. Deletion is real

An implementation MUST be able to delete an item's bytes. Deleting a blob
MUST leave the log intact: the fact and its reference remain, the bytes are
gone, and any read of the reference reports "content erased" rather than an
integrity failure. A missing provenance record MUST read as "origin unknown",
never as an error that blocks other items. Deletion MUST NOT rewrite or remove
a fact, so that a hash chain over the log stays verifiable.

### P4. Read models are rebuildable

Every read model MUST be derivable by folding the log (plus, where needed,
the private blobs it references). A read model MUST NOT be the source of
truth. An implementation SHOULD prove this by rebuilding from the log alone
and comparing with live state.

### P5. One stream per item

Each item MUST have its own event stream, and that stream MUST be the
consistency boundary: a command decides against one item's state and appends
to one item's stream at an expected position, so concurrent writers to the
same item get exactly one winner. A corpus or a compartment MUST NOT be a
single stream that every item serializes through.

### P6. The query surface is read models, never raw storage

Agents and tools MUST query read models (an API, a CLI) and MUST NOT be given
direct access to the log's storage or the object store. A read model returns
references; resolving a reference is a separate, authorizable hop through the
object store port. An implementation SHOULD ship agent-facing documentation
(an `AGENTS.md`) for the query surface.

### P7. The Compartment is the isolation unit

An item MUST belong to exactly one compartment. A reader of one compartment
MUST learn nothing about another from the log it can see. Implementations
SHOULD keep per-compartment keys, per-compartment stores and per-compartment
network isolation possible by never letting an item, a stream, or a read
model span compartments, and by never deduplicating blobs across compartments
once keys exist.

### P8. The log is built to be hash-chained

An implementation SHOULD serialize facts deterministically, append at an
expected position, and derive idempotency from a command id rather than the
payload, so that a hash chain over the log (with the content covered through
the references' hashes) and externally anchored, signed chain heads can be
added without changing a fact.

## 3. Normative artifact

There is none yet. The `examples/minimal/knowledge-fact.json` file shows one
fact that satisfies P1 and P2: it carries two references (content and
provenance), a compartment id, and nothing derived from the owner's text. It
is an illustration, not a schema.

## 4. Open questions

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
4. **Blob deduplication across compartments.** Content addressing
   deduplicates identical bytes; across compartments that leaks equality.
   Namespace blobs per compartment from the start, or only once keys exist?
5. **Where placement lives.** Spatial or presentational placement of an item
   (which wall, which screen) is state of the space, not of the knowledge; the
   reference implementation moved it to a separate bounded context. Is that
   split part of this standard or an implementation choice?
6. **The content-address under encryption.** Once blobs are encrypted at
   rest, is the reference's hash over plaintext (an oracle), over ciphertext,
   or a keyed hash?
7. **Schema evolution.** The reference implementation uses one schema
   version per event family and additive variants, with no upcasters. Is an
   upcasting seam required before this is a standard?
8. **What a conformance suite would test.** A leak scan over every fact kind,
   a rebuild-equals-live proof, a deletion-leaves-chain-intact proof, and a
   one-stream-per-item concurrency proof are the candidates.

## 5. Reference implementation

DreamShip's Knowledge Core (`contexts/knowledge/` in `dream-ship_v0`), over the
Syntropic137 Event Sourcing Platform. Its ADR-0042 and its threat model
(`docs/design/knowledge-core-threat-model.md`) are the source of every
principle above, and its open questions are the ones listed here.
