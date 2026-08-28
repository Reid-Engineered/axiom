import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Sheet } from './Sheet';

describe('Sheet', () => {
  it('renders conditionally and exposes a close control', () => {
    const onClose = vi.fn();
    const { rerender } = render(<Sheet open={false} onClose={onClose} title="Offline">Body</Sheet>);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    rerender(<Sheet open onClose={onClose} title="Offline">Body</Sheet>);
    fireEvent.click(screen.getByRole('button', { name: 'Close sheet' }));
    expect(onClose).toHaveBeenCalledOnce();
  });
});
