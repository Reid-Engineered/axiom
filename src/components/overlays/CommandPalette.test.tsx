import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { CommandPalette, CommandPaletteText } from './CommandPalette';

describe('CommandPalette', () => {
  it('renders only when open and reports query changes', () => {
    const onQueryChange = vi.fn();
    const { rerender } = render(
      <CommandPalette
        open={false}
        onClose={() => {}}
        query=""
        onQueryChange={onQueryChange}
        groups={[]}
      />,
    );

    expect(screen.queryByRole('dialog')).toBeNull();
    rerender(
      <CommandPalette open onClose={() => {}} query="" onQueryChange={onQueryChange} groups={[]} />,
    );
    fireEvent.change(screen.getByRole('textbox', { name: 'Search commands' }), {
      target: { value: 'shell' },
    });
    expect(onQueryChange).toHaveBeenCalledWith('shell');
  });

  it('closes on Escape', () => {
    const onClose = vi.fn();
    render(<CommandPalette open onClose={onClose} query="" onQueryChange={() => {}} groups={[]} />);

    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it('renders real group labels, consequences, shortcuts, and the key legend', () => {
    render(
      <CommandPalette
        open
        onClose={() => {}}
        query="shell"
        onQueryChange={() => {}}
        scopeLabel="Calculus II"
        groups={[
          {
            label: 'Actions',
            items: [
              {
                id: 'practice',
                content: (
                  <CommandPaletteText
                    label="Practice the Shell Method"
                    detail="12 problems · adaptive"
                  />
                ),
                shortcut: '⏎',
                onSelect: () => {},
              },
            ],
          },
        ]}
      />,
    );

    expect(screen.getByText('Actions')).toBeVisible();
    expect(screen.getByText('Practice the Shell Method')).toBeVisible();
    expect(screen.getByText('12 problems · adaptive')).toBeVisible();
    expect(screen.getByText('↑↓ move · ⏎ run · ⇥ scope · esc close')).toBeVisible();
  });
});
