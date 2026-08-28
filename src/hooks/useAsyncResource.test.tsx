import { renderHook, waitFor } from '@testing-library/react';
import { useCallback } from 'react';
import { describe, expect, it } from 'vitest';

import { getWorkspaces } from '../services/workspaceService';
import { mockWorkspaces } from '../services/mockData/workspaces';
import { useAsyncResource } from './useAsyncResource';

describe('useAsyncResource', () => {
  it('loads data from the real workspace fixture service', async () => {
    const { result } = renderHook(() => {
      const load = useCallback(() => getWorkspaces(), []);
      return useAsyncResource(load);
    });

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.data?.[0].id).toBe(mockWorkspaces[0].id);
    expect(result.current.error).toBeNull();
  });
});
