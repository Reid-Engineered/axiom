import type { ReactNode } from 'react';
import styles from './SessionShell.module.css';

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

export function SessionShell({
  toolbar,
  visualization,
  problem,
  tutor,
  className = '',
}: SessionShellProps) {
  return (
    <div className={`${styles.root} ${className}`}>
      <header className={styles.toolbar}>{toolbar}</header>
      <div className={styles.body}>
        <div className={styles.visualization}>{visualization}</div>
        <div className={styles.lowerRow}>
          <div className={styles.problem}>{problem}</div>
          <aside className={styles.tutor}>{tutor}</aside>
        </div>
      </div>
    </div>
  );
}
