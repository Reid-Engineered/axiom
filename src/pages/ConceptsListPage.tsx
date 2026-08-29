import { useMemo, useState, type ReactNode } from 'react';

import { ConceptRow } from '../components/concept/ConceptRow';
import { ChapterStateProfile } from '../components/mastery/ChapterStateProfile';
import { Placeholder } from '../components/primitives/Placeholder';
import { useConcepts } from '../hooks/useConcepts';
import { useNavigation } from '../hooks/useNavigation';
import { AppShell } from '../layouts/AppShell';
import type { Concept, MasteryState } from '../types';
import styles from './ConceptsListPage.module.css';

export interface ConceptsListPageProps {
  workspaceId: string;
  sidebar?: ReactNode;
}

type ConceptFilter = 'needs-work' | 'due' | 'in-progress' | 'exam' | 'not-started' | 'all';

const NEEDS_WORK_COUNT = 6;
const INITIAL_NEEDS_WORK_COUNT = 3;

function getNeedsWork(concepts: Concept[]) {
  return concepts
    .filter(
      (concept) =>
        concept.masteryState === 'New' ||
        concept.masteryState === 'Developing' ||
        concept.dueForReviewInDays !== undefined,
    )
    .sort(
      (left, right) =>
        right.blocksConceptIds.length - left.blocksConceptIds.length ||
        left.name.localeCompare(right.name),
    )
    .slice(0, NEEDS_WORK_COUNT);
}

function getMasteryCounts(concepts: Concept[]) {
  return concepts.reduce<Partial<Record<MasteryState, number>>>((counts, concept) => {
    counts[concept.masteryState] = (counts[concept.masteryState] ?? 0) + 1;
    return counts;
  }, {});
}

function getStatusText(concept: Concept) {
  if (concept.blocksConceptIds.length > 0) {
    const label = concept.blocksConceptIds.length === 1 ? 'concept' : 'concepts';
    return `blocks ${concept.blocksConceptIds.length} ${label}`;
  }
  if (concept.dueForReviewInDays !== undefined) return 'due for review';
  return concept.onExam ? 'on the exam' : concept.meaning;
}

