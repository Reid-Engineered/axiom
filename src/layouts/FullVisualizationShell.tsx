import type { ReactNode } from 'react';

/**
 * Full-bleed, no sidebar. `header` holds "‹ Session", the scene name, and actions
 * (Save to notes, Share, Inspector). Sidebar is hidden only in this mode, which offers
 * a single "‹ Session" return (AXIOM-HANDOFF.md §3, screen 6).
 */
export interface FullVisualizationShellProps {
  header: ReactNode;
  children: ReactNode;
  className?: string;
}

export function FullVisualizationShell(_props: FullVisualizationShellProps) {
  return null;
}
