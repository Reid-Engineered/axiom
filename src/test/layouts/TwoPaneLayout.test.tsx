import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { TwoPaneLayout } from '../../layouts/TwoPaneLayout';

describe('TwoPaneLayout', () => {
  it('renders main content and right rail simultaneously', () => {
    render(
      <TwoPaneLayout rail={<div>Right Rail Content</div>}>
        <div>Main Pane Content</div>
      </TwoPaneLayout>
    );

    expect(screen.getByText('Main Pane Content')).toBeInTheDocument();
    expect(screen.getByText('Right Rail Content')).toBeInTheDocument();
  });
});
