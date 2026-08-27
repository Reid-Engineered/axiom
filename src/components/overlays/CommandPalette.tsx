import type { ReactNode } from 'react';

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

export function CommandPalette(_props: CommandPaletteProps) {
  return null;
}
