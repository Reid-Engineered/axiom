import { useEffect, useMemo, useState, type ReactNode } from 'react';

import { MaterialResultRow } from '../components/material/MaterialResultRow';
import { useConcepts } from '../hooks/useConcepts';
import { useMaterial } from '../hooks/useMaterial';
import { useNavigation } from '../hooks/useNavigation';
import { AppShell } from '../layouts/AppShell';
import styles from './MaterialPage.module.css';

export interface MaterialPageProps {
  workspaceId: string;
  sidebar?: ReactNode;
}

/** Concept-linked material search for one workspace. */
export function MaterialPage({ workspaceId, sidebar }: MaterialPageProps) {
  const { material, searchResults, search, loading, error } = useMaterial(workspaceId);
  const { concepts } = useConcepts(workspaceId);
  const { navigate } = useNavigation();
  const [query, setQuery] = useState('shell radius');
  const conceptsById = useMemo(
    () => new Map(concepts.map((concept) => [concept.id, concept])),
    [concepts],
  );

  useEffect(() => {
    void search(query);
  }, [query, search]);

  return (
    <AppShell sidebar={sidebar}>
      <div className={styles.page}>
        <div className={styles.toolbar}>
          <label className={styles.searchLabel}>
            <span className={styles.srOnly}>Search this material</span>
            <input
              type="search"
              value={query}
              placeholder="in Stewart 9e"
              onChange={(event) => setQuery(event.target.value)}
            />
          </label>
          <div className={styles.toolbarRight}>
            <span className={styles.downloaded}>
              <span className={styles.downloadSquare} aria-hidden="true" />
              Downloaded
            </span>
            <button type="button" className={styles.contentsButton}>
              Contents
            </button>
          </div>
        </div>

        <main className={styles.content}>
          {loading ? <p>Loading material…</p> : null}
          {error ? <p role="alert">Material could not be loaded.</p> : null}
          {material ? (
            <>
              <header className={styles.header}>
                <div className={styles.headerTitleRow}>
                  <h1>
                    {material.title}, {material.edition}
                  </h1>
                  <span className={styles.headerStats}>
                    {material.totalPages} pages · {material.totalChapters} chapters
                  </span>
                </div>
                <p className={styles.headerDescription}>
                  Sections are matched to the {concepts.length} concepts in this workspace, so you
                  reach pages through what you are learning rather than through a folder.
                </p>
              </header>

              <div className={styles.columns}>
                <div className={styles.mainColumn}>
                  <section className={styles.results} aria-labelledby="results-heading">
                    <div className={styles.sectionHeading}>
                      <h2 id="results-heading">Matching “{query}”</h2>
                      <span className={styles.sectionDivider} />
                      <span className={styles.resultsBreakdown}>
                        {searchResults.length} typed{' '}
                        {searchResults.length === 1 ? 'result' : 'results'}
                      </span>
                    </div>
                    <div className={styles.resultList}>
                      <div className={styles.rows}>
                        {searchResults.map((result) => {
                          const concept = conceptsById.get(result.conceptId);
                          if (!concept) return null;
                          return (
                            <MaterialResultRow
                              key={result.id}
                              result={result}
                              conceptName={concept.name}
                              masteryState={concept.masteryState}
                              onOpen={() => undefined}
                              onConceptSelect={() =>
                                navigate({
                                  type: 'conceptView',
                                  workspaceId,
                                  conceptId: concept.id,
                                })
                              }
                            />
                          );
                        })}
                      </div>
                    </div>
                  </section>

                  <section className={styles.bookPosition} aria-labelledby="position-heading">
                    <div className={styles.sectionHeading}>
                      <h2 id="position-heading">Where you are in the book</h2>
                      <span className={styles.sectionDivider} />
                    </div>
                    <ol className={styles.segments}>
                      {material.segments.map((segment) => (
                        <li key={segment.label} data-status={segment.status}>
                          <strong>{segment.label}</strong>
                          <span>
                            {segment.status === 'inProgress'
                              ? ' · in progress'
                              : segment.status === 'outOfSyllabus'
                                ? ' · not in your course'
                                : segment.status === 'read'
                                  ? ' · read'
                                  : segment.status === 'next'
                                    ? ' · next'
                                    : ` · ${segment.status}`}
                            {segment.detail ? `, ${segment.detail}` : ''}
                          </span>
                        </li>
                      ))}
                    </ol>
                    <p className={styles.syllabusNote}>
                      Chapters outside your syllabus stay in the book but never appear in
                      recommendations or search-first results.
                    </p>
                  </section>
                </div>

                <aside className={styles.aside} aria-labelledby="marks-heading">
                  <div className={styles.asideSection}>
                    <h2 id="marks-heading" className={styles.asideHeading}>
                      Your marks in this book
                    </h2>
                    <p className={styles.marksSummary}>
                      {material.highlightsCount} highlights · {material.notesCount} notes
                    </p>
                    <p className={styles.marksMost}>
                      Most marked: {material.mostMarkedSections.join(', ')}
                    </p>
                    <button type="button" className={styles.asideLink}>
                      Browse marks
                    </button>
                  </div>

                  <div className={styles.calloutCard}>
                    <h3>Reading in the app pins the tutor to the page</h3>
                    <p>
                      Select a paragraph and ask about it; the answer is anchored to that passage
                      and stays with the concept.
                    </p>
                  </div>
                </aside>
              </div>
            </>
          ) : null}
        </main>
      </div>
    </AppShell>
  );
}
