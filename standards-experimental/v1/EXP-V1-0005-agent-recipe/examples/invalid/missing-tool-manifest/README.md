# Expect: RECIPE_MISSING_TOOL_MANIFEST

`tools/extract_citations/` is a direct child of `tools/` with no
`tool.yaml`. Every tool package MUST carry a conforming manifest
(section 5.2); skipping one in silence would certify a structurally
incomplete tool package.
