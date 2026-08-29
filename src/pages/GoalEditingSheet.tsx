import { useEffect, useState, type FormEvent } from 'react';

import { Sheet } from '../components/overlays/Sheet';
import { Button } from '../components/primitives/Button';
import { Chip } from '../components/primitives/Chip';
import { useGoals } from '../hooks/useGoals';
import { useWorkspaces } from '../hooks/useWorkspaces';
import styles from './GoalEditingSheet.module.css';

export interface GoalEditingSheetProps {
  open: boolean;
  workspaceId: string;
  goalId: string;
  onClose: () => void;
}

/** Dismissible goal editor that previews consequences without deleting prior work. */
export function GoalEditingSheet({ open, workspaceId, goalId, onClose }: GoalEditingSheetProps) {
  const { goals, updateGoal, revertGoal } = useGoals(workspaceId);
  const { workspaces } = useWorkspaces();
  const goal = goals.find((candidate) => candidate.id === goalId);
  const workspace = workspaces.find((candidate) => candidate.id === workspaceId);
  const [text, setText] = useState('');
  const [hiddenFacets, setHiddenFacets] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    if (goal) setText(goal.text);
  }, [goal]);
  useEffect(() => {
    if (!open) {
      setHiddenFacets([]);
      setError('');
    }
  }, [open]);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const trimmed = text.trim();
    if (!trimmed) {
      setError('Describe what you are working toward.');
      return;
    }
    setSaving(true);
    setError('');
    try {
      await updateGoal(goalId, trimmed);
      onClose();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : 'Could not update the goal.');
    } finally {
      setSaving(false);
    }
  };
  const revert = async () => {
    try {
      const updated = await revertGoal(goalId);
      setText(updated.text);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : 'Could not revert the goal.');
    }
  };
  const facets = goal
    ? [
        { key: 'deadline', label: `Deadline · ${goal.inferred.deadline ?? 'none'}` },
        { key: 'mastery', label: `Mastery · ${goal.inferred.masteryType ?? 'conceptual'}` },
        { key: 'pacing', label: `Pacing · ${goal.inferred.pacing ?? 'steady'}` },
        { key: 'scope', label: `Scope · ${goal.inferred.conceptScope ?? 'all'} concepts` },
        {
          key: 'tools',
          label: `Tools · ${goal.inferred.tools && goal.inferred.tools.length > 0 ? goal.inferred.tools.join(', ') : 'none'}`,
        },
      ].filter((facet) => !hiddenFacets.includes(facet.key))
    : [];

  return (
    <Sheet
      open={open}
      onClose={onClose}
      eyebrow={`${workspace?.name ?? 'Workspace'} · Primary goal`}
      title="What are you working toward?"
      className={styles.goalSheet}
      footer={
        <div className={styles.footerContent}>
          <div className={styles.footerActions}>
            <Button type="submit" form="goal-editing-form" disabled={saving || !goal}>
              {saving ? 'Updating…' : 'Update goal'}
            </Button>
            <Button variant="secondary" onClick={onClose}>
              Cancel
            </Button>
          </div>
          <span className={styles.history}>Goal history</span>
        </div>
      }
    >
      <form id="goal-editing-form" className={styles.form} onSubmit={submit}>
        <div className={styles.inputGroup}>
          <label>
            <span className={styles.srOnly}>Goal</span>
            <textarea
              className={styles.textarea}
              value={text}
              onChange={(event) => setText(event.target.value)}
              rows={2}
            />
          </label>
          {goal?.previousText ? (
            <div className={styles.previousRow}>
              <span className={styles.previousText}>Was: “{goal.previousText}”</span>
              <button type="button" className={styles.revertButton} onClick={revert}>
                Revert
              </button>
            </div>
          ) : null}
        </div>
        <div className={styles.facets}>
          {facets.map((facet) => (
            <Chip
              key={facet.key}
              label={facet.label}
              removable
              onRemove={() => setHiddenFacets((current) => [...current, facet.key])}
            />
          ))}
          <Chip label="+ Add" variant="subtle" className={styles.addChip} />
        </div>
        <section className={styles.consequences} aria-labelledby="changes-title">
          <h3 id="changes-title" className={styles.consequencesTitle}>
            What changes
          </h3>
          <ul className={styles.consequencesList}>
            <li className={styles.consequenceItemAccent}>
              Recommendations will prioritize concepts that support this wording.
            </li>
            <li className={styles.consequenceItemAccent}>
              Practice pacing will follow the inferred deadline and cadence.
            </li>
            <li className={styles.consequenceItemMuted}>
              Relevant tools will appear when they can help with this goal.
            </li>
            <li className={styles.consequenceItemMuted}>
              <strong>Nothing is deleted.</strong> Notes, mastery, history, and the previous goal
              are kept.
            </li>
          </ul>
        </section>
        {error ? (
          <p role="alert" className={styles.error}>
            {error}
          </p>
        ) : null}
      </form>
    </Sheet>
  );
}
