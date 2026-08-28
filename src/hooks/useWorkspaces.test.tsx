import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { mockWorkspaces } from '../services/mockData/workspaces';
import { useWorkspaceDetails, useWorkspaces } from './useWorkspaces';

describe('workspace domain hooks', () => {
  it('loads the real workspace fixtures and creates a workspace', async () => {
    const { result } = renderHook(() => useWorkspaces());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.workspaces).toHaveLength(mockWorkspaces.length);

    await act(async () => {
      await result.current.createWorkspace({ subject: 'Topology', goalText: 'Understand compactness.' });
    });
    expect(result.current.workspaces[result.current.workspaces.length - 1]?.name).toBe('Topology');
  });

  it('loads fixture detail and updates one offline kind', async () => {
    const workspaceId = mockWorkspaces[0].id;
    const { result } = renderHook(() => useWorkspaceDetails(workspaceId));
    await waitFor(() => expect(result.current.workspace?.id).toBe(workspaceId));

    await act(async () => {
      await result.current.setOfflineAvailability('courseVideos', true);
    });
    expect(result.current.workspace?.offlineAvailability.find(
      (item) => item.kind === 'courseVideos',
    )?.enabled).toBe(true);
    await act(async () => {
      await result.current.setOfflineAvailability('courseVideos', false);
    });
  });
});
