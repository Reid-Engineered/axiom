import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { mockWorkspaces } from '../services/mockData/workspaces';
import { useRecentWorkspaceActivity, useWorkspaceDetails, useWorkspaces } from './useWorkspaces';

describe('workspace domain hooks', () => {
  it('loads the real workspace fixtures, creates a workspace, and imports the sample', async () => {
    const { result } = renderHook(() => useWorkspaces());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.workspaces).toHaveLength(mockWorkspaces.length);

    await act(async () => {
      await result.current.createWorkspace({
        subject: 'Topology',
        goalText: 'Understand compactness.',
      });
    });
    expect(result.current.workspaces[result.current.workspaces.length - 1]?.name).toBe('Topology');

    await act(async () => {
      await result.current.importSampleWorkspace();
    });
    expect(
      result.current.workspaces.filter((workspace) => workspace.id === 'workspace-calculus-ii'),
    ).toHaveLength(1);
  });

  it('loads fixture detail and updates one offline kind', async () => {
    const workspaceId = mockWorkspaces[0].id;
    const { result } = renderHook(() => useWorkspaceDetails(workspaceId));
    await waitFor(() => expect(result.current.workspace?.id).toBe(workspaceId));

    await act(async () => {
      await result.current.setOfflineAvailability('courseVideos', true);
    });
    expect(
      result.current.workspace?.offlineAvailability.find((item) => item.kind === 'courseVideos')
        ?.enabled,
    ).toBe(true);
    await act(async () => {
      await result.current.setOfflineAvailability('courseVideos', false);
    });
  });

  it('loads an oldest-first recovery recap bounded to three events', async () => {
    const { result } = renderHook(() => useRecentWorkspaceActivity('workspace-physics'));
    await waitFor(() => expect(result.current.events).toHaveLength(3));
    expect(result.current.events.map((event) => event.id)).toEqual([
      'physics-activity-1',
      'physics-activity-2',
      'physics-activity-3',
    ]);
  });
});
