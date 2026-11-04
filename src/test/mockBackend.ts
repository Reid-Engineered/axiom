import type { InvokeArgs } from '@tauri-apps/api/core';

import type {
  AttemptState,
  AttemptStatus,
  Attribution,
  Available,
  BloomEvidence,
  Concept,
  ConceptEvidence,
  CreatePlanInput,
  EditPatternRuleInput,
  Goal,
  MasterySnapshot,
  Material,
  MaterialResult,
  Module,
  Note,
  PatternDetail,
  PatternListResponse,
  PatternSnapshot,
  Plan,
  PracticeDifficulty,
  ResponseType,
  Session,
  SessionActionInput,
  SessionActionReceipt,
  SessionHistoryEntry,
  SessionSummary,
  SessionTiming,
  Stamp,
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

interface MockAttempt {
  id: string;
  prompt: string;
  responseType: ResponseType;
  hintTexts: string[];
  hintsRevealed: number;
  status: AttemptStatus;
  submissionCount: number;
}

let mockAttempts: Map<string, MockAttempt>;
let mockPlans: Map<string, Plan>;
let mockAttemptStates: Map<string, AttemptState>;
let mockMasterySnapshots: Map<string, MasterySnapshot>;
let mockBloomSnapshots: Map<string, BloomEvidence>;
let mockPatterns: Map<string, PatternSnapshot>;
let mockPatternAttributions: Map<string, Attribution[]>;
let mockSessionTimings: Map<string, SessionTiming>;
let mockSessionSummaries: Map<string, SessionSummary>;
let mockReceipts: Map<string, SessionActionReceipt>;
let mockTotalAnswers: number;
let mockSinceDate: string | null;

const MOCK_POLICY_REF = {
  id: 'policy.reid_default',
  revision: 1,
  digest: 'sha256:9b5781d0a8a3e5a638f18ef40219c6a09446e0470b14eb93eb5f07c7bf01b102',
};

function makeMockStamp(workspaceId: string, capabilityId: string, evidenceIds: string[] = []): Stamp {
  return {
    schemaVersion: 1,
    workspaceId,
    provider: {
      moduleId: 'org.axiom.practice',
      moduleVersion: '0.1.0',
      capabilityId,
      capabilityVersion: 1,
    },
    policyRef: MOCK_POLICY_REF,
    contentRef: {
      packageId: 'org.axiom.reference.calculus.cylindrical_shells',
      packageVersion: '0.3.0',
      schemaVersion: 1,
      digest: 'sha256:org.axiom.reference.calculus.cylindrical_shells:0.3.0:v1',
    },
    evidenceIds,
    recordedAt: new Date().toISOString(),
  };
}

const MOCK_PRACTICE_FAMILY = {
  prompt:
    'A region is bounded by y = 4x - x^2 and the x-axis. Find the volume when revolved about the y-axis.',
  responseType: 'numeric' as ResponseType,
  correctValue: 42.7,
  hints: ['Set up the shell method integral.', 'Integrate from x = 0 to x = 4.'],
};

/** Mirrors the Rust seed's crosswalk: only the Shell method concept has practice content. */
const MAPPED_CONCEPT_NAMES = new Set(['Shell method']);

function createMockAttempt(): MockAttempt {
  const attempt: MockAttempt = {
    id: `attempt-${crypto.randomUUID()}`,
    prompt: MOCK_PRACTICE_FAMILY.prompt,
    responseType: MOCK_PRACTICE_FAMILY.responseType,
    hintTexts: [...MOCK_PRACTICE_FAMILY.hints],
    hintsRevealed: 0,
    status: 'open',
    submissionCount: 0,
  };
  mockAttempts.set(attempt.id, attempt);
  return attempt;
}

export function resetMockBackend() {
  mockAttempts = new Map();
  mockPlans = new Map();
  mockAttemptStates = new Map();
  mockMasterySnapshots = new Map();
  mockBloomSnapshots = new Map();
  mockPatterns = new Map();
  mockPatternAttributions = new Map();
  mockSessionTimings = new Map();
  mockSessionSummaries = new Map();
  mockReceipts = new Map();
  mockTotalAnswers = 0;
  mockSinceDate = null;
  concepts = structuredClone(mockConcepts);
  goals = structuredClone(mockGoals);
  materialResults = structuredClone(mockMaterialResults);
  materials = structuredClone(mockMaterials);
  modules = structuredClone(mockModules);
  notes = structuredClone(mockNotes);
  sessions = [];
  templates = structuredClone(mockWorkspaceTemplates);
  workspaceActivity = structuredClone(mockWorkspaceActivity);
  workspaces = structuredClone(mockWorkspaces);
}

resetMockBackend();

/** Loads explicit learner activity for tests whose scenario requires an existing session. */
export function loadMockSessionsForTest(nextSessions: Session[] = mockSessions) {
  sessions = structuredClone(nextSessions);
}

function args(payload?: InvokeArgs): Record<string, unknown> {
  return (payload ?? {}) as Record<string, unknown>;
}

function findWorkspace(id: string) {
  const resolvedId = id === 'workspace-calculus' ? 'workspace-calculus-ii' : id;
  const workspace = workspaces.find((candidate) => candidate.id === resolvedId);
  if (!workspace) throw new Error(`Workspace not found: ${id}`);
  return workspace;
}

function findGoal(id: string) {
  const goal = goals.find((candidate) => candidate.id === id);
  if (!goal) throw new Error(`Goal not found: ${id}`);
  return goal;
}

function findConcept(id: string) {
  const resolvedId = id === 'concept-shells' ? 'calc-concept-22' : id;
  const concept = concepts.find((candidate) => candidate.id === resolvedId);
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

function touchSession(session: Session) {
  const activityAt = new Date().toISOString();
  session.lastActivityAt = activityAt;
  findWorkspace(session.workspaceId).lastActivityAt = activityAt;
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
    case 'importSampleWorkspace':
      return structuredClone(findWorkspace('workspace-calculus-ii'));
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
        createdAt: new Date().toISOString(),
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
      const session = sessions
        .filter(
          (candidate) =>
            candidate.workspaceId === parameters.workspaceId && candidate.status !== 'completed',
        )
        .sort((left, right) => {
          if (left.lastActivityAt && right.lastActivityAt) {
            return right.lastActivityAt.localeCompare(left.lastActivityAt);
          }
          if (left.lastActivityAt) return -1;
          if (right.lastActivityAt) return 1;
          return sessions.indexOf(right) - sessions.indexOf(left);
        })[0];
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
      const workspace = findWorkspace(input.workspaceId);
      const concept = findConcept(input.conceptId);
      if (concept.workspaceId !== workspace.id && concept.workspaceId !== input.workspaceId) {
        throw new Error(`Concept not found in workspace: ${input.conceptId}`);
      }
      const existing = sessions
        .filter(
          (candidate) =>
            candidate.workspaceId === input.workspaceId &&
            candidate.conceptId === input.conceptId &&
            candidate.status !== 'completed',
        )
        .sort((left, right) => {
          if (left.lastActivityAt && right.lastActivityAt) {
            return right.lastActivityAt.localeCompare(left.lastActivityAt);
          }
          if (left.lastActivityAt) return -1;
          if (right.lastActivityAt) return 1;
          return sessions.indexOf(right) - sessions.indexOf(left);
        })[0];
      if (existing) {
        const attempt = existing.currentAttemptId
          ? mockAttempts.get(existing.currentAttemptId)
          : undefined;
        if (
          MAPPED_CONCEPT_NAMES.has(concept.name) &&
          (!existing.currentAttemptId || attempt?.status === 'solved')
        ) {
          existing.currentAttemptId = createMockAttempt().id;
          existing.problemIndex = (existing.problemIndex ?? 0) + 1;
        }
        existing.status = 'active';
        delete existing.pausedAt;
        touchSession(existing);
        return structuredClone(existing);
      }
      const activityAt = new Date().toISOString();
      const session: Session = {
        id: `session-${crypto.randomUUID()}`,
        workspaceId: input.workspaceId,
        conceptId: input.conceptId,
        currentAttemptId: MAPPED_CONCEPT_NAMES.has(concept.name)
          ? createMockAttempt().id
          : undefined,
        status: 'active',
        intent: structuredClone(input.intent),
        resumeSummary: `Ready to continue with ${concept.name}.`,
        elapsedMinutes: 0,
        problemIndex: MAPPED_CONCEPT_NAMES.has(concept.name) ? 1 : undefined,
        exchanges: [],
        settledConclusions: [],
        startedAt: new Date().toISOString(),
        lastActivityAt: activityAt,
      };
      sessions.push(session);
      workspace.lastActivityAt = activityAt;
      return structuredClone(session);
    }
    case 'pauseSession': {
      const session = findSession(parameters.id as string);
      if (session.status === 'completed') throw new Error(`Session is completed: ${session.id}`);
      session.status = 'paused';
      session.pausedAt = new Date().toISOString();
      touchSession(session);
      return structuredClone(session);
    }
    case 'resumeSession': {
      const session = findSession(parameters.id as string);
      if (session.status === 'completed') throw new Error(`Session is completed: ${session.id}`);
      session.status = 'active';
      delete session.pausedAt;
      touchSession(session);
      return structuredClone(session);
    }
    case 'addTutorExchange': {
      const session = findSession(parameters.sessionId as string);
      if (session.status === 'completed') throw new Error(`Session is completed: ${session.id}`);
      return {
        ...structuredClone(session),
        tutorStatus: 'unavailable',
      };
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
    case 'generateAttempt': {
      const input = parameters.input as { workspaceId: string; familyId: string };
      findWorkspace(input.workspaceId);
      const attempt = createMockAttempt();
      return {
        attemptId: attempt.id,
        prompt: attempt.prompt,
        responseType: attempt.responseType,
        hintsTotal: attempt.hintTexts.length,
      };
    }
    case 'describeAttempt': {
      const input = parameters.input as { workspaceId: string; attemptId: string };
      const attempt = mockAttempts.get(input.attemptId);
      if (!attempt) throw new Error(`Attempt not found: ${input.attemptId}`);
      return {
        prompt: attempt.prompt,
        responseType: attempt.responseType,
        hintsTotal: attempt.hintTexts.length,
        hintsRevealed: attempt.hintsRevealed,
        status: attempt.status,
        submissionCount: attempt.submissionCount,
      };
    }
    case 'nextProblem': {
      const session = findSession(parameters.sessionId as string);
      const concept = findConcept(session.conceptId);
      if (MAPPED_CONCEPT_NAMES.has(concept.name)) {
        session.currentAttemptId = createMockAttempt().id;
        session.problemIndex = (session.problemIndex ?? 1) + 1;
      }
      touchSession(session);
      return structuredClone(session);
    }
    case 'evaluateAttempt': {
      const input = parameters.input as {
        workspaceId: string;
        attemptId: string;
        response: { responseType: string; value: string | number };
      };
      const attempt = mockAttempts.get(input.attemptId);
      if (!attempt) throw new Error(`Attempt not found: ${input.attemptId}`);
      if (attempt.status === 'solved') throw new Error(`Attempt already solved: ${input.attemptId}`);
      const submitted = Number(input.response.value);
      const correct = Math.abs(submitted - MOCK_PRACTICE_FAMILY.correctValue) <= 1e-6;
      attempt.submissionCount += 1;
      if (correct) attempt.status = 'solved';
      const session = sessions.find((candidate) => candidate.currentAttemptId === input.attemptId);
      if (session) touchSession(session);
      return {
        correct,
        status: attempt.status,
        submissionCount: attempt.submissionCount,
      };
    }
    case 'requestHint': {
      const input = parameters.input as { workspaceId: string; attemptId: string };
      const attempt = mockAttempts.get(input.attemptId);
      if (!attempt) throw new Error(`Attempt not found: ${input.attemptId}`);
      if (attempt.hintsRevealed >= attempt.hintTexts.length) {
        throw new Error(`No more hints for attempt: ${input.attemptId}`);
      }
      attempt.hintsRevealed += 1;
      return {
        hintText: attempt.hintTexts[attempt.hintsRevealed - 1],
        hintsRevealed: attempt.hintsRevealed,
        hintsTotal: attempt.hintTexts.length,
      };
    }
    case 'createPracticePlan': {
      const input = parameters.input as CreatePlanInput;
      findWorkspace(input.workspaceId);
      const difficulty: PracticeDifficulty = input.difficulty ?? 'Steady';
      const targetMs = Math.min(2_700_000, Math.max(600_000, input.targetMs ?? 1_500_000));
      const limit = Math.min(8, Math.max(3, input.questionLimit ?? 5));
      const guidingGoal = goals.find(
        (g) => g.workspaceId === input.workspaceId && g.state === 'Guiding',
      );
      const planId = `plan-${input.operationId}`;
      const phases: Plan['phases'] = [
        {
          id: 'phase-warmup',
          name: 'Warm-up',
          reason: 'Recall core setup before full volume evaluation.',
          evidenceIds: [],
        },
        {
          id: 'phase-stretch',
          name: 'Stretch',
          reason: input.patternId
            ? `Targeted practice watching for pattern ${input.patternId}.`
            : 'Build unassisted fluency on active syllabus concepts.',
          evidenceIds: [],
        },
        {
          id: 'phase-combined',
          name: 'Combined',
          reason: 'Multi-objective synthesis combining setup and evaluation.',
          evidenceIds: [],
        },
      ];
      const queue: Plan['queue'] = Array.from({ length: limit }, (_, idx) => {
        const phaseId =
          idx === 0 ? 'phase-warmup' : idx === limit - 1 ? 'phase-combined' : 'phase-stretch';
        return {
          id: `slot-${idx + 1}`,
          phaseId,
          conceptId: input.conceptIds?.[0] ?? 'calc-concept-22',
          objectiveIds: [
            'shell.setup_radius_height_y_axis',
            'shell.compute_volume_y_axis_single_curve',
          ],
          familyId: 'problem.shell_y_poly',
          seed: String(17 + idx * 25),
          attemptId: null,
          state: 'pending',
          reason:
            phaseId === 'phase-warmup'
              ? 'Selected for Warm-up: verify shell radius and height setup.'
              : phaseId === 'phase-combined'
                ? 'Selected for Combined synthesis across setup and integration objectives.'
                : 'Selected for Stretch: build unassisted accuracy on cylindrical shells.',
          evidenceIds: [],
          replacesId: null,
        };
      });
      const plan: Plan = {
        id: planId,
        revision: 1,
        goalId: guidingGoal?.id ?? null,
        goalSnapshot: guidingGoal
          ? {
              id: guidingGoal.id,
              text: guidingGoal.text,
              state: guidingGoal.state,
              inferred: { ...guidingGoal.inferred },
              updatedAt: guidingGoal.updatedAt,
            }
          : null,
        targetMs,
        difficulty,
        policyRef: MOCK_POLICY_REF,
        phases,
        queue,
        changes: [],
        availability: 'limited',
        limitations: [
          'Single verified problem family available in package; seeds vary across queue slots.',
        ],
      };
      if (!input.previewOnly) {
        mockPlans.set(planId, structuredClone(plan));
      }
      return {
        state: 'available',
        value: structuredClone(plan),
        stamp: makeMockStamp(input.workspaceId, 'practice.plan'),
      } satisfies Available<Plan>;
    }
    case 'getConceptEvidence': {
      const input = parameters.input as {
        workspaceId: string;
        conceptIds?: string[];
        asOf?: string | null;
      };
      const wsConcepts = concepts.filter((c) => c.workspaceId === input.workspaceId);
      const targetIds =
        input.conceptIds && input.conceptIds.length > 0
          ? input.conceptIds
          : wsConcepts.map((c) => c.id);
      return targetIds.map((conceptId): ConceptEvidence => {
        const snap = mockMasterySnapshots.get(`${input.workspaceId}:${conceptId}`);
        const bloom = mockBloomSnapshots.get(`${input.workspaceId}:${conceptId}`);
        const concept = wsConcepts.find((c) => c.id === conceptId);
        const prereqIds = concept?.prerequisiteConceptIds ?? [];
        const prereqs = prereqIds.map((pid) => {
          const ps = mockMasterySnapshots.get(`${input.workspaceId}:${pid}`);
          return {
            conceptId: pid,
            snapshotId: ps?.id ?? null,
            state: (ps ? (ps.score >= 0.6 ? 'ready' : 'not-ready') : 'unknown') as
              | 'ready'
              | 'not-ready'
              | 'unknown',
          };
        });
        const readinessState: 'ready' | 'not-ready' | 'unknown' =
          prereqs.length === 0
            ? 'ready'
            : prereqs.some((p) => p.state === 'not-ready')
              ? 'not-ready'
              : prereqs.some((p) => p.state === 'unknown')
                ? 'unknown'
                : 'ready';
        return {
          conceptId,
          mastery: snap
            ? {
                state: 'available',
                value: structuredClone(snap),
                stamp: makeMockStamp(input.workspaceId, 'practice.evidence', [snap.id]),
              }
            : { state: 'unavailable', reason: 'no-evidence', lastKnown: null },
          bloom: bloom
            ? {
                state: 'available',
                value: structuredClone(bloom),
                stamp: makeMockStamp(input.workspaceId, 'practice.evidence'),
              }
            : { state: 'unavailable', reason: 'no-evidence', lastKnown: null },
          readiness: {
            state: 'available',
            value: {
              state: readinessState,
              prerequisites: prereqs,
              reason:
                prereqs.length === 0
                  ? 'No prerequisite concepts required.'
                  : readinessState === 'ready'
                    ? 'All prerequisite concepts meet the 0.60 readiness threshold.'
                    : 'Prerequisite readiness requires observed practice evidence.',
            },
            stamp: makeMockStamp(input.workspaceId, 'practice.evidence'),
          },
        };
      });
    }
    case 'getPracticePatterns': {
      const input = parameters.input as {
        workspaceId: string;
        status?: string | null;
        search?: string | null;
      };
      let list = [...mockPatterns.values()];
      if (input.status && input.status !== 'All') {
        list = list.filter((p) => p.status.toLowerCase() === input.status!.toLowerCase());
      }
      if (input.search && input.search.trim()) {
        const q = input.search.trim().toLowerCase();
        list = list.filter(
          (p) => p.name.toLowerCase().includes(q) || p.patternId.toLowerCase().includes(q),
        );
      }
      return {
        patterns:
          list.length === 0 && mockTotalAnswers === 0
            ? { state: 'unavailable', reason: 'no-evidence', lastKnown: null }
            : {
                state: 'available',
                value: structuredClone(list),
                stamp: makeMockStamp(input.workspaceId, 'practice.patterns'),
              },
        totalAnswers: mockTotalAnswers,
        sinceDate: mockSinceDate,
        requiredCleanDefault: 3,
        nextCursor: null,
      } satisfies PatternListResponse;
    }
    case 'getPatternDetail': {
      const input = parameters.input as {
        workspaceId: string;
        patternId: string;
      };
      const snap = mockPatterns.get(input.patternId);
      if (!snap) {
        return {
          state: 'unavailable',
          reason: 'no-evidence',
          lastKnown: null,
        } satisfies Available<PatternDetail>;
      }
      const attrs = mockPatternAttributions.get(input.patternId) ?? [];
      const rem = Math.max(0, snap.resolution.requiredClean - snap.cleanStreak);
      return {
        state: 'available',
        value: {
          snapshot: structuredClone(snap),
          explanation: `Pattern ${snap.name} tracked across ${snap.opportunities} eligible opportunities.`,
          nextAction:
            rem === 0
              ? 'Pattern is resolved under the current rule.'
              : `Complete ${rem} more consecutive clean unassisted problem${rem === 1 ? '' : 's'} on this pattern to mark it Resolved.`,
          attributions: structuredClone(attrs),
          sessionBuckets: [
            {
              sessionId: 'session-1',
              recordedAt: snap.lastSeenAt ?? new Date().toISOString(),
              opportunities: snap.opportunities,
              occurrences: snap.occurrences,
              cleanCount: snap.cleanStreak,
              unknownCount: snap.unknownCount,
            },
          ],
        },
        stamp: makeMockStamp(input.workspaceId, 'practice.patterns'),
      } satisfies Available<PatternDetail>;
    }
    case 'editPatternRule': {
      const input = parameters.input as EditPatternRuleInput;
      if (input.requiredClean < 2 || input.requiredClean > 5) {
        throw new Error('requiredClean must be between 2 and 5');
      }
      const existing = mockPatterns.get(input.patternId) ?? {
        id: `patsnap-${input.patternId}`,
        patternId: input.patternId,
        name: input.patternId,
        status: 'Unknown' as const,
        opportunities: 0,
        occurrences: 0,
        unknownCount: 0,
        cleanStreak: 0,
        firstSeenAt: null,
        lastSeenAt: null,
        recentAttributionIds: [],
        ruleRevision: input.ruleRevision,
        resolution: {
          requiredClean: input.requiredClean,
          eligibleClean: 0,
          explanation: '',
        },
      };
      const updated: PatternSnapshot = {
        ...structuredClone(existing),
        ruleRevision: existing.ruleRevision + 1,
        cleanStreak: 0,
        status: existing.occurrences >= 2 ? 'Recurring' : existing.occurrences === 1 ? 'Improving' : 'Unknown',
        resolution: {
          requiredClean: input.requiredClean,
          eligibleClean: 0,
          explanation: `Rule updated to require ${input.requiredClean} consecutive clean unassisted attempts (streak reset at revision ${existing.ruleRevision + 1}).`,
        },
      };
      if (!input.previewOnly) {
        mockPatterns.set(input.patternId, structuredClone(updated));
      }
      return {
        state: 'available',
        value: updated,
        stamp: makeMockStamp(input.workspaceId, 'practice.patterns'),
      } satisfies Available<PatternSnapshot>;
    }
    case 'getSessionHistory': {
      const workspaceId = parameters.workspaceId as string;
      return sessions
        .filter((s) => s.workspaceId === workspaceId)
        .map((s): SessionHistoryEntry => {
          const sum = mockSessionSummaries.get(s.id);
          return {
            sessionId: s.id,
            workspaceId: s.workspaceId,
            status: s.status,
            startedAt: s.startedAt,
            completedAt: s.status === 'completed' ? (s.lastActivityAt ?? s.startedAt) : null,
            difficulty: 'Steady',
            summary: sum
              ? {
                  state: 'available',
                  value: structuredClone(sum),
                  stamp: makeMockStamp(workspaceId, 'practice.session'),
                }
              : { state: 'unavailable', reason: 'no-evidence', lastKnown: null },
          };
        });
    }
    case 'executeSessionAction': {
      const input = parameters.input as SessionActionInput;
      return handleMockSessionAction(input);
    }
    default:
      throw new Error(`Unhandled test command: ${command}`);
  }
}

function getOrCreateMockSessionPlan(workspaceId: string, sessionId: string): Plan {
  const existing = mockPlans.get(`session-plan-${sessionId}`);
  if (existing) return existing;
  const created = handleMockInvoke('createPracticePlan', {
    input: {
      workspaceId,
      operationId: `auto-${sessionId}`,
    },
  }) as Available<Plan>;
  if (created.state !== 'available') {
    throw new Error('Could not create mock plan');
  }
  mockPlans.set(`session-plan-${sessionId}`, created.value);
  return created.value;
}

function ensureMockSlotAttempt(
  plan: Plan,
  slotIndex: number,
  sessionId?: string,
): AttemptState {
  const slot = plan.queue[slotIndex];
  if (slot.attemptId && mockAttemptStates.has(slot.attemptId)) {
    return mockAttemptStates.get(slot.attemptId)!;
  }
  const existingSession = sessionId ? sessions.find((s) => s.id === sessionId) : undefined;
  const legacyAttempt =
    slotIndex === 0 && existingSession?.currentAttemptId
      ? mockAttempts.get(existingSession.currentAttemptId)
      : undefined;
  const attempt = legacyAttempt ?? createMockAttempt();
  const hintBudget =
    plan.difficulty === 'Steady' ? 3 : plan.difficulty === 'Push' ? 1 : 0;
  const state: AttemptState = {
    id: attempt.id,
    slotId: slot.id,
    status: attempt.status === 'solved' ? 'solved' : 'open',
    instanceRef: `problem.shell_y_poly:${slot.seed}`,
    prompt: attempt.prompt,
    responseType: attempt.responseType,
    restrictions: [],
    bloomTarget: 'apply',
    draftRevision: 0,
    answerText: '',
    workingText: '',
    hintsTotal: attempt.hintTexts.length,
    hintsRevealed: attempt.hintsRevealed,
    hintBudget: Math.min(hintBudget, attempt.hintTexts.length),
    revealedHints: attempt.hintTexts.slice(0, attempt.hintsRevealed),
    canonicalSolutionDisplay: attempt.status === 'solved' ? '42.7' : null,
    workedSolution:
      attempt.status === 'solved'
        ? 'V = 2*pi * integral_0^4 x*(4x - x^2) dx = 128*pi/3 ≈ 42.7'
        : null,
    submissionIds: [],
    lastFeedbackId: null,
    lastVerification: null,
  };
  slot.attemptId = state.id;
  if (slot.state === 'pending') {
    slot.state = 'current';
  }
  mockAttemptStates.set(state.id, state);
  return state;
}

function handleMockSessionAction(input: SessionActionInput): SessionActionReceipt {
  const receiptKey = `${input.workspaceId}:${input.sessionId}:${input.operationId}`;
  if (input.action !== 'read' && mockReceipts.has(receiptKey)) {
    return structuredClone(mockReceipts.get(receiptKey)!);
  }

  const now = new Date().toISOString();
  const payload = input.payload ?? {};
  const planIdFromPayload =
    (payload.planId as string | undefined) ??
    (payload.plan_id as string | undefined);
  const plan =
    (planIdFromPayload ? mockPlans.get(planIdFromPayload) : undefined) ??
    getOrCreateMockSessionPlan(input.workspaceId, input.sessionId);
  mockPlans.set(`session-plan-${input.sessionId}`, plan);

  let session = sessions.find((s) => s.id === input.sessionId);
  if (!session && input.action === 'start') {
    session = {
      id: input.sessionId,
      workspaceId: input.workspaceId,
      conceptId: plan.queue[0]?.conceptId ?? 'calc-concept-22',
      status: 'active',
      intent: { activity: 'Practising', targetMinutes: 25 },
      resumeSummary: 'Adaptive practice session in progress.',
      elapsedMinutes: 0,
      problemIndex: 1,
      problemCount: plan.queue.length,
      exchanges: [],
      settledConclusions: [],
      startedAt: now,
      lastActivityAt: now,
    };
    sessions.push(session);
  }

  const prevTiming = mockSessionTimings.get(input.sessionId) ?? {
    elapsedMs: 0,
    attemptElapsedMs: 0,
    targetMs: plan.targetMs,
    checkpointSequence: 0,
    active: true,
    measuredAt: now,
    recovered: false,
  };
  const dEl = Math.max(0, input.elapsedDeltaMs ?? 0);
  const dAtt = Math.max(0, input.attemptElapsedDeltaMs ?? dEl);
  const timing: SessionTiming = {
    elapsedMs: prevTiming.elapsedMs + dEl,
    attemptElapsedMs:
      input.action === 'advance' || input.action === 'skip'
        ? 0
        : prevTiming.attemptElapsedMs + dAtt,
    targetMs: plan.targetMs,
    checkpointSequence: prevTiming.checkpointSequence + 1,
    active:
      input.action === 'pause' || input.action === 'end'
        ? false
        : input.action === 'resume' || input.action === 'start'
          ? true
          : prevTiming.active,
    measuredAt: now,
    recovered: false,
  };
  mockSessionTimings.set(input.sessionId, timing);

  const activeIdx =
    plan.queue.findIndex((s) => s.state === 'current') !== -1
      ? plan.queue.findIndex((s) => s.state === 'current')
      : plan.queue.findIndex((s) => s.state === 'pending');

  let currentAttempt: AttemptState | null =
    activeIdx >= 0 ? ensureMockSlotAttempt(plan, activeIdx, input.sessionId) : null;
  if (session && currentAttempt) {
    session.currentAttemptId = currentAttempt.id;
  }

  const masteryChanges: Available<MasterySnapshot>[] = [];
  const bloomChanges: Available<BloomEvidence>[] = [];
  const patternChanges: Available<PatternSnapshot>[] = [];

  if (input.action === 'save-working' && currentAttempt) {
    const draftRev = Number(payload.draftRevision ?? currentAttempt.draftRevision + 1);
    if (draftRev >= currentAttempt.draftRevision) {
      currentAttempt.draftRevision = draftRev;
      if (typeof payload.answerText === 'string') {
        currentAttempt.answerText = payload.answerText;
      }
      if (typeof payload.workingText === 'string') {
        currentAttempt.workingText = payload.workingText;
      }
    }
  } else if (input.action === 'hint' && currentAttempt) {
    const legacy = mockAttempts.get(currentAttempt.id);
    if (
      currentAttempt.hintsRevealed >= currentAttempt.hintBudget ||
      currentAttempt.hintsRevealed >= currentAttempt.hintsTotal
    ) {
      throw new Error('budget-exhausted: hint budget exhausted');
    }
    currentAttempt.hintsRevealed += 1;
    if (legacy) {
      legacy.hintsRevealed = currentAttempt.hintsRevealed;
      currentAttempt.revealedHints = legacy.hintTexts.slice(0, currentAttempt.hintsRevealed);
    }
  } else if (input.action === 'reveal-solution' && currentAttempt) {
    const legacy = mockAttempts.get(currentAttempt.id);
    currentAttempt.hintsRevealed = currentAttempt.hintsTotal;
    if (legacy) {
      legacy.hintsRevealed = currentAttempt.hintsTotal;
      legacy.status = 'solved';
      currentAttempt.revealedHints = [...legacy.hintTexts];
    }
    currentAttempt.status = 'solved';
    currentAttempt.canonicalSolutionDisplay = String(MOCK_PRACTICE_FAMILY.correctValue);
    currentAttempt.workedSolution =
      'V = 2*pi * integral_0^4 x*(4x - x^2) dx = 128*pi/3 \u2248 42.7';
    if (activeIdx >= 0) {
      plan.queue[activeIdx].state = 'miss';
    }
  } else if (input.action === 'check' && currentAttempt) {
    const resp = payload.response as { responseType?: string; value?: string | number } | undefined;
    const rawVal = resp?.value ?? currentAttempt.answerText;
    currentAttempt.answerText = String(rawVal ?? '');
    const numVal = Number(rawVal);
    const isFiniteNum = Number.isFinite(numVal) && String(rawVal).trim() !== '';
    const isCorrect =
      isFiniteNum && Math.abs(numVal - MOCK_PRACTICE_FAMILY.correctValue) <= 1e-6;
    const verdict: 'correct' | 'incorrect' | 'unverifiable' = !isFiniteNum
      ? 'unverifiable'
      : isCorrect
        ? 'correct'
        : 'incorrect';

    const verId = `ver-${crypto.randomUUID()}`;
    currentAttempt.lastFeedbackId = verId;
    currentAttempt.submissionIds.push(`sub-${crypto.randomUUID()}`);
    currentAttempt.lastVerification = {
      id: verId,
      verdict,
      method: 'numeric-tolerance',
      checkerVersion: '1.0.0',
      absoluteTolerance: 1e-9,
      relativeTolerance: 1e-9,
      canonicalValue: MOCK_PRACTICE_FAMILY.correctValue,
      studentValue: isFiniteNum ? numVal : null,
      restrictions: [],
      parseError: !isFiniteNum ? `Could not parse numeric response "${String(rawVal)}"` : null,
      limitations: ['finite-constant-evaluation-only'],
      objectiveResults: [
        {
          objectiveId: 'shell.setup_radius_height_y_axis',
          verdict,
          evidence: isCorrect ? 'numeric-tolerance-match' : 'numeric-tolerance-mismatch',
        },
      ],
      samples: null,
      terms: null,
    };

    const legacy = mockAttempts.get(currentAttempt.id);
    if (legacy) {
      legacy.submissionCount += 1;
      if (isCorrect) legacy.status = 'solved';
    }
    if (isCorrect) {
      currentAttempt.status = 'solved';
      currentAttempt.canonicalSolutionDisplay = String(MOCK_PRACTICE_FAMILY.correctValue);
      currentAttempt.workedSolution =
        'V = 2*pi * integral_0^4 x*(4x - x^2) dx = 128*pi/3 \u2248 42.7';
      if (activeIdx >= 0) plan.queue[activeIdx].state = 'correct';
    }

    if (verdict !== 'unverifiable') {
      mockTotalAnswers += 1;
      if (!mockSinceDate) mockSinceDate = now;
      const cid = activeIdx >= 0 ? plan.queue[activeIdx].conceptId : 'calc-concept-22';
      const prevSnap = mockMasterySnapshots.get(`${input.workspaceId}:${cid}`);
      const hintCost =
        currentAttempt.hintsRevealed *
        (plan.difficulty === 'Steady' ? 0.2 : plan.difficulty === 'Push' ? 0.35 : 1.0);
      const effCredit = isCorrect ? Math.max(0, 1.0 - hintCost) : 0.0;
      const prevScore = prevSnap?.score ?? null;
      const newScore = prevScore !== null ? 0.75 * prevScore + 0.25 * effCredit : 0.25 * effCredit;
      const snap: MasterySnapshot = {
        id: `msnap-${crypto.randomUUID()}`,
        conceptId: cid,
        objectiveIds: ['shell.setup_radius_height_y_axis'],
        score: newScore,
        previousSnapshotId: prevSnap?.id ?? null,
        previousScore: prevScore,
        delta: prevScore !== null ? newScore - prevScore : null,
        namedState:
          newScore >= 0.85
            ? 'Mastered'
            : newScore >= 0.7
              ? 'Strong'
              : newScore >= 0.45
                ? 'Familiar'
                : 'Developing',
        asOf: now,
        lastPracticedAt: now,
        decay: null,
      };
      mockMasterySnapshots.set(`${input.workspaceId}:${cid}`, snap);
      masteryChanges.push({
        state: 'available',
        value: structuredClone(snap),
        stamp: makeMockStamp(input.workspaceId, 'practice.evidence', [verId]),
      });

      if (!isCorrect) {
        const patId = 'pattern.shell_missing_radius_factor';
        const existingPat = mockPatterns.get(patId);
        const occ = (existingPat?.occurrences ?? 0) + 1;
        const opps = (existingPat?.opportunities ?? 0) + 1;
        const patSnap: PatternSnapshot = {
          id: `patsnap-${patId}`,
          patternId: patId,
          name: 'Omitting shell radius factor r(x) = x in integrand',
          conceptIds: [cid],
          status: 'Recurring',
          opportunities: opps,
          occurrences: occ,
          unknownCount: 0,
          cleanStreak: 0,
          firstSeenAt: existingPat?.firstSeenAt ?? now,
          lastSeenAt: now,
          recentAttributionIds: [verId],
          ruleRevision: existingPat?.ruleRevision ?? 1,
          resolution: {
            requiredClean: existingPat?.resolution.requiredClean ?? 3,
            eligibleClean: 0,
            explanation: `0 / ${existingPat?.resolution.requiredClean ?? 3} consecutive clean unassisted opportunities.`,
          },
        };
        mockPatterns.set(patId, patSnap);
        const list = mockPatternAttributions.get(patId) ?? [];
        list.unshift({
          id: `attr-${crypto.randomUUID()}`,
          attemptId: currentAttempt.id,
          patternId: patId,
          ruleRevision: patSnap.ruleRevision,
          opportunityId: 'opp.problem.shell_y_poly',
          eligible: true,
          outcome: 'observed',
          evidenceIds: [verId],
          detectorId: 'det.shell_missing_radius',
          detectorVersion: 1,
          explanation: 'Answer omits multiplying by shell radius r(x) = x.',
          recordedAt: now,
          answerText: currentAttempt.answerText,
          conceptId: cid,
          sessionId: input.sessionId,
        });
        mockPatternAttributions.set(patId, list);
        patternChanges.push({
          state: 'available',
          value: structuredClone(patSnap),
          stamp: makeMockStamp(input.workspaceId, 'practice.patterns', [verId]),
        });

        // P6: at most 1 queue replacement per session
        if (plan.changes.length === 0) {
          const pendingIdx = plan.queue.findIndex((s) => s.state === 'pending');
          if (pendingIdx !== -1) {
            const oldSlot = plan.queue[pendingIdx];
            const addedId = `${oldSlot.id}-rep`;
            const reason = `Queue adjusted after observing "${patSnap.name}": replaced pending slot ${oldSlot.id} with a targeted follow-up check.`;
            plan.queue[pendingIdx] = {
              ...oldSlot,
              id: addedId,
              seed: String(Number(oldSlot.seed) + 19),
              reason,
              replacesId: oldSlot.id,
            };
            plan.changes.push({
              eventId: verId,
              removedSlotId: oldSlot.id,
              addedSlotId: addedId,
              reason,
            });
            plan.revision += 1;
          }
        }
      }
    }
  } else if (input.action === 'skip' || input.action === 'advance') {
    if (activeIdx >= 0) {
      if (input.action === 'skip' && currentAttempt?.status === 'open') {
        currentAttempt.status = 'skipped';
        plan.queue[activeIdx].state = 'skipped';
      } else if (plan.queue[activeIdx].state === 'current') {
        plan.queue[activeIdx].state =
          currentAttempt?.status === 'solved' ? 'correct' : 'skipped';
      }
    }
    const nextPending = plan.queue.findIndex((s) => s.state === 'pending');
    if (nextPending >= 0) {
      currentAttempt = ensureMockSlotAttempt(plan, nextPending, input.sessionId);
      if (session) {
        session.currentAttemptId = currentAttempt.id;
        session.problemIndex = nextPending + 1;
      }
    } else {
      currentAttempt = null;
    }
  } else if (input.action === 'edit-plan') {
    if (typeof payload.difficulty === 'string') {
      plan.difficulty = payload.difficulty as PracticeDifficulty;
    }
    if (typeof payload.targetMs === 'number') {
      plan.targetMs = payload.targetMs;
    }
    plan.revision += 1;
  }

  let summaryAvailable: Available<SessionSummary> = {
    state: 'unavailable',
    reason: 'no-evidence',
    lastKnown: null,
  };

  if (input.action === 'end' || ( (input.action === 'skip' || input.action === 'advance') && !currentAttempt )) {
    if (session) {
      session.status = 'completed';
      delete session.pausedAt;
    }
    const existingSum = mockSessionSummaries.get(input.sessionId);
    if (existingSum) {
      summaryAvailable = {
        state: 'available',
        value: structuredClone(existingSum),
        stamp: makeMockStamp(input.workspaceId, 'practice.session'),
      };
    } else {
      const correctCount = plan.queue.filter((s) => s.state === 'correct').length;
      const skippedCount = plan.queue.filter((s) => s.state === 'skipped').length;
      const assessedCount = plan.queue.filter((s) =>
        ['correct', 'miss', 'partial'].includes(s.state),
      ).length;
      const sum: SessionSummary = {
        sessionId: input.sessionId,
        completedAt: now,
        timing: {
          state: 'available',
          value: structuredClone(timing),
          stamp: makeMockStamp(input.workspaceId, 'practice.session'),
        },
        difficultyHistory: plan.queue.map((s) => ({
          slotId: s.id,
          difficulty: plan.difficulty,
          policyRef: MOCK_POLICY_REF,
        })),
        correct: correctCount,
        assessedTotal: assessedCount,
        attemptedTotal: assessedCount,
        skippedTotal: skippedCount,
        unverifiableTotal: 0,
        hintsUsed: 0,
        hintBudgetTotal: plan.queue.length * 3,
        masteryChanges,
        repeatedPatternIds: [...mockPatterns.keys()],
        questionRows: plan.queue
          .filter((s) => s.attemptId)
          .map((s) => ({
            slotId: s.id,
            attemptId: s.attemptId!,
            phaseId: s.phaseId,
            phaseName: plan.phases.find((p) => p.id === s.phaseId)?.name,
            conceptId: s.conceptId,
            prompt: MOCK_PRACTICE_FAMILY.prompt,
            result:
              s.state === 'correct'
                ? 'correct'
                : s.state === 'skipped'
                  ? 'skipped'
                  : 'incorrect',
            feedbackId: null,
            beforeSnapshotId: null,
            afterSnapshotId: null,
          })),
        nextPlan: {
          state: 'available',
          value: structuredClone(plan),
          stamp: makeMockStamp(input.workspaceId, 'practice.plan'),
        },
      };
      mockSessionSummaries.set(input.sessionId, sum);
      summaryAvailable = {
        state: 'available',
        value: structuredClone(sum),
        stamp: makeMockStamp(input.workspaceId, 'practice.session'),
      };
    }
  }

  const receipt: SessionActionReceipt = {
    operationId: input.operationId,
    sessionId: input.sessionId,
    revision: (input.expectedRevision ?? 0) + 1,
    eventIds: [],
    attempt: currentAttempt ? structuredClone(currentAttempt) : null,
    masteryChanges,
    bloomChanges,
    patternChanges,
    plan: structuredClone(plan),
    timing: {
      state: 'available',
      value: structuredClone(timing),
      stamp: makeMockStamp(input.workspaceId, 'practice.session'),
    },
    summary: summaryAvailable,
  };

  if (input.action !== 'read') {
    mockReceipts.set(receiptKey, structuredClone(receipt));
  }

  return receipt;
}
