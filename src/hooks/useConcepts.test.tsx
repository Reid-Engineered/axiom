import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { mockConcepts } from '../services/mockData/concepts';
import { useConcept, useConcepts } from './useConcepts';

describe('concept domain hooks', () => {
  it('loads the 87-concept graph and searches the real fixtures', async () => {
    const workspaceId = 'workspace-calculus-ii';
    const { result } = renderHook(() => useConcepts(workspaceId));
    await waitFor(() => expect(result.current.concepts).toHaveLength(87));
    expect(result.current.concepts.some((concept) => concept.leadsToConceptIds.length > 0)).toBe(true);

    await act(async () => {
      await result.current.search('shell');
    });
    expect(result.current.searchResults.some((concept) => concept.name === 'Shell method')).toBe(true);
  });

  it('loads a fully populated concept from the fixture graph', async () => {
    const fixture = mockConcepts.find((concept) => concept.name === 'Shell method');
    if (!fixture) throw new Error('Shell method fixture missing');
    const { result } = renderHook(() => useConcept(fixture.id));
    await waitFor(() => expect(result.current.concept?.id).toBe(fixture.id));
    expect(result.current.concept?.recentDiagnostics).toHaveLength(2);
  });
});
