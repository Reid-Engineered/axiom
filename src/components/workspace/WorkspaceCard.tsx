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

export function WorkspaceCard(_props: WorkspaceCardProps) {
  return null;
}
