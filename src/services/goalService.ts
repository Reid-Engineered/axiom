import type { Goal } from '../types';

export async function getGoal(_id: string): Promise<Goal> {
  throw new Error('not implemented');
}

/** All goals for a workspace — Guiding plus any Waiting/Met/Resting (screen 21, "+2 more"). */
export async function getGoalsByWorkspace(_workspaceId: string): Promise<Goal[]> {
  throw new Error('not implemented');
}

/**
 * Goal Editing sheet (screen 11) submit. `text` is the new verbatim goal; inferred
 * structure is re-derived server-side. Previous text is preserved on the returned Goal
 * for the "Was: '...'" / Revert affordance.
 */
export async function updateGoal(_id: string, _text: string): Promise<Goal> {
  throw new Error('not implemented');
}

/** Goal Editing sheet's Revert link — restores `previousText` as the active text. */
export async function revertGoal(_id: string): Promise<Goal> {
  throw new Error('not implemented');
}
