import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { ConceptTag } from './ConceptTag';

describe('ConceptTag', () => {
  it('renders as text or as an interactive tag from the locked props', () => {
    const onSelect = vi.fn();
    const { rerender } = render(<ConceptTag label="Reduction formulas" />);
    expect(screen.getByText('Reduction formulas').tagName).toBe('SPAN');

    rerender(<ConceptTag label="Reduction formulas" onSelect={onSelect} />);
    fireEvent.click(screen.getByRole('button', { name: 'Reduction formulas' }));
    expect(onSelect).toHaveBeenCalledOnce();
  });
});
