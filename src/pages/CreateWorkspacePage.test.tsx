import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { WorkspaceProvider } from '../hooks/WorkspaceProvider';
import { useWorkspace } from '../hooks/useWorkspace';
import { CreateWorkspacePage } from './CreateWorkspacePage';

function Harness() {
  const { route } = useNavigation();
  const { activeWorkspaceId } = useWorkspace();
  return (
    <>
      <span data-testid="route">{route.type}</span>
      <span data-testid="workspace">{activeWorkspaceId}</span>
      <CreateWorkspacePage />
    </>
  );
}

describe('CreateWorkspacePage', () => {
  it('renders inferred facets, allows adjustment, and creates through the real hook', async () => {
    render(
      <WorkspaceProvider>
        <NavigationProvider initialRoute={{ type: 'createWorkspace' }}>
          <Harness />
        </NavigationProvider>
      </WorkspaceProvider>,
    );

    expect(screen.getByText('Axiom read that as')).toBeVisible();
    expect(screen.getByText('Deadline · Dec 12')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Adjust' }));
    expect(screen.getByText('Comfort level')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Create Workspace' }));

    await waitFor(() => expect(screen.getByTestId('route')).toHaveTextContent('home'));
    expect(screen.getByTestId('workspace').textContent).toMatch(/^workspace-/);
  });
});
