import type { OfflineContentKind, Workspace } from '../types';

export interface CreateWorkspaceInput {
  /** e.g. "Calculus II" (screen 2's Subject field). */
  subject: string;
  /** Verbatim natural-language text — inference happens server-side, never as form fields. */
  goalText: string;
}

/** All workspaces for the sidebar tree and Home's Workspace cards. */
export async function getWorkspaces(): Promise<Workspace[]> {
  throw new Error('not implemented');
}

export async function getWorkspace(_id: string): Promise<Workspace> {
  throw new Error('not implemented');
}

/** Screen 2 — Create Workspace. Also provisions the initial Guiding goal. */
export async function createWorkspace(_input: CreateWorkspaceInput): Promise<Workspace> {
  throw new Error('not implemented');
}

/** Screen 21's "Make available offline" sheet — one call per per-kind toggle. */
export async function setWorkspaceOfflineAvailability(
  _id: string,
  _kind: OfflineContentKind,
  _enabled: boolean,
): Promise<Workspace> {
  throw new Error('not implemented');
}
