import { useCallback, useState } from 'react';

import { getMaterial, searchMaterial } from '../services/materialService';
import { useAsyncResource } from './useAsyncResource';

/** Loads a workspace's book state and searches its concept-linked material results. */
export function useMaterial(workspaceId: string) {
  const load = useCallback(() => getMaterial(workspaceId), [workspaceId]);
  const resource = useAsyncResource(load);
  const [searchResults, setSearchResults] = useState<Awaited<ReturnType<typeof searchMaterial>>>(
    [],
  );

  const search = useCallback(
    async (query: string) => {
      const results = await searchMaterial(workspaceId, query);
      setSearchResults(results);
      return results;
    },
    [workspaceId],
  );

  return { material: resource.data, searchResults, search, ...resource };
}
