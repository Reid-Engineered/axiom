import { act, renderHook } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { mockSessions } from '../services/mockData/sessions';
import { useResizablePanes } from './useResizablePanes';

describe('useResizablePanes', () => {
  it('resizes adjacent panes, preserves their total, and resets', () => {
    const exchangeWeight = mockSessions[0].exchanges.length / 40;
    const initialSizes = [2, exchangeWeight];
    const { result } = renderHook(() => useResizablePanes({ initialSizes }));

    expect(result.current.sizes).toEqual([2 / 3, 1 / 3]);
    act(() => result.current.resize(0, 0.1));
    expect(result.current.sizes[0]).toBeCloseTo(2 / 3 + 0.1);
    expect(result.current.sizes.reduce((sum, size) => sum + size, 0)).toBeCloseTo(1);
    act(() => result.current.reset());
    expect(result.current.sizes).toEqual([2 / 3, 1 / 3]);
  });
});
