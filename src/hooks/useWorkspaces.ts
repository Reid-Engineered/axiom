import { useCallback } from 'react';

import {
  createWorkspace,
  getRecentActivity,
  getWorkspace,
  getWorkspaces,
  setWorkspaceOfflineAvailability,
  type CreateWorkspaceInput,
} from '../services/workspaceService';
import type { OfflineContentKind, Workspace } from '../types';
import { useAsyncResource } from './useAsyncResource';

/** Loads all workspace summaries and exposes workspace creation. */
export function useWorkspaces() {
  const load = useCallback(() => getWorkspaces(), []);
  const resource = useAsyncResource(load);
  const { setData } = resource;

  const create = useCallback(
    async (input: CreateWorkspaceInput) => {
      const workspace = await createWorkspace(input);
      setData((current) => [...(current ?? []), workspace]);
      return workspace;
    },
    [setData],
  );

  return { workspaces: resource.data ?? [], ...resource, createWorkspace: create };
}

/** Loads one workspace and exposes its per-kind offline toggle. */
export function useWorkspaceDetails(workspaceId: string) {
  const load = useCallback(() => getWorkspace(workspaceId), [workspaceId]);
  const resource = useAsyncResource(load);
  const { setData } = resource;

  const setOfflineAvailability = useCallback(
    async (kind: OfflineContentKind, enabled: boolean) => {
      const workspace = await setWorkspaceOfflineAvailability(workspaceId, kind, enabled);
      setData(workspace);
      return workspace;
    },
    [setData, workspaceId],
  );

  return {
    workspace: resource.data as Workspace | undefined,
    ...resource,
    setOfflineAvailability,
  };
}

/** Loads the bounded three-line recap used after a long workspace absence. */
export function useRecentWorkspaceActivity(workspaceId: string) {
  const load = useCallback(() => getRecentActivity(workspaceId), [workspaceId]);
  const resource = useAsyncResource(load);
  return { events: resource.data ?? [], ...resource };
}
