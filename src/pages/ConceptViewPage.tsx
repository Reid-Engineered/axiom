import { TwoPaneLayout } from '../layouts/TwoPaneLayout';

export interface ConceptViewPageProps {
  workspaceId: string;
  conceptId: string;
}

/** Detailed mastery, explanation, relationships, and activity for one concept. */
export function ConceptViewPage(_props: ConceptViewPageProps) {
  return <TwoPaneLayout rail={null}>{null}</TwoPaneLayout>;
}
