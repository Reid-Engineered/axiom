import type { Session, TutorExchange } from '../../types';

const longTutorHistory: TutorExchange[] = Array.from({ length: 40 }, (_, index) => ({
  id: `exchange-shell-${index + 1}`,
  question: index % 3 === 0
    ? `How should I identify the shell radius in example ${index + 1}?`
    : index % 3 === 1
      ? `Why does the height change at this boundary in example ${index + 1}?`
      : `Can I check this setup before integrating example ${index + 1}?`,
  answer: index % 3 === 0
    ? 'Measure from the axis of rotation to the representative shell, then keep that direction consistent.'
    : index % 3 === 1
      ? 'The top or bottom curve changes where the region’s boundary changes; split the integral there.'
      : 'Check radius, height, thickness, and bounds against the picture before simplifying.',
  occurredAt: `2026-08-27T${String(17 + Math.floor(index / 12)).padStart(2, '0')}:${String((index * 5) % 60).padStart(2, '0')}:00.000Z`,
  pinnedToVisualization: index === 18 || index === 34,
}));

export const mockSessions: Session[] = [
  {
    id: 'session-shell-method',
    workspaceId: 'workspace-calculus-ii',
    conceptId: 'calc-concept-22',
    status: 'paused',
    intent: { activity: 'Practising', detail: 'Set up shell-method integrals', targetMinutes: 35 },
    resumeSummary: 'You had drawn the shell radius and were checking where the height changes.',
    thumbnailUrl: '/mock-assets/shell-method.svg',
    elapsedMinutes: 47,
    problemIndex: 6,
    problemCount: 12,
    exchanges: longTutorHistory,
    settledConclusions: [
      'The shell radius is measured from the axis of rotation.',
      'A changing boundary means the height expression may need separate integrals.',
    ],
    openQuestion: 'How can the intersection be found without solving both curves explicitly?',
    startedAt: '2026-08-27T17:00:00.000Z',
    pausedAt: '2026-08-27T20:27:00.000Z',
  },
  {
    id: 'session-integration-by-parts',
    workspaceId: 'workspace-calculus-ii',
    conceptId: 'calc-concept-14',
    status: 'completed',
    intent: { activity: 'Reviewing', detail: 'Choose u and dv deliberately', targetMinutes: 20 },
    resumeSummary: 'Completed a mixed set and wrote a choice rule in your own words.',
    elapsedMinutes: 24,
    problemIndex: 8,
    problemCount: 8,
    exchanges: longTutorHistory.slice(0, 4),
    settledConclusions: ['Choose u so its derivative simplifies the product.'],
    startedAt: '2026-08-24T14:00:00.000Z',
  },
  {
    id: 'session-eigenvectors',
    workspaceId: 'workspace-linear-algebra',
    conceptId: 'linear-concept-3',
    status: 'active',
    intent: { activity: 'Exploring', detail: 'Connect geometry to the characteristic equation' },
    resumeSummary: 'You were comparing invariant directions before computing a determinant.',
    elapsedMinutes: 18,
    exchanges: longTutorHistory.slice(0, 3).map((exchange, index) => ({
      ...exchange,
      id: `exchange-eigen-${index + 1}`,
    })),
    settledConclusions: ['Eigenvectors keep their direction under the transformation.'],
    openQuestion: 'What does a repeated eigenvalue imply geometrically?',
    startedAt: '2026-08-28T13:00:00.000Z',
  },
];
