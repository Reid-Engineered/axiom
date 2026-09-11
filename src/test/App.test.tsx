import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import App from '../App';
import { mockWorkspaces } from '../services/mockData/workspaces';
import { describeAttempt } from '../services/practiceService';
import { startSession } from '../services/sessionService';
import { loadMockSessionsForTest, setMockWorkspacesForTest } from './mockBackend';

/**
 * "Explore a sample workspace" appears twice once the empty sidebar is visible alongside
 * First Launch's own content (same action, two entry points) — this picks the one outside
 * the sidebar, matching a learner reading the page rather than using the persistent chrome.
 */
async function enterSampleWorkspace() {
  const buttons = await screen.findAllByRole('button', { name: 'Explore a sample workspace' });
  const sidebar = screen.getByRole('navigation', { name: 'Primary' });
  const pageButton = buttons.find((button) => !sidebar?.contains(button)) ?? buttons[0];
  fireEvent.click(pageButton);
  await waitFor(() => expect(document.querySelector('[data-route="home"]')).not.toBeNull());
  await screen.findByRole('button', { name: 'Calculus II' });
}

describe('development gallery route', () => {
  beforeEach(() => {
    window.location.hash = '';
  });

  it('does not show the development gallery by default', () => {
    render(<App />);

    expect(screen.queryByRole('heading', { name: 'Design System Primitives' })).toBeNull();
  });

  it('shows the primitive gallery at its development hash', () => {
    window.location.hash = '#/dev/gallery';
    render(<App />);

    expect(screen.getByRole('heading', { name: 'Design System Primitives' })).toBeVisible();
  });
});

describe('startup restoration', () => {
  beforeEach(() => {
    window.location.hash = '';
  });

  it('blocks first paint and renders First Launch only when there are no workspaces', async () => {
    setMockWorkspacesForTest([]);
    const { container } = render(<App />);

    expect(container.querySelector('[data-route]')).toBeNull();
    expect(screen.queryByRole('heading', { name: 'What are you learning?' })).toBeNull();
    expect(await screen.findByRole('heading', { name: 'What are you learning?' })).toBeVisible();
  });

  it('boots Home with the last-worked workspace instead of the first returned workspace', async () => {
    setMockWorkspacesForTest([
      { ...mockWorkspaces[1], lastActivityAt: '2026-09-07T12:00:00.000Z' },
      { ...mockWorkspaces[0], lastActivityAt: '2026-09-09T12:00:00.000Z' },
    ]);
    loadMockSessionsForTest();
    const session = await startSession({
      workspaceId: 'workspace-calculus-ii',
      conceptId: 'calc-concept-22',
      intent: { activity: 'Practising', targetMinutes: 8 },
    });
    expect(session.currentAttemptId).toEqual(expect.any(String));
    if (!session.currentAttemptId) throw new Error('Expected a bound practice attempt');
    const attempt = await describeAttempt(session.workspaceId, session.currentAttemptId);
    render(<App />);

    expect(await screen.findByRole('heading', { name: 'Workspaces' })).toBeVisible();
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Calculus II' })).toHaveAttribute(
        'aria-expanded',
        'true',
      ),
    );
    expect(screen.queryByRole('heading', { name: 'What are you learning?' })).toBeNull();
    expect(await screen.findByRole('heading', { name: 'Calculus II — Shell method' })).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Resume session' }));
    expect(await screen.findByText(attempt.prompt)).toBeVisible();
  });

  it('keeps the workspace returned by creation active instead of rerunning startup selection', async () => {
    setMockWorkspacesForTest([
      { ...mockWorkspaces[0], lastActivityAt: '2026-09-09T12:00:00.000Z' },
    ]);
    render(<App />);

    fireEvent.click(await screen.findByRole('button', { name: '+ New Workspace' }));
    fireEvent.change(screen.getByLabelText('Subject'), {
      target: { value: 'Returned Workspace' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create Workspace' }));

    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Returned Workspace' })).toHaveAttribute(
        'aria-expanded',
        'true',
      ),
    );
  });

  it('keeps the workspace returned by sample import active instead of rerunning startup selection', async () => {
    setMockWorkspacesForTest([
      { ...mockWorkspaces[1], lastActivityAt: '2026-09-09T12:00:00.000Z' },
    ]);
    render(<App />);

    await screen.findByRole('heading', { name: 'Workspaces' });
    fireEvent.click(screen.getByText('Page stubs'));
    fireEvent.click(screen.getByRole('button', { name: 'First launch' }));
    const sidebar = screen.getByRole('navigation', { name: 'Primary' });
    const sampleButton = screen
      .getAllByRole('button', { name: 'Explore a sample workspace' })
      .find((button) => !sidebar.contains(button));
    fireEvent.click(sampleButton!);

    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Calculus II' })).toHaveAttribute(
        'aria-expanded',
        'true',
      ),
    );
  });
});

