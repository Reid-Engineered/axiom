import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { ReasonedRecommendation } from './ReasonedRecommendation';
import { SuggestionPanel } from './SuggestionPanel';

describe('feedback components', () => {
  it('expands recommendation evidence and starts the action', () => {
    const onStart = vi.fn();
    render(<ReasonedRecommendation action="Practise shell radius" evidence="Two setups used the wrong radius." ctaLabel="Start · 8 min" onStart={onStart} observations={[{ date: 'Tuesday', text: 'The axis moved.' }]} />);
    fireEvent.click(screen.getByRole('button', { name: 'Why this?' }));
    expect(screen.getByText('The axis moved.')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Start · 8 min' }));
    expect(onStart).toHaveBeenCalledOnce();
  });

  it('exposes accept and dismiss suggestion actions', () => {
    const onAccept = vi.fn();
    const onDismiss = vi.fn();
    render(<SuggestionPanel message="Series is next in the course." acceptLabel="Add to plan" onAccept={onAccept} onDismiss={onDismiss} />);
    fireEvent.click(screen.getByRole('button', { name: 'Add to plan' }));
    fireEvent.click(screen.getByRole('button', { name: 'Not now' }));
    expect(onAccept).toHaveBeenCalledOnce();
    expect(onDismiss).toHaveBeenCalledOnce();
  });
});
