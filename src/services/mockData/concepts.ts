import type { Concept, MasteryState } from '../../types';

const calculusChapters = [
  '1 · Review of Functions',
  '2 · Limits and Continuity',
  '3 · Differentiation',
  '4 · Integration Foundations',
  '5 · Techniques of Integration',
  '6 · Applications of Integration',
  '7 · Differential Equations',
  '8 · Sequences and Series',
  '9 · Parametric and Polar Curves',
];

const conceptNames = [
  'Function composition',
  'Inverse functions',
  'Trigonometric identities',
  'Limit laws',
  'Continuity',
  'Intermediate value theorem',
  'Product rule',
  'Chain rule',
  'Implicit differentiation',
  'Antiderivatives',
  'Riemann sums',
  'Fundamental theorem',
  'Substitution',
  'Integration by parts',
  'Trigonometric integrals',
  'Trig substitution',
  'Partial fractions',
  'Improper integrals',
  'Area between curves',
  'Disk method',
  'Washer method',
  'Shell method',
  'Arc length',
  'Surface area',
  'Work',
  'Average value',
  'Separable equations',
  'Exponential growth',
  'Logistic models',
  'Slope fields',
  'Sequence convergence',
  'Geometric series',
  'Integral test',
  'Comparison tests',
  'Alternating series',
  'Power series',
  'Taylor series',
  'Parametric derivatives',
  'Polar coordinates',
  'Polar area',
];

const masteryStates: MasteryState[] = ['New', 'Developing', 'Familiar', 'Strong', 'Mastered'];

const calculusConcepts: Concept[] = Array.from({ length: 87 }, (_, index) => {
  const id = `calc-concept-${index + 1}`;
  const previousId = index > 0 ? `calc-concept-${index}` : undefined;
  const nextIds = [index + 1, index + 2, index + 3]
    .filter((target) => target < 87)
    .map((target) => `calc-concept-${target + 1}`);
  const relatedIds = [index - 2, index + 2]
    .filter((target) => target >= 0 && target < 87)
    .map((target) => `calc-concept-${target + 1}`);
  const masteryState = masteryStates[index % masteryStates.length];
  const decayed = index % 19 === 0 && masteryState !== 'New';

  return {
    id,
    workspaceId: 'workspace-calculus-ii',
    name:
      conceptNames[index % conceptNames.length] +
      (index >= conceptNames.length ? ` ${Math.floor(index / conceptNames.length) + 1}` : ''),
    chapter: calculusChapters[Math.floor(index / 10)],
    masteryState: index === 13 ? 'Mastered' : masteryState,
    ...(decayed
      ? { wasMasteryState: 'Strong' as const, decayedAt: '2026-08-15T12:00:00.000Z' }
      : {}),
    meaning:
      masteryState === 'New'
        ? 'Not yet explored in this workspace.'
        : masteryState === 'Developing'
          ? 'Works with support but not yet independently.'
          : masteryState === 'Familiar'
            ? 'Recognized and applied in familiar problems.'
            : masteryState === 'Strong'
              ? 'Held up across different problem forms.'
              : 'Explained and applied reliably weeks apart.',
    ...(index === 13
      ? { dueForReviewInDays: 2 }
      : index % 8 === 0
        ? { dueForReviewInDays: 0 }
        : index % 7 === 0
          ? { dueForReviewInDays: 4 }
          : {}),
    onExam: index < 22 || index % 6 === 0,
    blocksConceptIds: nextIds,
    prerequisiteConceptIds:
      index === 13 ? ['calc-concept-7', 'calc-concept-13'] : previousId ? [previousId] : [],
    relatedConceptIds: relatedIds,
    leadsToConceptIds: index === 13 ? ['calc-concept-15', 'calc-concept-37'] : nextIds.slice(0, 2),
    ...(index === 13
      ? {
          meaning: 'Held up weeks apart without review.',
          displayFormula: '∫ u dv = uv − ∫ v du',
          explanation:
            'Integration by parts is the product rule read backward. You trade one integral for another, choosing u so the new integral is easier than the one you started with.',
          learnerHeuristic: 'Differentiate the messy part; integrate the easy part.',
          heuristicEvidence: 'That heuristic held up in four of your last five problems.',
          whereItShowsUp: [
            'Reduction formulas',
            'Taylor remainder',
            'Laplace transforms',
            'Work integrals',
          ],
          recentDiagnostics: [
            {
              id: 'diagnostic-parts-log',
              expression: '∫ x ln x dx',
              type: 'positive' as const,
              note: 'Correct on the first try.',
              occurredAt: '2026-08-27T18:10:00.000Z',
            },
            {
              id: 'diagnostic-parts-exponential',
              expression: '∫ x²eˣ dx',
              type: 'positive' as const,
              note: 'Correct after two passes.',
              occurredAt: '2026-08-27T18:18:00.000Z',
            },
            {
              id: 'diagnostic-parts-choice',
              expression: '∫ eˣ sin x dx',
              type: 'mistake' as const,
              note: 'Chose u backwards.',
              occurredAt: '2026-08-27T18:24:00.000Z',
            },
          ],
        }
      : {}),
    ...(index === 21
      ? {
          displayFormula: 'V = 2π∫ r(x)h(x) dx',
          explanation:
            'Cylindrical shells accumulate circumference times height across the radius.',
          learnerHeuristic: 'Shells are easier when the slices stay parallel to the axis.',
          heuristicEvidence: 'This held for three mixed axis-of-rotation problems.',
          whereItShowsUp: ['Volume by rotation', 'Physical modeling', 'Comparing setup methods'],
          recentDiagnostics: [
            {
              id: 'diagnostic-shell-radius',
              expression: '2πx(4-x²)',
              type: 'positive' as const,
              note: 'Radius and shell height were identified correctly.',
              occurredAt: '2026-08-27T19:10:00.000Z',
            },
            {
              id: 'diagnostic-shell-bounds',
              expression: '∫₀²',
              type: 'mistake' as const,
              note: 'Bounds were copied before checking the intersection.',
              occurredAt: '2026-08-27T19:14:00.000Z',
            },
          ],
        }
      : {}),
    lastActivityAt: `2026-08-${String(28 - (index % 20)).padStart(2, '0')}T15:00:00.000Z`,
    notesCount: index % 5,
  };
});

