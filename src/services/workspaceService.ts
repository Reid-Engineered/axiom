import type { OfflineContentKind, Workspace } from '../types';
import { mockGoals } from './mockData/goals';
import { mockWorkspaces } from './mockData/workspaces';

export interface CreateWorkspaceInput {
  /** e.g. "Calculus II" (screen 2's Subject field). */
  subject: string;
  /** Verbatim natural-language text — inference happens server-side, never as form fields. */
  goalText: string;
}

/** All workspaces for the sidebar tree and Home's Workspace cards. */
export async function getWorkspaces(): Promise<Workspace[]> {
  return structuredClone(mockWorkspaces);
}

export async function getWorkspace(id: string): Promise<Workspace> {
  const workspace = mockWorkspaces.find((candidate) => candidate.id === id);
  if (!workspace) throw new Error(`Workspace not found: ${id}`);
  return structuredClone(workspace);
}

/** Screen 2 — Create Workspace. Also provisions the initial Guiding goal. */
export async function createWorkspace(input: CreateWorkspaceInput): Promise<Workspace> {
  const suffix = crypto.randomUUID();
  const goalId = `goal-${suffix}`;
  const workspace: Workspace = {
    id: `workspace-${suffix}`,
    name: input.subject.trim(),
    guidingGoalId: goalId,
    progress: 0,
    paused: false,
    offlineAvailability: [
      { kind: 'textbookAndLectureNotes', enabled: false, sizeBytes: 0 },
      { kind: 'problemBanks', enabled: false, sizeBytes: 0 },
      { kind: 'visualAssetsAndModuleData', enabled: false, sizeBytes: 0 },
      { kind: 'courseVideos', enabled: false, sizeBytes: 0 },
    ],
    enabledModuleIds: [],
  };
  const createdAt = new Date().toISOString();

  mockWorkspaces.push(workspace);
  mockGoals.push({
    id: goalId,
    workspaceId: workspace.id,
    text: input.goalText.trim(),
    state: 'Guiding',
    inferred: {},
    createdAt,
    updatedAt: createdAt,
  });

  return structuredClone(workspace);
}

/** Screen 21's "Make available offline" sheet — one call per per-kind toggle. */
export async function setWorkspaceOfflineAvailability(
  id: string,
  kind: OfflineContentKind,
  enabled: boolean,
): Promise<Workspace> {
  const workspace = mockWorkspaces.find((candidate) => candidate.id === id);
  if (!workspace) throw new Error(`Workspace not found: ${id}`);
  const availability = workspace.offlineAvailability.find((item) => item.kind === kind);
  if (!availability) throw new Error(`Offline content kind not found: ${kind}`);
  availability.enabled = enabled;
  return structuredClone(workspace);
}
