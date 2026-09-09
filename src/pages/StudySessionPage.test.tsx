import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { getSession, startSession } from '../services/sessionService';
import { StudySessionPage } from './StudySessionPage';

function RouteObserver() {
  const { route } = useNavigation();
  return <output aria-label="Current route">{route.type}</output>;
}

function renderSession(sessionId = 'session-shell-method') {
  render(
    <NavigationProvider initialRoute={{ type: 'studySession', sessionId }}>
      <StudySessionPage sessionId={sessionId} />
      <RouteObserver />
    </NavigationProvider>,
  );
}

describe('StudySessionPage', () => {
  it('renders the real long session as a settled summary with only the current exchange expanded', async () => {
    renderSession();

    expect(await screen.findByText('Shell method')).toBeVisible();
    expect(screen.getByRole('heading', { name: 'Tutor · Coach' })).toBeVisible();
    expect(screen.getByText('40 exchanges today')).toBeVisible();
    expect(screen.getByRole('heading', { name: 'What we’ve settled' })).toBeVisible();
    expect(
      screen.getByText('The shell radius is measured from the axis of rotation.'),
    ).toBeVisible();
    expect(
      within(screen.getByRole('article', { name: 'Current tutor exchange' })).getByText(
        /Measure from the axis of rotation/,
      ),
    ).toBeVisible();
    expect(screen.getByText('Earlier today')).toBeVisible();
    expect(
      screen.getByText('How should I identify the shell radius in example 1?'),
    ).not.toBeVisible();
  });

  it('submits a tutor question, pauses, edits working, and opens the visualization detour', async () => {
    renderSession();
    const working = await screen.findByRole('textbox', { name: 'Your working' });

    fireEvent.change(working, { target: { value: 'r = 2 − x' } });
    expect(working).toHaveValue('r = 2 − x');

    fireEvent.change(screen.getByRole('textbox', { name: 'Ask about this step' }), {
      target: { value: 'Why is the radius measured from the axis?' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Ask' }));
    await waitFor(() => expect(screen.getByText('41 exchanges today')).toBeVisible());

    fireEvent.click(screen.getByRole('button', { name: 'Pause' }));
    await waitFor(async () =>
      expect((await getSession('session-shell-method')).status).toBe('paused'),
    );

    fireEvent.click(screen.getByRole('button', { name: 'Full visualization' }));
    expect(screen.getByRole('status', { name: 'Current route' })).toHaveTextContent(
      'fullVisualization',
    );
  });

  it('renders the bound attempt prompt and no problem total', async () => {
    const session = await startSession({
      workspaceId: 'workspace-calculus',
      conceptId: 'concept-shells',
      intent: { activity: 'Practising', targetMinutes: 8 },
    });
    renderSession(session.id);

    expect(await screen.findByText(/revolved about the y-axis/i)).toBeVisible();
    expect(screen.getByRole('textbox', { name: 'Answer' })).toBeVisible();
    expect(screen.queryByText(/of 1/)).not.toBeInTheDocument();
  });

  it('states plainly when a concept has no practice content', async () => {
    const session = await startSession({
      workspaceId: 'workspace-linear-algebra',
      conceptId: 'linear-concept-1',
      intent: { activity: 'Practising', targetMinutes: 8 },
    });
    renderSession(session.id);

    expect(await screen.findByText(/no practice content for this concept yet/i)).toBeVisible();
    expect(screen.queryByRole('textbox', { name: 'Answer' })).not.toBeInTheDocument();
  });

  it('offers a hint as the explanation when an answer is wrong', async () => {
    const session = await startSession({
      workspaceId: 'workspace-calculus',
      conceptId: 'concept-shells',
      intent: { activity: 'Practising', targetMinutes: 8 },
    });
    renderSession(session.id);
    const answer = await screen.findByRole('textbox', { name: 'Answer' });

    fireEvent.change(answer, { target: { value: '1' } });
    fireEvent.click(screen.getByRole('button', { name: 'Check' }));

    expect(await screen.findByText('Set up the shell method integral.')).toBeVisible();
  });

  it('shows a plain confirmation and a next problem action once solved', async () => {
    const session = await startSession({
      workspaceId: 'workspace-calculus',
      conceptId: 'concept-shells',
      intent: { activity: 'Practising', targetMinutes: 8 },
    });
    renderSession(session.id);
    const answer = await screen.findByRole('textbox', { name: 'Answer' });

    fireEvent.change(answer, { target: { value: '42.7' } });
    fireEvent.click(screen.getByRole('button', { name: 'Check' }));

    expect(await screen.findByText('Correct.')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Next problem' }));
    await waitFor(() => expect(screen.getByText('Problem 2')).toBeVisible());
  });
});
