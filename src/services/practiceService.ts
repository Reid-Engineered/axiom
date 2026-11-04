import { invoke } from '@tauri-apps/api/core';

import type {
  Attempt,
  AttemptDescription,
  Available,
  ConceptEvidence,
  CreatePlanInput,
  EditPatternRuleInput,
  EvaluationResult,
  Hint,
  PatternDetail,
  PatternListResponse,
  PatternSnapshot,
  Plan,
  ResponseValue,
  SessionActionInput,
  SessionActionReceipt,
  SessionHistoryEntry,
} from '../types';

export async function generateAttempt(workspaceId: string, familyId: string): Promise<Attempt> {
  return invoke<Attempt>('generateAttempt', { input: { workspaceId, familyId } });
}

export async function describeAttempt(
  workspaceId: string,
  attemptId: string,
): Promise<AttemptDescription> {
  return invoke<AttemptDescription>('describeAttempt', { input: { workspaceId, attemptId } });
}

export async function evaluateAttempt(
  workspaceId: string,
  attemptId: string,
  response: ResponseValue,
): Promise<EvaluationResult> {
  return invoke<EvaluationResult>('evaluateAttempt', {
    input: { workspaceId, attemptId, response },
  });
}

export async function requestHint(workspaceId: string, attemptId: string): Promise<Hint> {
  return invoke<Hint>('requestHint', { input: { workspaceId, attemptId } });
}

export async function createPracticePlan(input: CreatePlanInput): Promise<Available<Plan>> {
  return invoke<Available<Plan>>('createPracticePlan', { input });
}

export async function executeSessionAction(
  input: SessionActionInput,
): Promise<SessionActionReceipt> {
  return invoke<SessionActionReceipt>('executeSessionAction', { input });
}

export async function getConceptEvidence(
  workspaceId: string,
  conceptIds?: string[],
  asOf?: string,
): Promise<ConceptEvidence[]> {
  return invoke<ConceptEvidence[]>('getConceptEvidence', {
    input: {
      workspaceId,
      conceptIds: conceptIds ?? [],
      asOf: asOf ?? null,
    },
  });
}

export async function getPracticePatterns(
  workspaceId: string,
  options?: { status?: string; search?: string; cursor?: string; limit?: number },
): Promise<PatternListResponse> {
  return invoke<PatternListResponse>('getPracticePatterns', {
    input: {
      workspaceId,
      status: options?.status ?? null,
      search: options?.search ?? null,
      cursor: options?.cursor ?? null,
      limit: options?.limit ?? null,
    },
  });
}

export async function getPatternDetail(
  workspaceId: string,
  patternId: string,
  ruleRevision?: number,
): Promise<Available<PatternDetail>> {
  return invoke<Available<PatternDetail>>('getPatternDetail', {
    input: {
      workspaceId,
      patternId,
      ruleRevision: ruleRevision ?? null,
    },
  });
}

export async function editPatternRule(
  input: EditPatternRuleInput,
): Promise<Available<PatternSnapshot>> {
  return invoke<Available<PatternSnapshot>>('editPatternRule', { input });
}

export async function getSessionHistory(workspaceId: string): Promise<SessionHistoryEntry[]> {
  return invoke<SessionHistoryEntry[]>('getSessionHistory', { workspaceId });
}
