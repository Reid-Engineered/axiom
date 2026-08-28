import { act, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it } from 'vitest';

import { mockWorkspaces } from '../services/mockData/workspaces';
import { NavigationProvider } from './NavigationProvider';
import { useCommandPalette } from './useCommandPalette';

function wrapper({ children }: { children: ReactNode }) {
  return (
    <NavigationProvider initialRoute={{ type: 'workspaceOverview', workspaceId: mockWorkspaces[0].id }}>
      {children}
    </NavigationProvider>
  );
}

describe('useCommandPalette', () => {
  it('opens, tracks a query, and clears it when closing', () => {
    const { result } = renderHook(() => useCommandPalette(), { wrapper });

    act(() => result.current.open());
    expect(result.current.isOpen).toBe(true);
    act(() => result.current.setQuery('shell'));
    expect(result.current.query).toBe('shell');
    act(() => result.current.close());
    expect(result.current).toMatchObject({ isOpen: false, query: '' });
  });
});
