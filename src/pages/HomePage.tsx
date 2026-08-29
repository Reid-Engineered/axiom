import { useState, type ReactNode } from 'react';

import { MathDisplay } from '../components/math/MathDisplay';
import { Mastery } from '../components/mastery/Mastery';
import { Button } from '../components/primitives/Button';
import { Placeholder } from '../components/primitives/Placeholder';
import { WorkspaceCard } from '../components/workspace/WorkspaceCard';
import { useConcept, useConcepts } from '../hooks/useConcepts';
import { useGoals } from '../hooks/useGoals';
import { useNavigation } from '../hooks/useNavigation';
import { useActiveSession } from '../hooks/useSessions';
import { useWorkspace } from '../hooks/useWorkspace';
import { useRecentWorkspaceActivity, useWorkspaces } from '../hooks/useWorkspaces';
import { AppShell } from '../layouts/AppShell';
import type { Session, Workspace } from '../types';
import styles from './HomePage.module.css';

export type HomePageVariant = 'default' | 'session-intent' | 'library';

export interface HomePageProps {
  variant?: HomePageVariant;
  sidebar?: ReactNode;
}

const LONG_ABSENCE_DAYS = 30;

/** Home context and workspace entry points in one of the three specified variants. */
export function HomePage({ variant = 'default', sidebar }: HomePageProps) {
  const { workspaces, loading } = useWorkspaces();
  const { activeWorkspaceId } = useWorkspace();
  const primaryWorkspace =
    workspaces.find((workspace) => workspace.id === activeWorkspaceId) ?? workspaces[0];

  if (variant === 'library') {
    return (
      <AppShell>
        <LibraryHome
          workspaces={workspaces}
          loading={loading}
          primaryWorkspace={primaryWorkspace}
        />
      </AppShell>
    );
  }

  if (variant === 'session-intent') {
    return (
      <AppShell>
        <SessionIntentHome primaryWorkspace={primaryWorkspace} />
      </AppShell>
    );
  }

  return (
    <AppShell sidebar={sidebar}>
      <DefaultHome workspaces={workspaces} loading={loading} primaryWorkspace={primaryWorkspace} />
    </AppShell>
  );
}

function DefaultHome({
  workspaces,
  loading,
  primaryWorkspace,
}: {
  workspaces: Workspace[];
  loading: boolean;
  primaryWorkspace?: Workspace;
}) {
  const { navigate } = useNavigation();

  return (
    <main className={styles.defaultMain}>
      <p className={styles.rememberedContext}>
        {primaryWorkspace?.lastActivityAt
          ? `${relativeActivity(primaryWorkspace.lastActivityAt)} since your last session`
          : 'Your workspace context is ready.'}
      </p>
      {primaryWorkspace ? <PrimaryContextCard workspace={primaryWorkspace} /> : null}
      <section className={styles.workspaceSection}>
        <header>
          <h1>Workspaces</h1>
          <button type="button" onClick={() => navigate({ type: 'createWorkspace' })}>
            New Workspace
          </button>
        </header>
        {loading ? (
          <p className={styles.loading}>Loading workspaces…</p>
        ) : (
          <div className={styles.workspaceGrid}>
            {workspaces.map((workspace) => (
              <WorkspaceGoalCard key={workspace.id} workspace={workspace} />
            ))}
          </div>
        )}
      </section>
      <nav className={styles.footerLinks} aria-label="Workspace resources">
        <button type="button" onClick={() => navigate({ type: 'marketplace' })}>
          Templates
        </button>
        <button
          type="button"
          onClick={() => navigate({ type: 'marketplace', forWorkspaceId: primaryWorkspace?.id })}
        >
          Marketplace
        </button>
        <button type="button" onClick={() => navigate({ type: 'createWorkspace' })}>
          Import material
        </button>
      </nav>
    </main>
  );
}

function PrimaryContextCard({ workspace }: { workspace: Workspace }) {
  const longAbsence = workspace.lastActivityAt
    ? daysSince(workspace.lastActivityAt) >= LONG_ABSENCE_DAYS
    : false;
  return longAbsence ? (
    <ContextRecovery workspace={workspace} />
  ) : (
    <ContinueCard workspace={workspace} />
  );
}

