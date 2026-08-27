import type { ReactNode } from 'react';

/**
 * Session toolbar (44px) plus a resizable pane grid: visualization (upper, `flex: 1.35`),
 * problem (lower-left, `flex: 1.55`), tutor (lower-right). Sidebar stays visible — a
 * session is a mode of the workspace, not a separate window (screen 5).
 */
export interface SessionShellProps {
  toolbar: ReactNode;
  visualization: ReactNode;
  problem: ReactNode;
  tutor: ReactNode;
  className?: string;
}

export function SessionShell(_props: SessionShellProps) {
  return null;
}