const supportingConcepts: Concept[] = [
  ['linear-concept-1', 'workspace-linear-algebra', 'Linear combinations', '1 · Vectors'],
  ['linear-concept-2', 'workspace-linear-algebra', 'Span and independence', '1 · Vectors'],
  ['linear-concept-3', 'workspace-linear-algebra', 'Eigenvectors', '4 · Eigenvalues'],
  ['physics-concept-1', 'workspace-physics', 'Newton’s second law', '2 · Forces'],
  ['physics-concept-2', 'workspace-physics', 'Angular momentum', '7 · Rotation'],
].map(([id, workspaceId, name, chapter], index, entries) => ({
  id,
  workspaceId,
  name,
  chapter,
  masteryState:
    id === 'physics-concept-2' ? 'Developing' : masteryStates[index % masteryStates.length],
  ...(id === 'physics-concept-2'
    ? { wasMasteryState: 'Strong' as const, decayedAt: '2026-07-10T12:00:00.000Z' }
    : {}),
  meaning:
    id === 'physics-concept-2'
      ? 'The setup needs a short refresh after time away.'
      : 'Applied successfully in recent work.',
  onExam: true,
  blocksConceptIds:
    index + 1 < entries.length && entries[index + 1][1] === workspaceId
      ? [entries[index + 1][0]]
      : [],
  prerequisiteConceptIds:
    index > 0 && entries[index - 1][1] === workspaceId ? [entries[index - 1][0]] : [],
  relatedConceptIds: [],
  leadsToConceptIds:
    index + 1 < entries.length && entries[index + 1][1] === workspaceId
      ? [entries[index + 1][0]]
      : [],
  notesCount: index % 3,
}));

export const mockConcepts: Concept[] = [...calculusConcepts, ...supportingConcepts];
