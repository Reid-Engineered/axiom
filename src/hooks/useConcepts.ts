import { useCallback, useState } from 'react';

import { getConcept, getConceptsByWorkspace, searchConcepts } from '../services/conceptService';
import { useAsyncResource } from './useAsyncResource';

/** Loads the complete concept graph for a workspace and supports scoped search. */
export function useConcepts(workspaceId: string) {
  const load = useCallback(() => getConceptsByWorkspace(workspaceId), [workspaceId]);
  const resource = useAsyncResource(load);
  const [searchResults, setSearchResults] = useState(resource.data ?? []);

  const search = useCallback(async (query: string) => {
    const results = await searchConcepts(workspaceId, query);
    setSearchResults(results);
    return results;
  }, [workspaceId]);

  return { concepts: resource.data ?? [], searchResults, search, ...resource };
}

/** Loads one concept with its graph edges and learning evidence. */
export function useConcept(conceptId: string) {
  const load = useCallback(() => getConcept(conceptId), [conceptId]);
  const resource = useAsyncResource(load);
  return { concept: resource.data, ...resource };
}
