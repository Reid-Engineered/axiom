import { useState, type ReactNode } from 'react';

import { Button } from '../components/primitives/Button';
import { useErrorProfile } from '../hooks/useLearningEvidence';
import { useNavigation } from '../hooks/useNavigation';
import { useActiveSession } from '../hooks/useSessions';
import { useWorkspaceDetails } from '../hooks/useWorkspaces';
import { AppShell } from '../layouts/AppShell';
import type { PatternSnapshot } from '../types';
import styles from './ErrorProfilePage.module.css';

export interface ErrorProfilePageProps {
  workspaceId: string;
  initialPatternId?: string;
  sidebar?: ReactNode;
}

const STATUS_FILTERS = ['All', 'Recurring', 'Improving', 'Resolved'] as const;
const RULE_THRESHOLDS = [2, 3, 4, 5] as const;

function statusClass(status: string): string {
  if (status === 'Recurring') return styles.statusRecurring;
  if (status === 'Improving') return styles.statusImproving;
  if (status === 'Resolved') return styles.statusResolved;
  return '';
}

function formatShortDate(iso?: string | null): string {
  if (!iso) return 'Not yet observed';
  const parsed = new Date(iso);
  if (Number.isNaN(parsed.getTime())) return iso;
  return new Intl.DateTimeFormat('en-US', { month: 'short', day: 'numeric' }).format(parsed);
}

