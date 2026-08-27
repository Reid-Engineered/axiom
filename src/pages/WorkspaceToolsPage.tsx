import type { ReactNode } from 'react';

import { AppShell } from '../layouts/AppShell';

export interface WorkspaceToolsPageProps {
  workspaceId: string;
  sidebar?: ReactNode;
}

/** Module visibility and offline controls for one workspace. */
export function WorkspaceToolsPage({ sidebar }: WorkspaceToolsPageProps) {
  return <AppShell sidebar={sidebar}>{null}</AppShell>;
}
