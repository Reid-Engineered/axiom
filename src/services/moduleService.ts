import { invoke } from '@tauri-apps/api/core';

import type { Module, WorkspaceTemplate } from '../types';

/** A workspace's modules with enabled state derived from that workspace's installed ids. */
export async function getModulesByWorkspace(workspaceId: string): Promise<Module[]> {
  return invoke<Module[]>('getModulesByWorkspace', { workspaceId });
}

/** Marketplace catalog (screen 9), optionally personalized "for your workspace". */
export async function getMarketplaceModules(forWorkspaceId?: string): Promise<Module[]> {
  return invoke<Module[]>('getMarketplaceModules', { forWorkspaceId });
}

export async function getWorkspaceTemplates(): Promise<WorkspaceTemplate[]> {
  return invoke<WorkspaceTemplate[]>('getWorkspaceTemplates');
}

export async function getModule(id: string): Promise<Module> {
  return invoke<Module>('getModule', { id });
}

/** "Install to Calculus II" (screen 10) — install is always workspace-scoped. */
export async function installModule(workspaceId: string, moduleId: string): Promise<Module> {
  return invoke<Module>('installModule', { workspaceId, moduleId });
}

/** Workspace Tools on/off toggle (screen 8). */
export async function setModuleEnabled(
  workspaceId: string,
  moduleId: string,
  enabled: boolean,
): Promise<Module> {
  return invoke<Module>('setModuleEnabled', { workspaceId, moduleId, enabled });
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
  return invoke<Module>('setModuleVisibility', { workspaceId, moduleId, visibility });
}
