import { AppShell } from '../layouts/AppShell';

export interface MarketplacePageProps {
  forWorkspaceId?: string;
}

/** Module marketplace, optionally personalized for the active workspace. */
export function MarketplacePage(_props: MarketplacePageProps) {
  return <AppShell>{null}</AppShell>;
}
