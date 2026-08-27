import type { Module } from '../types';

/** A workspace's modules, on and off (screen 8) — enabled/visibility scoped to this workspace. */
export async function getModulesByWorkspace(_workspaceId: string): Promise<Module[]> {
  throw new Error('not implemented');
}

/** Marketplace catalog (screen 9), optionally personalized "for your workspace". */
export async function getMarketplaceModules(_forWorkspaceId?: string): Promise<Module[]> {
  throw new Error('not implemented');
}

export async function getModule(_id: string): Promise<Module> {
  throw new Error('not implemented');
}

/** "Install to Calculus II" (screen 10) — install is always workspace-scoped. */
export async function installModule(_workspaceId: string, _moduleId: string): Promise<Module> {
  throw new Error('not implemented');
}

/** Workspace Tools on/off toggle (screen 8). */
export async function setModuleEnabled(
  _workspaceId: string,
  _moduleId: string,
  _enabled: boolean,
): Promise<Module> {
  throw new Error('not implemented');
}

/**
 * Moves a module between the three groupings screen 21 describes ("In the workspace" /
 * "Appear when relevant" / "Off in this workspace"). Separate from `setModuleEnabled`:
 * a module can be enabled but only surfaced contextually, not pinned to Overview tiles.
 */
export async function setModuleVisibility(
  _workspaceId: string,
  _moduleId: string,
  _visibility: Module['visibility'],
): Promise<Module> {
  throw new Error('not implemented');
}
