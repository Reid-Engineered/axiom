import { invoke } from '@tauri-apps/api/core';

import type { Concept } from '../types';

/** Full concept set for a workspace — chapter grouping and filtering happen in the hook/page. */
export async function getConceptsByWorkspace(workspaceId: string): Promise<Concept[]> {
  return invoke<Concept[]>('getConceptsByWorkspace', { workspaceId });
}

export async function getConcept(id: string): Promise<Concept> {
  return invoke<Concept>('getConcept', { id });
}

/** Backs Material's search-typed results and the Command Palette's Concepts group. */
export async function searchConcepts(workspaceId: string, query: string): Promise<Concept[]> {
  return invoke<Concept[]>('searchConcepts', { workspaceId, query });
}
