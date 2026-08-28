import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { mockConcepts } from '../../services/mockData/concepts';
import { ConceptRow } from './ConceptRow';

describe('ConceptRow', () => {
  it('renders fixture mastery with its word and handles selection', () => {
    const concept = mockConcepts[21];
    const onSelect = vi.fn();
    render(<ConceptRow name={concept.name} masteryState={concept.masteryState} statusText="active" onSelect={onSelect} />);
    expect(screen.getByText(concept.masteryState)).toBeVisible();
    expect(screen.getByText('active')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: /Shell method/ }));
    expect(onSelect).toHaveBeenCalledOnce();
  });
});
