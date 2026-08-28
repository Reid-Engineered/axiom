import type { Workspace } from '../../types';

export const mockWorkspaces: Workspace[] = [
  {
    id: 'workspace-calculus-ii',
    name: 'Calculus II',
    guidingGoalId: 'goal-calculus-exam',
    progress: 0.58,
    lastConceptName: 'Shell method',
    lastActivityAt: '2026-08-27T19:20:00.000Z',
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
    progress: 0.34,
    lastConceptName: 'Eigenvectors',
    lastActivityAt: '2026-08-23T14:10:00.000Z',
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
    progress: 0.76,
    lastConceptName: 'Angular momentum',
    lastActivityAt: '2026-05-12T16:45:00.000Z',
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
