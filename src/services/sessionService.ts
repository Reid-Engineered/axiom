import type { Session, SessionIntent } from '../types';
import { mockConcepts } from './mockData/concepts';
import { mockSessions } from './mockData/sessions';
import { mockWorkspaces } from './mockData/workspaces';

export interface StartSessionInput {
  workspaceId: string;
  conceptId: string;
  intent: SessionIntent;
}

/** The workspace's resumable session, if any — backs Home's Continue card. */
export async function getActiveSessionByWorkspace(
  workspaceId: string,
): Promise<Session | undefined> {
  const session = mockSessions.find(
    (candidate) => candidate.workspaceId === workspaceId && candidate.status !== 'completed',
  );
  return session ? structuredClone(session) : undefined;
}

export async function getSession(id: string): Promise<Session> {
  const session = mockSessions.find((candidate) => candidate.id === id);
  if (!session) throw new Error(`Session not found: ${id}`);
  return structuredClone(session);
}

/** "Practice this" / "Start · 8 min" — begins a new session against one concept. */
export async function startSession(input: StartSessionInput): Promise<Session> {
  if (!mockWorkspaces.some((workspace) => workspace.id === input.workspaceId)) {
    throw new Error(`Workspace not found: ${input.workspaceId}`);
  }
  const concept = mockConcepts.find((candidate) => candidate.id === input.conceptId);
  if (!concept || concept.workspaceId !== input.workspaceId) {
    throw new Error(`Concept not found in workspace: ${input.conceptId}`);
  }

  const session: Session = {
    id: `session-${crypto.randomUUID()}`,
    workspaceId: input.workspaceId,
    conceptId: input.conceptId,
    status: 'active',
    intent: structuredClone(input.intent),
    resumeSummary: `Ready to continue with ${concept.name}.`,
    elapsedMinutes: 0,
    exchanges: [],
    settledConclusions: [],
    startedAt: new Date().toISOString(),
  };
  mockSessions.push(session);
  return structuredClone(session);
}

export async function pauseSession(id: string): Promise<Session> {
  const session = mockSessions.find((candidate) => candidate.id === id);
  if (!session) throw new Error(`Session not found: ${id}`);
  if (session.status === 'completed') throw new Error(`Session is completed: ${id}`);
  session.status = 'paused';
  session.pausedAt = new Date().toISOString();
  return structuredClone(session);
}

export async function resumeSession(id: string): Promise<Session> {
  const session = mockSessions.find((candidate) => candidate.id === id);
  if (!session) throw new Error(`Session not found: ${id}`);
  if (session.status === 'completed') throw new Error(`Session is completed: ${id}`);
  session.status = 'active';
  delete session.pausedAt;
  return structuredClone(session);
}

/**
 * One tutor Q&A turn (screen 5's Socratic panel). Returns the whole session so the
 * caller sees the updated `exchanges` / `settledConclusions` in one shape.
 */
export async function addTutorExchange(sessionId: string, question: string): Promise<Session> {
  const session = mockSessions.find((candidate) => candidate.id === sessionId);
  if (!session) throw new Error(`Session not found: ${sessionId}`);
  if (session.status === 'completed') throw new Error(`Session is completed: ${sessionId}`);
  session.exchanges.push({
    id: `exchange-${crypto.randomUUID()}`,
    question: question.trim(),
    answer: 'Start with what the current representation makes visible, then test one step against the goal.',
    occurredAt: new Date().toISOString(),
    pinnedToVisualization: false,
  });
  return structuredClone(session);
}

export async function endSession(id: string): Promise<Session> {
  const session = mockSessions.find((candidate) => candidate.id === id);
  if (!session) throw new Error(`Session not found: ${id}`);
  session.status = 'completed';
  delete session.pausedAt;
  return structuredClone(session);
}
