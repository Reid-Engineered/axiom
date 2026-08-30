import type { InvokeArgs } from '@tauri-apps/api/core';

import type {
  Concept,
  Goal,
  Material,
  MaterialResult,
  Module,
  Note,
  Session,
  Workspace,
  WorkspaceActivityEvent,
  WorkspaceTemplate,
} from '../types';
import { mockConcepts } from '../services/mockData/concepts';
import { mockGoals } from '../services/mockData/goals';
import { mockMaterialResults, mockMaterials } from '../services/mockData/material';
import { mockModules, mockWorkspaceTemplates } from '../services/mockData/modules';
import { mockNotes } from '../services/mockData/notes';
import { mockSessions } from '../services/mockData/sessions';
import { mockWorkspaceActivity } from '../services/mockData/workspaceActivity';
import { mockWorkspaces } from '../services/mockData/workspaces';

let concepts: Concept[];
let goals: Goal[];
let materialResults: MaterialResult[];
let materials: Material[];
let modules: Module[];
let notes: Note[];
let sessions: Session[];
let templates: WorkspaceTemplate[];
let workspaceActivity: WorkspaceActivityEvent[];
let workspaces: Workspace[];

export function resetMockBackend() {
  concepts = structuredClone(mockConcepts);
  goals = structuredClone(mockGoals);
  materialResults = structuredClone(mockMaterialResults);
  materials = structuredClone(mockMaterials);
  modules = structuredClone(mockModules);
  notes = structuredClone(mockNotes);
  sessions = structuredClone(mockSessions);
  templates = structuredClone(mockWorkspaceTemplates);
  workspaceActivity = structuredClone(mockWorkspaceActivity);
  workspaces = structuredClone(mockWorkspaces);
}

resetMockBackend();

function args(payload?: InvokeArgs): Record<string, unknown> {
  return (payload ?? {}) as Record<string, unknown>;
}

function findWorkspace(id: string) {
  const workspace = workspaces.find((candidate) => candidate.id === id);
  if (!workspace) throw new Error(`Workspace not found: ${id}`);
  return workspace;
}

function findGoal(id: string) {
  const goal = goals.find((candidate) => candidate.id === id);
  if (!goal) throw new Error(`Goal not found: ${id}`);
  return goal;
}

function findConcept(id: string) {
  const concept = concepts.find((candidate) => candidate.id === id);
  if (!concept) throw new Error(`Concept not found: ${id}`);
  return concept;
}

function findModule(id: string) {
  const module = modules.find((candidate) => candidate.id === id);
  if (!module) throw new Error(`Module not found: ${id}`);
  return module;
}

function findSession(id: string) {
  const session = sessions.find((candidate) => candidate.id === id);
  if (!session) throw new Error(`Session not found: ${id}`);
  return session;
}

function modulesForWorkspace(workspaceId: string) {
  const workspace = findWorkspace(workspaceId);
  return modules.map((module) => ({
    ...structuredClone(module),
    enabled: workspace.enabledModuleIds.includes(module.id),
  }));
}

function updateModuleEnabled(workspaceId: string, moduleId: string, enabled: boolean) {
  const workspace = findWorkspace(workspaceId);
  const module = findModule(moduleId);
  workspace.enabledModuleIds = enabled
    ? [...new Set([...workspace.enabledModuleIds, moduleId])]
    : workspace.enabledModuleIds.filter((id) => id !== moduleId);
  return { ...structuredClone(module), enabled };
}

function materialForWorkspace(workspaceId: string) {
  const material = materials.find((candidate) => candidate.workspaceId === workspaceId);
  if (!material) throw new Error(`Material not found for workspace: ${workspaceId}`);
  return material;
}

