import { SessionShell } from '../layouts/SessionShell';

export interface StudySessionPageProps {
  sessionId: string;
}

/** Active study session scoped to one resumable session record. */
export function StudySessionPage(_props: StudySessionPageProps) {
  return <SessionShell toolbar={null} visualization={null} problem={null} tutor={null} />;
}
