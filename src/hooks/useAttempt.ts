import { useCallback, useState } from 'react';

import { describeAttempt, evaluateAttempt, requestHint } from '../services/practiceService';
import { nextProblem } from '../services/sessionService';
import type { AttemptDescription, EvaluationResult, ResponseValue, Session } from '../types';
import { useAsyncResource } from './useAsyncResource';

/**
 * One session's Practice attempt: its current state, the learner's answer, revealed
 * hints, and advancing to the next problem. Called only from StudySessionPage.
 */
export function useAttempt(
  session: Session | undefined,
  onSessionChange: (session: Session) => void,
) {
  const workspaceId = session?.workspaceId;
  const attemptId = session?.currentAttemptId;
  const load = useCallback(
    async () =>
      workspaceId && attemptId ? describeAttempt(workspaceId, attemptId) : undefined,
    [attemptId, workspaceId],
  );
  const resource = useAsyncResource<AttemptDescription | undefined>(load);
  const { setData } = resource;
  const [answer, setAnswer] = useState('');
  const [revealedHints, setRevealedHints] = useState<string[]>([]);
  const [evaluation, setEvaluation] = useState<EvaluationResult>();

  const hint = useCallback(async () => {
    if (!workspaceId || !attemptId) return;
    const revealed = await requestHint(workspaceId, attemptId);
    setRevealedHints((hints) => [...hints, revealed.hintText]);
    setData((current) =>
      current ? { ...current, hintsRevealed: revealed.hintsRevealed } : current,
    );
  }, [attemptId, setData, workspaceId]);

  const check = useCallback(async () => {
    const attempt = resource.data;
    if (!workspaceId || !attemptId || !attempt) return;
    const response: ResponseValue =
      attempt.responseType === 'numeric'
        ? { responseType: 'numeric', value: Number(answer) }
        : { responseType: 'symbolic-expression', value: answer };
    const result = await evaluateAttempt(workspaceId, attemptId, response);
    setEvaluation(result);
    setData((current) =>
      current
        ? { ...current, status: result.status, submissionCount: result.submissionCount }
        : current,
    );
    if (!result.correct && attempt.hintsRevealed < attempt.hintsTotal) await hint();
  }, [answer, attemptId, hint, resource.data, setData, workspaceId]);

  const next = useCallback(async () => {
    if (!session) return;
    const advanced = await nextProblem(session.id);
    setAnswer('');
    setRevealedHints([]);
    setEvaluation(undefined);
    onSessionChange(advanced);
  }, [onSessionChange, session]);

  return {
    attempt: resource.data,
    loading: resource.loading,
    error: resource.error,
    answer,
    setAnswer,
    revealedHints,
    evaluation,
    check,
    hint,
    next,
  };
}
