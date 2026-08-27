import type { Session, SessionIntent } from '../types';

export interface StartSessionInput {
  workspaceId: string;
  conceptId: string;
  intent: SessionIntent;
}

/** The workspace's resumable session, if any — backs Home's Continue card. */
export async function getActiveSessionByWorkspace(
  _workspaceId: string,
): Promise<Session | undefined> {
  throw new Error('not implemented');
}

export async function getSession(_id: string): Promise<Session> {
  throw new Error('not implemented');
}

/** "Practice this" / "Start · 8 min" — begins a new session against one concept. */
export async function startSession(_input: StartSessionInput): Promise<Session> {
  throw new Error('not implemented');
}

export async function pauseSession(_id: string): Promise<Session> {
  throw new Error('not implemented');
}

export async function resumeSession(_id: string): Promise<Session> {
  throw new Error('not implemented');
}

/**
 * One tutor Q&A turn (screen 5's Socratic panel). Returns the whole session so the
 * caller sees the updated `exchanges` / `settledConclusions` in one shape.
 */
export async function addTutorExchange(_sessionId: string, _question: string): Promise<Session> {
  throw new Error('not implemented');
}

export async function endSession(_id: string): Promise<Session> {
  throw new Error('not implemented');
}
