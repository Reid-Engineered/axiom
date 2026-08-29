import type { WorkspaceActivityEvent } from '../../types';

export const mockWorkspaceActivity: WorkspaceActivityEvent[] = [
  {
    id: 'physics-activity-1',
    workspaceId: 'workspace-physics',
    occurredAt: '2026-05-20T13:00:00.000Z',
    summary: 'Your course moved from forces into rotational dynamics.',
  },
  {
    id: 'physics-activity-2',
    workspaceId: 'workspace-physics',
    occurredAt: '2026-06-03T15:30:00.000Z',
    summary: 'Angular-momentum review was added to your guiding goal.',
  },
  {
    id: 'physics-activity-3',
    workspaceId: 'workspace-physics',
    occurredAt: '2026-06-18T11:15:00.000Z',
    summary: 'A worked rotation example was linked to Angular momentum.',
  },
];
