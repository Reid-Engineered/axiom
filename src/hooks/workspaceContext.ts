import { createContext } from 'react';

export interface WorkspaceContextValue {
  activeWorkspaceId: string | null;
  setActiveWorkspaceId: (workspaceId: string | null) => void;
}

export const WorkspaceContext = createContext<WorkspaceContextValue | null>(null);
