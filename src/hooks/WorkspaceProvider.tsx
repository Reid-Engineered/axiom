import { useMemo, useState, type ReactNode } from 'react';

import { WorkspaceContext } from './workspaceContext';

export interface WorkspaceProviderProps {
  children: ReactNode;
  initialWorkspaceId?: string | null;
}

/** Owns only the active workspace identity shared across workspace-scoped hooks. */
export function WorkspaceProvider({
  children,
  initialWorkspaceId = null,
}: WorkspaceProviderProps) {
  const [activeWorkspaceId, setActiveWorkspaceId] = useState<string | null>(initialWorkspaceId);
  const value = useMemo(
    () => ({ activeWorkspaceId, setActiveWorkspaceId }),
    [activeWorkspaceId],
  );

  return <WorkspaceContext.Provider value={value}>{children}</WorkspaceContext.Provider>;
}
