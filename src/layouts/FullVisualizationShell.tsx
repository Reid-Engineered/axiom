import type { ReactNode } from 'react';
import styles from './FullVisualizationShell.module.css';

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

export function FullVisualizationShell({
  header,
  children,
  className = '',
}: FullVisualizationShellProps) {
  return (
    <div className={`${styles.root} ${className}`}>
      <header className={styles.header}>{header}</header>
      <main className={styles.content}>{children}</main>
    </div>
  );
}
