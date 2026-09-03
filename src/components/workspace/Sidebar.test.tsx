import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Sidebar } from './Sidebar';

const workspaces = [
  { id: 'calculus', name: 'Calculus II' },
  { id: 'linear-algebra', name: 'Linear Algebra' },
];

describe('Sidebar', () => {
  it('shows the intentional empty state instead of a blank tree when there are no workspaces', () => {
    render(<Sidebar workspaces={[]} />);

    expect(screen.getByText('No workspaces yet.')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Create workspace' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Explore a sample workspace' })).toBeVisible();
    // The always-populated tree affordance never renders alongside the empty state.
    expect(screen.queryByRole('button', { name: '+ New Workspace' })).toBeNull();
    // Global nav stays available even with no workspace yet.
    expect(screen.getByRole('button', { name: 'Home' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Marketplace' })).toBeVisible();
  });

  it('invokes the create-workspace handler from the empty state\'s primary action', () => {
    const onCreateWorkspace = vi.fn();
    render(<Sidebar workspaces={[]} onCreateWorkspace={onCreateWorkspace} />);

    fireEvent.click(screen.getByRole('button', { name: 'Create workspace' }));

    expect(onCreateWorkspace).toHaveBeenCalledOnce();
  });

  it('invokes the sample-import handler from the empty state\'s secondary action, and only on click', () => {
    const onExploreSample = vi.fn();
    render(<Sidebar workspaces={[]} onExploreSample={onExploreSample} />);

    expect(onExploreSample).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Explore a sample workspace' }));

    expect(onExploreSample).toHaveBeenCalledOnce();
  });

  it('disables the secondary action while a sample import is in flight', () => {
    render(<Sidebar workspaces={[]} exploringSample />);

    expect(screen.getByRole('button', { name: 'Explore a sample workspace' })).toBeDisabled();
  });

  it('surfaces a sample-import error next to the empty-state actions', () => {
    render(<Sidebar workspaces={[]} exploreSampleError="The sample workspace could not be prepared. Try again." />);

    expect(screen.getByRole('alert')).toHaveTextContent(
      'The sample workspace could not be prepared. Try again.',
    );
  });

  it('renders the real workspace tree, not the empty state, once a workspace exists', () => {
    render(<Sidebar workspaces={workspaces} />);

    expect(screen.queryByText('No workspaces yet.')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Create workspace' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Calculus II' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Linear Algebra' })).toBeVisible();
    expect(screen.getByRole('button', { name: '+ New Workspace' })).toBeVisible();
  });
});