/** Domain-scoped Stage 9 Error profile screen (`Profile.dc.html`). */
export function ErrorProfilePage({
  workspaceId,
  initialPatternId,
  sidebar,
}: ErrorProfilePageProps) {
  const { workspace } = useWorkspaceDetails(workspaceId);
  const {
    listResponse,
    loading,
    error,
    statusFilter,
    setStatusFilter,
    search,
    setSearch,
    selectedPatternId,
    setSelectedPatternId,
    selectedDetail,
    savingRule,
    updateRule,
  } = useErrorProfile(workspaceId, initialPatternId);
  const { startSession } = useActiveSession(workspaceId);
  const { navigate } = useNavigation();
  const [actionError, setActionError] = useState('');

  const patternsAvail = listResponse?.patterns;
  const patterns: PatternSnapshot[] =
    patternsAvail?.state === 'available' ? patternsAvail.value : [];

  const handleDrillPattern = async (pattern: PatternSnapshot) => {
    setActionError('');
    try {
      const targetConceptId = pattern.conceptIds?.[0] ?? 'calc-concept-22';
      const session = await startSession({
        workspaceId,
        conceptId: targetConceptId,
        intent: {
          activity: 'Practising',
          detail: `Drill pattern: ${pattern.name}`,
          targetMinutes: 15,
        },
      });
      navigate({ type: 'studySession', sessionId: session.id });
    } catch (caught) {
      setActionError(
        caught instanceof Error ? caught.message : 'Could not start pattern drill session.',
      );
    }
  };

  const handleRuleChange = async (patternId: string, requiredClean: number) => {
    setActionError('');
    try {
      await updateRule(patternId, requiredClean);
    } catch (caught) {
      setActionError(
        caught instanceof Error ? caught.message : 'Could not update resolution rule.',
      );
    }
  };

  const detailAvail = selectedDetail;
  const activeDetail = detailAvail?.state === 'available' ? detailAvail.value : null;

  return (
    <AppShell sidebar={sidebar}>
      <main className={styles.page}>
        <header className={styles.header}>
          <div>
            <div className={styles.eyebrow}>{workspace?.name ?? 'Domain'} · Error profile</div>
            <h1 className={styles.title}>Error profile</h1>
            <p className={styles.subtitle}>
              {listResponse
                ? `Patterns detected across ${listResponse.totalAnswers} checked answers${
                    listResponse.sinceDate ? ` · since ${formatShortDate(listResponse.sinceDate)}` : ''
                  }. A pattern resolves after ${listResponse.requiredCleanDefault} clean problems in a row.`
                : 'Inspecting domain misconception patterns and clean-streak resolution rules.'}
            </p>
          </div>
          <div className={styles.controls}>
            <label>
              <input
                type="search"
                className={styles.searchInput}
                value={search}
                placeholder="Search patterns..."
                aria-label="Search patterns"
                onChange={(event) => setSearch(event.target.value)}
              />
            </label>
            <div className={styles.filterGroup} role="group" aria-label="Pattern status filter">
              {STATUS_FILTERS.map((filter) => {
                const pressed =
                  filter === 'All'
                    ? statusFilter === 'All' || !statusFilter
                    : statusFilter === filter;
                return (
                  <button
                    key={filter}
                    type="button"
                    className={styles.filterButton}
                    aria-pressed={pressed}
                    onClick={() => setStatusFilter(filter)}
                  >
                    {filter}
                  </button>
                );
              })}
            </div>
          </div>
        </header>

        {loading ? <p role="status">Loading error profile…</p> : null}
        {error ? (
          <p className={styles.errorAlert} role="alert">
            {error.message}
          </p>
        ) : null}
        {actionError ? (
          <p className={styles.errorAlert} role="alert">
            {actionError}
          </p>
        ) : null}

        {!loading && patternsAvail?.state === 'unavailable' ? (
          <section className={styles.emptyCard} aria-label="No error patterns">
            <h2 className={styles.emptyTitle}>No error patterns recorded yet</h2>
            <p className={styles.emptyCopy}>
              Error patterns are only attributed when verified misconception opportunities are
              evaluated during practice. Missing evidence is never shown as zero occurrences.
            </p>
            <Button onClick={() => navigate({ type: 'home' })}>Return to Today plan</Button>
          </section>
        ) : null}

        {!loading && patternsAvail?.state === 'available' && patterns.length === 0 ? (
          <section className={styles.emptyCard} aria-label="No matching patterns">
            <h2 className={styles.emptyTitle}>No patterns match the current filter</h2>
            <p className={styles.emptyCopy}>
              Try clearing your search or switching the status filter back to All.
            </p>
            <Button variant="secondary" onClick={() => setStatusFilter('All')}>
              Show all patterns
            </Button>
          </section>
        ) : null}

        {!loading && patterns.length > 0 ? (
          <div className={styles.splitGrid}>
            <div className={styles.listCol} role="list" aria-label="Detected error patterns">
              {patterns.map((pat) => {
                const selected = pat.patternId === selectedPatternId;
                const reqClean = pat.resolution.requiredClean;
                return (
                  <button
                    key={pat.patternId}
                    type="button"
                    className={`${styles.patternRow} ${selected ? styles.patternRowSelected : ''}`}
                    aria-pressed={selected}
                    onClick={() => setSelectedPatternId(pat.patternId)}
                  >
                    <div className={styles.rowTop}>
                      <span className={`${styles.statusBadge} ${statusClass(pat.status)}`}>
                        {pat.status}
                      </span>
                      <span
                        className={styles.streakDots}
                        title={`${pat.cleanStreak} of ${reqClean} clean in a row`}
                      >
                        {Array.from({ length: reqClean }).map((_, idx) => (
                          <span
                            key={idx}
                            className={`${styles.streakDot} ${
                              idx < pat.cleanStreak ? styles.streakDotClean : ''
                            }`}
                          />
                        ))}
                        <span>
                          {pat.cleanStreak}/{reqClean}
                        </span>
                      </span>
                    </div>
                    <h2 className={styles.patternName}>{pat.name}</h2>
                    <span className={styles.rowMeta}>
                      {pat.occurrences}× observed · {pat.opportunities} eligible opportunities
                    </span>
                  </button>
                );
              })}
            </div>

            {activeDetail ? (
              <section className={styles.detailCard} aria-label="Pattern detail">
                <div className={styles.detailHeader}>
                  <div>
                    <span
                      className={`${styles.statusBadge} ${statusClass(
                        activeDetail.snapshot.status,
                      )}`}
                    >
                      {activeDetail.snapshot.status}
                    </span>
                    <h2 className={styles.detailTitle}>{activeDetail.snapshot.name}</h2>
                  </div>
                  <Button onClick={() => void handleDrillPattern(activeDetail.snapshot)}>
                    Drill this pattern
                  </Button>
                </div>

                <div className={styles.statsGrid}>
                  <div className={styles.statItem}>
                    <span className={styles.statLabel}>Occurrences</span>
                    <span className={styles.statValue}>{activeDetail.snapshot.occurrences}</span>
                    <span className={styles.statSub}>
                      across {activeDetail.snapshot.opportunities} eligible
                    </span>
                  </div>
                  <div className={styles.statItem}>
                    <span className={styles.statLabel}>Clean streak</span>
                    <span className={styles.statValue}>
                      {activeDetail.snapshot.cleanStreak} /{' '}
                      {activeDetail.snapshot.resolution.requiredClean}
                    </span>
                    <span className={styles.statSub}>
                      {activeDetail.snapshot.resolution.explanation}
                    </span>
                  </div>
                  <div className={styles.statItem}>
                    <span className={styles.statLabel}>Last seen</span>
                    <span className={styles.statValue}>
                      {formatShortDate(activeDetail.snapshot.lastSeenAt)}
                    </span>
                    <span className={styles.statSub}>
                      First: {formatShortDate(activeDetail.snapshot.firstSeenAt)}
                    </span>
                  </div>
                  <div className={styles.statItem}>
                    <span className={styles.statLabel}>Resolution rule</span>
                    <span className={styles.statValue}>
                      {activeDetail.snapshot.resolution.requiredClean} clean
                    </span>
                    <span className={styles.statSub}>
                      Revision {activeDetail.snapshot.ruleRevision}
                    </span>
                  </div>
                </div>

                <div className={styles.explanationBox}>
                  <div>
                    <h3 className={styles.sectionTitle}>Why it happens</h3>
                    <p className={styles.bodyCopy}>{activeDetail.explanation}</p>
                  </div>
                  <div>
                    <h3 className={styles.sectionTitle}>What to do next</h3>
                    <p className={styles.bodyCopy}>{activeDetail.nextAction}</p>
                  </div>
                </div>

                <div>
                  <h3 className={styles.sectionTitle}>Occurrences by session</h3>
                  {activeDetail.sessionBuckets.length > 0 ? (
                    <div className={styles.bucketList}>
                      {activeDetail.sessionBuckets.map((bucket) => (
                        <div key={bucket.sessionId} className={styles.bucketItem}>
                          <span>{bucket.occurrences}× miss</span>
                          <span>{bucket.cleanCount} clean</span>
                          <span>{formatShortDate(bucket.recordedAt)}</span>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p className={styles.bodyCopy}>
                      No session buckets recorded yet for this pattern.
                    </p>
                  )}
                </div>

                <div>
                  <h3 className={styles.sectionTitle}>Recent evidence</h3>
                  {activeDetail.attributions.length > 0 ? (
                    <ul className={styles.attributionList}>
                      {activeDetail.attributions.slice(0, 5).map((attr) => (
                        <li key={attr.id} className={styles.attributionItem}>
                          <span>
                            <strong>{attr.outcome.toUpperCase()}</strong> — {attr.explanation}
                          </span>
                          {attr.answerText ? (
                            <code className={styles.rowMeta}>{attr.answerText}</code>
                          ) : null}
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <p className={styles.bodyCopy}>
                      No individual attempt attributions recorded yet.
                    </p>
                  )}
                </div>

                <div className={styles.ruleEditor}>
                  <div>
                    <h3 className={styles.sectionTitle}>Resolution rule threshold</h3>
                    <p className={styles.bodyCopy}>
                      Consecutive unassisted clean opportunities required to mark Resolved (2–5).
                      Rule edits preserve history and recalculate status immediately.
                    </p>
                  </div>
                  <div className={styles.ruleButtons} role="group" aria-label="Required clean count">
                    {RULE_THRESHOLDS.map((threshold) => {
                      const active =
                        activeDetail.snapshot.resolution.requiredClean === threshold;
                      return (
                        <Button
                          key={threshold}
                          size="sm"
                          variant={active ? 'dark' : 'secondary'}
                          disabled={savingRule}
                          aria-pressed={active}
                          onClick={() =>
                            void handleRuleChange(activeDetail.snapshot.patternId, threshold)
                          }
                        >
                          {threshold} clean
                        </Button>
                      );
                    })}
                  </div>
                </div>
              </section>
            ) : null}
          </div>
        ) : null}
      </main>
    </AppShell>
  );
}
