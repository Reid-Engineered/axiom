import type { ReactNode } from 'react';

import { AppShell } from '../layouts/AppShell';

export interface MaterialPageProps {
  workspaceId: string;
  sidebar?: ReactNode;
}

/** Concept-linked material search for one workspace. */
export function MaterialPage({ sidebar }: MaterialPageProps) {
  return <AppShell sidebar={sidebar}>{null}</AppShell>;
}
