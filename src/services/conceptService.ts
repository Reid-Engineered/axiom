import type { Concept } from '../types';
import { mockConcepts } from './mockData/concepts';

/** Full concept set for a workspace — chapter grouping and filtering happen in the hook/page. */
export async function getConceptsByWorkspace(workspaceId: string): Promise<Concept[]> {
  return structuredClone(mockConcepts.filter((concept) => concept.workspaceId === workspaceId));
}

export async function getConcept(id: string): Promise<Concept> {
  const concept = mockConcepts.find((candidate) => candidate.id === id);
  if (!concept) throw new Error(`Concept not found: ${id}`);
  return structuredClone(concept);
}

/** Backs Material's search-typed results and the Command Palette's Concepts group. */
export async function searchConcepts(workspaceId: string, query: string): Promise<Concept[]> {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const workspaceConcepts = mockConcepts.filter((concept) => concept.workspaceId === workspaceId);
  if (!normalizedQuery) return structuredClone(workspaceConcepts);

  return structuredClone(
    workspaceConcepts.filter((concept) =>
      [concept.name, concept.chapter, concept.explanation, ...(concept.whereItShowsUp ?? [])]
        .filter((value): value is string => Boolean(value))
        .some((value) => value.toLocaleLowerCase().includes(normalizedQuery)),
    ),
  );
}
