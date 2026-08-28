import { useCallback } from 'react';

import { getGoalsByWorkspace, revertGoal, updateGoal } from '../services/goalService';
import { useAsyncResource } from './useAsyncResource';

/** Loads a workspace's goal history and exposes edit/revert mutations. */
export function useGoals(workspaceId: string) {
  const load = useCallback(() => getGoalsByWorkspace(workspaceId), [workspaceId]);
  const resource = useAsyncResource(load);
  const { setData } = resource;

  const replaceGoal = useCallback((updatedGoal: Awaited<ReturnType<typeof updateGoal>>) => {
    setData((current) =>
      current?.map((goal) => goal.id === updatedGoal.id ? updatedGoal : goal),
    );
    return updatedGoal;
  }, [setData]);

  const update = useCallback(async (goalId: string, text: string) =>
    replaceGoal(await updateGoal(goalId, text)), [replaceGoal]);
  const revert = useCallback(async (goalId: string) =>
    replaceGoal(await revertGoal(goalId)), [replaceGoal]);

  return { goals: resource.data ?? [], ...resource, updateGoal: update, revertGoal: revert };
}
