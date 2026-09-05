import { invoke } from '@tauri-apps/api/core';

import type { Attempt, EvaluationResult, Hint, ResponseValue } from '../types';

export async function generateAttempt(workspaceId: string, familyId: string): Promise<Attempt> {
  return invoke<Attempt>('generateAttempt', { input: { workspaceId, familyId } });
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
