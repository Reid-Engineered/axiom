import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { mockWorkspaces } from '../../services/mockData/workspaces';
import { WorkspaceCard } from './WorkspaceCard';

describe('WorkspaceCard', () => {
  it('renders fixture progress context and selects the workspace', () => {
    const workspace = mockWorkspaces[0];
    const onSelect = vi.fn();
    render(
      <WorkspaceCard
        name={workspace.name}
        goalSentence="Prepare for the December final."
        progress={workspace.progress}
        lastConceptName={workspace.lastConceptName}
        lastActivityLabel="yesterday"
        onSelect={onSelect}
      />,
    );

    expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', String(workspace.progress));
    fireEvent.click(screen.getByRole('button', { name: /Calculus II/ }));
    expect(onSelect).toHaveBeenCalledOnce();
  });

  it('shows Paused instead of activity details', () => {
    const workspace = mockWorkspaces.find((item) => item.paused);
    if (!workspace) throw new Error('Paused fixture missing');
    render(<WorkspaceCard name={workspace.name} goalSentence="Retain mechanics." progress={workspace.progress} paused />);
    expect(screen.getByText('Paused')).toBeVisible();
  });
});
