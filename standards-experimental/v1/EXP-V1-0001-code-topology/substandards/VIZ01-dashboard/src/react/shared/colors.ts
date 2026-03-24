/**
 * VIZ01 — Bounded-context color mapping for topology graphs.
 *
 * Maps keywords found in module IDs to semantic colors. Projects can
 * override or extend these by providing their own mapping.
 */

export const CONTEXT_COLORS: Record<string, string> = {
  orchestration: '#4D80FF',
  workflow: '#4D80FF',
  workspace: '#4D80FF',
  session: '#1A80B3',
  observability: '#1A80B3',
  github: '#8C50DC',
  artifact: '#22cc88',
  'agentic-primitives': '#ff8844',
  'event-sourcing-platform': '#44aaff',
  cost: '#ffcc44',
  token: '#ffcc44',
};

const DEFAULT_COLOR = '#555';

/** Derive a bounded-context color from a module ID string. */
export function getContextColor(
  moduleId: string,
  overrides?: Record<string, string>,
): string {
  const lower = moduleId.toLowerCase();
  const palette = overrides ? { ...CONTEXT_COLORS, ...overrides } : CONTEXT_COLORS;
  for (const [keyword, color] of Object.entries(palette)) {
    if (lower.includes(keyword)) return color;
  }
  return DEFAULT_COLOR;
}
