import { FullVisualizationShell } from '../layouts/FullVisualizationShell';

export interface FullVisualizationPageProps {
  sessionId: string;
}

/** Full-bleed visualization detour within an active study session. */
export function FullVisualizationPage(_props: FullVisualizationPageProps) {
  return <FullVisualizationShell header={null}>{null}</FullVisualizationShell>;
}
