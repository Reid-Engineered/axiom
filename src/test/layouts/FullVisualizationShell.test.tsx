import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { FullVisualizationShell } from '../../layouts/FullVisualizationShell';

describe('FullVisualizationShell', () => {
  it('renders header actions and full-bleed scene children', () => {
    render(
      <FullVisualizationShell header={<div>‹ Session Navigation</div>}>
        <div>Full Bleed 3D Scene</div>
      </FullVisualizationShell>
    );

    expect(screen.getByText('‹ Session Navigation')).toBeInTheDocument();
    expect(screen.getByText('Full Bleed 3D Scene')).toBeInTheDocument();
  });
});
