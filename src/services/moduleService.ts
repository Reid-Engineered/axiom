import type { Module } from '../types';
import { mockModules } from './mockData/modules';
import { mockWorkspaces } from './mockData/workspaces';

/** A workspace's modules, on and off (screen 8) — enabled/visibility scoped to this workspace. */
export async function getModulesByWorkspace(workspaceId: string): Promise<Module[]> {
  const workspace = mockWorkspaces.find((candidate) => candidate.id === workspaceId);
  if (!workspace) throw new Error(`Workspace not found: ${workspaceId}`);
  return structuredClone(mockModules);
}

/** Marketplace catalog (screen 9), optionally personalized "for your workspace". */
export async function getMarketplaceModules(forWorkspaceId?: string): Promise<Module[]> {
  if (forWorkspaceId && !mockWorkspaces.some((workspace) => workspace.id === forWorkspaceId)) {
    throw new Error(`Workspace not found: ${forWorkspaceId}`);
  }
  return structuredClone(mockModules);
}

export async function getModule(id: string): Promise<Module> {
  const module = mockModules.find((candidate) => candidate.id === id);
  if (!module) throw new Error(`Module not found: ${id}`);
  return structuredClone(module);
}

/** "Install to Calculus II" (screen 10) — install is always workspace-scoped. */
export async function installModule(workspaceId: string, moduleId: string): Promise<Module> {
  const workspace = mockWorkspaces.find((candidate) => candidate.id === workspaceId);
  if (!workspace) throw new Error(`Workspace not found: ${workspaceId}`);
  const module = mockModules.find((candidate) => candidate.id === moduleId);
  if (!module) throw new Error(`Module not found: ${moduleId}`);
  if (!workspace.enabledModuleIds.includes(moduleId)) workspace.enabledModuleIds.push(moduleId);
  module.enabled = true;
  if (module.visibility === 'off') module.visibility = 'contextual';
  return structuredClone(module);
}

/** Workspace Tools on/off toggle (screen 8). */
export async function setModuleEnabled(
  workspaceId: string,
  moduleId: string,
  enabled: boolean,
): Promise<Module> {
  const workspace = mockWorkspaces.find((candidate) => candidate.id === workspaceId);
  if (!workspace) throw new Error(`Workspace not found: ${workspaceId}`);
  const module = mockModules.find((candidate) => candidate.id === moduleId);
  if (!module) throw new Error(`Module not found: ${moduleId}`);

  module.enabled = enabled;
  if (enabled && module.visibility === 'off') module.visibility = 'contextual';
  workspace.enabledModuleIds = enabled
    ? [...new Set([...workspace.enabledModuleIds, moduleId])]
    : workspace.enabledModuleIds.filter((id) => id !== moduleId);
  if (!enabled) module.visibility = 'off';
  return structuredClone(module);
}

/**
 * Moves a module between the three groupings screen 21 describes ("In the workspace" /
 * "Appear when relevant" / "Off in this workspace"). Separate from `setModuleEnabled`:
 * a module can be enabled but only surfaced contextually, not pinned to Overview tiles.
 */
export async function setModuleVisibility(
  workspaceId: string,
  moduleId: string,
  visibility: Module['visibility'],
): Promise<Module> {
  const workspace = mockWorkspaces.find((candidate) => candidate.id === workspaceId);
  if (!workspace) throw new Error(`Workspace not found: ${workspaceId}`);
  const module = mockModules.find((candidate) => candidate.id === moduleId);
  if (!module) throw new Error(`Module not found: ${moduleId}`);

  module.visibility = visibility;
  module.enabled = visibility !== 'off';
  workspace.enabledModuleIds = module.enabled
    ? [...new Set([...workspace.enabledModuleIds, moduleId])]
    : workspace.enabledModuleIds.filter((id) => id !== moduleId);
  return structuredClone(module);
}
