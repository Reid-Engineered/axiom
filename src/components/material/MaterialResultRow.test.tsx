import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import type { MaterialResult } from '../../types';
import { MaterialResultRow } from './MaterialResultRow';

const baseResult: MaterialResult = {
  id: 'result',
  kind: 'section',
  page: 442,
  title: '§7.3 · Volumes by Cylindrical Shells',
  reason: 'The radius is measured from the axis.',
  conceptId: 'calc-concept-22',
  inSyllabus: true,
};

describe('MaterialResultRow', () => {
  it.each([
    ['section', 'Section'],
    ['workedExample', 'Worked example'],
    ['exerciseRange', 'Exercise range'],
  ] as const)('renders the %s result kind with resolved concept data', (kind, label) => {
    render(
      <MaterialResultRow
        result={{
          ...baseResult,
          kind,
          ...(kind === 'exerciseRange' ? { exerciseTotal: 14, exerciseAttempted: 3 } : {}),
        }}
        conceptName="Shell method"
        masteryState="Developing"
        onConceptSelect={() => undefined}
      />,
    );

    expect(screen.getByText(label)).toBeVisible();
    expect(screen.getByRole('button', { name: /Shell method Developing/ })).toBeVisible();
    expect(screen.getByTitle('Mastery: Developing')).toBeVisible();
    if (kind === 'exerciseRange')
      expect(screen.getByText('14 exercises · 3 attempted')).toBeVisible();
  });
});
