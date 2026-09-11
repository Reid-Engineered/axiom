import { useEffect, useRef, useState, type FormEvent } from 'react';

import { Button } from '../components/primitives/Button';
import { SessionToolbar } from '../components/session/SessionToolbar';
import { WorkingArea } from '../components/session/WorkingArea';
import { useAttempt } from '../hooks/useAttempt';
import { useConcept } from '../hooks/useConcepts';
import { useNavigation } from '../hooks/useNavigation';
import { useSession } from '../hooks/useSessions';
import { useWorkspaceDetails } from '../hooks/useWorkspaces';
import { SessionShell } from '../layouts/SessionShell';
import type { Session } from '../types';
import styles from './StudySessionPage.module.css';

export interface StudySessionPageProps {
  sessionId: string;
}

/** Active study session scoped to one resumable session record. */
export function StudySessionPage({ sessionId }: StudySessionPageProps) {
  const { session, loading, error, pauseSession, resumeSession, addTutorExchange, setData } =
    useSession(sessionId);
  const attempt = useAttempt(session, setData);
  const { concept } = useConcept(session?.conceptId ?? '');
  const { workspace } = useWorkspaceDetails(session?.workspaceId ?? '');
  const { navigate } = useNavigation();
  const [working, setWorking] = useState('');
  const [question, setQuestion] = useState('');
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
      await addTutorExchange(trimmed);
      setQuestion('');
    } catch (caught) {
      setMutationError(caught instanceof Error ? caught.message : 'Could not ask the tutor.');
    }
  };

  const pause = async () => {
    setMutationError('');
    try {
      await pauseSession();
    } catch (caught) {
      setMutationError(caught instanceof Error ? caught.message : 'Could not pause the session.');
    }
  };

  const check = async () => {
    setMutationError('');
    try {
      await attempt.check();
    } catch (caught) {
      setMutationError(caught instanceof Error ? caught.message : 'Could not check the answer.');
    }
  };

  const hint = async () => {
    setMutationError('');
    try {
      await attempt.hint();
    } catch (caught) {
      setMutationError(caught instanceof Error ? caught.message : 'Could not request a hint.');
    }
  };

  const next = async () => {
    setMutationError('');
    try {
      await attempt.next();
    } catch (caught) {
      setMutationError(caught instanceof Error ? caught.message : 'Could not advance to the next problem.');
    }
  };

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
            problemCount={session.problemCount}
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
            working={working}
            onWorkingChange={setWorking}
            onCheck={check}
            onHint={hint}
            onNext={next}
          />
        }
        tutor={
          <TutorPane
            session={session}
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
      <p className={styles.placeholder}>Visualization workspace</p>
    </section>
  );
}

function ProblemPane({
  session,
  attempt,
  working,
  onWorkingChange,
  onCheck,
  onHint,
  onNext,
}: {
  session: Session;
  attempt: ReturnType<typeof useAttempt>;
  working: string;
  onWorkingChange: (value: string) => void;
  onCheck: () => void | Promise<void>;
  onHint: () => void | Promise<void>;
  onNext: () => void | Promise<void>;
}) {
  const solved = attempt.attempt?.status === 'solved';
  const counter = `Problem ${session.problemIndex ?? 1}${
    session.problemCount === undefined || session.problemCount === null
      ? ''
      : ` of ${session.problemCount}`
  }`;

  return (
    <section className={styles.problemPane} aria-labelledby="problem-heading">
      <p className={styles.eyebrow} id="problem-heading">
        {counter}
      </p>
      {attempt.error ? (
        <p className={styles.state} role="status">
          {attempt.error.message}
        </p>
      ) : null}
      {!attempt.attempt && !attempt.loading && !attempt.error ? (
        <>
          <p className={styles.problemText}>
            There is no practice content for this concept yet.
          </p>
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
            <p className={styles.feedback}>That does not match yet.</p>
          ) : null}
          {solved ? <p className={styles.feedback}>Correct.</p> : null}
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
              <Button onClick={() => void onNext()}>Next problem</Button>
            ) : (
              <>
                <Button onClick={() => void onCheck()}>Check</Button>
                <Button variant="secondary" onClick={() => void onHint()}>
                  Hint
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
  question,
  onQuestionChange,
  onSubmit,
}: {
  session: Session;
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
