import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { SessionToolbar } from './SessionToolbar';
import { WorkingArea } from './WorkingArea';

describe('session components', () => {
  it('renders a five-dash toolbar and exposes its optional actions', () => {
    const onChangeIntent = vi.fn();
    const onPause = vi.fn();
    render(
      <SessionToolbar
        conceptName="Shell method"
        subjectLine="Calculus II"
        intent={{ activity: 'Practising', targetMinutes: 35 }}
        onChangeIntent={onChangeIntent}
        problemIndex={6}
        problemCount={12}
        elapsedMinutes={20}
        targetMinutes={35}
        onPause={onPause}
      />,
    );

    const progress = screen.getByLabelText('Problem 6 of 12');
    expect(progress.children).toHaveLength(5);
    expect(progress.querySelectorAll('[data-complete="true"]')).toHaveLength(3);
    fireEvent.click(screen.getByRole('button', { name: 'Change intent' }));
    fireEvent.click(screen.getByRole('button', { name: 'Pause' }));
    expect(onChangeIntent).toHaveBeenCalledOnce();
    expect(onPause).toHaveBeenCalledOnce();
  });

  it('renders WorkingArea as a controlled learner input', () => {
    const onChange = vi.fn();
    render(<WorkingArea value="r = x" onChange={onChange} />);
    fireEvent.change(screen.getByRole('textbox', { name: 'Your working' }), {
      target: { value: 'r = 2 − x' },
    });
    expect(onChange).toHaveBeenCalledWith('r = 2 − x');
  });
});
