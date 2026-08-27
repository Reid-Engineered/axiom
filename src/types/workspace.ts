import type { OfflineStatus } from './common';

/**
 * A curated learning environment for one subject. Owns goals, concepts, material,
 * history, enabled modules, and learning preferences (AXIOM-HANDOFF.md §1).
 */
export interface Workspace {
  id: string;
  name: string;
  guidingGoalId: string;
  /** 0-1 fraction driving the unlabelled ProgressBar fill — never rendered as text. */
  progress: number;
  lastConceptName?: string;
  lastActivityAt?: string;
  paused: boolean;
  offlineStatus: OfflineStatus;
  /** Set once "Make available offline" has run for this workspace (screen 21). */
  offlineSizeBytes?: number;
  enabledModuleIds: string[];
}
