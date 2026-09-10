import { mockIPC } from '@tauri-apps/api/mocks';
import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { WorkspaceProvider } from '../hooks/WorkspaceProvider';
import { mockWorkspaces } from '../services/mockData/workspaces';
import { startSession } from '../services/sessionService';
import { handleMockInvoke } from '../test/mockBackend';
import type { HomePageVariant } from './HomePage';
import { HomePage } from './HomePage';

function RouteObserver() {
  const { route } = useNavigation();
  return <output aria-label="Current route">{route.type}</output>;
}

function renderHome(variant: HomePageVariant, workspaceId = 'workspace-calculus-ii') {
  return render(
    <WorkspaceProvider initialWorkspaceId={workspaceId}>
      <NavigationProvider initialRoute={{ type: 'home', variant }}>
        <HomePage variant={variant} sidebar={<span>Sidebar content</span>} />
        <RouteObserver />
      </NavigationProvider>
    </WorkspaceProvider>,
  );
}

describe('HomePage', () => {
  it('renders no Continue card immediately after sample import', async () => {
    renderHome('default');
    await waitFor(() => expect(screen.getByRole('heading', { name: 'Workspaces' })).toBeVisible());
    expect(screen.getAllByRole('progressbar')).toHaveLength(3);
    expect(screen.queryByText('Continue')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Resume session' })).toBeNull();
  });

  it('renders the session-intent plan', async () => {
    renderHome('session-intent');
    expect(screen.getByRole('heading', { name: 'How much time do you have?' })).toBeVisible();
    await waitFor(() => expect(screen.getByText(/Finish the shell method/)).toBeVisible());
    expect(screen.getByText('Three problems on choosing radius vs. height')).toBeVisible();
  });

  it('renders the library variant with explicitly created learner activity', async () => {
    await startSession({
      workspaceId: 'workspace-calculus-ii',
      conceptId: 'calc-concept-22',
      intent: { activity: 'Practising' },
    });
    renderHome('library');
    await waitFor(() => expect(screen.getByText('Pick up: Shell method')).toBeVisible());
    expect(screen.queryByText('Sidebar content')).toBeNull();
    expect(screen.getByRole('button', { name: 'New' })).toBeVisible();
  });

  it('replaces Continue with bounded context recovery after a long absence', async () => {
    await startSession({
      workspaceId: 'workspace-physics',
      conceptId: 'physics-concept-2',
      intent: { activity: 'Practising' },
    });
    const workspacesWithLearnerActivity = mockWorkspaces.map((workspace) =>
      workspace.id === 'workspace-physics'
        ? { ...workspace, lastActivityAt: '2026-05-12T16:45:00.000Z' }
        : workspace,
    );
    mockIPC((command, payload) =>
      command === 'getWorkspaces'
        ? structuredClone(workspacesWithLearnerActivity)
        : handleMockInvoke(command, payload),
    );
    renderHome('default', 'workspace-physics');

    const title = await screen.findByRole('heading', {
      name: 'You were working with Angular momentum',
    });
    const recovery = title.closest('section');
    expect(within(recovery!).queryByText('Continue')).toBeNull();
    expect(within(recovery!).getByRole('button', { name: '5-minute refresher' })).toBeVisible();
    const resumeButton = await within(recovery!).findByRole('button', {
      name: /^Straight back to problem/,
    });
    expect(resumeButton).toBeVisible();

    const recoveryLines = within(recovery!)
      .getByText(/held up while you were away/)
      .closest('ul');
    expect(within(recoveryLines!).getAllByRole('listitem')).toHaveLength(3);
    expect(screen.getByText('Angular momentum · was Strong')).toBeVisible();

    const away = screen.getByRole('heading', { name: 'While you were away' }).closest('section');
    expect(within(away!).getAllByRole('listitem')).toHaveLength(3);

    fireEvent.click(resumeButton);
    await waitFor(() =>
      expect(screen.getByLabelText('Current route')).toHaveTextContent('studySession'),
    );
  });
});
