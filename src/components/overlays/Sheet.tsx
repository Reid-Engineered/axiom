import type { ReactNode } from 'react';
import styles from './Sheet.module.css';

/**
 * A sheet over the dimmed workspace, not a settings page — e.g. Goal Editing's 560px
 * sheet (screen 11). 95% white, hairline border, large soft shadow, always dismissible.
 */
export interface SheetProps {
  open: boolean;
  onClose: () => void;
  eyebrow?: string;
  title?: string;
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
}

export function Sheet({ open, onClose, eyebrow, title, children, footer, className = '' }: SheetProps) {
  if (!open) return null;
  return <div className={styles.backdrop} role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}><section className={`${styles.sheet} ${className}`} role="dialog" aria-modal="true" aria-labelledby="sheet-title"><header>{eyebrow ? <span>{eyebrow}</span> : null}{title ? <h2 id="sheet-title">{title}</h2> : null}<button type="button" onClick={onClose} aria-label="Close sheet">×</button></header><div className={styles.body}>{children}</div>{footer ? <footer>{footer}</footer> : null}</section></div>;
}
