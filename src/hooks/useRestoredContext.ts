import { useCallback } from 'react';

import { getWorkspaces } from '../services/workspaceService';
import type { Workspace } from '../types';
import type { Route } from './useNavigation';
import { useAsyncResource } from './useAsyncResource';
import { selectStartupWorkspace } from './selectStartupWorkspace';

export interface RestoredContext {
  initialRoute: Route;
  initialWorkspaceId: string | null;
  workspaces: Workspace[];
  refreshWorkspaces: () => Promise<void>;
}

/** Derives startup navigation and workspace context from persisted domain state. */
export function useRestoredContext(): RestoredContext | null {
  const load = useCallback(() => getWorkspaces(), []);
  const { data, loading, error, refresh } = useAsyncResource(load);

  if (error) throw error;
  if (loading || !data) return null;

  const workspace = selectStartupWorkspace(data);
  return workspace
    ? {
        initialRoute: { type: 'home' },
        initialWorkspaceId: workspace.id,
        workspaces: data,
        refreshWorkspaces: refresh,
      }
    : {
        initialRoute: { type: 'firstLaunch' },
        initialWorkspaceId: null,
        workspaces: data,
        refreshWorkspaces: refresh,
      };
}