/** Chapter-grouped concept list for one workspace. */
export function ConceptsListPage({ workspaceId, sidebar }: ConceptsListPageProps) {
  const { concepts, loading, error } = useConcepts(workspaceId);
  const { navigate } = useNavigation();
  const [filter, setFilter] = useState<ConceptFilter>('needs-work');
  const [showAllNeedsWork, setShowAllNeedsWork] = useState(false);
  const [view, setView] = useState<'list' | 'graph'>('list');
  const [query, setQuery] = useState('');

  const dueCount = concepts.filter((concept) => concept.dueForReviewInDays !== undefined).length;
  const inProgressCount = concepts.filter(
    (concept) =>
      concept.masteryState === 'Developing' ||
      concept.masteryState === 'Familiar' ||
      concept.masteryState === 'Strong',
  ).length;
  const examCount = concepts.filter((concept) => concept.onExam).length;
  const notStartedCount = concepts.filter((concept) => concept.masteryState === 'New').length;

  const needsWork = useMemo(() => getNeedsWork(concepts), [concepts]);
  const visibleNeedsWork = needsWork.filter(
    (concept) => !query || concept.name.toLowerCase().includes(query.toLowerCase()),
  );

  const chapterConcepts = concepts.filter((concept) => {
    if (query && !concept.name.toLowerCase().includes(query.toLowerCase())) return false;
    if (filter === 'due') return concept.dueForReviewInDays !== undefined;
    if (filter === 'in-progress')
      return (
        concept.masteryState === 'Developing' ||
        concept.masteryState === 'Familiar' ||
        concept.masteryState === 'Strong'
      );
    if (filter === 'exam') return concept.onExam;
    if (filter === 'not-started') return concept.masteryState === 'New';
    return true;
  });

  const chapters = Array.from(
    chapterConcepts.reduce((groups, concept) => {
      const group = groups.get(concept.chapter) ?? [];
      group.push(concept);
      groups.set(concept.chapter, group);
      return groups;
    }, new Map<string, Concept[]>()),
  );
  const openConcept = (conceptId: string) =>
    navigate({ type: 'conceptView', workspaceId, conceptId });

  return (
    <AppShell sidebar={sidebar}>
      <div className={styles.page}>
        <div className={styles.toolbar}>
          <label className={styles.search}>
            <span className={styles.srOnly}>Search concepts</span>
            <input
              type="search"
              value={query}
              placeholder={`Search ${concepts.length} concepts`}
              onChange={(event) => setQuery(event.target.value)}
            />
          </label>
          <div className={styles.viewToggle} aria-label="Concept view">
            <button type="button" aria-pressed={view === 'list'} onClick={() => setView('list')}>
              List
            </button>
            <button type="button" aria-pressed={view === 'graph'} onClick={() => setView('graph')}>
              Graph
            </button>
          </div>
        </div>

        <main className={styles.content}>
          <header className={styles.header}>
            <h1>Concepts</h1>
            <p>{concepts.length} in this workspace</p>
          </header>

          {loading ? <p>Loading concepts…</p> : null}
          {error ? <p role="alert">Concepts could not be loaded.</p> : null}

          {!loading && !error ? (
            <>
              <div className={styles.filters} aria-label="Concept filters">
                <button
                  type="button"
                  aria-pressed={filter === 'needs-work'}
                  onClick={() => setFilter('needs-work')}
                >
                  Needs work · {needsWork.length}
                </button>
                <button
                  type="button"
                  aria-pressed={filter === 'due'}
                  onClick={() => setFilter('due')}
                >
                  Due for review · {dueCount}
                </button>
                <button
                  type="button"
                  aria-pressed={filter === 'in-progress'}
                  onClick={() => setFilter('in-progress')}
                >
                  In progress · {inProgressCount}
                </button>
                <button
                  type="button"
                  aria-pressed={filter === 'exam'}
                  onClick={() => setFilter('exam')}
                >
                  On the exam · {examCount}
                </button>
                <button
                  type="button"
                  aria-pressed={filter === 'not-started'}
                  onClick={() => setFilter('not-started')}
                >
                  Not started · {notStartedCount}
                </button>
                <button
                  type="button"
                  aria-pressed={filter === 'all'}
                  onClick={() => setFilter('all')}
                >
                  All
                </button>
              </div>

              {view === 'graph' ? (
                <Placeholder label="Concept graph" className={styles.graphPlaceholder} />
              ) : (
                <>
                  {filter === 'needs-work' || filter === 'all' ? (
                    <section
                      className={styles.needsWorkSection}
                      aria-labelledby="needs-work-heading"
                    >
                      <div className={styles.sectionHeading}>
                        <h2 id="needs-work-heading">Needs work</h2>
                        <span className={styles.sectionDivider} />
                        <span className={styles.sectionNote}>ordered by what it blocks</span>
                      </div>
                      <div className={styles.needsWorkCard}>
                        <div className={styles.rows}>
                          {visibleNeedsWork
                            .slice(
                              0,
                              showAllNeedsWork ? NEEDS_WORK_COUNT : INITIAL_NEEDS_WORK_COUNT,
                            )
                            .map((concept) => (
                              <ConceptRow
                                key={concept.id}
                                name={concept.name}
                                masteryState={concept.masteryState}
                                statusText={getStatusText(concept)}
                                onSelect={() => openConcept(concept.id)}
                              />
                            ))}
                        </div>
                        {visibleNeedsWork.length > INITIAL_NEEDS_WORK_COUNT ? (
                          <button
                            type="button"
                            className={styles.showMoreButton}
                            onClick={() => setShowAllNeedsWork((shown) => !shown)}
                            aria-label={
                              showAllNeedsWork
                                ? 'Show fewer'
                                : `${visibleNeedsWork.length - INITIAL_NEEDS_WORK_COUNT} more need work`
                            }
                          >
                            <span className={styles.footerNote}>
                              {showAllNeedsWork
                                ? 'Showing all 6'
                                : `${visibleNeedsWork.length - INITIAL_NEEDS_WORK_COUNT} more need work`}
                            </span>
                            <span className={styles.showAction}>
                              {showAllNeedsWork ? 'Show fewer' : 'Show'}
                            </span>
                          </button>
                        ) : null}
                      </div>
                    </section>
                  ) : null}

                  <section className={styles.chapters} aria-labelledby="chapters-heading">
                    <div className={styles.sectionHeading}>
                      <h2 id="chapters-heading">By chapter</h2>
                      <span className={styles.sectionDivider} />
                    </div>
                    {chapters.length > 0 ? (
                      chapters.map(([chapter, chapterItems]) => (
                        <details key={chapter} className={styles.chapter}>
                          <summary>
                            <span>{chapter}</span>
                            <div className={styles.summaryMeta}>
                              <ChapterStateProfile counts={getMasteryCounts(chapterItems)} />
                              <span className={styles.chapterCount}>
                                {chapterItems.length} concepts
                              </span>
                            </div>
                          </summary>
                          <div className={styles.rows}>
                            {chapterItems.map((concept) => (
                              <ConceptRow
                                key={concept.id}
                                name={concept.name}
                                masteryState={concept.masteryState}
                                statusText={getStatusText(concept)}
                                onSelect={() => openConcept(concept.id)}
                              />
                            ))}
                          </div>
                        </details>
                      ))
                    ) : (
                      <p className={styles.empty}>No concepts match this filter.</p>
                    )}
                  </section>
                </>
              )}
            </>
          ) : null}
        </main>
      </div>
    </AppShell>
  );
}
