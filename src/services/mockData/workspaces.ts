import type { Workspace } from '../../types';

export const mockWorkspaces: Workspace[] = [
  {
    id: 'workspace-calculus-ii',
    name: 'Calculus II',
    guidingGoalId: 'goal-calculus-exam',
    progress: 0,
    lastConceptName: 'Shell method',
    createdAt: '2026-08-18T13:00:00.000Z',
    paused: false,
    offlineAvailability: [
      { kind: 'textbookAndLectureNotes', enabled: true, sizeBytes: 880803840 },
      { kind: 'problemBanks', enabled: true, sizeBytes: 125829120 },
      { kind: 'visualAssetsAndModuleData', enabled: true, sizeBytes: 429916160 },
      {
        kind: 'courseVideos',
        enabled: false,
        sizeBytes: 2254857830,
        partial: {
          availableCount: 9,
          totalCount: 32,
          limitReason: 'The remaining videos are streamed by your school.',
        },
      },
    ],
    enabledModuleIds: Array.from({ length: 13 }, (_, index) => `module-${index + 1}`),
  },
  {
    id: 'workspace-linear-algebra',
    name: 'Linear Algebra',
    guidingGoalId: 'goal-linear-algebra-proof',
    progress: 0,
    lastConceptName: 'Eigenvectors',
    createdAt: '2026-08-19T13:00:00.000Z',
    paused: false,
    offlineAvailability: [
      { kind: 'textbookAndLectureNotes', enabled: false, sizeBytes: 524288000 },
      { kind: 'problemBanks', enabled: false, sizeBytes: 94371840 },
      { kind: 'visualAssetsAndModuleData', enabled: false, sizeBytes: 314572800 },
      { kind: 'courseVideos', enabled: false, sizeBytes: 1572864000 },
    ],
    enabledModuleIds: ['module-1', 'module-3', 'module-5', 'module-8'],
  },
  {
    id: 'workspace-physics',
    name: 'Mechanics',
    guidingGoalId: 'goal-physics-review',
    progress: 0,
    lastConceptName: 'Angular momentum',
    createdAt: '2026-05-01T13:00:00.000Z',
    paused: true,
    offlineAvailability: [
      { kind: 'textbookAndLectureNotes', enabled: true, sizeBytes: 734003200 },
      { kind: 'problemBanks', enabled: true, sizeBytes: 104857600 },
      { kind: 'visualAssetsAndModuleData', enabled: false, sizeBytes: 367001600 },
      { kind: 'courseVideos', enabled: false, sizeBytes: 1887436800 },
    ],
    enabledModuleIds: ['module-1', 'module-2', 'module-4'],
  },
];
