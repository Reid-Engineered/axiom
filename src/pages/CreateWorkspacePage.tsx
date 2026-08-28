import { useState, type FormEvent } from 'react';

import { Button } from '../components/primitives/Button';
import { Chip } from '../components/primitives/Chip';
import { useNavigation } from '../hooks/useNavigation';
import { useWorkspace } from '../hooks/useWorkspace';
import { useWorkspaces } from '../hooks/useWorkspaces';
import { CenteredColumnLayout } from '../layouts/CenteredColumnLayout';
import styles from './CreateWorkspacePage.module.css';

export type CreateWorkspacePageProps = Record<string, never>;

/** Natural-language workspace creation and inferred-goal confirmation. */
const initialFacets = [
  'Deadline · Dec 12',
  'Mastery · conceptual, not just procedural',
  '14 concepts',
  'Pacing · 4 sessions / week',
  'Tools · Tutor, Practice, Visualizer, Notes',
];

export function CreateWorkspacePage(_props: CreateWorkspacePageProps) {
  const { navigate } = useNavigation();
  const { setActiveWorkspaceId } = useWorkspace();
  const { createWorkspace } = useWorkspaces();
  const [subject, setSubject] = useState('Calculus II');
  const [goalText, setGoalText] = useState(
    'I want to deeply understand Calc II and be ready for my final in December.',
  );
  const [facets, setFacets] = useState(initialFacets);
  const [adjusting, setAdjusting] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!subject.trim() || !goalText.trim()) return;
    setSubmitting(true);
    setError(null);
    try {
      const workspace = await createWorkspace({ subject, goalText });
      setActiveWorkspaceId(workspace.id);
      navigate({ type: 'home' });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Workspace creation could not be completed.');
      setSubmitting(false);
    }
  };

  return (
    <CenteredColumnLayout width="wide" className={styles.page}>
      <button className={styles.back} type="button" onClick={() => navigate({ type: 'firstLaunch' })}>
        ‹ Back
      </button>
      <form className={styles.form} onSubmit={submit}>
        <header className={styles.header}>
          <span>New workspace</span>
          <h1>Set it up in two answers</h1>
        </header>

        <label className={styles.field}>
          <span>Subject</span>
          <input value={subject} onChange={(event) => setSubject(event.target.value)} />
        </label>

        <label className={styles.field}>
          <span>What are you trying to accomplish?</span>
          <small>Plain language is fine.</small>
          <textarea value={goalText} onChange={(event) => setGoalText(event.target.value)} rows={3} />
        </label>

        <section className={styles.inference} aria-labelledby="inference-title">
          <div className={styles.inferenceHeader}>
            <h2 id="inference-title"><span aria-hidden="true" />Axiom read that as</h2>
            <button type="button" onClick={() => setAdjusting((value) => !value)}>
              {adjusting ? 'Done' : 'Adjust'}
            </button>
          </div>
          <div className={styles.chips}>
            {facets.map((facet) => (
              <Chip
                key={facet}
                label={facet}
                variant="subtle"
                removable
                onRemove={() => setFacets((current) => current.filter((item) => item !== facet))}
              />
            ))}
          </div>
          {adjusting ? (
            <div className={styles.adjustments}>
              <label>Comfort level<select defaultValue="building"><option value="building">Building confidence</option><option value="comfortable">Comfortable</option></select></label>
              <label>Materials<input type="file" multiple /></label>
              <label>Pacing<select defaultValue="steady"><option value="steady">Four sessions per week</option><option value="flexible">Flexible</option></select></label>
            </div>
          ) : null}
        </section>

        {error ? <p className={styles.error} role="alert">{error}</p> : null}
        <footer className={styles.footer}>
          <div>
            <Button size="lg" type="submit" disabled={submitting || !subject.trim() || !goalText.trim()}>
              {submitting ? 'Creating…' : 'Create Workspace'}
            </Button>
            <Button size="lg" variant="secondary" onClick={() => navigate({ type: 'firstLaunch' })}>Cancel</Button>
          </div>
          <span>Nothing here is permanent</span>
        </footer>
      </form>
    </CenteredColumnLayout>
  );
}
