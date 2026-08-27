import type { ReactNode } from 'react';

import { AppShell } from '../layouts/AppShell';

export interface MarketplacePageProps {
  forWorkspaceId?: string;
  sidebar?: ReactNode;
}

/** Module marketplace, optionally personalized for the active workspace. */
export function MarketplacePage({ sidebar }: MarketplacePageProps) {
  return <AppShell sidebar={sidebar}>{null}</AppShell>;
}
