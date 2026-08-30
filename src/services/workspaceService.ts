import { invoke } from '@tauri-apps/api/core';

import type { OfflineContentKind, Workspace, WorkspaceActivityEvent } from '../types';

export interface CreateWorkspaceInput {
  /** e.g. "Calculus II" (screen 2's Subject field). */
  subject: string;
  /** Verbatim natural-language text — inference happens server-side, never as form fields. */
  goalText: string;
}

/** All workspaces for the sidebar tree and Home's Workspace cards. */
export async function getWorkspaces(): Promise<Workspace[]> {
  return invoke<Workspace[]>('getWorkspaces');
}

export async function getWorkspace(id: string): Promise<Workspace> {
  return invoke<Workspace>('getWorkspace', { id });
}

/** Returns the bounded context-recovery recap, oldest first and never more than three. */
export async function getRecentActivity(workspaceId: string): Promise<WorkspaceActivityEvent[]> {
  return invoke<WorkspaceActivityEvent[]>('getRecentActivity', { workspaceId });
}

/** Screen 2 — Create Workspace. Also provisions the initial Guiding goal. */
export async function createWorkspace(input: CreateWorkspaceInput): Promise<Workspace> {
  return invoke<Workspace>('createWorkspace', { input });
}

/** Screen 21's "Make available offline" sheet — one call per per-kind toggle. */
export async function setWorkspaceOfflineAvailability(
  id: string,
  kind: OfflineContentKind,
  enabled: boolean,
): Promise<Workspace> {
  return invoke<Workspace>('setWorkspaceOfflineAvailability', { id, kind, enabled });
}
