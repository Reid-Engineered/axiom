import { useEffect, useRef, useState, type FormEvent } from 'react';

import { Button } from '../components/primitives/Button';
import { SessionToolbar } from '../components/session/SessionToolbar';
import { WorkingArea } from '../components/session/WorkingArea';
import { useAttempt } from '../hooks/useAttempt';
import { useConcept } from '../hooks/useConcepts';
import { useAdaptiveSession } from '../hooks/useLearningEvidence';
import { useNavigation } from '../hooks/useNavigation';
import { useSession } from '../hooks/useSessions';
import { useWorkspaceDetails } from '../hooks/useWorkspaces';
import { SessionShell } from '../layouts/SessionShell';
import type { Session } from '../types';
import styles from './StudySessionPage.module.css';

export interface StudySessionPageProps {
  sessionId: string;
}

function formatClockMs(ms: number): string {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const mins = Math.floor(totalSeconds / 60);
  const secs = totalSeconds % 60;
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

/** Active study session scoped to one resumable session record. */
export function StudySessionPage({ sessionId }: StudySessionPageProps) {
  const { session, loading, error, pauseSession, resumeSession, addTutorExchange, setData } =
    useSession(sessionId);
  const attempt = useAttempt(session, setData);
  const adaptive = useAdaptiveSession(session?.workspaceId, sessionId);
  const { concept } = useConcept(session?.conceptId ?? '');
  const { workspace } = useWorkspaceDetails(session?.workspaceId ?? '');
  const { navigate } = useNavigation();
  const [working, setWorking] = useState('r = x, h = x² − 1\n= 2π∫ (x³ − x) dx');
  const [question, setQuestion] = useState('');
  const [tutorUnavailableFor, setTutorUnavailableFor] = useState<string | null>(null);
  const [intentNoteVisible, setIntentNoteVisible] = useState(false);
  const [mutationError, setMutationError] = useState('');
  const resumedOnOpen = useRef(false);

  useEffect(() => {
    if (!resumedOnOpen.current && session?.status === 'paused') {
      resumedOnOpen.current = true;
      void resumeSession();
    }
  }, [resumeSession, session?.status]);

  if (loading || !session)
    return (
      <div className={styles.state} role="status">
        {error ? error.message : 'Opening session…'}
      </div>
    );

  const submitQuestion = async (event: FormEvent) => {
    event.preventDefault();
    const trimmed = question.trim();
    if (!trimmed) return;
    setMutationError('');
    try {
      const response = await addTutorExchange(trimmed);
      setTutorUnavailableFor(response.tutorStatus === 'unavailable' ? sessionId : null);
      if (!response.tutorStatus) setQuestion('');
    } catch (caught) {
      setMutationError(caught instanceof Error ? caught.message : 'Could not ask the tutor.');
    }
  };

  const pause = async () => {
    setMutationError('');
    try {
      await pauseSession();
      void adaptive.pause().catch(() => undefined);
    } catch (caught) {
      setMutationError(caught instanceof Error ? caught.message : 'Could not pause the session.');
    }
  };

  const check = async () => {
    setMutationError('');
    try {
      const currentAnswer = attempt.answer;
      await attempt.check();
      void adaptive.checkAnswer(currentAnswer).catch(() => undefined);
    } catch (caught) {
      setMutationError(caught instanceof Error ? caught.message : 'Could not check the answer.');
    }
  };

  const hint = async () => {
    setMutationError('');
    try {
      await attempt.hint();
      void adaptive.requestHint().catch(() => undefined);
    } catch (caught) {
      setMutationError(caught instanceof Error ? caught.message : 'Could not request a hint.');
    }
  };

  const next = async () => {
    setMutationError('');
    try {
      await attempt.next();
      void adaptive.advanceProblem().catch(() => undefined);
    } catch (caught) {
      setMutationError(
        caught instanceof Error ? caught.message : 'Could not advance to the next problem.',
      );
    }
  };

  const finishSession = async () => {
    setMutationError('');
    try {
      await adaptive.endSession();
      navigate({ type: 'sessionHistory', workspaceId: session.workspaceId, sessionId });
    } catch (caught) {
      setMutationError(
        caught instanceof Error ? caught.message : 'Could not complete the session.',
      );
    }
  };

  const liveElapsedMinutes = Math.max(
    session.elapsedMinutes,
    Math.floor(adaptive.liveElapsedMs / 60_000),
  );
  const targetMinutes =
    adaptive.plan?.targetMs != null
      ? Math.round(adaptive.plan.targetMs / 60_000)
      : (session.intent.targetMinutes ?? liveElapsedMinutes);

  return (
    <>
      <SessionShell
        toolbar={
          <SessionToolbar
            conceptName={concept?.name ?? 'Current concept'}
            subjectLine={workspace?.name ?? 'Workspace'}
            intent={session.intent}
            onChangeIntent={() => setIntentNoteVisible((visible) => !visible)}
            problemIndex={session.problemIndex ?? 1}
            problemCount={session.problemCount ?? 1}
            elapsedMinutes={liveElapsedMinutes}
            targetMinutes={targetMinutes}
            onPause={pause}
          />
        }
        visualization={
          <VisualizationPane onExpand={() => navigate({ type: 'fullVisualization', sessionId })} />
        }
        problem={
          <ProblemPane
            session={session}
            attempt={attempt}
            adaptive={adaptive}
            working={working}
            onWorkingChange={(val) => {
              setWorking(val);
              adaptive.setWorkingLines(val.split('\n'));
            }}
            onCheck={check}
            onHint={hint}
            onNext={next}
            onFinishSession={finishSession}
            onInspectPattern={(patternId) =>
              navigate({ type: 'errorProfile', workspaceId: session.workspaceId, patternId })
            }
          />
        }
        tutor={
          <TutorPane
            session={session}
            unavailable={tutorUnavailableFor === sessionId}
            question={question}
            onQuestionChange={setQuestion}
            onSubmit={submitQuestion}
          />
        }
      />
      {intentNoteVisible ? (
        <p className={styles.intentNote}>
          Intent changes steer the session without limiting what you can open.
        </p>
      ) : null}
      {mutationError ? (
        <p className={styles.error} role="alert">
          {mutationError}
        </p>
      ) : null}
    </>
  );
}

function VisualizationPane({ onExpand }: { onExpand: () => void }) {
  return (
    <section className={styles.visualization} aria-label="Shell-method visualization">
      <div className={styles.visualizationTools}>
        <Button variant="dark" size="sm">
          Rotate
        </Button>
        <Button variant="tertiary" size="sm">
          Slice
        </Button>
        <Button variant="tertiary" size="sm">
          Revolve
        </Button>
      </div>
      <Button className={styles.expand} variant="secondary" size="sm" onClick={onExpand}>
        Full visualization
      </Button>
      <p className={styles.placeholder}>
        solid of revolution — shells
        <br />
        region y = x² − 1 on [1, 3], revolved about x = 0
      </p>
      <div className={styles.readout}>
        <span>radius r = x</span>
        <span>height h = x² − 1</span>
        <span>drag a shell to inspect</span>
      </div>
    </section>
  );
}

function ProblemPane({
  session,
  attempt,
  adaptive,
  working,
  onWorkingChange,
  onCheck,
  onHint,
  onNext,
  onFinishSession,
  onInspectPattern,
}: {
  session: Session;
  attempt: ReturnType<typeof useAttempt>;
  adaptive: ReturnType<typeof useAdaptiveSession>;
  working: string;
  onWorkingChange: (value: string) => void;
  onCheck: () => void | Promise<void>;
  onHint: () => void | Promise<void>;
  onNext: () => void | Promise<void>;
  onFinishSession: () => void | Promise<void>;
  onInspectPattern: (patternId: string) => void;
}) {
  const [whyOpen, setWhyOpen] = useState(false);
  const [changesDismissed, setChangesDismissed] = useState(false);

  const solved = attempt.attempt?.status === 'solved';
  const counter = attempt.attempt
    ? `Problem ${session.problemIndex ?? 1}`
    : `Problem ${session.problemIndex ?? 1} of ${session.problemCount ?? 1}`;

  const plan = adaptive.plan;
  const currentSlot =
    plan?.queue.find((s) => s.state === 'current') ??
    plan?.queue[(session.problemIndex ?? 1) - 1] ??
    plan?.queue[0];
  const currentPhase = plan?.phases.find((p) => p.id === currentSlot?.phaseId);
  const verification = adaptive.attempt?.lastVerification;
  const masterySnap =
    adaptive.receipt?.masteryChanges[0]?.state === 'available'
      ? adaptive.receipt.masteryChanges[0].value
      : undefined;
  const patternSnap =
    adaptive.receipt?.patternChanges[0]?.state === 'available'
      ? adaptive.receipt.patternChanges[0].value
      : undefined;
  const latestPlanChange = plan?.changes[plan.changes.length - 1];

  return (
    <section className={styles.problemPane} aria-labelledby="problem-heading">
      <div className={styles.problemHeaderBar}>
        <p className={styles.eyebrow} id="problem-heading">
          {counter}
        </p>
        <div className={styles.metaBadges}>
          {currentPhase ? <span className={styles.metaPill}>{currentPhase.name}</span> : null}
          {plan?.difficulty ? <span className={styles.metaPill}>{plan.difficulty}</span> : null}
          <span
            className={`${styles.metaPill} ${styles.timerPill}`}
            data-testid="live-session-timer"
          >
            {formatClockMs(adaptive.liveElapsedMs)}
          </span>
          {currentSlot?.reason ? (
            <Button
              size="sm"
              variant="tertiary"
              aria-expanded={whyOpen}
              onClick={() => setWhyOpen((open) => !open)}
            >
              Why this question
            </Button>
          ) : null}
        </div>
      </div>

      {whyOpen && currentSlot?.reason ? (
        <aside className={styles.inspectorCard} aria-label="Why this question was selected">
          <div className={styles.inspectorHeader}>
            <h3 className={styles.inspectorTitle}>Why this question</h3>
            <Button size="sm" variant="tertiary" onClick={() => setWhyOpen(false)}>
              Dismiss
            </Button>
          </div>
          <p className={styles.feedback}>{currentSlot.reason}</p>
        </aside>
      ) : null}

      {attempt.error ? (
        <p className={styles.state} role="status">
          {attempt.error.message}
        </p>
      ) : null}
      {!attempt.attempt && !attempt.loading && !attempt.error ? (
        <>
          <p className={styles.problemText}>There is no practice content for this concept yet.</p>
          <WorkingArea value={working} onChange={onWorkingChange} />
        </>
      ) : null}
      {attempt.attempt ? (
        <>
          <p className={styles.problemText}>{attempt.attempt.prompt}</p>
          <WorkingArea value={working} onChange={onWorkingChange} />
          <label className={styles.answerLabel}>
            <span className={styles.answerLabelText}>Answer</span>
            <input
              className={styles.answerInput}
              value={attempt.answer}
              inputMode={attempt.attempt.responseType === 'numeric' ? 'decimal' : 'text'}
              onChange={(event) => attempt.setAnswer(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter' && event.metaKey) void onCheck();
              }}
            />
          </label>
          {attempt.evaluation && !attempt.evaluation.correct ? (
            <p className={`${styles.feedback} ${styles.feedbackMiss}`} role="status">
              That does not match yet.
            </p>
          ) : null}
          {solved ? (
            <p className={`${styles.feedback} ${styles.feedbackCorrect}`} role="status">
              Correct.
            </p>
          ) : null}

          {!changesDismissed && (verification || masterySnap || patternSnap || latestPlanChange) ? (
            <section
              className={styles.inspectorCard}
              aria-label="What this check changed"
            >
              <div className={styles.inspectorHeader}>
                <h3 className={styles.inspectorTitle}>What this check changed</h3>
                <Button size="sm" variant="tertiary" onClick={() => setChangesDismissed(true)}>
                  Dismiss
                </Button>
              </div>
              {verification ? (
                <p className={styles.feedback}>
                  Checker ({verification.method}): verdict <strong>{verification.verdict}</strong>
                  {typeof verification.studentValue === 'number'
                    ? ` · evaluated ${verification.studentValue.toFixed(4)}`
                    : ''}
                  {verification.parseError ? ` · ${verification.parseError}` : ''}
                </p>
              ) : null}
              {masterySnap ? (
                <p className={styles.feedback}>
                  Mastery updated to <strong>{Math.round(masterySnap.score * 100)}%</strong> (
                  {masterySnap.namedState})
                </p>
              ) : null}
              {patternSnap ? (
                <div>
                  <p className={styles.feedback}>
                    Pattern <strong>{patternSnap.name}</strong>: {patternSnap.status} (
                    {patternSnap.cleanStreak}/{patternSnap.resolution.requiredClean} clean streak)
                  </p>
                  <Button
                    size="sm"
                    variant="tertiary"
                    onClick={() => onInspectPattern(patternSnap.patternId)}
                  >
                    Inspect in Error profile
                  </Button>
                </div>
              ) : null}
              {latestPlanChange ? (
                <div className={styles.queueReplacedNotice}>
                  Queue adapted: {latestPlanChange.reason}
                </div>
              ) : null}
              {adaptive.attempt?.workedSolution ? (
                <div className={styles.workedSolutionBox}>
                  Worked structure: {adaptive.attempt.workedSolution}
                </div>
              ) : null}
            </section>
          ) : null}

          {attempt.revealedHints.length ? (
            <ul className={styles.hintList}>
              {attempt.revealedHints.map((text, index) => (
                <li key={index} className={styles.hintItem}>
                  {text}
                </li>
              ))}
            </ul>
          ) : null}
          {attempt.evaluation &&
          !attempt.evaluation.correct &&
          attempt.attempt.hintsRevealed >= attempt.attempt.hintsTotal ? (
            <p className={styles.feedback}>No further hints remain for this problem.</p>
          ) : null}
          <div className={styles.problemActions}>
            {solved ? (
              <>
                <Button onClick={() => void onNext()}>Next problem</Button>
                <Button variant="secondary" onClick={() => void onFinishSession()}>
                  Finish session
                </Button>
              </>
            ) : (
              <>
                <Button onClick={() => void onCheck()}>Check</Button>
                <Button variant="secondary" onClick={() => void onHint()}>
                  Hint
                </Button>
                <Button variant="tertiary" onClick={() => void onFinishSession()}>
                  Finish session
                </Button>
                <span className={styles.shortcutHint}>⌘↵ to check</span>
              </>
            )}
          </div>
        </>
      ) : null}
    </section>
  );
}

function TutorPane({
  session,
  unavailable,
  question,
  onQuestionChange,
  onSubmit,
}: {
  session: Session;
  unavailable: boolean;
  question: string;
  onQuestionChange: (value: string) => void;
  onSubmit: (event: FormEvent) => void;
}) {
  const current = session.exchanges[session.exchanges.length - 1];
  const earlier = session.exchanges.slice(0, -1);
  const hasSettled = Boolean(session.settledConclusions.length || session.openQuestion);
  const tutorMode = hasSettled ? 'Coach' : 'Socratic';

  return (
    <section className={styles.tutorPane} aria-labelledby="tutor-heading">
      <header className={styles.tutorHeader}>
        <h2 id="tutor-heading" className={styles.tutorHeading}>
          <span className={styles.tutorDot} aria-hidden="true" />
          Tutor · {tutorMode}
        </h2>
        <span className={styles.tutorCount}>{session.exchanges.length} exchanges today</span>
      </header>
      {session.settledConclusions.length || session.openQuestion ? (
        <section className={styles.settled} aria-labelledby="settled-heading">
          <h3 id="settled-heading" className={styles.settledTitle}>
            What we’ve settled
          </h3>
          <ul className={styles.settledList}>
            {session.settledConclusions.slice(0, 2).map((conclusion) => (
              <li key={conclusion} className={styles.settledItem}>
                {conclusion}
              </li>
            ))}
            {session.openQuestion ? (
              <li className={styles.openQuestion}>Open: {session.openQuestion}</li>
            ) : null}
          </ul>
          <div className={styles.settledFooter}>
            <Button variant="tertiary" size="sm">
              Save to concept notes
            </Button>
            {earlier.length ? (
              <details className={styles.earlierDetails}>
                <summary className={styles.earlierSummary}>Earlier today</summary>
                <ol className={styles.earlierList}>
                  {earlier.map((exchange) => (
                    <li key={exchange.id} className={styles.earlierItem}>
                      <p>{exchange.question}</p>
                      <p>{exchange.answer}</p>
                    </li>
                  ))}
                </ol>
              </details>
            ) : null}
          </div>
        </section>
      ) : null}
      {current ? (
        <article className={styles.currentExchange} aria-label="Current tutor exchange">
          <div className={styles.timestampDivider}>
            <span>Now · problem {session.problemIndex ?? 1}</span>
          </div>
          <p className={styles.currentAnswer}>{current.answer}</p>
        </article>
      ) : (
        <p className={styles.emptyPrompt}>Ask about the step you are working on.</p>
      )}
      <div role="status" aria-label="Tutor availability">
        {unavailable ? (
          <p className={styles.currentAnswer}>
            Tutor replies are not available in this build yet.
          </p>
        ) : null}
      </div>
      {session.elapsedMinutes >= 90 ? (
        <p className={styles.breakSuggestion}>
          A break now would protect what you’ve built. Stop here and review tomorrow?
        </p>
      ) : null}
      <form className={styles.tutorForm} onSubmit={onSubmit}>
        <label>
          <span className={styles.srOnly}>Ask about this step</span>
          <input
            className={styles.tutorInput}
            value={question}
            onChange={(event) => onQuestionChange(event.target.value)}
            placeholder="Ask about this step…"
          />
        </label>
        <Button type="submit" variant="secondary" disabled={!question.trim()}>
          Ask
        </Button>
      </form>
    </section>
  );
}
