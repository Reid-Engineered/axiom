import type { SessionIntent } from '../../types';
import { Button } from '../primitives/Button';
import styles from './SessionToolbar.module.css';

/**
 * 44px session toolbar: concept name, subject line, session-intent label with "Change
 * intent" (never mode tabs), five-dash problem progress, elapsed/target time, Pause
 * (screen 5 — the five dashes mirror the problem pane's "Problem 3 of 5").
 */
export interface SessionToolbarProps {
  conceptName: string;
  subjectLine: string;
  intent: SessionIntent;
  onChangeIntent?: () => void;
  /** Drives the five-dash indicator, e.g. 3 of 5 — matches `Session.problemIndex/Count`. */
  problemIndex: number;
  problemCount: number;
  elapsedMinutes: number;
  targetMinutes: number;
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
  elapsedMinutes,
  targetMinutes,
  onPause,
  className = '',
}: SessionToolbarProps) {
  const dashCount = 5;
  const completedDashes = Math.min(
    dashCount,
    Math.ceil((problemIndex / Math.max(problemCount, 1)) * dashCount),
  );

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
      <div className={styles.progress} aria-label={`Problem ${problemIndex} of ${problemCount}`}>
        {Array.from({ length: dashCount }, (_, index) => (
          <span key={index} data-complete={index < completedDashes} />
        ))}
      </div>
      <span className={styles.time}>
        {elapsedMinutes}′ of {targetMinutes}′
      </span>
      {onPause ? (
        <Button variant="secondary" size="sm" onClick={onPause}>
          Pause
        </Button>
      ) : null}
    </div>
  );
}
