import { useContext } from 'react';

import { WorkspaceContext, type WorkspaceContextValue } from './workspaceContext';

/** Reads and updates the active workspace identity. */
export function useWorkspace(): WorkspaceContextValue {
  const value = useContext(WorkspaceContext);

  if (!value) {
    throw new Error('useWorkspace must be used within WorkspaceProvider');
  }

  return value;
}
