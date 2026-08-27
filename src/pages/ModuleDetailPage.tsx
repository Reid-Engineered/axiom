import { TwoPaneLayout } from '../layouts/TwoPaneLayout';

export interface ModuleDetailPageProps {
  moduleId: string;
  forWorkspaceId?: string;
}

/** Learning value, trust, context access, and workspace-scoped actions for one module. */
export function ModuleDetailPage(_props: ModuleDetailPageProps) {
  return <TwoPaneLayout rail={null}>{null}</TwoPaneLayout>;
}
