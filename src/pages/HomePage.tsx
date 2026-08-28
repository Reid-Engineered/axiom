import { useState, type ReactNode } from 'react';

import { MathDisplay } from '../components/math/MathDisplay';
import { Button } from '../components/primitives/Button';
import { Placeholder } from '../components/primitives/Placeholder';
import { WorkspaceCard } from '../components/workspace/WorkspaceCard';
import { useConcept } from '../hooks/useConcepts';
import { useGoals } from '../hooks/useGoals';
import { useNavigation } from '../hooks/useNavigation';
import { useActiveSession } from '../hooks/useSessions';
import { useWorkspace } from '../hooks/useWorkspace';
import { useWorkspaces } from '../hooks/useWorkspaces';
import { AppShell } from '../layouts/AppShell';
import type { Session, Workspace } from '../types';
import styles from './HomePage.module.css';

export type HomePageVariant = 'default' | 'session-intent' | 'library';

export interface HomePageProps {
  variant?: HomePageVariant;
  sidebar?: ReactNode;
}

/** Home context and workspace entry points in one of the three specified variants. */
export function HomePage({ variant = 'default', sidebar }: HomePageProps) {
  const { workspaces, loading } = useWorkspaces();
  const { activeWorkspaceId } = useWorkspace();
  const primaryWorkspace = workspaces.find((workspace) => workspace.id === activeWorkspaceId)
    ?? workspaces[0];

  if (variant === 'library') {
    return (
      <AppShell>
        <LibraryHome workspaces={workspaces} loading={loading} primaryWorkspace={primaryWorkspace} />
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
      <p className={styles.rememberedContext}>Thursday afternoon · 3 days since your last session</p>
      {primaryWorkspace ? <ContinueCard workspace={primaryWorkspace} /> : null}
      <section className={styles.workspaceSection}>
        <header><h1>Workspaces</h1><button type="button" onClick={() => navigate({ type: 'createWorkspace' })}>New Workspace</button></header>
        {loading ? <p className={styles.loading}>Loading workspaces…</p> : (
          <div className={styles.workspaceGrid}>
            {workspaces.map((workspace) => <WorkspaceGoalCard key={workspace.id} workspace={workspace} />)}
          </div>
        )}
      </section>
      <nav className={styles.footerLinks} aria-label="Workspace resources">
        <button type="button" onClick={() => navigate({ type: 'marketplace' })}>Templates</button>
        <button type="button" onClick={() => navigate({ type: 'marketplace', forWorkspaceId: primaryWorkspace?.id })}>Marketplace</button>
        <button type="button" onClick={() => navigate({ type: 'createWorkspace' })}>Import material</button>
      </nav>
    </main>
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
        <h1 id="continue-title">{workspace.name} — {concept?.name ?? workspace.lastConceptName}</h1>
        <p>{session.resumeSummary}</p>
        {concept?.displayFormula ? <MathDisplay expression={concept.displayFormula} className={styles.formula} /> : null}
        <div className={styles.actions}>
          <Button size="lg" onClick={() => navigate({ type: 'studySession', sessionId: session.id })}>Resume session</Button>
          <Button size="lg" variant="secondary" onClick={() => navigate({ type: 'workspaceOverview', workspaceId: workspace.id })}>Open workspace</Button>
        </div>
      </div>
      <Placeholder label="solid of revolution · last camera position" className={styles.thumbnail} />
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
      lastActivityLabel={workspace.lastActivityAt ? relativeActivity(workspace.lastActivityAt) : undefined}
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
    { duration: '12′', title: `Finish the ${primaryWorkspace?.lastConceptName?.toLowerCase() ?? 'problem'} you left open`, reason: session?.resumeSummary ?? 'Return to the exact place you stopped.' },
    { duration: '10′', title: 'Three problems on choosing radius vs. height', reason: 'You missed this twice on Tuesday.' },
    { duration: '8′', title: 'Review: Integration by Parts', reason: 'Due for review — solid last time.' },
  ];

  return (
    <div className={styles.intentLayout}>
      <aside className={styles.intentRail} aria-label="Workspace shortcuts"><span>⌂</span><span>■</span><span>■</span><span>■</span><span>JR</span></aside>
      <main className={styles.intentMain}>
        <h1>How much time do you have?</h1>
        <p>{primaryWorkspace?.name ?? 'Workspace'} · final in 15 days</p>
        <div className={styles.timeChoices}>
          {['30 min', '15 min', 'An hour', 'Just browsing'].map((choice) => (
            <Button key={choice} variant={choice === selectedTime ? 'dark' : 'secondary'} onClick={() => setSelectedTime(choice)}>{choice}</Button>
          ))}
        </div>
        <div className={styles.planList}>
          {plans.map((plan, index) => (
            <article key={plan.title}>
              <span>{plan.duration}</span><div><h2>{plan.title}</h2><p>{plan.reason}</p></div>
              <Button variant={index === 0 ? 'primary' : 'tertiary'} onClick={() => session && navigate({ type: 'studySession', sessionId: session.id })}>{index === 0 ? 'Resume' : 'Start'}</Button>
            </article>
          ))}
        </div>
        <button className={styles.directLink} type="button" onClick={() => primaryWorkspace && navigate({ type: 'workspaceOverview', workspaceId: primaryWorkspace.id })}>Or open a workspace directly →</button>
      </main>
    </div>
  );
}

function LibraryHome({ workspaces, loading, primaryWorkspace }: { workspaces: Workspace[]; loading: boolean; primaryWorkspace?: Workspace }) {
  const { navigate } = useNavigation();
  const { session } = useActiveSession(primaryWorkspace?.id ?? '');

  return (
    <main className={styles.libraryMain}>
      <header className={styles.libraryHeader}><strong>Axiom</strong><nav><button type="button" onClick={() => navigate({ type: 'marketplace' })}>Templates</button><button type="button" onClick={() => navigate({ type: 'marketplace' })}>Marketplace</button><Button variant="dark" onClick={() => navigate({ type: 'createWorkspace' })}>New</Button></nav></header>
      {session && primaryWorkspace ? (
        <button className={styles.resumeStrip} type="button" onClick={() => navigate({ type: 'studySession', sessionId: session.id })}>
          <span>●</span><strong>Pick up: {primaryWorkspace.lastConceptName}</strong><span>{primaryWorkspace.name} · mid-problem</span><b>Resume ↵</b>
        </button>
      ) : null}
      {!loading ? <div className={styles.libraryGrid}>{workspaces.map((workspace) => (
        <div className={styles.libraryCard} key={workspace.id}>
          <Placeholder label={`${workspace.lastConceptName ?? workspace.name} visualization`} />
          <WorkspaceGoalCard workspace={workspace} />
        </div>
      ))}</div> : null}
    </main>
  );
}

function relativeActivity(timestamp: string) {
  const days = Math.max(0, Math.round((Date.now() - new Date(timestamp).getTime()) / 86_400_000));
  if (days === 0) return 'today';
  if (days === 1) return 'yesterday';
  return `${days} days ago`;
}
