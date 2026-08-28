import { afterEach, describe, expect, it } from 'vitest';

import { mockModules } from './mockData/modules';
import { mockWorkspaces } from './mockData/workspaces';
import {
  getMarketplaceModules,
  getModulesByWorkspace,
  setModuleEnabled,
} from './moduleService';

const calculusId = 'workspace-calculus-ii';
const linearAlgebraId = 'workspace-linear-algebra';
const sharedModuleId = 'module-1';
const originalEnabledIds = new Map(
  mockWorkspaces.map((workspace) => [workspace.id, [...workspace.enabledModuleIds]]),
);

afterEach(() => {
  for (const workspace of mockWorkspaces) {
    workspace.enabledModuleIds = [...(originalEnabledIds.get(workspace.id) ?? [])];
  }
});

describe('moduleService workspace scoping', () => {
  it('derives enabled state from each workspace and its personalized marketplace', async () => {
    const [calculusModules, linearModules, linearMarketplace] = await Promise.all([
      getModulesByWorkspace(calculusId),
      getModulesByWorkspace(linearAlgebraId),
      getMarketplaceModules(linearAlgebraId),
    ]);

    expect(calculusModules.filter((module) => module.enabled)).toHaveLength(13);
    expect(linearModules.filter((module) => module.enabled)).toHaveLength(4);
    expect(linearMarketplace.filter((module) => module.enabled)).toHaveLength(4);
  });

  it('does not change another workspace or the global fixture when toggled', async () => {
    await setModuleEnabled(linearAlgebraId, sharedModuleId, false);

    const [calculusModules, linearModules] = await Promise.all([
      getModulesByWorkspace(calculusId),
      getModulesByWorkspace(linearAlgebraId),
    ]);
    expect(calculusModules.find((module) => module.id === sharedModuleId)?.enabled).toBe(true);
    expect(linearModules.find((module) => module.id === sharedModuleId)?.enabled).toBe(false);
    expect(mockModules.find((module) => module.id === sharedModuleId)?.enabled).toBe(true);
  });
});
