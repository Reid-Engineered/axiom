import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { GoalEditingSheet } from './GoalEditingSheet';

describe('GoalEditingSheet', () => {
  it('edits and reverts the real guiding-goal fixture while previewing consequences', async () => {
    const onClose = vi.fn();
    render(
      <GoalEditingSheet
        open
        workspaceId="workspace-calculus-ii"
        goalId="goal-calculus-exam"
        onClose={onClose}
      />,
    );
    const field = await screen.findByRole('textbox', { name: 'Goal' });
    await waitFor(() =>
      expect(field).toHaveValue(
        'Be ready to explain and solve every integration technique on the December final.',
      ),
    );
    expect(await screen.findByText(/Nothing is deleted/)).toBeVisible();
    expect(await screen.findByText('Deadline · December 12')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Revert' }));
    await waitFor(() => expect(field).toHaveValue('Pass the Calculus II final in December.'));
    fireEvent.change(field, { target: { value: 'Understand integration deeply.' } });
    fireEvent.click(screen.getByRole('button', { name: 'Update goal' }));
    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
  });

  it('allows inferred facets to be corrected without changing the goal text', async () => {
    render(
      <GoalEditingSheet
        open
        workspaceId="workspace-calculus-ii"
        goalId="goal-calculus-exam"
        onClose={vi.fn()}
      />,
    );
    expect(await screen.findByText('Tools · Practice, Visualizer, Tutor')).toBeVisible();
    fireEvent.click(
      screen.getByRole('button', { name: 'Remove Tools · Practice, Visualizer, Tutor' }),
    );
    expect(screen.queryByText('Tools · Practice, Visualizer, Tutor')).not.toBeInTheDocument();
    fireEvent.click(
      screen.getByRole('button', { name: 'Remove Pacing · Four focused sessions each week' }),
    );
    expect(screen.queryByText('Pacing · Four focused sessions each week')).not.toBeInTheDocument();
  });
});
