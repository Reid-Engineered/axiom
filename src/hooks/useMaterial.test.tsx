import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { useMaterial } from './useMaterial';

describe('useMaterial', () => {
  it('loads the real book fixture and searches all three shell result kinds', async () => {
    const { result } = renderHook(() => useMaterial('workspace-calculus-ii'));

    await waitFor(() => expect(result.current.material?.totalPages).toBe(712));
    expect(result.current.material?.segments).toHaveLength(4);

    await act(async () => {
      await result.current.search('shell radius');
    });
    expect(result.current.searchResults.map((item) => item.kind)).toEqual([
      'section',
      'workedExample',
      'exerciseRange',
    ]);
    expect(result.current.searchResults.every((item) => item.conceptId === 'calc-concept-22')).toBe(
      true,
    );
  });
});
