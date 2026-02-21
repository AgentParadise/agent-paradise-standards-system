/**
 * VIZ01 — Topology data types.
 *
 * These interfaces describe the JSON artifacts produced by a topology analyzer
 * (e.g. EXP-V1-0001 code-topology). They are the props contract consumed by
 * all VIZ01 visualization components.
 */

/* ---------- Input data (from .topology/ artifacts) ---------- */

/** A single dependency edge from dependencies.json */
export interface DependencyEdge {
  from: string;
  to: string;
  weight: number;
}

/** A single module entry from modules.json */
export interface ModuleMetric {
  id: string;
  metrics: {
    lines_of_code: number;
    function_count: number;
    avg_cyclomatic: number;
    martin?: {
      instability?: number;
      abstractness?: number;
    };
  };
}

/* ---------- Derived graph types ---------- */

export interface TopoNode {
  id: string;
  name: string;
  loc: number;
  color: string;
  context: string;
  functionCount: number;
  avgCyclomatic: number;
  instability: number;
}

export interface TopoLink {
  source: string;
  target: string;
  weight: number;
}
