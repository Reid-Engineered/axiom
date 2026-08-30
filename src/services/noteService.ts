import { invoke } from '@tauri-apps/api/core';

import type { Note } from '../types';

/** Backs the Command Palette's "From your work" group. */
export async function getRecentNotes(workspaceId: string): Promise<Note[]> {
  return invoke<Note[]>('getRecentNotes', { workspaceId });
}