function ContextRecovery({ workspace }: { workspace: Workspace }) {
  const { concepts } = useConcepts(workspace.id);
  const { events } = useRecentWorkspaceActivity(workspace.id);
  const { session, startSession } = useActiveSession(workspace.id);
  const { navigate } = useNavigation();
  const current = concepts.find((concept) => concept.name === workspace.lastConceptName);
  const held = concepts.find((concept) => !concept.wasMasteryState) ?? concepts[0];
  const decayed = concepts.find((concept) => concept.wasMasteryState);

  const startRefresher = async () => {
    if (!current) return;
    const refresher = await startSession({
      workspaceId: workspace.id,
      conceptId: current.id,
      intent: {
        activity: 'Refreshing',
        detail: `Rebuild context for ${current.name}`,
        targetMinutes: 5,
      },
    });
    navigate({ type: 'studySession', sessionId: refresher.id });
  };

  return (
    <section className={styles.recovery} aria-labelledby="recovery-title">
      <div className={styles.recoveryMain}>
        <span className={styles.eyebrow}>
          Welcome back — {daysSince(workspace.lastActivityAt!)} days
        </span>
        <h1 id="recovery-title">
          You were working with {current?.name ?? workspace.lastConceptName}
        </h1>
        <ul className={styles.recoveryLines}>
          {held ? (
            <li>
              <Mastery state={held.masteryState} size="sm" />
              <span>{held.name} held up while you were away.</span>
            </li>
          ) : null}
          {current ? (
            <li>
              <Mastery state={current.masteryState} size="sm" />
              <span>{current.name} still needs support at the point where you stopped.</span>
            </li>
          ) : null}
          {decayed?.wasMasteryState ? (
            <li>
              <Mastery state={decayed.masteryState} size="sm" />
              <span>
                {decayed.name} changed from {decayed.wasMasteryState} while you were away.
              </span>
            </li>
          ) : null}
        </ul>
        <div className={styles.actions}>
          <Button size="lg" onClick={() => void startRefresher()}>
            5-minute refresher
          </Button>
          <Button
            size="lg"
            variant="secondary"
            disabled={!session}
            onClick={() => session && navigate({ type: 'studySession', sessionId: session.id })}
          >
            Straight back to problem {session?.problemIndex ?? ''}
          </Button>
        </div>
      </div>

      <aside className={styles.faded} aria-labelledby="faded-heading">
        <h2 id="faded-heading">Faded while away</h2>
        {decayed?.wasMasteryState ? (
          <div>
            <Mastery state={decayed.masteryState} size="sm" />
            <span>
              {decayed.name} · was {decayed.wasMasteryState}
            </span>
          </div>
        ) : null}
      </aside>

      <section className={styles.awayEvents} aria-labelledby="away-heading">
        <h2 id="away-heading">While you were away</h2>
        <ol>
          {events.slice(0, 3).map((event) => (
            <li key={event.id}>
              <time dateTime={event.occurredAt}>{formatShortDate(event.occurredAt)}</time>
              <span>{event.summary}</span>
            </li>
          ))}
        </ol>
      </section>
    </section>
  );
}

function ContinueCard({ workspace }: { workspace: Workspace }) {
  const { session } = useActiveSession(workspace.id);
  if (!session) return null;
  return <ContinueSession workspace={workspace} session={session} />;
}

function ContinueSession({ workspace, session }: { workspace: Workspace; session: Session }) {
  const { concept } = useConcept(session.conceptId);
  const { navigate } = useNavigation();

  return (
    <section className={styles.continueCard} aria-labelledby="continue-title">
      <div className={styles.continueCopy}>
        <span className={styles.eyebrow}>Continue</span>
        <h1 id="continue-title">
          {workspace.name} — {concept?.name ?? workspace.lastConceptName}
        </h1>
        <p>{session.resumeSummary}</p>
        {concept?.displayFormula ? (
          <MathDisplay expression={concept.displayFormula} className={styles.formula} />
        ) : null}
        <div className={styles.actions}>
          <Button
            size="lg"
            onClick={() => navigate({ type: 'studySession', sessionId: session.id })}
          >
            Resume session
          </Button>
          <Button
            size="lg"
            variant="secondary"
            onClick={() => navigate({ type: 'workspaceOverview', workspaceId: workspace.id })}
          >
            Open workspace
          </Button>
        </div>
      </div>
      <Placeholder
        label="solid of revolution · last camera position"
        className={styles.thumbnail}
      />
    </section>
  );
}

function WorkspaceGoalCard({ workspace }: { workspace: Workspace }) {
  const { goals } = useGoals(workspace.id);
  const { navigate } = useNavigation();
  const { setActiveWorkspaceId } = useWorkspace();
  const guidingGoal = goals.find((goal) => goal.id === workspace.guidingGoalId);

  return (
    <WorkspaceCard
      name={workspace.name}
      goalSentence={guidingGoal?.text ?? 'Goal details are being prepared.'}
      progress={workspace.progress}
      lastConceptName={workspace.lastConceptName}
      lastActivityLabel={
        workspace.lastActivityAt ? relativeActivity(workspace.lastActivityAt) : undefined
      }
      paused={workspace.paused}
      onSelect={() => {
        setActiveWorkspaceId(workspace.id);
        navigate({ type: 'workspaceOverview', workspaceId: workspace.id });
      }}
    />
  );
}

