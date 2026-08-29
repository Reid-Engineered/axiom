import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Inspector } from './Inspector';

describe('Inspector', () => {
  it('renders only while selected and remains dismissible', () => {
    const onClose = vi.fn();
    const { rerender } = render(
      <Inspector open={false} onClose={onClose} title="Selected shell">
        Shell details
      </Inspector>,
    );
    expect(screen.queryByRole('complementary')).not.toBeInTheDocument();

    rerender(
      <Inspector open onClose={onClose} title="Selected shell">
        Shell details
      </Inspector>,
    );
    expect(screen.getByRole('complementary', { name: 'Selected shell' })).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Close Selected shell' }));
    expect(onClose).toHaveBeenCalledOnce();
  });
});
