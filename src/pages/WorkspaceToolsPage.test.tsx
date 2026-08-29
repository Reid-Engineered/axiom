import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { WorkspaceToolsPage } from './WorkspaceToolsPage';

describe('WorkspaceToolsPage', () => {
  it('groups the real 20 modules and reports the real four workspace goals', async () => {
    render(
      <NavigationProvider
        initialRoute={{ type: 'workspaceTools', workspaceId: 'workspace-calculus-ii' }}
      >
        <WorkspaceToolsPage workspaceId="workspace-calculus-ii" />
      </NavigationProvider>,
    );
    await waitFor(() => expect(screen.getByText('20 modules · 4 goals')).toBeVisible());
    expect(screen.getByRole('heading', { name: 'In the workspace · 4' })).toBeVisible();
    expect(screen.getByRole('heading', { name: 'Appear when relevant · 9' })).toBeVisible();
    expect(screen.getByRole('heading', { name: 'Off in this workspace · 7' })).toBeVisible();
  });

  it('offers one offline sheet with four per-kind fixture toggles', async () => {
    render(
      <NavigationProvider
        initialRoute={{ type: 'workspaceTools', workspaceId: 'workspace-calculus-ii' }}
      >
        <WorkspaceToolsPage workspaceId="workspace-calculus-ii" />
      </NavigationProvider>,
    );
    await waitFor(() => expect(screen.getByText('20 modules · 4 goals')).toBeVisible());
    fireEvent.click(screen.getByRole('button', { name: 'Make available offline' }));
    const sheet = screen.getByRole('dialog', { name: 'Make available offline' });
    expect(within(sheet).getAllByRole('switch')).toHaveLength(4);
    expect(within(sheet).getByText(/9 of 32 downloadable/)).toBeVisible();
    expect(within(sheet).getByRole('heading', { name: 'Voice tutoring' })).toBeVisible();
    expect(within(sheet).getByText('Internet required')).toBeVisible();
  });
});
