import type { GoalState } from './common';

/**
 * Inferred structure shown as removable chips on Create Workspace / Goal Editing
 * (screens 2, 11). Never collected as form fields — always inferred and correctable.
 */
export interface GoalInferredStructure {
  deadline?: string;
  masteryType?: string;
  conceptScope?: number;
  pacing?: string;
  tools?: string[];
}

/**
 * A living object holding verbatim natural-language text plus inferred structure.
 * One primary active goal per workspace (state 'Guiding'); history retained.
 */
export interface Goal {
  id: string;
  workspaceId: string;
  text: string;
  state: GoalState;
  inferred: GoalInferredStructure;
  /** Previous goal text, shown as "Was: '...'" on Goal Editing with Revert. */
  previousText?: string;
  /** For state 'Met' — "archived with what they achieved" (screen 21). */
  achievedSummary?: string;
  createdAt: string;
  updatedAt: string;
}
