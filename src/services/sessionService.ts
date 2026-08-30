import { invoke } from '@tauri-apps/api/core';

import type { Session, SessionIntent } from '../types';

export interface StartSessionInput {
  workspaceId: string;
  conceptId: string;
  intent: SessionIntent;
}

/** The workspace's resumable session, if any — backs Home's Continue card. */
export async function getActiveSessionByWorkspace(
  workspaceId: string,
): Promise<Session | undefined> {
  return (
    (await invoke<Session | null>('getActiveSessionByWorkspace', { workspaceId })) ?? undefined
  );
}

export async function getSession(id: string): Promise<Session> {
  return invoke<Session>('getSession', { id });
}

/** "Practice this" / "Start · 8 min" — begins a new session against one concept. */
export async function startSession(input: StartSessionInput): Promise<Session> {
  return invoke<Session>('startSession', { input });
}

export async function pauseSession(id: string): Promise<Session> {
  return invoke<Session>('pauseSession', { id });
}

export async function resumeSession(id: string): Promise<Session> {
  return invoke<Session>('resumeSession', { id });
}

/**
 * One tutor Q&A turn (screen 5's Socratic panel). Returns the whole session so the
 * caller sees the updated `exchanges` / `settledConclusions` in one shape.
 */
export async function addTutorExchange(sessionId: string, question: string): Promise<Session> {
  return invoke<Session>('addTutorExchange', { sessionId, question });
}

export async function endSession(id: string): Promise<Session> {
  return invoke<Session>('endSession', { id });
}
