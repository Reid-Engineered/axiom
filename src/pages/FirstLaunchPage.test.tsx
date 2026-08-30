import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { WorkspaceProvider } from '../hooks/WorkspaceProvider';
import { useWorkspace } from '../hooks/useWorkspace';
import { FirstLaunchPage } from './FirstLaunchPage';

function Harness() {
  const { route } = useNavigation();
  const { activeWorkspaceId } = useWorkspace();
  return (
    <>
      <span data-testid="route">{route.type}</span>
      <span data-testid="workspace">{activeWorkspaceId}</span>
      <FirstLaunchPage />
    </>
  );
}

describe('FirstLaunchPage', () => {
  it('imports and opens the sample workspace through the real hook', async () => {
    render(
      <WorkspaceProvider>
        <NavigationProvider>
          <Harness />
        </NavigationProvider>
      </WorkspaceProvider>,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Explore a sample workspace' }));

    await waitFor(() => expect(screen.getByTestId('route')).toHaveTextContent('home'));
    expect(screen.getByTestId('workspace')).toHaveTextContent('workspace-calculus-ii');
  });

  it('keeps the learner on first launch when the sample import fails', async () => {
    mockIPC((command) => {
      if (command === 'getWorkspaces') return [];
      throw new Error('Database unavailable');
    });
    render(
      <WorkspaceProvider>
        <NavigationProvider>
          <Harness />
        </NavigationProvider>
      </WorkspaceProvider>,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Explore a sample workspace' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'The sample workspace could not be prepared. Try again.',
    );
    expect(screen.getByTestId('route')).toHaveTextContent('firstLaunch');
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Explore a sample workspace' })).toBeEnabled(),
    );
  });
});
