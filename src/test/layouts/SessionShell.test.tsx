import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { SessionShell } from '../../layouts/SessionShell';

describe('SessionShell', () => {
  it('renders toolbar, visualization, problem, and tutor panes', () => {
    render(
      <SessionShell
        toolbar={<div>Session Toolbar 44px</div>}
        visualization={<div>Visualization Pane</div>}
        problem={<div>Problem Working Area</div>}
        tutor={<div>Tutor Socratic Panel</div>}
      />
    );

    expect(screen.getByText('Session Toolbar 44px')).toBeInTheDocument();
    expect(screen.getByText('Visualization Pane')).toBeInTheDocument();
    expect(screen.getByText('Problem Working Area')).toBeInTheDocument();
    expect(screen.getByText('Tutor Socratic Panel')).toBeInTheDocument();
  });
});
