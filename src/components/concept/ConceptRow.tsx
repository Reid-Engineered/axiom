import type { MasteryState } from '../../types';
import { Mastery } from '../mastery/Mastery';
import styles from './ConceptRow.module.css';

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

export function ConceptRow({ name, masteryState, statusText, onSelect, className = '' }: ConceptRowProps) {
  const content = (
    <>
      <span className={styles.name}>{name}</span>
      {statusText ? <span className={styles.status}>{statusText}</span> : null}
      <Mastery state={masteryState} size="sm" />
    </>
  );

  return onSelect ? (
    <button type="button" className={`${styles.row} ${className}`} onClick={onSelect}>{content}</button>
  ) : (
    <div className={`${styles.row} ${className}`}>{content}</div>
  );
}
