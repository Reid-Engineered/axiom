import type { ReactNode } from 'react';

/** Main content plus a 250px right rail (used by WorkspaceOverviewPage, ConceptViewPage, ModuleDetailPage). */
export interface TwoPaneLayoutProps {
  children: ReactNode;
  rail: ReactNode;
  className?: string;
}

export function TwoPaneLayout(_props: TwoPaneLayoutProps) {
  return null;
}
