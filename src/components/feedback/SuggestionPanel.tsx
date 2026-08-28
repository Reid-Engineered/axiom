import { Button } from '../primitives/Button';
import styles from './SuggestionPanel.module.css';

/**
 * Grey panel proposing one suggestion justified by observed behaviour, with an accept
 * and a dismiss action (screens 4, 8).
 */
export interface SuggestionPanelProps {
  message: string;
  onAccept: () => void;
  acceptLabel: string;
  onDismiss: () => void;
  dismissLabel?: string;
  className?: string;
}

export function SuggestionPanel({
  message,
  onAccept,
  acceptLabel,
  onDismiss,
  dismissLabel = 'Not now',
  className = '',
}: SuggestionPanelProps) {
  return (
    <section className={`${styles.panel} ${className}`}>
      <p>{message}</p>
      <div>
        <Button variant="tertiary" onClick={onAccept}>{acceptLabel}</Button>
        <Button variant="tertiary" onClick={onDismiss}>{dismissLabel}</Button>
      </div>
    </section>
  );
}
