import type { ReactNode } from 'react';
import styles from './TwoPaneLayout.module.css';

/** Main content plus a 250px right rail (used by WorkspaceOverviewPage, ConceptViewPage, ModuleDetailPage). */
export interface TwoPaneLayoutProps {
  children: ReactNode;
  rail: ReactNode;
  className?: string;
}

export function TwoPaneLayout({ children, rail, className = '' }: TwoPaneLayoutProps) {
  return (
    <div className={`${styles.root} ${className}`}>
      <main className={styles.main}>{children}</main>
      <aside className={styles.rail}>{rail}</aside>
    </div>
  );
}
