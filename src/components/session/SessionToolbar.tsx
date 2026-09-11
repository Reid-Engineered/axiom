import type { SessionIntent } from '../../types';
import { Button } from '../primitives/Button';
import styles from './SessionToolbar.module.css';

/**
 * 44px session toolbar: concept name, subject line, session-intent label with "Change
 * intent" (never mode tabs), optional five-dash problem progress, and Pause.
 */
export interface SessionToolbarProps {
  conceptName: string;
  subjectLine: string;
  intent: SessionIntent;
  onChangeIntent?: () => void;
  /** Drives the five-dash indicator only when a persisted problem total exists. */
  problemIndex: number;
  problemCount?: number | null;
  onPause?: () => void;
  className?: string;
}

export function SessionToolbar({
  conceptName,
  subjectLine,
  intent,
  onChangeIntent,
  problemIndex,
  problemCount,
  onPause,
  className = '',
}: SessionToolbarProps) {
  const dashCount = 5;
  const hasProblemCount = problemCount !== undefined && problemCount !== null;
  const completedDashes =
    !hasProblemCount
      ? 0
      : Math.min(dashCount, Math.ceil((problemIndex / Math.max(problemCount, 1)) * dashCount));

  return (
    <div className={`${styles.toolbar} ${className}`}>
      <div className={styles.context}>
        <strong>{conceptName}</strong>
        <span>{subjectLine}</span>
      </div>
      <div className={styles.intent}>
        <span>
          {intent.activity}
          {intent.detail ? ` · ${intent.detail}` : ''}
        </span>
        {onChangeIntent ? (
          <Button variant="tertiary" size="sm" onClick={onChangeIntent}>
            Change intent
          </Button>
        ) : null}
      </div>
      {!hasProblemCount ? null : (
        <div className={styles.progress} aria-label={`Problem ${problemIndex} of ${problemCount}`}>
          {Array.from({ length: dashCount }, (_, index) => (
            <span key={index} data-complete={index < completedDashes} />
          ))}
        </div>
      )}
      {onPause ? (
        <Button variant="secondary" size="sm" onClick={onPause}>
          Pause
        </Button>
      ) : null}
    </div>
  );
}
