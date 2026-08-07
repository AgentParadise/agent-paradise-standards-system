# Session Capture Standard Examples

This directory demonstrates the two producing conformance profiles of
APS-V1-0004. The third profile, Reconstitutor, consumes stored envelopes rather
than producing them; its per-harness knowledge lives in
[`../registry/reconstitution.toml`](../registry/reconstitution.toml).

## Files

| File | Profile | What it shows |
|------|---------|---------------|
| `claude-source-envelope.json` | Source (pull) | A single valid session envelope produced by reading a Claude Code transcript from local disk. `raw` holds the provider transcript verbatim; the header fields and `metadata` power backup and search. |
| `workflow-exporter-batch.json` | Exporter (push) | The body of `POST /v1/sessions/batch` for one syntropic137 multi-agent run: one envelope per agent, grouped by a shared `metadata.workflow_id`. |

## Validating an envelope against the schema

Every example envelope is valid against
[`../schemas/session-envelope.schema.json`](../schemas/session-envelope.schema.json).
Only the header fields and `raw` are required; `metadata` is optional and
freeform. Any JSON Schema validator works, for example:

```bash
# Using check-jsonschema (pip install check-jsonschema)
check-jsonschema --schemafile ../schemas/session-envelope.schema.json \
  claude-source-envelope.json
```

The crate in this package (`src/lib.rs`) also parses and structurally validates
envelopes; see [`../tests/`](../tests/).

## Key points the examples illustrate

- **`raw` is opaque and verbatim.** The examples never flatten or normalize the
  provider transcript. A Claude envelope and a Codex envelope differ only in
  `source_format` and the shape inside `raw`; the envelope header is identical.
- **Metadata is first-class for search.** `repo`, `origin.environment`, `agent`,
  `model`, and `tags` are what a store filters and queries on. See section 7 of
  the spec.
- **Idempotency.** The batch endpoint dedups on `(session_id, content_hash)`, so
  re-sends are safe. Note that **no example carries `content_hash`**: these model
  envelopes in flight, and producers do not compute the hash. The store derives it
  at ingest over the captured content and overwrites anything a producer sent
  (spec 4.2.3). A test enforces that the examples stay this way.
- **`raw` is a verbatim string.** Both `source_format` values here appear in the
  reconstitution registry, so their transcripts MUST be captured as exact bytes
  rather than parsed into objects (spec 4.3.1). Parsing would forfeit resume.
- **Origin attribution.** The Source example is `origin.environment = local`;
  the Exporter examples are `origin.environment = workflow`.
