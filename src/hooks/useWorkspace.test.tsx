import { act, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it } from 'vitest';

import { WorkspaceProvider } from './WorkspaceProvider';
import { useWorkspace } from './useWorkspace';

function wrapper({ children }: { children: ReactNode }) {
  return <WorkspaceProvider initialWorkspaceId="calculus">{children}</WorkspaceProvider>;
}

describe('useWorkspace', () => {
  it('owns only the active workspace identity', () => {
    const { result } = renderHook(() => useWorkspace(), { wrapper });

    expect(result.current.activeWorkspaceId).toBe('calculus');
    act(() => result.current.setActiveWorkspaceId('linear-algebra'));
    expect(result.current.activeWorkspaceId).toBe('linear-algebra');
  });

  it('fails clearly outside its provider', () => {
    expect(() => renderHook(() => useWorkspace())).toThrow(
      'useWorkspace must be used within WorkspaceProvider',
    );
  });
});
