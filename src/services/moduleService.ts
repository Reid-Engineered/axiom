import type { Module } from '../types';
import { mockModules } from './mockData/modules';
import { mockWorkspaces } from './mockData/workspaces';

function findWorkspace(workspaceId: string) {
  const workspace = mockWorkspaces.find((candidate) => candidate.id === workspaceId);
  if (!workspace) throw new Error(`Workspace not found: ${workspaceId}`);
  return workspace;
}

function findModule(moduleId: string) {
  const module = mockModules.find((candidate) => candidate.id === moduleId);
  if (!module) throw new Error(`Module not found: ${moduleId}`);
  return module;
}

function forWorkspace(module: Module, enabledModuleIds: string[]): Module {
  return { ...structuredClone(module), enabled: enabledModuleIds.includes(module.id) };
}

/** A workspace's modules with enabled state derived from that workspace's installed ids. */
export async function getModulesByWorkspace(workspaceId: string): Promise<Module[]> {
  const workspace = findWorkspace(workspaceId);
  return mockModules.map((module) => forWorkspace(module, workspace.enabledModuleIds));
}

/** Marketplace catalog (screen 9), optionally personalized "for your workspace". */
export async function getMarketplaceModules(forWorkspaceId?: string): Promise<Module[]> {
  if (forWorkspaceId) return getModulesByWorkspace(forWorkspaceId);
  return structuredClone(mockModules);
}

export async function getModule(id: string): Promise<Module> {
  return structuredClone(findModule(id));
}

/** "Install to Calculus II" (screen 10) — install is always workspace-scoped. */
export async function installModule(workspaceId: string, moduleId: string): Promise<Module> {
  const workspace = findWorkspace(workspaceId);
  const module = findModule(moduleId);
  if (!workspace.enabledModuleIds.includes(moduleId)) workspace.enabledModuleIds.push(moduleId);
  return forWorkspace(module, workspace.enabledModuleIds);
}

/** Workspace Tools on/off toggle (screen 8). */
export async function setModuleEnabled(
  workspaceId: string,
  moduleId: string,
  enabled: boolean,
): Promise<Module> {
  const workspace = findWorkspace(workspaceId);
  const module = findModule(moduleId);

  workspace.enabledModuleIds = enabled
    ? [...new Set([...workspace.enabledModuleIds, moduleId])]
    : workspace.enabledModuleIds.filter((id) => id !== moduleId);
  return forWorkspace(module, workspace.enabledModuleIds);
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
  const workspace = findWorkspace(workspaceId);
  const module = findModule(moduleId);

  module.visibility = visibility;
  workspace.enabledModuleIds = visibility !== 'off'
    ? [...new Set([...workspace.enabledModuleIds, moduleId])]
    : workspace.enabledModuleIds.filter((id) => id !== moduleId);
  return forWorkspace(module, workspace.enabledModuleIds);
}
