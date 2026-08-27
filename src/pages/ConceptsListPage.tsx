import type { ReactNode } from 'react';

import { AppShell } from '../layouts/AppShell';

export interface ConceptsListPageProps {
  workspaceId: string;
  sidebar?: ReactNode;
}

/** Chapter-grouped concept list for one workspace. */
export function ConceptsListPage({ sidebar }: ConceptsListPageProps) {
  return <AppShell sidebar={sidebar}>{null}</AppShell>;
}
