# VIZ01-dashboard — TopologyDependencyGraph

A generic, reusable React component that renders an interactive force-directed dependency graph from `.topology/` artifacts.

## Features

- **Canvas-based** force-directed graph powered by `d3-force`
- **Pan & zoom** with pointer/scroll interaction
- **Hover tooltips** showing module metrics (LOC, cyclomatic complexity, instability)
- **Bounded-context coloring** — automatic color mapping by module ID keywords
- **Filtering** — excludes worktree modules and small modules (< 100 LOC by default)
- **Fully self-contained** — copy into any React/Next.js project

## Installation

Copy `src/react/` into your project. Requires peer dependencies:

```bash
npm install react d3-force
npm install -D @types/d3-force   # if using TypeScript
```

## Usage

```tsx
import { TopologyDependencyGraph } from './path-to/VIZ01-dashboard/src/react';
import depData from './.topology/graphs/dependencies.json';
import modData from './.topology/metrics/modules.json';

<TopologyDependencyGraph
  dependencies={depData.edges}
  modules={modData.modules}
  height={600}
/>
```

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `dependencies` | `DependencyEdge[]` | **required** | Edges from `dependencies.json` |
| `modules` | `ModuleMetric[]` | **required** | Module entries from `modules.json` |
| `height` | `number` | `600` | Container height in px |
| `className` | `string` | — | CSS class on wrapper div |
| `filterOptions` | `FilterOptions` | `{}` | Min LOC, exclude prefixes, color overrides |
| `legendItems` | `[label, color][]` | built-in | Custom legend entries |

## Data Format

### `DependencyEdge`
```ts
{ from: string; to: string; weight: number }
```

### `ModuleMetric`
```ts
{
  id: string;
  metrics: {
    lines_of_code: number;
    function_count: number;
    avg_cyclomatic: number;
    martin?: { instability?: number };
  };
}
```

## MDX Integration

See `examples/nextjs-mdx/` for a complete example of using this component in FumaDocs/Next.js MDX pages.

## Source Standard

Part of [EXP-V1-0001 Code Topology](../../) in the [Agent Paradise Standards System](https://github.com/AgentParadise/agent-paradise-standards-system).
