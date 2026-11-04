import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import {
  createPracticePlan,
  editPatternRule,
  executeSessionAction,
  getConceptEvidence,
  getPatternDetail,
  getPracticePatterns,
  getSessionHistory,
} from '../services/practiceService';
import type {
  Available,
  ConceptEvidence,
  CreatePlanInput,
  PatternDetail,
  PatternListResponse,
  PatternSnapshot,
  Plan,
  PracticeDifficulty,
  ResponseValue,
  SessionActionInput,
  SessionActionReceipt,
  SessionHistoryEntry,
} from '../types';
import { useAsyncResource } from './useAsyncResource';

/** Loads numeric mastery, Bloom evidence, and readiness for a workspace's concepts. */
export function useConceptEvidence(workspaceId: string, conceptIds?: string | string[]) {
  const normalizedIds =
    typeof conceptIds === 'string'
      ? conceptIds
        ? [conceptIds]
        : undefined
      : conceptIds;
  const key = normalizedIds ? normalizedIds.join(',') : '';
  const load = useCallback(
    () => (workspaceId ? getConceptEvidence(workspaceId, normalizedIds) : Promise.resolve([])),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [workspaceId, key],
  );
  const resource = useAsyncResource<ConceptEvidence[]>(load);
  const evidenceList = resource.data ?? [];
  return {
    evidence: evidenceList[0],
    evidenceList,
    ...resource,
  };
}

/** Manages the Today session plan (creation, difficulty/duration/length edits, and preview). */
export function usePracticePlan(workspaceId: string, initialPatternId?: string | null) {
  const [difficulty, setDifficulty] = useState<PracticeDifficulty>('Steady');
  const [targetMs, setTargetMs] = useState<number>(1_500_000);
  const [questionLimit, setQuestionLimit] = useState<number>(5);
  const [patternId, setPatternId] = useState<string | null>(initialPatternId ?? null);

  const load = useCallback(async () => {
    if (!workspaceId) return undefined;
    return createPracticePlan({
      workspaceId,
      operationId: `plan-${workspaceId}-${difficulty}-${targetMs}-${questionLimit}-${patternId ?? 'none'}`,
      difficulty,
      targetMs,
      questionLimit,
      patternId,
    });
  }, [difficulty, patternId, questionLimit, targetMs, workspaceId]);

  const resource = useAsyncResource<Available<Plan> | undefined>(load);

  const rebuildPlan = useCallback(
    async (overrides?: Partial<CreatePlanInput>) => {
      if (!workspaceId) return undefined;
      const nextDiff = overrides?.difficulty ?? difficulty;
      const nextTarget = overrides?.targetMs ?? targetMs;
      const nextLimit = overrides?.questionLimit ?? questionLimit;
      const nextPattern = overrides?.patternId !== undefined ? overrides.patternId : patternId;
      if (overrides?.difficulty) setDifficulty(overrides.difficulty);
      if (overrides?.targetMs) setTargetMs(overrides.targetMs);
      if (overrides?.questionLimit) setQuestionLimit(overrides.questionLimit);
      if (overrides?.patternId !== undefined) setPatternId(overrides.patternId);

      const created = await createPracticePlan({
        workspaceId,
        operationId: crypto.randomUUID(),
        difficulty: nextDiff,
        targetMs: nextTarget,
        questionLimit: nextLimit,
        patternId: nextPattern,
        conceptIds: overrides?.conceptIds,
        previewOnly: overrides?.previewOnly,
      });
      resource.setData(created);
      return created;
    },
    [difficulty, patternId, questionLimit, resource, targetMs, workspaceId],
  );

  return {
    plan: resource.data,
    planAvailable: resource.data,
    planValue: resource.data?.state === 'available' ? resource.data.value : undefined,
    loading: resource.loading,
    error: resource.error,
    difficulty,
    setDifficulty,
    targetMs,
    setTargetMs,
    questionLimit,
    setQuestionLimit,
    patternId,
    setPatternId,
    rebuildPlan,
  };
}

