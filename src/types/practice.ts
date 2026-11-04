export type ResponseType = 'symbolic-expression' | 'numeric';

export interface Attempt {
  attemptId: string;
  prompt: string;
  responseType: ResponseType;
  hintsTotal: number;
}

export type ResponseValue =
  | { responseType: 'symbolic-expression'; value: string }
  | { responseType: 'numeric'; value: number };

export type AttemptStatus = 'open' | 'solved';

export interface EvaluationResult {
  correct: boolean;
  status: AttemptStatus;
  submissionCount: number;
}

export interface Hint {
  hintText: string;
  hintsRevealed: number;
  hintsTotal: number;
}

/** An attempt's current learner-facing state, the single hydration path for a session. */
export interface AttemptDescription {
  prompt: string;
  responseType: ResponseType;
  hintsTotal: number;
  hintsRevealed: number;
  status: AttemptStatus;
  submissionCount: number;
}

export type UnavailableReason =
  | 'no-evidence'
  | 'legacy-only'
  | 'unsupported'
  | 'provider-unavailable'
  | 'content-unavailable'
  | 'policy-unapproved';

export interface PolicyRef {
  id: string;
  revision: number;
  digest: string;
}

export interface ContentRef {
  packageId: string;
  packageVersion: string;
  schemaVersion: number;
  digest: string;
}

export interface Stamp {
  schemaVersion: 1;
  workspaceId: string;
  provider: {
    moduleId: string;
    moduleVersion: string;
    capabilityId: string;
    capabilityVersion: number;
  };
  policyRef: PolicyRef | null;
  contentRef: ContentRef | null;
  evidenceIds: string[];
  recordedAt: string;
}

export type Available<T> =
  | { state: 'available'; value: T; stamp: Stamp }
  | {
      state: 'unavailable';
      reason: UnavailableReason;
      lastKnown: { value: T; stamp: Stamp } | null;
    };

export type BloomLevel = 'remember' | 'understand' | 'apply' | 'analyze' | 'evaluate' | 'create';

export type PracticeDifficulty = 'Steady' | 'Push' | 'Exam';

export interface Observation {
  id: string;
  stamp: Stamp;
  sessionId: string | null;
  attemptId: string | null;
  submissionId: string | null;
  ordinal: number;
  kind: 'check' | 'hint' | 'skip' | 'solution' | 'decay' | 'rule-change';
  result: 'correct' | 'incorrect' | 'partial' | 'unverifiable' | null;
  objectiveIds: string[];
  verificationId: string | null;
  hintsUsed: number;
  assisted: boolean;
}

export interface MasterySnapshot {
  id: string;
  conceptId: string;
  objectiveIds: string[];
  score: number;
  previousSnapshotId: string | null;
  previousScore: number | null;
  delta: number | null;
  namedState: 'Developing' | 'Familiar' | 'Strong' | 'Mastered';
  asOf: string;
  lastPracticedAt: string;
  decay: {
    fromSnapshotId: string;
    fromScore: number;
    elapsedMs: number;
    amount: number;
  } | null;
}

export interface BloomEvidence {
  demonstratedLevel: BloomLevel;
  objectiveIds: string[];
  qualifyingAttemptIds: string[];
  previousLevel: BloomLevel | null;
}

export interface Readiness {
  state: 'ready' | 'not-ready' | 'unknown';
  prerequisites: {
    conceptId: string;
    snapshotId: string | null;
    state: 'ready' | 'not-ready' | 'unknown';
  }[];
  reason: string;
}

export interface ConceptEvidence {
  conceptId: string;
  mastery: Available<MasterySnapshot>;
  bloom: Available<BloomEvidence>;
  readiness: Available<Readiness>;
}

export interface Attribution {
  id: string;
  attemptId: string;
  patternId: string;
  ruleRevision: number;
  opportunityId: string;
  eligible: boolean;
  outcome: 'observed' | 'clean' | 'unknown' | 'not-applicable';
  evidenceIds: string[];
  detectorId: string | null;
  detectorVersion: number | null;
  explanation: string;
  recordedAt?: string;
  answerText?: string | null;
  conceptId?: string | null;
  sessionId?: string | null;
}

export type PatternStatus = 'Recurring' | 'Improving' | 'Resolved' | 'Unknown';

export interface PatternSnapshot {
  id: string;
  patternId: string;
  name: string;
  conceptIds?: string[];
  status: PatternStatus;
  opportunities: number;
  occurrences: number;
  unknownCount: number;
  cleanStreak: number;
  firstSeenAt: string | null;
  lastSeenAt: string | null;
  recentAttributionIds: string[];
  ruleRevision: number;
  resolution: {
    requiredClean: number;
    eligibleClean: number;
    explanation: string;
  };
}

export interface PatternSessionBucket {
  sessionId: string;
  recordedAt: string;
  opportunities: number;
  occurrences: number;
  cleanCount: number;
  unknownCount: number;
}

export interface PatternDetail {
  snapshot: PatternSnapshot;
  explanation: string;
  nextAction: string;
  attributions: Attribution[];
  sessionBuckets: PatternSessionBucket[];
}

export type PlanPhaseName = 'Warm-up' | 'Stretch' | 'Combined';

export type QueueSlotState = 'pending' | 'current' | 'correct' | 'miss' | 'partial' | 'skipped';

export interface PlanPhase {
  id: string;
  name: PlanPhaseName;
  reason: string;
  evidenceIds: string[];
}

export interface PlanQueueSlot {
  id: string;
  phaseId: string;
  conceptId: string;
  objectiveIds: string[];
  familyId: string;
  seed: string;
  attemptId: string | null;
  state: QueueSlotState;
  reason: string;
  evidenceIds: string[];
  replacesId: string | null;
}

