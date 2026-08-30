import { mockIPC } from '@tauri-apps/api/mocks';
import { describe, expect, it } from 'vitest';

import type {
  Concept,
  Goal,
  Material,
  MaterialResult,
  Module,
  Note,
  Session,
  Workspace,
} from '../types';
import { importSampleWorkspace } from './sampleWorkspaceService';

describe('sampleWorkspaceService', () => {
  it('sends the complete retained fixture graph with globally unique nested identities', async () => {
    let capturedSeed: Record<string, unknown> | undefined;
    mockIPC((command, payload) => {
      expect(command).toBe('importSampleWorkspace');
      capturedSeed = (payload as Record<string, unknown>).seed as Record<string, unknown>;
      return (capturedSeed.workspaces as Workspace[])[0];
    });

    const workspace = await importSampleWorkspace();
    const workspaces = capturedSeed?.workspaces as Workspace[];
    const goals = capturedSeed?.goals as Goal[];
    const concepts = capturedSeed?.concepts as Concept[];
    const modules = capturedSeed?.modules as Module[];
    const sessions = capturedSeed?.sessions as Session[];
    const materials = capturedSeed?.materials as Material[];
    const materialResults = capturedSeed?.materialResults as MaterialResult[];
    const notes = capturedSeed?.notes as Note[];
    const workspaceIds = new Set(workspaces.map((item) => item.id));
    const goalIds = new Set(goals.map((goal) => goal.id));
    const conceptIds = new Set(concepts.map((concept) => concept.id));
    const moduleIds = new Set(modules.map((module) => module.id));
    const exchangeIds = sessions.flatMap((session) =>
      session.exchanges.map((exchange) => exchange.id),
    );

    expect(workspace.id).toBe('workspace-calculus-ii');
    expect(concepts).toHaveLength(92);
    expect(workspaces).toHaveLength(3);
    expect(
      workspaces.every(
        (item) =>
          goalIds.has(item.guidingGoalId) &&
          item.enabledModuleIds.every((moduleId) => moduleIds.has(moduleId)),
      ),
    ).toBe(true);
    expect(goals.every((goal) => workspaceIds.has(goal.workspaceId))).toBe(true);
    expect(
      workspaces.every(
        (item) =>
          goals.filter((goal) => goal.workspaceId === item.id && goal.state === 'Guiding')
            .length === 1,
      ),
    ).toBe(true);
    expect(
      concepts.every(
        (concept) =>
          workspaceIds.has(concept.workspaceId) &&
          [
            ...concept.blocksConceptIds,
            ...concept.prerequisiteConceptIds,
            ...concept.relatedConceptIds,
            ...concept.leadsToConceptIds,
          ].every((conceptId) => conceptIds.has(conceptId)),
      ),
    ).toBe(true);
    expect(
      modules.every((module) =>
        (module.worksWithModuleIds ?? []).every((moduleId) => moduleIds.has(moduleId)),
      ),
    ).toBe(true);
    expect(
      sessions.every(
        (session) => workspaceIds.has(session.workspaceId) && conceptIds.has(session.conceptId),
      ),
    ).toBe(true);
    expect(new Set(exchangeIds).size).toBe(exchangeIds.length);
    expect(materials.every((material) => material.segments.length === 4)).toBe(true);
    expect(
      materialResults.every((result) => {
        const concept = concepts.find((candidate) => candidate.id === result.conceptId);
        return materials.some((material) => material.workspaceId === concept?.workspaceId);
      }),
    ).toBe(true);
    expect(
      notes.every((note) => workspaceIds.has(note.workspaceId) && conceptIds.has(note.conceptId)),
    ).toBe(true);
  });
});
