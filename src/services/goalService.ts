import { invoke } from '@tauri-apps/api/core';

import type { Goal } from '../types';

export async function getGoal(id: string): Promise<Goal> {
  return invoke<Goal>('getGoal', { id });
}

/** All goals for a workspace — Guiding plus any Waiting/Met/Resting (screen 21, "+2 more"). */
export async function getGoalsByWorkspace(workspaceId: string): Promise<Goal[]> {
  return invoke<Goal[]>('getGoalsByWorkspace', { workspaceId });
}

/**
 * Goal Editing sheet (screen 11) submit. `text` is the new verbatim goal; inferred
 * structure is re-derived server-side. Previous text is preserved on the returned Goal
 * for the "Was: '...'" / Revert affordance.
 */
export async function updateGoal(id: string, text: string): Promise<Goal> {
  return invoke<Goal>('updateGoal', { id, text });
}

/** Goal Editing sheet's Revert link — restores `previousText` as the active text. */
export async function revertGoal(id: string): Promise<Goal> {
  return invoke<Goal>('revertGoal', { id });
}
