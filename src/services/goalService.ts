import type { Goal } from '../types';
import { mockGoals } from './mockData/goals';

export async function getGoal(id: string): Promise<Goal> {
  const goal = mockGoals.find((candidate) => candidate.id === id);
  if (!goal) throw new Error(`Goal not found: ${id}`);
  return structuredClone(goal);
}

/** All goals for a workspace — Guiding plus any Waiting/Met/Resting (screen 21, "+2 more"). */
export async function getGoalsByWorkspace(workspaceId: string): Promise<Goal[]> {
  return structuredClone(mockGoals.filter((goal) => goal.workspaceId === workspaceId));
}

/**
 * Goal Editing sheet (screen 11) submit. `text` is the new verbatim goal; inferred
 * structure is re-derived server-side. Previous text is preserved on the returned Goal
 * for the "Was: '...'" / Revert affordance.
 */
export async function updateGoal(id: string, text: string): Promise<Goal> {
  const goal = mockGoals.find((candidate) => candidate.id === id);
  if (!goal) throw new Error(`Goal not found: ${id}`);
  goal.previousText = goal.text;
  goal.text = text.trim();
  goal.updatedAt = new Date().toISOString();
  return structuredClone(goal);
}

/** Goal Editing sheet's Revert link — restores `previousText` as the active text. */
export async function revertGoal(id: string): Promise<Goal> {
  const goal = mockGoals.find((candidate) => candidate.id === id);
  if (!goal) throw new Error(`Goal not found: ${id}`);
  if (!goal.previousText) throw new Error(`Goal has no previous text: ${id}`);
  const currentText = goal.text;
  goal.text = goal.previousText;
  goal.previousText = currentText;
  goal.updatedAt = new Date().toISOString();
  return structuredClone(goal);
}
