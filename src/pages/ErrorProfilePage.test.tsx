import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { executeSessionAction } from '../services/practiceService';
import { startSession } from '../services/sessionService';
import { ErrorProfilePage } from './ErrorProfilePage';

function RouteObserver() {
  const { route } = useNavigation();
  return <output aria-label="Current route">{route.type}</output>;
}

function renderProfile(workspaceId = 'workspace-calculus-ii', initialPatternId?: string) {
  return render(
    <NavigationProvider
      initialRoute={{ type: 'errorProfile', workspaceId, patternId: initialPatternId }}
    >
      <ErrorProfilePage workspaceId={workspaceId} initialPatternId={initialPatternId} />
      <RouteObserver />
    </NavigationProvider>,
  );
}

describe('ErrorProfilePage', () => {
  it('renders an honest empty state when a domain has no recorded error patterns', async () => {
    renderProfile('workspace-linear-algebra');

    expect(
      await screen.findByRole('heading', { name: 'No error patterns recorded yet' }),
    ).toBeVisible();
    expect(
      screen.getByText(/Missing evidence is never shown as zero occurrences/i),
    ).toBeVisible();
  });

  it('displays tracked patterns, updates resolution rule thresholds, and drills a pattern', async () => {
    const session = await startSession({
      workspaceId: 'workspace-calculus-ii',
      conceptId: 'calc-concept-22',
      intent: { activity: 'Practising', targetMinutes: 25 },
    });
    await executeSessionAction({
      operationId: 'op-seed-check-1',
      workspaceId: 'workspace-calculus-ii',
      sessionId: session.id,
      action: 'check',
      payload: {
        response: { responseType: 'numeric', value: '3.2' },
      },
    });

    renderProfile('workspace-calculus-ii');

    const headings = await screen.findAllByRole('heading', {
      name: /Omitting shell radius factor/i,
      level: 2,
    });
    expect(headings.length).toBeGreaterThanOrEqual(1);
    expect(headings[0]).toBeVisible();

    // Update resolution rule from 3 clean to 4 clean
    const fourCleanBtn = await screen.findByRole('button', { name: '4 clean' });
    fireEvent.click(fourCleanBtn);
    await waitFor(() => expect(fourCleanBtn).toHaveAttribute('aria-pressed', 'true'));

    // Drill this pattern
    fireEvent.click(screen.getByRole('button', { name: 'Drill this pattern' }));
    await waitFor(() =>
      expect(screen.getByLabelText('Current route')).toHaveTextContent('studySession'),
    );
  });
});
