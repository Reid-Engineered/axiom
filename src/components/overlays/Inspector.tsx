import type { ReactNode } from 'react';
import styles from './Inspector.module.css';

/**
 * Right-side inspector panel — e.g. Full Visualization's Selected shell inspector.
 * Appears only on selection and is dismissible (screen 6).
 */
export interface InspectorProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
  className?: string;
}

export function Inspector({ open, onClose, title, children, className = '' }: InspectorProps) {
  if (!open) return null;
  return (
    <section
      className={`${styles.inspector} ${className}`}
      role="complementary"
      aria-labelledby="inspector-title"
    >
      <header>
        <h2 id="inspector-title">{title}</h2>
        <button type="button" onClick={onClose} aria-label={`Close ${title}`}>
          ×
        </button>
      </header>
      <div className={styles.body}>{children}</div>
    </section>
  );
}