export interface PlanChange {
  eventId: string;
  removedSlotId: string;
  addedSlotId: string;
  reason: string;
}

export interface PlanGoalSnapshot {
  id: string;
  text: string;
  state: string;
  inferred: Record<string, unknown>;
  updatedAt: string;
}

export interface Plan {
  id: string;
  revision: number;
  goalId: string | null;
  goalSnapshot: PlanGoalSnapshot | null;
  targetMs: number | null;
  difficulty: PracticeDifficulty;
  policyRef: PolicyRef;
  phases: PlanPhase[];
  queue: PlanQueueSlot[];
  changes: PlanChange[];
  availability: 'complete' | 'limited';
  limitations: string[];
}

export interface Verification {
  id: string;
  verdict: 'correct' | 'incorrect' | 'partial' | 'unverifiable';
  method: 'numeric-tolerance' | 'constant-expression-numeric';
  checkerVersion: string;
  absoluteTolerance: number;
  relativeTolerance: number;
  canonicalValue: number | null;
  studentValue: number | null;
  restrictions: string[];
  parseError: string | null;
  limitations: string[];
  objectiveResults: {
    objectiveId: string;
    verdict: 'correct' | 'incorrect' | 'partial' | 'unverifiable';
    evidence: string;
  }[];
  samples:
    | { bindings: Record<string, number>; expected: number; actual: number; error: number }[]
    | null;
  terms: { expression: string; verdict: string; evidence: string }[] | null;
}

export interface AttemptState {
  id: string;
  slotId: string;
  status: 'open' | 'solved' | 'skipped';
  instanceRef: string;
  prompt: string;
  responseType: ResponseType;
  restrictions: string[];
  bloomTarget: BloomLevel | null;
  draftRevision: number;
  answerText: string;
  workingText: string;
  hintsTotal: number;
  hintsRevealed: number;
  hintBudget: number;
  revealedHints?: string[];
  canonicalSolutionDisplay?: string | null;
  workedSolution?: string | null;
  submissionIds: string[];
  lastFeedbackId: string | null;
  lastVerification?: Verification | null;
}

export interface SessionTiming {
  elapsedMs: number;
  attemptElapsedMs: number;
  targetMs: number | null;
  checkpointSequence: number;
  active: boolean;
  measuredAt: string;
  recovered: boolean;
}

export interface SessionQuestionRow {
  slotId: string;
  attemptId: string;
  phaseId: string;
  phaseName?: PlanPhaseName;
  conceptId?: string;
  prompt: string;
  result: 'correct' | 'incorrect' | 'partial' | 'unverifiable' | 'skipped';
  feedbackId: string | null;
  beforeSnapshotId: string | null;
  afterSnapshotId: string | null;
  beforeScore?: number | null;
  afterScore?: number | null;
}

export interface SessionSummary {
  sessionId: string;
  completedAt: string;
  timing: Available<SessionTiming>;
  difficultyHistory: {
    slotId: string;
    difficulty: PracticeDifficulty;
    policyRef: PolicyRef;
  }[];
  correct: number;
  assessedTotal: number;
  attemptedTotal: number;
  skippedTotal: number;
  unverifiableTotal: number;
  hintsUsed: number;
  hintBudgetTotal?: number;
  masteryChanges: Available<MasterySnapshot>[];
  repeatedPatternIds: string[];
  questionRows: SessionQuestionRow[];
  nextPlan: Available<Plan>;
}

export interface SessionActionReceipt {
  operationId: string;
  sessionId: string;
  revision: number;
  eventIds: string[];
  attempt: AttemptState | null;
  masteryChanges: Available<MasterySnapshot>[];
  bloomChanges: Available<BloomEvidence>[];
  patternChanges: Available<PatternSnapshot>[];
  plan: Plan;
  timing: Available<SessionTiming>;
  summary: Available<SessionSummary>;
}

export interface CreatePlanInput {
  workspaceId: string;
  operationId: string;
  goalId?: string | null;
  conceptIds?: string[];
  targetMs?: number | null;
  questionLimit?: number | null;
  difficulty?: PracticeDifficulty;
  patternId?: string | null;
  previewOnly?: boolean;
}

export type SessionActionKind =
  | 'read'
  | 'start'
  | 'resume'
  | 'pause'
  | 'check'
  | 'hint'
  | 'skip'
  | 'advance'
  | 'save-working'
  | 'reveal-solution'
  | 'end'
  | 'edit-plan'
  | 'edit-rule';

export interface SessionActionInput {
  workspaceId: string;
  sessionId: string;
  operationId: string;
  expectedRevision?: number;
  action: SessionActionKind;
  payload?: Record<string, unknown>;
  elapsedDeltaMs?: number;
  attemptElapsedDeltaMs?: number;
}

export interface EditPatternRuleInput {
  workspaceId: string;
  operationId: string;
  expectedRevision: number;
  patternId: string;
  ruleRevision: number;
  requiredClean: number;
  previewOnly?: boolean;
}

export interface PatternListResponse {
  patterns: Available<PatternSnapshot[]>;
  totalAnswers: number;
  sinceDate: string | null;
  requiredCleanDefault: number;
  nextCursor: string | null;
}

export interface SessionHistoryEntry {
  sessionId: string;
  workspaceId: string;
  conceptId?: string;
  intentActivity?: string;
  intentDetail?: string | null;
  status: 'active' | 'paused' | 'completed';
  startedAt: string;
  completedAt: string | null;
  difficulty: PracticeDifficulty | null;
  summary: Available<SessionSummary>;
}


