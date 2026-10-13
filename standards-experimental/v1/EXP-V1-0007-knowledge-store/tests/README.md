# Knowledge Store Tests

The tests pin the principle list (ids are unique and stable, every principle
has a statement) and check that the example fact carries references only,
each with an opaque, keyed (or ciphertext) address and no key material.

```bash
cargo test -p apss-v1-0007-knowledge-store
```

There is no conformance suite yet; what one would test is open question 8 in
the spec.
