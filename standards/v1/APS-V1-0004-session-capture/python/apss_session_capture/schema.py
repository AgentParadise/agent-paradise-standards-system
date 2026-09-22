"""Generate the structural schema. Semantic constraints also require model validation."""

import json
from .inventory import operation_adapter


def inventory_schema() -> str:
    schema = operation_adapter.json_schema()
    schema["$schema"] = "https://json-schema.org/draft/2020-12/schema"
    schema["$id"] = "urn:apss:session-inventory:1:operation"
    return json.dumps(schema, ensure_ascii=False, indent=2, sort_keys=True) + "\n"


if __name__ == "__main__":
    print(inventory_schema(), end="")
