import type { ReactNode } from 'react';

/**
 * 38px toolbar for workspace-scoped pages (Overview, Concepts, Material, Tools). Shows
 * the single "Available offline" chip once downloaded — nothing else mentions
 * connectivity (screen 21, note 4).
 */
export interface WorkspaceToolbarProps {
  workspaceName: string;
  offlineAvailable?: boolean;
  /** Page-specific right-aligned actions. */
  children?: ReactNode;
  className?: string;
}

export function WorkspaceToolbar(_props: WorkspaceToolbarProps) {
  return null;
}
