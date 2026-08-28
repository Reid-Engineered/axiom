import styles from './MathDisplay.module.css';

export interface MathSegment {
  text: string;
  /** This segment is the highlighted/selected term (screen 5: "the selected term x highlighted"). */
  selected?: boolean;
}

/**
 * Centered display math in a well, STIX Two Text (screens 5, 7). `expression` is a plain
 * pre-typeset string for the common case, or `MathSegment[]` when one term needs
 * independent highlighting and interaction — screen 5's integral with `x` highlighted and
 * an "Ask about x" affordance. Segmented, not a parser: `onSelectTerm` fires for the
 * segment the learner activates.
 */
export interface MathDisplayProps {
  expression: string | MathSegment[];
  onSelectTerm?: (segment: MathSegment) => void;
  className?: string;
}

export function MathDisplay({ expression, onSelectTerm, className = '' }: MathDisplayProps) {
  return (
    <div className={`${styles.display} ${className}`}>
      {typeof expression === 'string'
        ? expression
        : expression.map((segment, index) => (
            <button
              key={`${segment.text}-${index}`}
              type="button"
              className={segment.selected ? styles.selected : ''}
              onClick={() => onSelectTerm?.(segment)}
            >
              {segment.text}
            </button>
          ))}
    </div>
  );
}
