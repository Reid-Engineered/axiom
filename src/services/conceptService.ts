import type { Concept } from '../types';

/** Full concept set for a workspace — chapter grouping and filtering happen in the hook/page. */
export async function getConceptsByWorkspace(_workspaceId: string): Promise<Concept[]> {
  throw new Error('not implemented');
}

export async function getConcept(_id: string): Promise<Concept> {
  throw new Error('not implemented');
}

/** Backs Material's search-typed results and the Command Palette's Concepts group. */
export async function searchConcepts(_workspaceId: string, _query: string): Promise<Concept[]> {
  throw new Error('not implemented');
}
