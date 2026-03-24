/**
 * Example: Register TopologyDependencyGraph for MDX usage in FumaDocs / Next.js.
 *
 * Add this to your mdx-components.tsx (or merge into existing).
 */
import dynamic from 'next/dynamic';

// Lazy-load to avoid SSR issues with canvas/d3-force
const TopologyDependencyGraph = dynamic(
  () =>
    import('@/components/diagrams/topology/TopologyDependencyGraph').then(
      (m) => m.TopologyDependencyGraph,
    ),
  { ssr: false },
);

// Merge into your existing MDX components map:
export function useMDXComponents(components: Record<string, React.ComponentType>) {
  return {
    ...components,
    TopologyDependencyGraph,
  };
}
