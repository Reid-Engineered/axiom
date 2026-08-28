import { useCallback } from 'react';

import {
  getMarketplaceModules,
  getModule,
  getModulesByWorkspace,
  installModule,
  setModuleEnabled,
  setModuleVisibility,
} from '../services/moduleService';
import type { Module } from '../types';
import { useAsyncResource } from './useAsyncResource';

/** Loads workspace module groupings and exposes enable/visibility mutations. */
export function useModules(workspaceId: string) {
  const load = useCallback(() => getModulesByWorkspace(workspaceId), [workspaceId]);
  const resource = useAsyncResource(load);
  const { setData } = resource;
  const replace = useCallback((updated: Module) => {
    setData((current) => current?.map((module) => module.id === updated.id ? updated : module));
    return updated;
  }, [setData]);

  const setEnabled = useCallback(async (moduleId: string, enabled: boolean) =>
    replace(await setModuleEnabled(workspaceId, moduleId, enabled)), [replace, workspaceId]);
  const setVisibility = useCallback(async (moduleId: string, visibility: Module['visibility']) =>
    replace(await setModuleVisibility(workspaceId, moduleId, visibility)), [replace, workspaceId]);

  return { modules: resource.data ?? [], ...resource, setEnabled, setVisibility };
}

/** Loads the marketplace catalog and installs entries into an optional workspace. */
export function useMarketplaceModules(forWorkspaceId?: string) {
  const load = useCallback(() => getMarketplaceModules(forWorkspaceId), [forWorkspaceId]);
  const resource = useAsyncResource(load);
  const { setData } = resource;
  const install = useCallback(async (moduleId: string) => {
    if (!forWorkspaceId) throw new Error('A workspace is required to install a module');
    const installed = await installModule(forWorkspaceId, moduleId);
    setData((current) => current?.map((module) => module.id === installed.id ? installed : module));
    return installed;
  }, [forWorkspaceId, setData]);

  return { modules: resource.data ?? [], ...resource, installModule: install };
}

/** Loads one module's learner-facing capability details. */
export function useModule(moduleId: string) {
  const load = useCallback(() => getModule(moduleId), [moduleId]);
  const resource = useAsyncResource(load);
  return { module: resource.data, ...resource };
}
