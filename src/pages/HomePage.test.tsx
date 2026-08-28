import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { WorkspaceProvider } from '../hooks/WorkspaceProvider';
import type { HomePageVariant } from './HomePage';
import { HomePage } from './HomePage';

function renderHome(variant: HomePageVariant) {
  return render(
    <WorkspaceProvider initialWorkspaceId="workspace-calculus-ii">
      <NavigationProvider initialRoute={{ type: 'home', variant }}>
        <HomePage variant={variant} sidebar={<span>Sidebar content</span>} />
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
});
