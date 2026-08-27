import { TwoPaneLayout } from '../layouts/TwoPaneLayout';

export interface WorkspaceOverviewPageProps {
  workspaceId: string;
}

/** Goal-oriented overview for one workspace. */
export function WorkspaceOverviewPage(_props: WorkspaceOverviewPageProps) {
  return <TwoPaneLayout rail={null}>{null}</TwoPaneLayout>;
}
