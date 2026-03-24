# APS-V1-0001.VIZ01 — Mermaid Diagram Projector (Specification)

**Version**: 0.1.0
**Status**: Promoted
**Parent**: APS-V1-0001 (Code Topology and Coupling Analysis)

---

## 1. Scope

This substandard specifies the **Mermaid Diagram Projector** for generating Mermaid-compatible diagrams from code topology artifacts.

The projector reads `.topology/` artifacts and produces diagrams suitable for embedding in markdown documentation.

## 2. Supported Diagram Styles

| Style | Mermaid Type | Description |
|-------|-------------|-------------|
| `flowchart` | `graph LR/TD` | Dependency flowchart with coupling edges |
| `c4-context` | `C4Context` | C4 system context diagram |
| `c4-container` | `C4Container` | C4 container diagram with metrics |
| `class-diagram` | `classDiagram` | Module structure diagram |

## 3. Output Formats

| Format | Description |
|--------|-------------|
| `mermaid` | Raw Mermaid diagram text |
| `markdown` | Mermaid wrapped in markdown code fence |

## 4. Required Input Artifacts

- `graphs/coupling-matrix.json` — Module coupling coefficients
- `metrics/modules.json` — Module-level metrics (recommended, for styling)

## 5. Configuration

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `style` | string | `"flowchart"` | Diagram style |
| `direction` | string | `"LR"` | Graph direction (LR, TD, etc.) |
| `minCoupling` | number | `0.3` | Minimum coupling strength to render edge |
| `showStrength` | boolean | `true` | Show coupling percentage on edges |
| `theme` | string | `"dark"` | Mermaid theme (default, dark, forest, neutral) |

## 6. Error Codes

| Code | Description |
|------|-------------|
| `TOPOLOGY_NOT_FOUND` | `.topology/` directory does not exist |
| `UNSUPPORTED_FORMAT` | Requested format not supported |