/** Loads the Error Profile pattern list and detail for a selected misconception pattern. */
export function useErrorProfile(workspaceId: string, initialPatternId?: string) {
  const [statusFilter, setStatusFilter] = useState<string>('All');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [selectedPatternId, setSelectedPatternId] = useState<string | null>(
    initialPatternId ?? null,
  );
  const [savingRule, setSavingRule] = useState(false);

  const loadList = useCallback(
    () =>
      workspaceId
        ? getPracticePatterns(workspaceId, {
            status: statusFilter === 'All' ? undefined : statusFilter,
            search: searchQuery || undefined,
          })
        : Promise.resolve(undefined),
    [searchQuery, statusFilter, workspaceId],
  );
  const listResource = useAsyncResource<PatternListResponse | undefined>(loadList);

  const patterns = useMemo<PatternSnapshot[]>(
    () =>
      listResource.data?.patterns.state === 'available' ? listResource.data.patterns.value : [],
    [listResource.data?.patterns],
  );

  const activePatternId = selectedPatternId ?? patterns[0]?.patternId ?? null;

  const loadDetail = useCallback(
    () =>
      workspaceId && activePatternId
        ? getPatternDetail(workspaceId, activePatternId)
        : Promise.resolve(undefined),
    [activePatternId, workspaceId],
  );
  const detailResource = useAsyncResource<Available<PatternDetail> | undefined>(loadDetail);

  const updateRule = useCallback(
    async (patternId: string, requiredClean: number) => {
      setSavingRule(true);
      try {
        const currentSnap = patterns.find((p) => p.patternId === patternId);
        const ruleRevision = currentSnap?.ruleRevision ?? 1;
        const updated = await editPatternRule({
          workspaceId,
          operationId: crypto.randomUUID(),
          expectedRevision: ruleRevision,
          patternId,
          ruleRevision,
          requiredClean,
        });
        if (updated.state === 'available' && listResource.data?.patterns.state === 'available') {
          const nextList = listResource.data.patterns.value.map((p) =>
            p.patternId === patternId ? updated.value : p,
          );
          listResource.setData({
            ...listResource.data,
            patterns: {
              ...listResource.data.patterns,
              value: nextList,
            },
          });
        }
        if (detailResource.data?.state === 'available') {
          const freshDetail = await getPatternDetail(workspaceId, patternId);
          detailResource.setData(freshDetail);
        }
        return updated;
      } finally {
        setSavingRule(false);
      }
    },
    [detailResource, listResource, patterns, workspaceId],
  );

  return {
    listResponse: listResource.data,
    response: listResource.data,
    patterns,
    patternsAvailable: listResource.data?.patterns,
    loading: listResource.loading,
    error: listResource.error,
    statusFilter,
    setStatusFilter,
    search: searchQuery,
    setSearch: setSearchQuery,
    searchQuery,
    setSearchQuery,
    selectedPatternId: activePatternId,
    setSelectedPatternId,
    selectedDetail: detailResource.data,
    detailAvailable: detailResource.data,
    detail: detailResource.data?.state === 'available' ? detailResource.data.value : undefined,
    detailLoading: detailResource.loading,
    savingRule,
    updateRule,
  };
}

/** Loads completed and active session summaries for the Session History screen. */
export function useSessionHistory(workspaceId: string, initialSessionId?: string) {
  const [selectedSessionId, setSelectedSessionId] = useState<string | undefined>(
    initialSessionId,
  );
  const load = useCallback(
    () => (workspaceId ? getSessionHistory(workspaceId) : Promise.resolve([])),
    [workspaceId],
  );
  const resource = useAsyncResource<SessionHistoryEntry[]>(load);
  const entries = resource.data ?? [];
  const activeSessionId = selectedSessionId ?? entries[0]?.sessionId;
  const selectedEntry = entries.find((e) => e.sessionId === activeSessionId) ?? entries[0];

  return {
    entries,
    selectedEntry,
    selectedSessionId: activeSessionId,
    setSelectedSessionId,
    ...resource,
  };
}

/**
 * Manages the Stage 9 adaptive session protocol (`executeSessionAction`), including:
 * - Question-first Practice state & post-check Feedback state
 * - Working draft persistence (`save-working`)
 * - Live foreground elapsed time tracking (`076` + `P7`)
 */
