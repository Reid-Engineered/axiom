import type { Note } from '../types';
import { mockNotes } from './mockData/notes';

/** Backs the Command Palette's "From your work" group. */
export async function getRecentNotes(workspaceId: string): Promise<Note[]> {
  return structuredClone(
    mockNotes
      .filter((note) => note.workspaceId === workspaceId)
      .sort((left, right) => right.updatedAt.localeCompare(left.updatedAt)),
  );
}
