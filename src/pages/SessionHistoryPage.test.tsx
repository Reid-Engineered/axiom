import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { executeSessionAction } from '../services/practiceService';
import { startSession } from '../services/sessionService';
import { SessionHistoryPage } from './SessionHistoryPage';

function RouteObserver() {
  const { route } = useNavigation();
  return <output aria-label="Current route">{route.type}</output>;
}

function renderHistory(workspaceId = 'workspace-calculus-ii', initialSessionId?: string) {
  return render(
    <NavigationProvider
      initialRoute={{ type: 'sessionHistory', workspaceId, sessionId: initialSessionId }}
    >
      <SessionHistoryPage workspaceId={workspaceId} initialSessionId={initialSessionId} />
      <RouteObserver />
    </NavigationProvider>,
  );
}

describe('SessionHistoryPage', () => {
  it('renders an empty state when a domain has no recorded sessions', async () => {
    renderHistory('workspace-linear-algebra');

    expect(
      await screen.findByRole('heading', { name: 'No sessions recorded in this domain yet' }),
    ).toBeVisible();
  });

  it('renders completed session summary, mastery deltas, question breakdown, and starts follow-up', async () => {
    const session = await startSession({
      workspaceId: 'workspace-calculus-ii',
      conceptId: 'calc-concept-22',
      intent: { activity: 'Practising', detail: 'Shell method practice', targetMinutes: 25 },
    });
    await executeSessionAction({
      operationId: 'op-hist-start',
      workspaceId: 'workspace-calculus-ii',
      sessionId: session.id,
      action: 'start',
      elapsedDeltaMs: 120_000,
    });
    await executeSessionAction({
      operationId: 'op-hist-check',
      workspaceId: 'workspace-calculus-ii',
      sessionId: session.id,
      action: 'check',
      elapsedDeltaMs: 45_000,
      payload: {
        response: { responseType: 'numeric', value: '42.7' },
      },
    });
    await executeSessionAction({
      operationId: 'op-hist-end',
      workspaceId: 'workspace-calculus-ii',
      sessionId: session.id,
      action: 'end',
      elapsedDeltaMs: 15_000,
    });

    renderHistory('workspace-calculus-ii', session.id);

    expect(await screen.findByText('Session complete')).toBeVisible();
    expect(screen.getByText('Mastery changes in this session')).toBeVisible();
    expect(screen.getByText('Question breakdown')).toBeVisible();
    expect(screen.getByText('3m 00s')).toBeVisible();

    fireEvent.click(screen.getByRole('button', { name: 'Start next recommended session' }));
    await waitFor(() =>
      expect(screen.getByLabelText('Current route')).toHaveTextContent('studySession'),
    );
  });
});
