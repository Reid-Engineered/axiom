import { useCallback } from 'react';

import {
  addTutorExchange,
  endSession,
  getActiveSessionByWorkspace,
  getSession,
  pauseSession,
  resumeSession,
  startSession,
  type StartSessionInput,
} from '../services/sessionService';
import { useAsyncResource } from './useAsyncResource';

/** Loads a workspace's resumable session and supports starting a new one. */
export function useActiveSession(workspaceId: string) {
  const load = useCallback(() => getActiveSessionByWorkspace(workspaceId), [workspaceId]);
  const resource = useAsyncResource(load);
  const { setData } = resource;
  const start = useCallback(async (input: StartSessionInput) => {
    const session = await startSession(input);
    setData(session);
    return session;
  }, [setData]);

  return { session: resource.data, ...resource, startSession: start };
}

/** Loads one session and exposes its lifecycle and tutor-exchange mutations. */
export function useSession(sessionId: string) {
  const load = useCallback(() => getSession(sessionId), [sessionId]);
  const resource = useAsyncResource(load);
  const { setData } = resource;
  const replace = useCallback(<T extends Awaited<ReturnType<typeof getSession>>>(session: T) => {
    setData(session);
    return session;
  }, [setData]);

  const pause = useCallback(async () => replace(await pauseSession(sessionId)), [replace, sessionId]);
  const resume = useCallback(async () => replace(await resumeSession(sessionId)), [replace, sessionId]);
  const addExchange = useCallback(async (question: string) =>
    replace(await addTutorExchange(sessionId, question)), [replace, sessionId]);
  const end = useCallback(async () => replace(await endSession(sessionId)), [replace, sessionId]);

  return {
    session: resource.data,
    ...resource,
    pauseSession: pause,
    resumeSession: resume,
    addTutorExchange: addExchange,
    endSession: end,
  };
}
