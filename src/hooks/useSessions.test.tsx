import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { mockSessions } from '../services/mockData/sessions';
import { loadMockSessionsForTest } from '../test/mockBackend';
import { useActiveSession, useSession } from './useSessions';

describe('session domain hooks', () => {
  beforeEach(() => loadMockSessionsForTest());

  it('loads the real long session and preserves its collapsed-summary inputs', async () => {
    const fixture = mockSessions[0];
    const { result } = renderHook(() => useSession(fixture.id));
    await waitFor(() => expect(result.current.session?.id).toBe(fixture.id));
    expect(result.current.session?.exchanges).toHaveLength(40);
    expect(result.current.session?.settledConclusions).toHaveLength(2);
    expect(result.current.session?.openQuestion).toBeTruthy();

    await act(async () => {
      await result.current.resumeSession();
    });
    expect(result.current.session?.status).toBe('active');
    await act(async () => {
      await result.current.addTutorExchange('Why is the radius measured from the axis?');
    });
    expect(result.current.session?.tutorStatus).toBe('unavailable');
    expect(result.current.session?.exchanges).toHaveLength(40);
    await act(async () => {
      await result.current.pauseSession();
    });
    expect(result.current.session?.tutorStatus).toBeUndefined();
    expect(result.current.session?.exchanges).toHaveLength(40);
  });

  it('loads an active fixture session and can start a fixture-backed session', async () => {
    const workspaceId = 'workspace-linear-algebra';
    const { result } = renderHook(() => useActiveSession(workspaceId));
    await waitFor(() => expect(result.current.session?.workspaceId).toBe(workspaceId));
    await act(async () => {
      await result.current.startSession({
        workspaceId,
        conceptId: 'linear-concept-1',
        intent: { activity: 'Practising', targetMinutes: 15 },
      });
    });
    expect(result.current.session).toMatchObject({ workspaceId, conceptId: 'linear-concept-1', status: 'active' });
  });
});
