import type { ReactNode } from 'react';
import styles from './CenteredColumnLayout.module.css';

/**
 * Centred column on a soft radial wash (screen 1). `width` is a semantic variant, not a
 * pixel choice — the real implementation (Stage 3) sources the actual dimension from
 * `--layout-column-width` / `--layout-column-width-wide` in `tokens.css`, per the
 * design-token rule (`AGENTS.md` UI rules).
 */
export interface CenteredColumnLayoutProps {
  children: ReactNode;
  /** 'default' = 520px, 'wide' = 560px (`ARCHITECTURE.md` §2). */
  width?: 'default' | 'wide';
  className?: string;
}

export function CenteredColumnLayout({
  children,
  width = 'default',
  className = '',
}: CenteredColumnLayoutProps) {
  const widthClass = width === 'wide' ? styles.widthWide : styles.widthDefault;

  return (
    <div className={`${styles.root} ${className}`}>
      <div className={`${styles.column} ${widthClass}`}>{children}</div>
    </div>
  );
}
