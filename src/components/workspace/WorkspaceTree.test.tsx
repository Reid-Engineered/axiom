import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { WorkspaceTree } from './WorkspaceTree';

const workspaces = [
  { id: 'calculus', name: 'Calculus II' },
  { id: 'linear-algebra', name: 'Linear Algebra' },
  { id: 'circuits', name: 'Circuit Analysis' },
];

describe('WorkspaceTree', () => {
  it('expands only the open workspace and never renders a third level', () => {
    render(
      <WorkspaceTree
        workspaces={workspaces}
        openWorkspaceId="calculus"
        activeSubItem="concepts"
      />,
    );

    expect(screen.getAllByRole('button')).toHaveLength(7);
    expect(screen.getByRole('button', { name: 'Calculus II' })).toHaveAttribute(
      'aria-expanded',
      'true',
    );
    expect(screen.getByRole('button', { name: 'Linear Algebra' })).toHaveAttribute(
      'aria-expanded',
      'false',
    );
    expect(screen.getByRole('button', { name: 'Concepts' })).toHaveAttribute(
      'aria-current',
      'page',
    );
    expect(screen.queryByRole('button', { name: /module/i })).toBeNull();
  });

  it('reports workspace and sub-item selections', () => {
    const onSelectWorkspace = vi.fn();
    const onSelectSubItem = vi.fn();
    const { container } = render(
      <WorkspaceTree
        workspaces={workspaces}
        openWorkspaceId="calculus"
        onSelectWorkspace={onSelectWorkspace}
        onSelectSubItem={onSelectSubItem}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Linear Algebra' }));
    fireEvent.click(within(container).getByRole('button', { name: 'Tools' }));
    expect(onSelectWorkspace).toHaveBeenCalledWith('linear-algebra');
    expect(onSelectSubItem).toHaveBeenCalledWith('tools');
  });
});
