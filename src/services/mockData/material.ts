import type { Material, MaterialResult } from '../../types';

export const mockMaterials: Material[] = [
  {
    id: 'material-stewart-calculus-9e',
    workspaceId: 'workspace-calculus-ii',
    title: 'Calculus',
    edition: '9th edition',
    totalPages: 712,
    totalChapters: 18,
    segments: [
      { label: 'Ch 6–7', status: 'read' },
      { label: 'Ch 8', status: 'inProgress' },
      { label: 'Ch 10–11', status: 'next', detail: '33 sections' },
      { label: 'Ch 12–18', status: 'outOfSyllabus' },
    ],
    highlightsCount: 41,
    notesCount: 18,
    mostMarkedSections: ['§7.3', '§8.2', '§11.4'],
  },
];

export const mockMaterialResults: MaterialResult[] = [
  {
    id: 'material-result-shell-section',
    kind: 'section',
    page: 442,
    title: '§7.3 · Volumes by Cylindrical Shells',
    reason:
      'The radius of the shell is the distance from the axis of revolution, not from the y-axis.',
    conceptId: 'calc-concept-22',
    inSyllabus: true,
    highlightedAt: '2026-10-28T14:00:00.000Z',
  },
  {
    id: 'material-result-shell-example',
    kind: 'workedExample',
    page: 446,
    title: 'Example 3 · region revolved about x = 2',
    reason: 'The worked case closest to the shell-radius mistake you keep making.',
    conceptId: 'calc-concept-22',
    inSyllabus: true,
  },
  {
    id: 'material-result-shell-exercises',
    kind: 'exerciseRange',
    page: 449,
    title: 'Exercises 7.3 · 21–34',
    reason: 'Shell exercises on non-zero axes, matched to your current radius practice.',
    conceptId: 'calc-concept-22',
    inSyllabus: true,
    exerciseTotal: 14,
    exerciseAttempted: 3,
  },
  {
    id: 'material-result-parts-section',
    kind: 'section',
    page: 478,
    title: '§8.1 · Integration by Parts',
    reason: 'Connects the product rule to the setup you have used reliably in recent work.',
    conceptId: 'calc-concept-14',
    inSyllabus: true,
  },
  {
    id: 'material-result-series-section',
    kind: 'section',
    page: 641,
    title: '§11.4 · The Comparison Tests',
    reason: 'Comes up if you look ahead to series convergence, outside this course’s syllabus.',
    conceptId: 'calc-concept-14',
    inSyllabus: false,
  },
];
