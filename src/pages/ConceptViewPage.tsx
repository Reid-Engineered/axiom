import { useState } from 'react';

import { DiagnosticDot } from '../components/badges/DiagnosticDot';
import { ConceptRow } from '../components/concept/ConceptRow';
import { ConceptTag } from '../components/concept/ConceptTag';
import { Mastery } from '../components/mastery/Mastery';
import { MathDisplay } from '../components/math/MathDisplay';
import { Button } from '../components/primitives/Button';
import { useConcept, useConcepts } from '../hooks/useConcepts';
import { useNavigation } from '../hooks/useNavigation';
import { useActiveSession } from '../hooks/useSessions';
import { useWorkspaceDetails } from '../hooks/useWorkspaces';
import { TwoPaneLayout } from '../layouts/TwoPaneLayout';
import type { Concept, SessionIntent } from '../types';
import styles from './ConceptViewPage.module.css';

export interface ConceptViewPageProps {
  workspaceId: string;
  conceptId: string;
}

/** Detailed mastery, explanation, relationships, and activity for one concept. */
export function ConceptViewPage({ workspaceId, conceptId }: ConceptViewPageProps) {
  const { concept, loading, error } = useConcept(conceptId);
  const { concepts } = useConcepts(workspaceId);
  const { workspace } = useWorkspaceDetails(workspaceId);
  const { startSession } = useActiveSession(workspaceId);
  const { navigate } = useNavigation();
  const [actionError, setActionError] = useState('');

  if (loading || !concept)
    return (
      <div className={styles.state} role="status">
        {error ? error.message : 'Opening concept…'}
      </div>
    );

  const byId = new Map(concepts.map((candidate) => [candidate.id, candidate]));
  const prerequisites = concept.prerequisiteConceptIds
    .map((id) => byId.get(id))
    .filter((candidate): candidate is Concept => Boolean(candidate));
  const leadsTo = concept.leadsToConceptIds
    .map((id) => byId.get(id))
    .filter((candidate): candidate is Concept => Boolean(candidate));

  const begin = async (intent: SessionIntent) => {
    setActionError('');
    try {
      const session = await startSession({ workspaceId, conceptId, intent });
      navigate({ type: 'studySession', sessionId: session.id });
    } catch (caught) {
      setActionError(caught instanceof Error ? caught.message : 'Could not start the session.');
    }
  };

  return (
    <TwoPaneLayout
      rail={
        <ConceptRail
          concept={concept}
          prerequisites={prerequisites}
          leadsTo={leadsTo}
          onSelect={(selected) =>
            navigate({ type: 'conceptView', workspaceId, conceptId: selected.id })
          }
        />
      }
    >
      <article className={styles.article}>
        <nav className={styles.breadcrumb} aria-label="Breadcrumb">
          <span>{workspace?.name ?? 'Workspace'}</span>
          <span>›</span>
          <span>Concepts</span>
          <span>›</span>
          <strong>{concept.name}</strong>
        </nav>
        <header className={styles.header}>
          <h1>{concept.name}</h1>
          <div className={styles.masteryLine}>
            <Mastery state={concept.masteryState} />
            <span>{concept.meaning}</span>
            {concept.dueForReviewInDays !== undefined ? (
              <span className={styles.reviewDue}>
                {concept.dueForReviewInDays === 0
                  ? 'Due for review'
                  : `Due for review in ${concept.dueForReviewInDays} days`}
              </span>
            ) : null}
          </div>
        </header>
        {concept.displayFormula ? (
          <MathDisplay expression={concept.displayFormula} className={styles.formula} />
        ) : null}
        {concept.explanation ? <p className={styles.explanation}>{concept.explanation}</p> : null}
        {concept.learnerHeuristic ? (
          <p className={styles.heuristic}>
            Your notes say you think of it as “{concept.learnerHeuristic}”{' '}
            {concept.heuristicEvidence ? <span>{concept.heuristicEvidence}</span> : null}
          </p>
        ) : null}
        <div className={styles.actions}>
          <Button
            onClick={() =>
              begin({ activity: 'Practising', detail: concept.name, targetMinutes: 20 })
            }
          >
            Practice this
          </Button>
          <Button
            variant="secondary"
            onClick={() => begin({ activity: 'Exploring', detail: `Ask about ${concept.name}` })}
          >
            Ask the tutor
          </Button>
          <Button
            variant="secondary"
            onClick={() =>
              begin({ activity: 'Reflecting', detail: `Explain ${concept.name} back` })
            }
          >
            Explain it back
          </Button>
        </div>
        {concept.whereItShowsUp?.length ? (
          <section className={styles.tags} aria-labelledby="where-heading">
            <h2 id="where-heading">Where it shows up</h2>
            <div className={styles.tagList}>
              {concept.whereItShowsUp.map((label) => (
                <ConceptTag key={label} label={label} />
              ))}
            </div>
          </section>
        ) : null}
        {actionError ? (
          <p className={styles.error} role="alert">
            {actionError}
          </p>
        ) : null}
      </article>
    </TwoPaneLayout>
  );
}

function ConceptRail({
  concept,
  prerequisites,
  leadsTo,
  onSelect,
}: {
  concept: Concept;
  prerequisites: Concept[];
  leadsTo: Concept[];
  onSelect: (concept: Concept) => void;
}) {
  return (
    <div className={styles.rail}>
      <RelatedConcepts title="Builds on" concepts={prerequisites} onSelect={onSelect} />
      <RelatedConcepts title="Leads to" concepts={leadsTo} onSelect={onSelect} />
      <Button variant="tertiary" size="sm">
        See in concept map
      </Button>
      {concept.recentDiagnostics?.length ? (
        <section aria-labelledby="practice-heading">
          <h2 id="practice-heading">Recent practice</h2>
          <ul className={styles.diagnostics}>
            {concept.recentDiagnostics.map((diagnostic) => (
              <li key={diagnostic.id}>
                <DiagnosticDot type={diagnostic.type} tooltip={diagnostic.note} />
                <span className={styles.diagnosticExpression}>{diagnostic.expression}</span>
                <span>— {diagnostic.note}</span>
              </li>
            ))}
          </ul>
        </section>
      ) : null}
      <section aria-labelledby="notes-heading">
        <h2 id="notes-heading">Your notes</h2>
        <p className={styles.notesSummary}>
          {concept.notesCount
            ? `${concept.notesCount} ${concept.notesCount === 1 ? 'note' : 'notes'}`
            : 'No notes yet'}
        </p>
        {concept.notesCount > 1 ? (
          <Button variant="tertiary" size="sm">
            {concept.notesCount - 1} more notes
          </Button>
        ) : null}
      </section>
    </div>
  );
}

function RelatedConcepts({
  title,
  concepts,
  onSelect,
}: {
  title: string;
  concepts: Concept[];
  onSelect: (concept: Concept) => void;
}) {
  const id = `${title.toLowerCase().replace(/ /g, '-')}-heading`;
  return (
    <section aria-labelledby={id}>
      <h2 id={id}>{title}</h2>
      <div className={styles.relatedRows}>
        {concepts.map((concept) => (
          <ConceptRow
            key={concept.id}
            name={concept.name}
            masteryState={concept.masteryState}
            onSelect={() => onSelect(concept)}
          />
        ))}
      </div>
    </section>
  );
}
