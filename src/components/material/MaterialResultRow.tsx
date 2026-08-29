import type { MasteryState, MaterialResult } from '../../types';
import { Mastery } from '../mastery/Mastery';
import styles from './MaterialResultRow.module.css';

export interface MaterialResultRowProps {
  result: MaterialResult;
  conceptName: string;
  masteryState: MasteryState;
  onOpen?: () => void;
  onConceptSelect?: () => void;
}

const KIND_LABELS: Record<MaterialResult['kind'], string> = {
  section: 'Section',
  workedExample: 'Worked example',
  exerciseRange: 'Exercise range',
};

function formatHighlightDate(isoString: string) {
  try {
    const date = new Date(isoString);
    const month = date.toLocaleString('en-US', { month: 'short' });
    const day = date.getUTCDate();
    return `you highlighted this on ${month} ${day}`;
  } catch {
    return 'you highlighted this';
  }
}

/** Typed material search result linked to a separately resolved concept. */
export function MaterialResultRow({
  result,
  conceptName,
  masteryState,
  onOpen,
  onConceptSelect,
}: MaterialResultRowProps) {
  const exerciseSummary =
    result.kind === 'exerciseRange' && result.exerciseTotal !== undefined
      ? `${result.exerciseTotal} exercises · ${result.exerciseAttempted ?? 0} attempted`
      : null;

  return (
    <article className={styles.row}>
      <div className={styles.location}>
        <span className={styles.kindLabel}>{KIND_LABELS[result.kind]}</span>
        <span className={styles.pageNumber}>p. {result.page}</span>
      </div>
      <div className={styles.content}>
        <h3 className={styles.title}>{result.title}</h3>
        <p className={styles.reason}>
          {result.kind === 'section' ? `“${result.reason}”` : result.reason}
        </p>
        {exerciseSummary ? <p className={styles.exerciseSummary}>{exerciseSummary}</p> : null}
        <div className={styles.metaRow}>
          {onConceptSelect ? (
            <button type="button" className={styles.concept} onClick={onConceptSelect}>
              <span className={styles.conceptName}>{conceptName}</span>
              <Mastery state={masteryState} size="sm" />
            </button>
          ) : (
            <div className={styles.concept}>
              <span className={styles.conceptName}>{conceptName}</span>
              <Mastery state={masteryState} size="sm" />
            </div>
          )}
          {result.highlightedAt ? (
            <span className={styles.highlightNote}>
              {formatHighlightDate(result.highlightedAt)}
            </span>
          ) : null}
        </div>
      </div>
      <div className={styles.actions}>
        {result.kind === 'section' ? (
          <>
            <button
              type="button"
              className={styles.readButton}
              onClick={onOpen ?? (() => undefined)}
            >
              Read
            </button>
            <button
              type="button"
              className={styles.actionLink}
              onClick={onOpen ?? (() => undefined)}
            >
              Visualize
            </button>
          </>
        ) : (
          <button type="button" className={styles.actionLink} onClick={onOpen ?? (() => undefined)}>
            {result.kind === 'exerciseRange' ? 'Practise these' : 'Open'}
          </button>
        )}
      </div>
    </article>
  );
}
