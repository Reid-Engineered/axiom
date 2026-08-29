import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { getSession } from '../services/sessionService';
import { StudySessionPage } from './StudySessionPage';

function RouteObserver() {
  const { route } = useNavigation();
  return <output aria-label="Current route">{route.type}</output>;
}

function renderSession() {
  render(
    <NavigationProvider initialRoute={{ type: 'studySession', sessionId: 'session-shell-method' }}>
      <StudySessionPage sessionId="session-shell-method" />
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
});
