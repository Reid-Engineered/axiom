import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { WorkspaceProvider } from '../hooks/WorkspaceProvider';
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
  it('renders fixture-backed Continue and workspace cards by default', async () => {
    renderHome('default');
    await waitFor(() => expect(screen.getByText('Calculus II — Shell method')).toBeVisible());
    expect(screen.getAllByRole('progressbar')).toHaveLength(3);
    expect(screen.getByText(/checking where the height changes/)).toBeVisible();
  });

  it('renders the session-intent plan', async () => {
    renderHome('session-intent');
    expect(screen.getByRole('heading', { name: 'How much time do you have?' })).toBeVisible();
    await waitFor(() => expect(screen.getByText(/Finish the shell method/)).toBeVisible());
    expect(screen.getByText('Three problems on choosing radius vs. height')).toBeVisible();
  });

  it('renders the library variant without the supplied sidebar', async () => {
    renderHome('library');
    await waitFor(() => expect(screen.getByText('Pick up: Shell method')).toBeVisible());
    expect(screen.queryByText('Sidebar content')).toBeNull();
    expect(screen.getByRole('button', { name: 'New' })).toBeVisible();
  });

  it('replaces Continue with bounded context recovery after a long absence', async () => {
    renderHome('default', 'workspace-physics');

    const title = await screen.findByRole('heading', {
      name: 'You were working with Angular momentum',
    });
    const recovery = title.closest('section');
    expect(within(recovery!).queryByText('Continue')).toBeNull();
    expect(within(recovery!).getByRole('button', { name: '5-minute refresher' })).toBeVisible();
    expect(
      await within(recovery!).findByRole('button', { name: 'Straight back to problem 3' }),
    ).toBeVisible();

    const recoveryLines = within(recovery!)
      .getByText(/held up while you were away/)
      .closest('ul');
    expect(within(recoveryLines!).getAllByRole('listitem')).toHaveLength(3);
    expect(screen.getByText('Angular momentum · was Strong')).toBeVisible();

    const away = screen.getByRole('heading', { name: 'While you were away' }).closest('section');
    expect(within(away!).getAllByRole('listitem')).toHaveLength(3);

    fireEvent.click(within(recovery!).getByRole('button', { name: 'Straight back to problem 3' }));
    await waitFor(() =>
      expect(screen.getByLabelText('Current route')).toHaveTextContent('studySession'),
    );
  });
});
