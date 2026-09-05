import { describe, expect, it } from 'vitest';

import { evaluateAttempt, generateAttempt, requestHint } from './practiceService';

const workspaceId = 'workspace-calculus-ii';

describe('practiceService', () => {
  it('generates an attempt with the mock family prompt and hint count', async () => {
    const attempt = await generateAttempt(workspaceId, 'problem.shell_y_poly');

    expect(attempt.attemptId).toBeTruthy();
    expect(attempt.prompt).toContain('volume');
    expect(attempt.hintsTotal).toBe(2);
  });

  it('returns status open for an incorrect response', async () => {
    const attempt = await generateAttempt(workspaceId, 'problem.shell_y_poly');

    const result = await evaluateAttempt(workspaceId, attempt.attemptId, {
      responseType: 'numeric',
      value: 0,
    });

    expect(result.correct).toBe(false);
    expect(result.status).toBe('open');
    expect(result.submissionCount).toBe(1);
  });

  it('returns status solved for the correct response', async () => {
    const attempt = await generateAttempt(workspaceId, 'problem.shell_y_poly');

    const result = await evaluateAttempt(workspaceId, attempt.attemptId, {
      responseType: 'numeric',
      value: 42.7,
    });

    expect(result.correct).toBe(true);
    expect(result.status).toBe('solved');
  });

  it('reveals hints in order and throws once exhausted', async () => {
    const attempt = await generateAttempt(workspaceId, 'problem.shell_y_poly');

    const first = await requestHint(workspaceId, attempt.attemptId);
    expect(first.hintsRevealed).toBe(1);
    const second = await requestHint(workspaceId, attempt.attemptId);
    expect(second.hintsRevealed).toBe(2);

    await expect(requestHint(workspaceId, attempt.attemptId)).rejects.toThrow();
  });

  it('throws evaluating or hinting an unknown attempt', async () => {
    await expect(
      evaluateAttempt(workspaceId, 'attempt-unknown', { responseType: 'numeric', value: 1 }),
    ).rejects.toThrow();
    await expect(requestHint(workspaceId, 'attempt-unknown')).rejects.toThrow();
  });
});
