import { useState } from 'react';

import { ConceptRow } from '../components/concept/ConceptRow';
import { ReasonedRecommendation } from '../components/feedback/ReasonedRecommendation';
import { SuggestionPanel } from '../components/feedback/SuggestionPanel';
import { Button } from '../components/primitives/Button';
import { useConcepts } from '../hooks/useConcepts';
import { useGoals } from '../hooks/useGoals';
import { useNavigation } from '../hooks/useNavigation';
import { useActiveSession } from '../hooks/useSessions';
import { useWorkspaceDetails } from '../hooks/useWorkspaces';
import { TwoPaneLayout } from '../layouts/TwoPaneLayout';
import type { Concept } from '../types';
import styles from './WorkspaceOverviewPage.module.css';

export interface WorkspaceOverviewPageProps {
  workspaceId: string;
}

/** Goal-oriented overview for one workspace. */
export function WorkspaceOverviewPage({ workspaceId }: WorkspaceOverviewPageProps) {
  const { workspace } = useWorkspaceDetails(workspaceId);
  const { goals } = useGoals(workspaceId);
  const { concepts } = useConcepts(workspaceId);
  const { session } = useActiveSession(workspaceId);
  const { navigate, openOverlay } = useNavigation();
  const [showSuggestion, setShowSuggestion] = useState(true);
  const guidingGoal = goals.find((goal) => goal.id === workspace?.guidingGoalId);
  const conceptsInPlay = selectConceptsInPlay(concepts);

  return (
    <TwoPaneLayout
      className={styles.layout}
      rail={(
        <div className={styles.railContent}>
          <section className={styles.tools}>
            <span className={styles.eyebrow}>Tools</span>
            <div>{['Tutor', 'Practice', 'Visualize', 'Notes'].map((tool) => (
              <button
                type="button"
                key={tool}
                disabled={!session}
                onClick={() => session && navigate(
                  tool === 'Visualize'
                    ? { type: 'fullVisualization', sessionId: session.id }
                    : { type: 'studySession', sessionId: session.id },
                )}
              >
                {tool}
              </button>
            ))}</div>
            <button type="button" className={styles.link} onClick={() => navigate({ type: 'workspaceTools', workspaceId })}>All tools &amp; modules</button>
          </section>
          <section className={styles.recent}>
            <span className={styles.eyebrow}>Recent</span>
            <ul>
              <li>Tutor — “{session?.exchanges[session.exchanges.length - 1]?.question ?? 'How does the radius change when the axis moves?'}”</li>
              <li>Note — shells vs. washers, when to pick which</li>
              <li>Visualization — region revolved about y = 2</li>
            </ul>
          </section>
          {showSuggestion ? (
            <SuggestionPanel
              message="Series is next in the course. Nothing is scheduled yet. Add it to the goal?"
              acceptLabel="Add to plan"
              onAccept={() => guidingGoal && openOverlay({ type: 'goalEditing', workspaceId, goalId: guidingGoal.id })}
              onDismiss={() => setShowSuggestion(false)}
            />
          ) : null}
        </div>
      )}
    >
      <div className={styles.mainContent}>
        <header className={styles.header}>
          <h1>{workspace?.name ?? 'Workspace'}</h1>
          <p>
            {guidingGoal?.text ?? 'Your guiding goal is loading.'}
            {guidingGoal?.inferred.deadline ? <span> · {guidingGoal.inferred.deadline}</span> : null}
            {guidingGoal ? <button type="button" onClick={() => openOverlay({ type: 'goalEditing', workspaceId, goalId: guidingGoal.id })}>Edit goal</button> : null}
          </p>
        </header>

        {session ? (
          <section className={styles.continueCard}>
            <span className={styles.eyebrow}>Continue</span>
            <h2>{workspace?.lastConceptName ?? concepts.find((concept) => concept.id === session.conceptId)?.name}</h2>
            <p>{session.problemIndex && session.problemCount ? `Problem ${session.problemIndex} of ${session.problemCount} · ` : ''}{session.resumeSummary}</p>
            <div><Button onClick={() => navigate({ type: 'studySession', sessionId: session.id })}>Resume</Button><Button variant="secondary" onClick={() => navigate({ type: 'conceptsList', workspaceId })}>Start something else</Button></div>
          </section>
        ) : null}

        <ReasonedRecommendation
          action="Three problems on choosing radius vs. height"
          evidence="You've set up shells correctly twice but picked the wrong radius when the axis wasn't x = 0."
          ctaLabel="Start · 8 min"
          onStart={() => session && navigate({ type: 'studySession', sessionId: session.id })}
          onAlternative={() => navigate({ type: 'conceptsList', workspaceId })}
          observations={[
            { date: 'Tuesday', text: 'The radius was measured from the curve instead of the shifted axis.' },
            { date: 'Thursday', text: 'The setup was correct when the axis was x = 0.' },
          ]}
        />

        <section className={styles.concepts}>
          <header><h2>Concepts in play</h2><button type="button" onClick={() => navigate({ type: 'conceptsList', workspaceId })}>All {concepts.length}</button></header>
          <div>
            {conceptsInPlay.map((concept, index) => (
              <ConceptRow
                key={concept.id}
                name={concept.name}
                masteryState={concept.masteryState}
                statusText={conceptStatus(concept, index)}
                onSelect={() => navigate({ type: 'conceptView', workspaceId, conceptId: concept.id })}
              />
            ))}
          </div>
        </section>
      </div>
    </TwoPaneLayout>
  );
}

function selectConceptsInPlay(concepts: Concept[]) {
  const preferred = ['Shell method', 'Washer method', 'Integration by parts', 'Taylor series'];
  const selected = preferred.map((name) => concepts.find((concept) => concept.name === name)).filter((concept): concept is Concept => Boolean(concept));
  return selected.length === preferred.length ? selected : concepts.slice(0, 4);
}

function conceptStatus(concept: Concept, index: number) {
  if (index === 0) return 'active';
  if (concept.dueForReviewInDays === 0 || index === 2) return 'due for review';
  if (concept.masteryState === 'New') return 'not started';
  return '2 days ago';
}
