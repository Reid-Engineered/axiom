import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { WorkspaceProvider } from '../hooks/WorkspaceProvider';
import { WorkspaceOverviewPage } from './WorkspaceOverviewPage';

describe('WorkspaceOverviewPage', () => {
  it('populates the overview from real Stage 4 fixtures', async () => {
    render(
      <WorkspaceProvider initialWorkspaceId="workspace-calculus-ii">
        <NavigationProvider initialRoute={{ type: 'workspaceOverview', workspaceId: 'workspace-calculus-ii' }}>
          <WorkspaceOverviewPage workspaceId="workspace-calculus-ii" />
        </NavigationProvider>
      </WorkspaceProvider>,
    );

    await waitFor(() => expect(screen.getByRole('heading', { name: 'Calculus II' })).toBeVisible());
    expect(screen.getByText(/Be ready to explain and solve every integration technique/)).toBeVisible();
    expect(screen.getByRole('button', { name: /Shell method/ })).toBeVisible();
    expect(screen.getByText('Three problems on choosing radius vs. height')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Why this?' }));
    expect(screen.getByText(/shifted axis/)).toBeVisible();
    expect(screen.getByText(/All 87/)).toBeVisible();
  });
});