function SessionIntentHome({ primaryWorkspace }: { primaryWorkspace?: Workspace }) {
  const { navigate } = useNavigation();
  const { session } = useActiveSession(primaryWorkspace?.id ?? '');
  const [selectedTime, setSelectedTime] = useState('30 min');
  const plans = [
    {
      duration: '12′',
      title: `Finish the ${primaryWorkspace?.lastConceptName?.toLowerCase() ?? 'problem'} you left open`,
      reason: session?.resumeSummary ?? 'Return to the exact place you stopped.',
    },
    {
      duration: '10′',
      title: 'Three problems on choosing radius vs. height',
      reason: 'You missed this twice on Tuesday.',
    },
    {
      duration: '8′',
      title: 'Review: Integration by Parts',
      reason: 'Due for review — solid last time.',
    },
  ];

  return (
    <div className={styles.intentLayout}>
      <aside className={styles.intentRail} aria-label="Workspace shortcuts">
        <span>⌂</span>
        <span>■</span>
        <span>■</span>
        <span>■</span>
        <span>JR</span>
      </aside>
      <main className={styles.intentMain}>
        <h1>How much time do you have?</h1>
        <p>{primaryWorkspace?.name ?? 'Workspace'} · final in 15 days</p>
        <div className={styles.timeChoices}>
          {['30 min', '15 min', 'An hour', 'Just browsing'].map((choice) => (
            <Button
              key={choice}
              variant={choice === selectedTime ? 'dark' : 'secondary'}
              onClick={() => setSelectedTime(choice)}
            >
              {choice}
            </Button>
          ))}
        </div>
        <div className={styles.planList}>
          {plans.map((plan, index) => (
            <article key={plan.title}>
              <span>{plan.duration}</span>
              <div>
                <h2>{plan.title}</h2>
                <p>{plan.reason}</p>
              </div>
              <Button
                variant={index === 0 ? 'primary' : 'tertiary'}
                onClick={() => session && navigate({ type: 'studySession', sessionId: session.id })}
              >
                {index === 0 ? 'Resume' : 'Start'}
              </Button>
            </article>
          ))}
        </div>
        <button
          className={styles.directLink}
          type="button"
          onClick={() =>
            primaryWorkspace &&
            navigate({ type: 'workspaceOverview', workspaceId: primaryWorkspace.id })
          }
        >
          Or open a workspace directly →
        </button>
      </main>
    </div>
  );
}

function LibraryHome({
  workspaces,
  loading,
  primaryWorkspace,
}: {
  workspaces: Workspace[];
  loading: boolean;
  primaryWorkspace?: Workspace;
}) {
  const { navigate } = useNavigation();
  const { session } = useActiveSession(primaryWorkspace?.id ?? '');

  return (
    <main className={styles.libraryMain}>
      <header className={styles.libraryHeader}>
        <strong>Axiom</strong>
        <nav>
          <button type="button" onClick={() => navigate({ type: 'marketplace' })}>
            Templates
          </button>
          <button type="button" onClick={() => navigate({ type: 'marketplace' })}>
            Marketplace
          </button>
          <Button variant="dark" onClick={() => navigate({ type: 'createWorkspace' })}>
            New
          </Button>
        </nav>
      </header>
      {session && primaryWorkspace ? (
        <button
          className={styles.resumeStrip}
          type="button"
          onClick={() => navigate({ type: 'studySession', sessionId: session.id })}
        >
          <span>●</span>
          <strong>Pick up: {primaryWorkspace.lastConceptName}</strong>
          <span>{primaryWorkspace.name} · mid-problem</span>
          <b>Resume ↵</b>
        </button>
      ) : null}
      {!loading ? (
        <div className={styles.libraryGrid}>
          {workspaces.map((workspace) => (
            <div className={styles.libraryCard} key={workspace.id}>
              <Placeholder label={`${workspace.lastConceptName ?? workspace.name} visualization`} />
              <WorkspaceGoalCard workspace={workspace} />
            </div>
          ))}
        </div>
      ) : null}
    </main>
  );
}

function relativeActivity(timestamp: string) {
  const days = daysSince(timestamp);
  if (days === 0) return 'today';
  if (days === 1) return 'yesterday';
  return `${days} days ago`;
}

function daysSince(timestamp: string) {
  return Math.max(0, Math.round((Date.now() - new Date(timestamp).getTime()) / 86_400_000));
}

function formatShortDate(timestamp: string) {
  return new Intl.DateTimeFormat('en-US', { month: 'short', day: 'numeric' }).format(
    new Date(timestamp),
  );
}
