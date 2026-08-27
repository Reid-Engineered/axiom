import { act, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from './NavigationProvider';
import { useNavigation } from './useNavigation';

function wrapper({ children }: { children: ReactNode }) {
  return (
    <NavigationProvider initialRoute={{ type: 'home' }}>{children}</NavigationProvider>
  );
}

describe('useNavigation', () => {
  it('moves between typed routes and closes the active overlay', () => {
    const { result } = renderHook(() => useNavigation(), { wrapper });

    act(() => result.current.openOverlay({ type: 'commandPalette' }));
    expect(result.current.overlay).toEqual({ type: 'commandPalette' });

    act(() => result.current.navigate({ type: 'conceptsList', workspaceId: 'calculus' }));
    expect(result.current.route).toEqual({ type: 'conceptsList', workspaceId: 'calculus' });
    expect(result.current.overlay).toBeNull();
  });

  it('fails clearly outside its provider', () => {
    expect(() => renderHook(() => useNavigation())).toThrow(
      'useNavigation must be used within NavigationProvider',
    );
  });
});
