# Python inventory contract

`apss-session-capture` provides typed, immutable Pydantic models for the optional
APS-V1-0004 workflow inventory profile. It contains no store, inference, network,
or harness implementation. Rust and Python consume the same accepted/rejected
fixtures in `../fixtures/inventory-conformance.json`.

Validate exporter input with `apss_session_capture.inventory.operation_adapter`.
`validate_json` accepts one operation JSON document; `model_dump_json` serializes
its typed result. Unknown fields, cross-source references, invalid bindings,
forged retractions, unsupported variants, and numeric/array bounds are rejected.

From this directory, run:

```sh
python -m unittest discover -s tests -v
python -m apss_session_capture.schema > ../schemas/inventory-operation.schema.json
python check_package.py
```

The schema captures structural constraints. Cross-field constraints such as
namespace equality and manifest bounds require model validation too. The schema
freshness test fails if committed generated output differs from the models.

`check_package.py` builds an sdist and wheel, verifies version alignment and the
typing marker, installs into a temporary virtual environment, then runs fixtures
against the installed package with isolated imports. It requires package-index
access for build and runtime dependencies. Repository `just check` includes it;
CI exercises Python 3.11 and 3.14.

Version 2.1.0 is in development and has not been published. Coordinated consumer
pins and release automation remain required before shipping this profile.

`CaptureReceipt` is the shared qualified-capture acknowledgement model. Call
`validates_capture(identity, original_content_hash)` before retiring durable
delivery work. `stored_content_hash` identifies the sanitized representation and
can differ from the original hash. A receipt proves historical acceptance, not
current read authorization or availability. Rust and Python check the same
acceptance and rejection vectors in `../fixtures/capture-receipts.json`.