export function useAdaptiveSession(
  workspaceId: string | undefined,
  sessionId: string | undefined,
  paused = false,
) {
  const load = useCallback(async () => {
    if (!workspaceId || !sessionId) return undefined;
    return executeSessionAction({
      workspaceId,
      sessionId,
      operationId: `read-${sessionId}`,
      expectedRevision: 0,
      action: 'read',
      payload: {},
    });
  }, [sessionId, workspaceId]);

  const resource = useAsyncResource<SessionActionReceipt | undefined>(load);
  const receipt = resource.data;
  const [answerText, setAnswerText] = useState('');
  const [workingText, setWorkingText] = useState('');
  const [savingDraft, setSavingDraft] = useState(false);
  const [liveElapsedMs, setLiveElapsedMs] = useState(0);
  const [liveAttemptElapsedMs, setLiveAttemptElapsedMs] = useState(0);
  const pendingDeltaMsRef = useRef(0);
  const pendingAttemptDeltaMsRef = useRef(0);

  const attemptId = receipt?.attempt?.id;
  useEffect(() => {
    if (receipt?.attempt) {
      setAnswerText(receipt.attempt.answerText ?? '');
      setWorkingText(receipt.attempt.workingText ?? '');
    }
  }, [attemptId, receipt?.attempt]);

  useEffect(() => {
    if (receipt?.timing.state === 'available') {
      setLiveElapsedMs(receipt.timing.value.elapsedMs);
      setLiveAttemptElapsedMs(receipt.timing.value.attemptElapsedMs);
    }
  }, [receipt?.timing]);

  // 1-second foreground ticker (P7: pauses when session paused or document hidden)
  useEffect(() => {
    if (!workspaceId || !sessionId || paused || receipt?.summary.state === 'available') {
      return;
    }
    const interval = window.setInterval(() => {
      if (typeof document !== 'undefined' && document.visibilityState === 'hidden') {
        return;
      }
      setLiveElapsedMs((prev) => prev + 1000);
      setLiveAttemptElapsedMs((prev) => prev + 1000);
      pendingDeltaMsRef.current += 1000;
      pendingAttemptDeltaMsRef.current += 1000;
    }, 1000);
    return () => window.clearInterval(interval);
  }, [paused, receipt?.summary.state, sessionId, workspaceId]);

  const runAction = useCallback(
    async (
      action: SessionActionInput['action'],
      payload: Record<string, unknown> = {},
    ): Promise<SessionActionReceipt | undefined> => {
      if (!workspaceId || !sessionId) return undefined;
      const elapsedDelta = pendingDeltaMsRef.current;
      const attemptDelta = pendingAttemptDeltaMsRef.current;
      pendingDeltaMsRef.current = 0;
      pendingAttemptDeltaMsRef.current = 0;

      const nextReceipt = await executeSessionAction({
        workspaceId,
        sessionId,
        operationId: crypto.randomUUID(),
        expectedRevision: receipt?.revision ?? 1,
        action,
        payload,
        elapsedDeltaMs: elapsedDelta,
        attemptElapsedDeltaMs: attemptDelta,
      });
      resource.setData(nextReceipt);
      return nextReceipt;
    },
    [receipt?.revision, resource, sessionId, workspaceId],
  );

  const setWorkingLines = useCallback((lines: string[]) => {
    setWorkingText(lines.join('\n'));
  }, []);

  const saveWorking = useCallback(
    async (nextAnswer?: string, nextWorking?: string) => {
      if (!receipt?.attempt) return undefined;
      setSavingDraft(true);
      try {
        return await runAction('save-working', {
          attemptId: receipt.attempt.id,
          draftRevision: receipt.attempt.draftRevision + 1,
          answerText: nextAnswer ?? answerText,
          workingText: nextWorking ?? workingText,
        });
      } finally {
        setSavingDraft(false);
      }
    },
    [answerText, receipt?.attempt, runAction, workingText],
  );

  const checkAnswer = useCallback(
    async (explicitAnswer?: string) => {
      const text = explicitAnswer ?? answerText;
      const response: ResponseValue =
        receipt?.attempt?.responseType === 'numeric'
          ? { responseType: 'numeric', value: Number(text) }
          : { responseType: 'symbolic-expression', value: text };
      return runAction('check', {
        attemptId: receipt?.attempt?.id,
        response,
      });
    },
    [answerText, receipt?.attempt?.id, receipt?.attempt?.responseType, runAction],
  );

  const requestNextHint = useCallback(async () => {
    return runAction('hint', {
      attemptId: receipt?.attempt?.id,
      level: (receipt?.attempt?.hintsRevealed ?? 0) + 1,
    });
  }, [receipt?.attempt?.hintsRevealed, receipt?.attempt?.id, runAction]);

  const revealSolution = useCallback(async () => {
    return runAction('reveal-solution', {
      attemptId: receipt?.attempt?.id,
    });
  }, [receipt?.attempt?.id, runAction]);

  const skipQuestion = useCallback(async () => {
    return runAction('skip', {
      attemptId: receipt?.attempt?.id,
    });
  }, [receipt?.attempt?.id, runAction]);

  const advanceQuestion = useCallback(async () => {
    return runAction('advance', {
      attemptId: receipt?.attempt?.id,
    });
  }, [receipt?.attempt?.id, runAction]);

  const pause = useCallback(async () => {
    return runAction('pause', {});
  }, [runAction]);

  const endSession = useCallback(async () => {
    return runAction('end', {});
  }, [runAction]);

  return {
    receipt,
    plan: receipt?.plan,
    attempt: receipt?.attempt ?? undefined,
    loading: resource.loading,
    error: resource.error,
    answerText,
    setAnswerText,
    workingText,
    setWorkingText,
    setWorkingLines,
    savingDraft,
    liveElapsedMs,
    liveAttemptElapsedMs,
    saveWorking,
    checkAnswer,
    requestHint: requestNextHint,
    requestNextHint,
    revealSolution,
    skipProblem: skipQuestion,
    skipQuestion,
    advanceProblem: advanceQuestion,
    advanceQuestion,
    pause,
    endSession,
  };
}
