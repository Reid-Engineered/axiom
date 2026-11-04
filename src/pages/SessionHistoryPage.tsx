import { useState, type ReactNode } from 'react';

import { Button } from '../components/primitives/Button';
import { useConcepts } from '../hooks/useConcepts';
import { useSessionHistory } from '../hooks/useLearningEvidence';
import { useNavigation } from '../hooks/useNavigation';
import { useActiveSession } from '../hooks/useSessions';
import { useWorkspaceDetails } from '../hooks/useWorkspaces';
import { AppShell } from '../layouts/AppShell';
import styles from './SessionHistoryPage.module.css';

export interface SessionHistoryPageProps {
  workspaceId: string;
  initialSessionId?: string;
  sidebar?: ReactNode;
}

function formatDurationMs(elapsedMs: number): string {
  const totalSeconds = Math.max(0, Math.round(elapsedMs / 1000));
  const mins = Math.floor(totalSeconds / 60);
  const secs = totalSeconds % 60;
  return `${mins}m ${secs.toString().padStart(2, '0')}s`;
}

function formatDateTime(iso?: string): string {
  if (!iso) return 'In progress';
  const parsed = new Date(iso);
  if (Number.isNaN(parsed.getTime())) return iso;
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  }).format(parsed);
}

/** Domain-scoped Stage 9 Session history and completed session summary screen (`Summary.dc.html`). */
export function SessionHistoryPage({
  workspaceId,
  initialSessionId,
  sidebar,
}: SessionHistoryPageProps) {
  const { workspace } = useWorkspaceDetails(workspaceId);
  const { concepts } = useConcepts(workspaceId);
  const { entries, selectedEntry, selectedSessionId, setSelectedSessionId, loading, error } =
    useSessionHistory(workspaceId, initialSessionId);
  const { startSession } = useActiveSession(workspaceId);
  const { navigate } = useNavigation();
  const [actionError, setActionError] = useState('');

  const conceptNameById = new Map(concepts.map((c) => [c.id, c.name]));

  const handleStartNextSession = async (conceptId?: string) => {
    setActionError('');
    try {
      const targetConceptId = conceptId ?? selectedEntry?.conceptId ?? concepts[0]?.id ?? 'calc-concept-22';
      const session = await startSession({
        workspaceId,
        conceptId: targetConceptId,
        intent: {
          activity: 'Practising',
          detail: 'Adaptive follow-up session',
          targetMinutes: 25,
        },
      });
      navigate({ type: 'studySession', sessionId: session.id });
    } catch (caught) {
      setActionError(
        caught instanceof Error ? caught.message : 'Could not start next practice session.',
      );
    }
  };

  const summaryAvail = selectedEntry?.summary;
  const summary = summaryAvail?.state === 'available' ? summaryAvail.value : null;
  const timing = summary?.timing.state === 'available' ? summary.timing.value : null;

  return (
    <AppShell sidebar={sidebar}>
      <main className={styles.page}>
        <header className={styles.header}>
          <div>
            <div className={styles.eyebrow}>{workspace?.name ?? 'Domain'} · Session history</div>
            <h1 className={styles.title}>Session history</h1>
            <p className={styles.subtitle}>
              Completed and active practice sessions with frozen mastery deltas and question-by-question
              evidence.
            </p>
          </div>
          <div className={styles.actionsRow}>
            <Button onClick={() => void handleStartNextSession()}>
              Start next recommended session
            </Button>
          </div>
        </header>

        {loading ? <p role="status">Loading session history…</p> : null}
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

        {!loading && entries.length === 0 ? (
          <section className={styles.emptyCard} aria-label="No session history">
            <h2 className={styles.emptyTitle}>No sessions recorded in this domain yet</h2>
            <p className={styles.emptyCopy}>
              Once you start or complete a practice session, its measured foreground duration,
              assessed answers, and mastery deltas will appear here.
            </p>
            <Button onClick={() => navigate({ type: 'home' })}>Go to Today plan</Button>
          </section>
        ) : null}

        {!loading && entries.length > 0 ? (
          <div className={styles.splitGrid}>
            <div className={styles.sessionList} role="list" aria-label="Recorded sessions">
              {entries.map((entry) => {
                const selected = entry.sessionId === selectedSessionId;
                const cName =
                  (entry.conceptId && conceptNameById.get(entry.conceptId)) ||
                  entry.intentActivity ||
                  'Practice session';
                return (
                  <button
                    key={entry.sessionId}
                    type="button"
                    className={`${styles.sessionRow} ${
                      selected ? styles.sessionRowSelected : ''
                    }`}
                    aria-pressed={selected}
                    onClick={() => setSelectedSessionId(entry.sessionId)}
                  >
                    <div className={styles.rowTop}>
                      <span
                        className={`${styles.statusBadge} ${
                          entry.status === 'completed' ? styles.statusCompleted : ''
                        }`}
                      >
                        {entry.status}
                      </span>
                      <span className={styles.rowMeta}>
                        {formatDateTime(entry.completedAt ?? entry.startedAt)}
                      </span>
                    </div>
                    <h2 className={styles.sessionTitle}>{cName}</h2>
                    <span className={styles.rowMeta}>
                      {entry.summary.state === 'available'
                        ? `${entry.summary.value.correct} / ${entry.summary.value.assessedTotal} correct`
                        : 'Session in progress'}
                    </span>
                  </button>
                );
              })}
            </div>

            {selectedEntry ? (
              <section className={styles.summaryCard} aria-label="Session summary detail">
                <div className={styles.header}>
                  <div>
                    <span
                      className={`${styles.statusBadge} ${
                        selectedEntry.status === 'completed' ? styles.statusCompleted : ''
                      }`}
                    >
                      {selectedEntry.status === 'completed'
                        ? 'Session complete'
                        : selectedEntry.status}
                    </span>
                    <h2 className={styles.title}>
                      {(selectedEntry.conceptId &&
                        conceptNameById.get(selectedEntry.conceptId)) ||
                        selectedEntry.intentDetail ||
                        'Practice session summary'}
                    </h2>
                  </div>
                  {selectedEntry.status !== 'completed' ? (
                    <Button
                      onClick={() =>
                        navigate({ type: 'studySession', sessionId: selectedEntry.sessionId })
                      }
                    >
                      Resume this session
                    </Button>
                  ) : null}
                </div>

                {summary ? (
                  <>
                    <div className={styles.heroStrip}>
                      <div className={styles.statItem}>
                        <span className={styles.statLabel}>Assessed score</span>
                        <span className={styles.statValue}>
                          {summary.correct} / {summary.assessedTotal}
                        </span>
                        <span className={styles.statSub}>
                          {summary.skippedTotal} skipped · {summary.unverifiableTotal} unverifiable
                        </span>
                      </div>
                      <div className={styles.statItem}>
                        <span className={styles.statLabel}>Measured duration</span>
                        <span className={styles.statValue}>
                          {timing ? formatDurationMs(timing.elapsedMs) : '—'}
                        </span>
                        <span className={styles.statSub}>
                          {timing?.targetMs
                            ? `Target ${Math.round(timing.targetMs / 60000)}m`
                            : 'Active foreground time'}
                        </span>
                      </div>
                      <div className={styles.statItem}>
                        <span className={styles.statLabel}>Hints used</span>
                        <span className={styles.statValue}>
                          {summary.hintsUsed}
                          {typeof summary.hintBudgetTotal === 'number'
                            ? ` / ${summary.hintBudgetTotal}`
                            : ''}
                        </span>
                        <span className={styles.statSub}>Across attempted problems</span>
                      </div>
                      <div className={styles.statItem}>
                        <span className={styles.statLabel}>Completed at</span>
                        <span className={styles.statValue}>
                          {formatDateTime(summary.completedAt)}
                        </span>
                        <span className={styles.statSub}>Frozen summary record</span>
                      </div>
                    </div>

                    <div>
                      <h3 className={styles.sectionTitle}>Mastery changes in this session</h3>
                      {summary.masteryChanges.length > 0 ? (
                        <ul className={styles.deltaList}>
                          {summary.masteryChanges.map((mc, idx) => {
                            if (mc.state !== 'available') return null;
                            const snap = mc.value;
                            const cTitle =
                              conceptNameById.get(snap.conceptId) ?? snap.conceptId;
                            const pctAfter = Math.round(snap.score * 100);
                            const pctBefore =
                              typeof snap.previousScore === 'number'
                                ? Math.round(snap.previousScore * 100)
                                : null;
                            const deltaPct =
                              typeof snap.delta === 'number'
                                ? Math.round(snap.delta * 100)
                                : pctAfter;
                            return (
                              <li key={`${snap.id}-${idx}`} className={styles.deltaRow}>
                                <div>
                                  <strong>{cTitle}</strong> · {snap.namedState}
                                  {pctBefore !== null ? (
                                    <span className={styles.rowMeta}>
                                      {' '}
                                      ({pctBefore}% → {pctAfter}%)
                                    </span>
                                  ) : (
                                    <span className={styles.rowMeta}> (now {pctAfter}%)</span>
                                  )}
                                </div>
                                <span
                                  className={
                                    deltaPct >= 0 ? styles.deltaPositive : styles.deltaNegative
                                  }
                                >
                                  {deltaPct >= 0 ? `+${deltaPct}%` : `${deltaPct}%`}
                                </span>
                              </li>
                            );
                          })}
                        </ul>
                      ) : (
                        <p className={styles.subtitle}>
                          No assessable mastery transitions were recorded in this session.
                        </p>
                      )}
                    </div>

                    <div>
                      <h3 className={styles.sectionTitle}>Question breakdown</h3>
                      {summary.questionRows.length > 0 ? (
                        <ol className={styles.questionList}>
                          {summary.questionRows.map((q) => (
                            <li key={q.slotId} className={styles.questionRow}>
                              <div>
                                <span className={styles.phaseBadge}>
                                  {q.phaseName ?? q.phaseId}
                                </span>{' '}
                                <span>{q.prompt}</span>
                              </div>
                              <span className={styles.statusBadge}>{q.result}</span>
                            </li>
                          ))}
                        </ol>
                      ) : (
                        <p className={styles.subtitle}>No questions recorded in this session.</p>
                      )}
                    </div>

                    {summary.repeatedPatternIds.length > 0 ? (
                      <div className={styles.actionsRow}>
                        <Button
                          variant="secondary"
                          onClick={() =>
                            navigate({
                              type: 'errorProfile',
                              workspaceId,
                              patternId: summary.repeatedPatternIds[0],
                            })
                          }
                        >
                          Inspect repeated pattern in Error profile
                        </Button>
                      </div>
                    ) : null}
                  </>
                ) : (
                  <p className={styles.subtitle}>
                    This session has not been completed yet. Resume it to finish your practice
                    queue and freeze the session summary.
                  </p>
                )}
              </section>
            ) : null}
          </div>
        ) : null}
      </main>
    </AppShell>
  );
}
