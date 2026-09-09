import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { startSession } from '../services/sessionService';
import type { Session } from '../types';
import { useAttempt } from './useAttempt';

async function boundSession(): Promise<Session> {
  return startSession({
    workspaceId: 'workspace-calculus',
    conceptId: 'concept-shells',
    intent: { activity: 'Practising', targetMinutes: 8 },
  });
}

describe('useAttempt', () => {
  it('describes the session-bound attempt on mount', async () => {
    const session = await boundSession();
    const { result } = renderHook(() => useAttempt(session, vi.fn()));

    await waitFor(() => expect(result.current.attempt).toBeDefined());
    expect(result.current.attempt?.status).toBe('open');
    expect(result.current.attempt?.hintsRevealed).toBe(0);
    expect(result.current.error).toBeNull();
  });

  it('stays empty and error-free when the session has no bound attempt', async () => {
    const session = { ...(await boundSession()), currentAttemptId: undefined };
    const { result } = renderHook(() => useAttempt(session, vi.fn()));

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.attempt).toBeUndefined();
    expect(result.current.error).toBeNull();
  });

  it('reveals the next hint when an answer is wrong', async () => {
    const session = await boundSession();
    const { result } = renderHook(() => useAttempt(session, vi.fn()));
    await waitFor(() => expect(result.current.attempt).toBeDefined());

    act(() => result.current.setAnswer('1'));
    await act(async () => {
      await result.current.check();
    });

    expect(result.current.evaluation?.correct).toBe(false);
    expect(result.current.revealedHints).toHaveLength(1);
    expect(result.current.attempt?.hintsRevealed).toBe(1);
  });

  it('stops revealing hints once they are exhausted', async () => {
    const session = await boundSession();
    const { result } = renderHook(() => useAttempt(session, vi.fn()));
    await waitFor(() => expect(result.current.attempt).toBeDefined());

    act(() => result.current.setAnswer('1'));
    for (let attempt = 0; attempt < 3; attempt += 1) {
      await act(async () => {
        await result.current.check();
      });
    }

    expect(result.current.revealedHints).toHaveLength(2);
    expect(result.current.error).toBeNull();
  });

  it('marks the attempt solved on a correct answer', async () => {
    const session = await boundSession();
    const { result } = renderHook(() => useAttempt(session, vi.fn()));
    await waitFor(() => expect(result.current.attempt).toBeDefined());

    act(() => result.current.setAnswer('42.7'));
    await act(async () => {
      await result.current.check();
    });

    expect(result.current.evaluation?.correct).toBe(true);
    expect(result.current.attempt?.status).toBe('solved');
  });

  it('rebinds and resets when advancing to the next problem', async () => {
    const session = await boundSession();
    const onSessionChange = vi.fn();
    const { result, rerender } = renderHook(
      ({ current }) => useAttempt(current, onSessionChange),
      { initialProps: { current: session } },
    );
    await waitFor(() => expect(result.current.attempt).toBeDefined());
    act(() => result.current.setAnswer('42.7'));
    await act(async () => {
      await result.current.check();
    });

    await act(async () => {
      await result.current.next();
    });
    const advanced = onSessionChange.mock.calls.at(-1)?.[0] as Session;
    expect(advanced.currentAttemptId).not.toBe(session.currentAttemptId);
    expect(advanced.problemIndex).toBe(2);

    rerender({ current: advanced });
    await waitFor(() => expect(result.current.attempt?.status).toBe('open'));
    expect(result.current.answer).toBe('');
    expect(result.current.revealedHints).toEqual([]);
    expect(result.current.evaluation).toBeUndefined();
  });

  it('surfaces a describe failure as an error rather than throwing', async () => {
    const session = { ...(await boundSession()), currentAttemptId: 'attempt-missing' };
    const { result } = renderHook(() => useAttempt(session, vi.fn()));

    await waitFor(() => expect(result.current.error).not.toBeNull());
    expect(result.current.attempt).toBeUndefined();
  });
});
