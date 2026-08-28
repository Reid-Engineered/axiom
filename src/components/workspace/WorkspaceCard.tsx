import { ProgressBar } from '../primitives/ProgressBar';
import styles from './WorkspaceCard.module.css';

/**
 * Home's "Workspaces" cards — name, goal sentence, 3px unlabelled progress, last concept
 * plus relative time, or "Paused" (screen 3). No dashboard, no analytics.
 */
export interface WorkspaceCardProps {
  name: string;
  goalSentence: string;
  /** 0-1 fraction driving the unlabelled ProgressBar fill — never rendered as text. */
  progress: number;
  lastConceptName?: string;
  /** Relative-time text, e.g. "2 hours ago" — pre-formatted by the caller. */
  lastActivityLabel?: string;
  paused?: boolean;
  onSelect?: () => void;
  className?: string;
}

export function WorkspaceCard({
  name,
  goalSentence,
  progress,
  lastConceptName,
  lastActivityLabel,
  paused = false,
  onSelect,
  className = '',
}: WorkspaceCardProps) {
  return (
    <button type="button" className={`${styles.card} ${className}`} onClick={onSelect}>
      <span className={styles.title}><span aria-hidden="true" />{name}</span>
      <span className={styles.goal}>{goalSentence}</span>
      <ProgressBar value={progress} max={1} />
      <span className={styles.activity}>
        {paused ? 'Paused' : [lastConceptName, lastActivityLabel].filter(Boolean).join(' · ')}
      </span>
    </button>
  );
}
