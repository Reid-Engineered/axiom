import type { MasteryState } from '../../types';

/**
 * A concept's mastery ring, name, and status text, reused across WorkspaceOverviewPage's
 * "Concepts in play", ConceptsListPage, ConceptViewPage's Builds on/Leads to, and the
 * Command Palette's Concepts group. The word is always paired with the ring
 * (AXIOM-HANDOFF.md §1) — this component owns that pairing so no page reimplements it.
 */
export interface ConceptRowProps {
  name: string;
  masteryState: MasteryState;
  /** Right-aligned status, e.g. "active", "due for review", "blocks 3 concepts". */
  statusText?: string;
  onSelect?: () => void;
  className?: string;
}

export function ConceptRow(_props: ConceptRowProps) {
  return null;
}
