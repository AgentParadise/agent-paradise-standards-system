# Expect: RECIPE_INVALID_TOOL_MANIFEST

`tools/broken/tool.yaml` parses (well-formed YAML) but has an empty `name`
and an empty `command`.
