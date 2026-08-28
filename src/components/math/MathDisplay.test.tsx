import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { MathDisplay, type MathSegment } from './MathDisplay';

describe('MathDisplay', () => {
  it('renders a plain expression', () => {
    render(<MathDisplay expression="V = 2π∫ r(x)h(x) dx" />);
    expect(screen.getByText('V = 2π∫ r(x)h(x) dx')).toBeVisible();
  });

  it('exposes selectable expression segments', () => {
    const selected: MathSegment = { text: 'x', selected: true };
    const onSelect = vi.fn();
    render(<MathDisplay expression={[{ text: 'V = ' }, selected]} onSelectTerm={onSelect} />);
    fireEvent.click(screen.getByRole('button', { name: 'x' }));
    expect(onSelect).toHaveBeenCalledWith(selected);
  });
});
