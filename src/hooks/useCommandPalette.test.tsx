import { act, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it } from 'vitest';

import { mockWorkspaces } from '../services/mockData/workspaces';
import { NavigationProvider } from './NavigationProvider';
import { useCommandPalette } from './useCommandPalette';

function wrapper({ children }: { children: ReactNode }) {
  return (
    <NavigationProvider
      initialRoute={{ type: 'workspaceOverview', workspaceId: mockWorkspaces[0].id }}
    >
      {children}
    </NavigationProvider>
  );
}

describe('useCommandPalette', () => {
  it('opens, tracks a query, returns real grouped data, and clears on close', async () => {
    const { result } = renderHook(() => useCommandPalette(), { wrapper });

    await waitFor(() => expect(result.current.notes).toHaveLength(1));
    await waitFor(() => expect(result.current.concepts.length).toBeGreaterThan(0));

    act(() => result.current.open());
    expect(result.current.isOpen).toBe(true);
    act(() => result.current.setQuery('shell'));
    expect(result.current.query).toBe('shell');
    expect(result.current.actions[0]?.label).toBe('Practice the Shell Method');
    expect(result.current.actions[0]?.detail).toBe('12 problems · adaptive');
    expect(result.current.concepts.some((concept) => concept.name === 'Shell method')).toBe(true);
    expect(result.current.notes[0]?.id).toBe('note-shell-radius');
    expect(result.current.marketplaceModules).toHaveLength(0);
    act(() => result.current.close());
    expect(result.current).toMatchObject({ isOpen: false, query: '' });
  });

  it('derives actions and related concepts for a non-shell active concept', async () => {
    const { result } = renderHook(() => useCommandPalette('workspace-linear-algebra'), {
      wrapper,
    });

    await waitFor(() =>
      expect(
        result.current.actions.some((action) => action.label === 'Practice the Eigenvectors'),
      ).toBe(true),
    );
    act(() => result.current.setQuery('eigenvectors'));

    expect(result.current.actions.map((action) => action.label)).toContain(
      'Visualize the Eigenvectors',
    );
    expect(result.current.concepts.map((concept) => concept.name)).toEqual([
      'Eigenvectors',
      'Span and independence',
    ]);
  });
});
