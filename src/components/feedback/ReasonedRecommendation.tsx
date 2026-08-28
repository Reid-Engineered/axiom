export interface ReasonedRecommendationObservation {
  date: string;
  text: string;
}

/**
 * Accent-ruled block: action, one line of evidence, a primary CTA, and an optional
 * "Why this?" expanding to dated observations. Never mentions AI, never explains the
 * model (screen 4, `AXIOM-HANDOFF.md` §2).
 */
export interface ReasonedRecommendationProps {
  action: string;
  evidence: string;
  ctaLabel: string;
  onStart: () => void;
  observations?: ReasonedRecommendationObservation[];
  onAlternative?: () => void;
  className?: string;
}

export function ReasonedRecommendation({
  action,
  evidence,
  ctaLabel,
  onStart,
  observations = [],
  onAlternative,
  className = '',
}: ReasonedRecommendationProps) {
  const [showWhy, setShowWhy] = useState(false);

  return (
    <section className={`${styles.recommendation} ${className}`}>
      <span className={styles.eyebrow}>Recommended next</span>
      <h2>{action}</h2>
      <p>{evidence}</p>
      <div className={styles.actions}>
        <Button onClick={onStart}>{ctaLabel}</Button>
        {onAlternative ? <Button variant="tertiary" onClick={onAlternative}>Something else</Button> : null}
        {observations.length ? <button type="button" className={styles.why} onClick={() => setShowWhy((value) => !value)}>Why this?</button> : null}
      </div>
      {showWhy ? <ul>{observations.map((observation) => <li key={`${observation.date}-${observation.text}`}><time>{observation.date}</time>{observation.text}</li>)}</ul> : null}
    </section>
  );
}
import { useState } from 'react';

import { Button } from '../primitives/Button';
import styles from './ReasonedRecommendation.module.css';
