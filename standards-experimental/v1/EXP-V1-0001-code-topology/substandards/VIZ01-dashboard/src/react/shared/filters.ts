/**
 * VIZ01 — Topology data filtering utilities.
 *
 * Provides reusable filters for excluding worktree modules,
 * small modules (< N LOC), and building the derived graph nodes/links.
 */

import type { DependencyEdge, ModuleMetric, TopoNode, TopoLink } from './types';
import { CONTEXT_COLORS, getContextColor } from './colors';

/* ---------- Helpers ---------- */

function shortName(id: string): string {
  const parts = id.replace(/::/g, '.').split('.');
  return parts[parts.length - 1] || id;
}

// Derived from CONTEXT_COLORS keys, sorted longest-first for greedy matching
const CONTEXT_KEYWORDS = Object.keys(CONTEXT_COLORS).sort((a, b) => b.length - a.length);

function inferContext(id: string): string {
  const lower = id.toLowerCase();
  for (const c of CONTEXT_KEYWORDS) {
    if (lower.includes(c)) return c;
  }
  return 'other';
}

/* ---------- Public API ---------- */

export interface FilterOptions {
  /** Exclude modules whose ID starts with these prefixes (default: ['worktrees.', 'worktrees::']) */
  excludePrefixes?: string[];
  /** Minimum lines of code to include a module (default: 100) */
  minLoc?: number;
  /** Optional color overrides passed to getContextColor */
  colorOverrides?: Record<string, string>;
}

/**
 * Build derived graph data from raw topology artifacts.
 *
 * Filters out worktree modules and small modules, then returns
 * nodes and links ready for a force simulation.
 */
export function buildTopologyGraph(
  modules: ModuleMetric[],
  edges: DependencyEdge[],
  options: FilterOptions = {},
): { nodes: TopoNode[]; links: TopoLink[] } {
  const {
    excludePrefixes = ['worktrees.', 'worktrees::'],
    minLoc = 100,
    colorOverrides,
  } = options;

  const moduleMap = new Map<string, ModuleMetric>();
  for (const m of modules) {
    if (excludePrefixes.some((p) => m.id.startsWith(p))) continue;
    if (m.metrics.lines_of_code < minLoc) continue;
    moduleMap.set(m.id, m);
  }

  const nodes: TopoNode[] = [];
  const nodeIds = new Set<string>();
  for (const [id, m] of moduleMap) {
    nodeIds.add(id);
    nodes.push({
      id,
      name: shortName(id),
      loc: m.metrics.lines_of_code,
      color: getContextColor(id, colorOverrides),
      context: inferContext(id),
      functionCount: m.metrics.function_count,
      avgCyclomatic: m.metrics.avg_cyclomatic,
      instability: m.metrics.martin?.instability ?? 0.5,
    });
  }

  const links: TopoLink[] = [];
  for (const e of edges) {
    if (nodeIds.has(e.from) && nodeIds.has(e.to)) {
      links.push({ source: e.from, target: e.to, weight: e.weight });
    }
  }

  return { nodes, links };
}
