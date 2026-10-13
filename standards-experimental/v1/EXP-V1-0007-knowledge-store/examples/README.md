# Knowledge Store Examples

`minimal/knowledge-fact.json` is one fact that satisfies principles P1, P2
and P9: a random item id, a compartment id, a content reference and a
provenance reference. Each reference's `address` is a keyed MAC under the
compartment's key (`alg`, `key_scope`), never an unkeyed digest of plaintext;
the key itself never appears. Nothing in it is derived from the owner's text. The provenance
blob (not shown) holds the source path and title behind a random nonce.

It is an illustration of the shape, not a schema. This experiment defines no
wire format.
