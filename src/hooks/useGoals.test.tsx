import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { mockGoals } from '../services/mockData/goals';
import { useGoals } from './useGoals';

describe('useGoals', () => {
  it('loads all real goal states and updates then reverts a goal', async () => {
    const workspaceId = 'workspace-calculus-ii';
    const fixtureGoals = mockGoals.filter((goal) => goal.workspaceId === workspaceId);
    const originalText = fixtureGoals[0].text;
    const { result } = renderHook(() => useGoals(workspaceId));

    await waitFor(() => expect(result.current.goals).toHaveLength(4));
    expect(new Set(result.current.goals.map((goal) => goal.state))).toEqual(
      new Set(['Guiding', 'Waiting', 'Met', 'Resting']),
    );
    await act(async () => {
      await result.current.updateGoal(fixtureGoals[0].id, 'Explain every integration method clearly.');
    });
    expect(result.current.goals[0].text).not.toBe(originalText);
    await act(async () => {
      await result.current.revertGoal(fixtureGoals[0].id);
    });
    expect(result.current.goals[0].text).toBe(originalText);
  });
});
