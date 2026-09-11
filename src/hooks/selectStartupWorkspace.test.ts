import { describe, expect, it } from 'vitest';

import type { Workspace } from '../types';
import { selectStartupWorkspace } from './selectStartupWorkspace';

function workspace(id: string, lastActivityAt?: string, createdAt?: string): Workspace {
  return {
    id,
    name: id,
    guidingGoalId: `goal-${id}`,
    progress: 0,
    lastActivityAt,
    createdAt,
    paused: false,
    offlineAvailability: [],
    enabledModuleIds: [],
  };
}

describe('selectStartupWorkspace', () => {
  it('prefers the greatest non-null activity timestamp in a mixed list', () => {
    const selected = selectStartupWorkspace([
      workspace('no-activity', undefined, '2026-09-09T12:00:00.000Z'),
      workspace('older-activity', '2026-09-07T12:00:00.000Z'),
      workspace('newer-activity', '2026-09-08T12:00:00.000Z'),
    ]);

    expect(selected?.id).toBe('newer-activity');
  });

  it('falls through to the greatest non-null creation timestamp when all activity is null', () => {
    const selected = selectStartupWorkspace([
      workspace('no-created-at'),
      workspace('older', undefined, '2026-09-07T12:00:00.000Z'),
      workspace('newer', undefined, '2026-09-08T12:00:00.000Z'),
    ]);

    expect(selected?.id).toBe('newer');
  });

  it('uses descending id as the final tiebreak when activity and creation are all null', () => {
    const selected = selectStartupWorkspace([workspace('workspace-a'), workspace('workspace-z')]);

    expect(selected?.id).toBe('workspace-z');
  });

  it('returns a single workspace', () => {
    const only = workspace('only');

    expect(selectStartupWorkspace([only])).toBe(only);
  });

  it('returns undefined for an empty list', () => {
    expect(selectStartupWorkspace([])).toBeUndefined();
  });
});
