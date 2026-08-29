/**
 * A learner's own note, linked to the concept it's about. `Concept.notesCount` is only a
 * count; this is the first place actual note content is needed (screen 12's "From your
 * work" Command Palette result).
 */
export interface Note {
  id: string;
  workspaceId: string;
  conceptId: string;
  text: string;
  updatedAt: string;
}
