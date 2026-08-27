import type { SessionIntent } from '../../types';

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

export function SessionToolbar(_props: SessionToolbarProps) {
  return null;
}
