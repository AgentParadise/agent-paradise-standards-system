# Optional workflow inventory profile

Profile identifier: `session-inventory/1`.

This additive profile carries workflow membership and reconstructed relationship
evidence independently of session envelope transport. Existing Source, Exporter,
and Reconstitutor contracts remain unchanged. Implementing this profile does not
permit changing a native session ID, bypassing sanitization, or changing the
content-hash input. Inventory metadata is not transcript content.

## Identity and evidence

`QualifiedRun` identifies a source installation and one execution. A source ID is
stable across restarts and distinct from a hostname, process, or container ID.
`QualifiedTranscript` adds harness and native session ID. These keys are identity,
not authorization. Native IDs retain their original spelling.

The shared `QualifiedTranscript.storage_key` function produces an opaque storage
locator without rewriting the envelope's native `session_id`. Its value is
`qts1:` followed by lowercase SHA-256 hex. The hash input begins with UTF-8
`session-inventory/1:transcript` and one NUL byte, followed in order by source
installation ID, harness, and native session ID. Each component is its exact
UTF-8 bytes prefixed by its byte length as an unsigned 64-bit big-endian integer.
No trimming, case folding, Unicode normalization, or URL decoding is performed.
Stores MUST retain the original tuple and compare it on lookup and conflict;
a hash collision must fail explicitly rather than alias two transcripts. The
locator is neither a content hash nor an authorization credential. Legacy bare
IDs resolve only when unique; ambiguous reads must return a conflict.

## Qualified capture transport

The optional qualified capture endpoint is `POST /v1/transcripts`, with
`source_instance_id`, `harness`, and `native_session_id` query parameters and one
standard session envelope as its JSON body. The native envelope ID MUST match
the qualified identity. Namespace authorization is independent of identity.
Original-content hashing and unconditional server sanitization still follow the
base standard. No inventory publication is implied by accepting a capture.

After durable persistence the endpoint returns `CaptureReceipt`: `storage_key`,
`content_hash`, `stored_content_hash`, and `duplicate`. Clients MUST compare the
key and original-content hash with the submitted capture before acknowledging
their durable work. The stored hash describes the sanitized body, not the
original bytes. A duplicate receipt acknowledges an already persisted revision;
it MUST NOT move the latest pointer back to that revision.

`GET /v1/transcripts` returns the qualified stored envelope with its integrity
hash. An optional `content_hash` selects an exact version, with no fallback to
latest. `/v1/transcripts/raw` returns sanitized bytes with `X-Source-Format` and
`X-Stored-Content-Hash` headers. Stores apply their current read authorization on
every request, independently of inventory visibility.

`InventoryRecord` carries a run, producer ID, immutable record ID, monotonic
producer sequence, and a typed fact. Facts describe nodes, memberships, lineage,
identity bindings, capture receipts, gaps, or explicit corrections. Capture body
availability and relationship confidence remain independent. A missing body must
not erase an expected invocation or an authoritative registration.

Producers MUST call `InventoryRecord::validate` before transmission; receivers
MUST validate before persistence. Receivers MUST authorize the producer's source
namespace separately from validating its identifiers. Retrying a record ID or
producer sequence with different content is a conflict, never an update. Tags
and timestamp proximity do not establish verified parentage.

## Revisions and bounded delivery

`InventoryRevision` declares its qualified run, producer, parent revision,
monotonic revision sequence, record count, evidence sequence high-water mark,
resolver version, and coverage. All referenced transport records belong to that
replication producer; individual facts retain their original evidence provenance.

Receivers may stage revision headers and ordered record references before their
evidence arrives. A manifest batch contains at most 500 references. Publication
requires every declared ordinal and referenced record, within the declared
high-water mark and run, plus the committed parent revision. Publication and the
head update MUST be atomic. An older retry cannot move the head backwards.

Committed revisions remain immutable. Pagination pins a revision and uses bounded
ordinal continuation; absence of that revision cannot silently select a newer
head. Missing transcript bodies do not block inventory publication. Coverage can
remain unknown, open, missing, unsupported, or conflicting after transport has
completed. Without a published inventory, expected coverage is unknown.

## Compatibility

These Rust definitions live in `session_capture::inventory`. Optional `openapi`
provides schema metadata for HTTP implementors. Store adapters and exporters must
consume the shared definitions instead of maintaining independent wire types.
Transport storage, reconstruction algorithms, retry workers, and authorization
policy remain implementation responsibilities.

## Exporter input and language bindings

`InventoryOperation` is the canonical exporter input: an `operation` discriminator
(`record`, `stage`, `manifest`, `publish`) plus its typed `body`. Producers retain
operations durably until acknowledged. A pending publication response is not an
acknowledgement of publication. Records and manifests may arrive before their
prerequisites; successful publication still requires the complete revision.

The Rust crate and the `python/apss_session_capture` package implement the same
profile. Both validate `fixtures/inventory-conformance.json`, including rejected
inputs. The generated `schemas/inventory-operation.schema.json` defines structural
validation; namespace equality, correction authority, archive proof, and manifest
range constraints additionally require semantic validation. This profile does
not replace or alter the existing session-envelope schema.
