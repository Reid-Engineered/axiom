import { useEffect, type ReactNode } from 'react';

import styles from './CommandPalette.module.css';

export interface CommandPaletteResultItem {
  id: string;
  /**
   * The row's rendered content — plain text plus a consequence detail for Actions/notes,
   * or a fully composed component for Concepts/marketplace results (screen 12: "inline
   * mastery glyphs and status", "carrying a Community badge"). Task 035 reuses `ConceptRow`
   * and `TrustBadge` directly here rather than this overlay re-deriving their rendering —
   * a ReactNode, not parallel label/detail/leading fields that would duplicate what those
   * components already render internally.
   */
  content: ReactNode;
  shortcut?: string;
  onSelect: () => void;
}

export interface CommandPaletteResultGroup {
  /** "Actions", "Concepts", "From your work", a marketplace result group. */
  label: string;
  items: CommandPaletteResultItem[];
}

/**
 * 600px overlay, 96px from the top, over dimmed content (screen 12). Empty `groups` is
 * acceptable here — this is the contract only; real results are wired in Stage 6 (035).
 */
export interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
  query: string;
  onQueryChange: (query: string) => void;
  groups: CommandPaletteResultGroup[];
  /** Workspace scope badge, top-right. */
  scopeLabel?: string;
  className?: string;
}

export function CommandPalette({
  open,
  onClose,
  query,
  onQueryChange,
  groups,
  scopeLabel,
  className = '',
}: CommandPaletteProps) {
  useEffect(() => {
    if (!open) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose, open]);

  if (!open) {
    return null;
  }

  return (
    <div className={styles.backdrop} role="presentation" onMouseDown={onClose}>
      <section
        className={`${styles.palette} ${className}`}
        role="dialog"
        aria-modal="true"
        aria-label="Command palette"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className={styles.searchRow}>
          <span className={styles.searchIcon} aria-hidden="true">
            ⌕
          </span>
          <input
            autoFocus
            className={styles.input}
            value={query}
            onChange={(event) => onQueryChange(event.target.value)}
            aria-label="Search commands"
          />
          {scopeLabel ? <span className={styles.scope}>{scopeLabel}</span> : null}
        </div>
        <div className={styles.results}>
          {groups.map((group) => (
            <section key={group.label} className={styles.group}>
              <div className={styles.groupLabel}>{group.label}</div>
              {group.items.map((item) => (
                <button
                  type="button"
                  className={styles.result}
                  key={item.id}
                  onClick={item.onSelect}
                >
                  <span>{item.content}</span>
                  {item.shortcut ? <span className={styles.shortcut}>{item.shortcut}</span> : null}
                </button>
              ))}
            </section>
          ))}
        </div>
        <footer className={styles.footer}>↑↓ move · ⏎ run · ⇥ scope · esc close</footer>
      </section>
    </div>
  );
}

export function CommandPaletteText({ label, detail }: { label: string; detail?: string }) {
  return (
    <span className={styles.resultText}>
      <span>{label}</span>
      {detail ? <span className={styles.resultDetail}>{detail}</span> : null}
    </span>
  );
}

export function CommandPaletteMarketplaceResult({
  label,
  badge,
}: {
  label: string;
  badge: ReactNode;
}) {
  return (
    <span className={styles.marketplaceResult}>
      <span>{label}</span>
      {badge}
    </span>
  );
}