describe('Stage 3 navigation', () => {
  beforeEach(() => {
    window.location.hash = '';
    setMockWorkspacesForTest([]);
  });

  it('navigates through the permanent workspace areas', async () => {
    const { container } = render(<App />);

    await enterSampleWorkspace();
    expect(container.querySelector('[data-route="home"]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Calculus II' }));
    fireEvent.click(screen.getByRole('button', { name: 'Concepts' }));
    expect(container.querySelector('[data-route="conceptsList"]')).not.toBeNull();
    expect(screen.getByRole('button', { name: 'Concepts' })).toHaveAttribute(
      'aria-current',
      'page',
    );
    fireEvent.click(screen.getByRole('button', { name: 'Material' }));
    expect(container.querySelector('[data-route="material"]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Tools' }));
    expect(container.querySelector('[data-route="workspaceTools"]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Marketplace' }));
    expect(container.querySelector('[data-route="marketplace"]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Home' }));
    await waitFor(() => expect(container.querySelector('[data-route="home"]')).not.toBeNull());
    fireEvent.click(screen.getByRole('button', { name: 'Calculus II' }));
    expect(container.querySelector('[data-route="workspaceOverview"]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: '+ New Workspace' }));
    expect(container.querySelector('[data-route="createWorkspace"]')).not.toBeNull();
  });

  it('opens real command palette results from the advertised shortcut', async () => {
    setMockWorkspacesForTest([
      { ...mockWorkspaces[0], lastActivityAt: '2026-09-09T12:00:00.000Z' },
    ]);
    loadMockSessionsForTest();
    render(<App />);

    await screen.findByRole('heading', { name: 'Workspaces' });
    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    const dialog = screen.getByRole('dialog', { name: 'Command palette' });
    expect(dialog).toBeVisible();
    expect(await within(dialog).findByText('Practice the Shell Method')).toBeVisible();
    expect(within(dialog).getByText('From your work')).toBeVisible();
    expect(within(dialog).getByText(/Note —/)).toBeVisible();
    expect(within(dialog).getByText('Marketplace')).toBeVisible();
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(screen.queryByRole('dialog', { name: 'Command palette' })).toBeNull();
  });

  it.each([
    ['First launch', 'firstLaunch'],
    ['Study session', 'studySession'],
    ['Concept view', 'conceptView'],
    ['Module detail', 'moduleDetail'],
  ])('reaches the %s stub without adding permanent navigation', async (label, route) => {
    const { container } = render(<App />);

    await enterSampleWorkspace();
    fireEvent.click(screen.getByText('Page stubs'));
    fireEvent.click(screen.getByRole('button', { name: label }));
    expect(container.querySelector(`[data-route="${route}"]`)).not.toBeNull();
  });

  it('renders full visualization with a drag strip and without a sidebar', async () => {
    const { container } = render(<App />);

    await enterSampleWorkspace();
    fireEvent.click(screen.getByText('Page stubs'));
    fireEvent.click(screen.getByRole('button', { name: 'Full visualization' }));

    expect(container.querySelector('[data-route="fullVisualization"]')).not.toBeNull();
    expect(container.querySelector('[data-tauri-drag-region]')).not.toBeNull();
    expect(container.querySelector('aside')).toBeNull();
  });

  it('reaches the goal-editing overlay from the development stub menu', async () => {
    const { container } = render(<App />);

    await enterSampleWorkspace();
    fireEvent.click(screen.getByText('Page stubs'));
    fireEvent.click(screen.getByRole('button', { name: 'Goal editing' }));
    expect(container.querySelector('[data-overlay="goalEditing"]')).not.toBeNull();
  });

  it('shows a visible sidebar (not absent) on first launch and continues to workspace setup', async () => {
    const { container } = render(<App />);

    expect(await screen.findByRole('heading', { name: 'What are you learning?' })).toBeVisible();
    expect(container.querySelector('aside')).not.toBeNull();
    expect(screen.getByRole('navigation', { name: 'Primary' })).toBeVisible();

    expect(screen.getByRole('textbox', { name: 'Subject' })).toHaveValue('');
    expect(screen.getByPlaceholderText('Calculus II')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(container.querySelector('[data-route="createWorkspace"]')).not.toBeNull();
    expect(container.querySelector('aside')).not.toBeNull();
  });

  it('reflects a newly created workspace in the sidebar once Home is reached', async () => {
    const { container } = render(<App />);

    fireEvent.click(await screen.findByRole('button', { name: 'Continue' }));
    fireEvent.change(screen.getByLabelText('Subject'), {
      target: { value: 'Sidebar Regression Subject' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create Workspace' }));
    await waitFor(() => expect(container.querySelector('[data-route="home"]')).not.toBeNull());

    const sidebar = screen.getByRole('navigation', { name: 'Primary' });
    await waitFor(() =>
      expect(
        within(sidebar).getByRole('button', { name: 'Sidebar Regression Subject' }),
      ).toBeVisible(),
    );
  });

  it('completes the Stage 5 launch-to-populated-overview path', async () => {
    const { container } = render(<App />);

    await enterSampleWorkspace();

    fireEvent.click(screen.getByRole('button', { name: 'Calculus II' }));
    await waitFor(() =>
      expect(container.querySelector('[data-route="workspaceOverview"]')).not.toBeNull(),
    );
    expect(screen.getByText('Concepts in play')).toBeVisible();
    expect(screen.getByRole('button', { name: /Shell method/ })).toBeVisible();
  });
});
