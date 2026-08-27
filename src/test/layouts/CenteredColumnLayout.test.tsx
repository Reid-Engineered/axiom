import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CenteredColumnLayout } from '../../layouts/CenteredColumnLayout';

describe('CenteredColumnLayout', () => {
  it('renders children within centered column', () => {
    render(
      <CenteredColumnLayout>
        <div>Content Inside Column</div>
      </CenteredColumnLayout>
    );

    expect(screen.getByText('Content Inside Column')).toBeInTheDocument();
  });

  it('applies default width class', () => {
    const { container } = render(
      <CenteredColumnLayout width="default">
        <div>Default Column</div>
      </CenteredColumnLayout>
    );

    const column = container.querySelector('[class*="widthDefault"]');
    expect(column).toBeInTheDocument();
  });

  it('applies wide width class when specified', () => {
    const { container } = render(
      <CenteredColumnLayout width="wide">
        <div>Wide Column</div>
      </CenteredColumnLayout>
    );

    const column = container.querySelector('[class*="widthWide"]');
    expect(column).toBeInTheDocument();
  });
});
