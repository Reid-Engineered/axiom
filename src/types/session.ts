/**
 * The current activity, shown as a soft label with "Change intent" (screen 5). Steers;
 * never gates navigation. A short-term goal, distinct from the workspace Goal.
 */
export interface SessionIntent {
  /** A soft label, e.g. "Practising", "Reading", "Exploring" — not a fixed enum. */
  activity: string;
  detail?: string;
  targetMinutes?: number;
}

/**
 * One tutor question/answer pair. Full history sits behind "Earlier today" once
 * exchanges collapse into a settled summary (screen 19).
 */
export interface TutorExchange {
  id: string;
  question: string;
  answer: string;
  occurredAt: string;
  pinnedToVisualization: boolean;
}

/**
 * A study session against one concept. Owns the Continue-card resume state and the
 * tutor panel's rolling summary — the panel never becomes a transcript (screen 19).
 */
export interface Session {
  id: string;
  workspaceId: string;
  conceptId: string;
  status: 'active' | 'paused' | 'completed';
  intent: SessionIntent;
  /** One sentence of exactly where the learner stopped, for the Continue card. */
  resumeSummary: string;
  /** Visualization's last camera position, 206x150 on Home. */
  thumbnailUrl?: string;
  elapsedMinutes: number;
  problemIndex?: number;
  problemCount?: number;
  exchanges: TutorExchange[];
  /** "What we've settled" — up to two conclusions, derived from exchanges. */
  settledConclusions: string[];
  openQuestion?: string;
  startedAt: string;
  pausedAt?: string;
}