export function handleMockInvoke(command: string, payload?: InvokeArgs): unknown {
  const parameters = args(payload);

  switch (command) {
    case 'getWorkspaces':
      return structuredClone(workspaces);
    case 'getWorkspace':
      return structuredClone(findWorkspace(parameters.id as string));
    case 'getRecentActivity':
      return structuredClone(
        workspaceActivity
          .filter((event) => event.workspaceId === parameters.workspaceId)
          .sort((left, right) => left.occurredAt.localeCompare(right.occurredAt))
          .slice(0, 3),
      );
    case 'createWorkspace': {
      const input = parameters.input as { subject: string; goalText: string };
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
      workspaces.push(workspace);
      goals.push({
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
    case 'setWorkspaceOfflineAvailability': {
      const workspace = findWorkspace(parameters.id as string);
      const kind = parameters.kind as string;
      const availability = workspace.offlineAvailability.find((item) => item.kind === kind);
      if (!availability) throw new Error(`Offline content kind not found: ${kind}`);
      availability.enabled = parameters.enabled as boolean;
      return structuredClone(workspace);
    }
    case 'getGoal':
      return structuredClone(findGoal(parameters.id as string));
    case 'getGoalsByWorkspace':
      return structuredClone(goals.filter((goal) => goal.workspaceId === parameters.workspaceId));
    case 'updateGoal': {
      const goal = findGoal(parameters.id as string);
      goal.previousText = goal.text;
      goal.text = (parameters.text as string).trim();
      goal.updatedAt = new Date().toISOString();
      return structuredClone(goal);
    }
    case 'revertGoal': {
      const goal = findGoal(parameters.id as string);
      if (!goal.previousText) throw new Error(`Goal has no previous text: ${goal.id}`);
      const currentText = goal.text;
      goal.text = goal.previousText;
      goal.previousText = currentText;
      goal.updatedAt = new Date().toISOString();
      return structuredClone(goal);
    }
    case 'getConceptsByWorkspace':
      return structuredClone(
        concepts.filter((concept) => concept.workspaceId === parameters.workspaceId),
      );
    case 'getConcept':
      return structuredClone(findConcept(parameters.id as string));
    case 'searchConcepts': {
      const query = (parameters.query as string).trim().toLocaleLowerCase();
      const workspaceConcepts = concepts.filter(
        (concept) => concept.workspaceId === parameters.workspaceId,
      );
      if (!query) return structuredClone(workspaceConcepts);
      return structuredClone(
        workspaceConcepts.filter((concept) =>
          [concept.name, concept.chapter, concept.explanation, ...(concept.whereItShowsUp ?? [])]
            .filter((value): value is string => Boolean(value))
            .some((value) => value.toLocaleLowerCase().includes(query)),
        ),
      );
    }
    case 'getModulesByWorkspace':
      return structuredClone(modulesForWorkspace(parameters.workspaceId as string));
    case 'getMarketplaceModules':
      return structuredClone(
        parameters.forWorkspaceId
          ? modulesForWorkspace(parameters.forWorkspaceId as string)
          : modules,
      );
    case 'getWorkspaceTemplates':
      return structuredClone(templates);
    case 'getModule':
      return structuredClone(findModule(parameters.id as string));
    case 'installModule':
      return updateModuleEnabled(
        parameters.workspaceId as string,
        parameters.moduleId as string,
        true,
      );
    case 'setModuleEnabled':
      return updateModuleEnabled(
        parameters.workspaceId as string,
        parameters.moduleId as string,
        parameters.enabled as boolean,
      );
    case 'setModuleVisibility': {
      const module = findModule(parameters.moduleId as string);
      const visibility = parameters.visibility as Module['visibility'];
      module.visibility = visibility;
      return updateModuleEnabled(parameters.workspaceId as string, module.id, visibility !== 'off');
    }
    case 'getActiveSessionByWorkspace': {
      const session = sessions.find(
        (candidate) =>
          candidate.workspaceId === parameters.workspaceId && candidate.status !== 'completed',
      );
      return session ? structuredClone(session) : null;
    }
    case 'getSession':
      return structuredClone(findSession(parameters.id as string));
    case 'startSession': {
      const input = parameters.input as {
        workspaceId: string;
        conceptId: string;
        intent: Session['intent'];
      };
      findWorkspace(input.workspaceId);
      const concept = findConcept(input.conceptId);
      if (concept.workspaceId !== input.workspaceId) {
        throw new Error(`Concept not found in workspace: ${input.conceptId}`);
      }
      const session: Session = {
        id: `session-${crypto.randomUUID()}`,
        workspaceId: input.workspaceId,
        conceptId: input.conceptId,
        status: 'active',
        intent: structuredClone(input.intent),
        resumeSummary: `Ready to continue with ${concept.name}.`,
        elapsedMinutes: 0,
        exchanges: [],
        settledConclusions: [],
        startedAt: new Date().toISOString(),
      };
      sessions.push(session);
      return structuredClone(session);
    }
    case 'pauseSession': {
      const session = findSession(parameters.id as string);
      if (session.status === 'completed') throw new Error(`Session is completed: ${session.id}`);
      session.status = 'paused';
      session.pausedAt = new Date().toISOString();
      return structuredClone(session);
    }
    case 'resumeSession': {
      const session = findSession(parameters.id as string);
      if (session.status === 'completed') throw new Error(`Session is completed: ${session.id}`);
      session.status = 'active';
      delete session.pausedAt;
      return structuredClone(session);
    }
    case 'addTutorExchange': {
      const session = findSession(parameters.sessionId as string);
      if (session.status === 'completed') throw new Error(`Session is completed: ${session.id}`);
      session.exchanges.push({
        id: `exchange-${crypto.randomUUID()}`,
        question: (parameters.question as string).trim(),
        answer:
          'Start with what the current representation makes visible, then test one step against the goal.',
        occurredAt: new Date().toISOString(),
        pinnedToVisualization: false,
      });
      return structuredClone(session);
    }
    case 'endSession': {
      const session = findSession(parameters.id as string);
      session.status = 'completed';
      delete session.pausedAt;
      return structuredClone(session);
    }
    case 'getMaterial':
      return structuredClone(materialForWorkspace(parameters.workspaceId as string));
    case 'searchMaterial': {
      const material = materialForWorkspace(parameters.workspaceId as string);
      const terms = (parameters.query as string)
        .trim()
        .toLocaleLowerCase()
        .split(/\s+/)
        .filter(Boolean);
      return structuredClone(
        materialResults.filter((result) => {
          if (!result.inSyllabus) return false;
          const concept = findConcept(result.conceptId);
          if (concept.workspaceId !== material.workspaceId) return false;
          const searchable = `${result.title} ${result.reason}`.toLocaleLowerCase();
          return terms.every((term) => searchable.includes(term));
        }),
      );
    }
    case 'getRecentNotes':
      return structuredClone(
        notes
          .filter((note) => note.workspaceId === parameters.workspaceId)
          .sort((left, right) => right.updatedAt.localeCompare(left.updatedAt)),
      );
    default:
      throw new Error(`Unhandled test command: ${command}`);
  }
}
