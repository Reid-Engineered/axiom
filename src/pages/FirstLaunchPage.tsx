import { useState, type FormEvent } from 'react';

import { Button } from '../components/primitives/Button';
import { useExploreSampleWorkspace } from '../hooks/useExploreSampleWorkspace';
import { useNavigation } from '../hooks/useNavigation';
import { CenteredColumnLayout } from '../layouts/CenteredColumnLayout';
import styles from './FirstLaunchPage.module.css';

export type FirstLaunchPageProps = Record<string, never>;

/** First-run entry for choosing what the learner wants to study. */
export function FirstLaunchPage(_props: FirstLaunchPageProps) {
  const { navigate } = useNavigation();
  const { explore: exploreSampleWorkspace, importing: importingSample, error: sampleImportError } =
    useExploreSampleWorkspace();
  const [subject, setSubject] = useState('');

  const continueToSetup = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const submittedSubject = subject.trim() || 'Calculus II';
    navigate({ type: 'createWorkspace', subject: submittedSubject });
  };

  return (
    <CenteredColumnLayout className={styles.page}>
      <div className={styles.logoLockup} aria-label="Axiom">
        <span className={styles.logoMark} aria-hidden="true">
          <span />
        </span>
        <span className={styles.logoType}>Axiom</span>
      </div>

      <header className={styles.header}>
        <h1>What are you learning?</h1>
        <p>Name a subject and Axiom builds a workspace around it. Everything is editable later.</p>
      </header>

      <form className={styles.subjectForm} onSubmit={continueToSetup}>
        <label className={styles.visuallyHidden} htmlFor="first-launch-subject">
          Subject
        </label>
        <input
          id="first-launch-subject"
          value={subject}
          placeholder="Calculus II"
          onChange={(event) => setSubject(event.target.value)}
          autoComplete="off"
        />
        <Button size="lg" type="submit">
          Continue
        </Button>
      </form>

      <nav className={styles.alternatives} aria-label="Other ways to begin">
        <button type="button" onClick={() => navigate({ type: 'marketplace' })}>
          <span>Install a workspace template</span>
          <span aria-hidden="true">›</span>
        </button>
        <button type="button" onClick={() => navigate({ type: 'createWorkspace' })}>
          <span>Import a syllabus, PDF, or notes</span>
          <span aria-hidden="true">›</span>
        </button>
        <button type="button" onClick={exploreSampleWorkspace} disabled={importingSample}>
          <span>Explore a sample workspace</span>
          <span aria-hidden="true">›</span>
        </button>
      </nav>
      {sampleImportError ? <p role="alert">{sampleImportError}</p> : null}
    </CenteredColumnLayout>
  );
}
