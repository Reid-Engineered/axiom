import type { MasteryState } from "../../types";
import { Mastery } from "./Mastery";
import styles from "./ChapterStateProfile.module.css";

export interface ChapterStateProfileProps {
  counts: Partial<Record<MasteryState, number>>;
  showTotalCount?: boolean;
  className?: string;
}

const MASTERY_ORDER: MasteryState[] = [
  "Mastered",
  "Strong",
  "Familiar",
  "Developing",
  "New",
];

/**
 * Summary row of up to 5 mastery rings representing a chapter or group's mastery distribution.
 */
export function ChapterStateProfile({
  counts,
  showTotalCount = true,
  className = "",
}: ChapterStateProfileProps) {
  const activeStates = MASTERY_ORDER.filter((st) => (counts[st] ?? 0) > 0);
  const totalConcepts = Object.values(counts).reduce((acc, n) => acc + (n || 0), 0);

  return (
    <div className={`${styles.container} ${className}`}>
      <div className={styles.ringsRow}>
        {activeStates.length > 0 ? (
          activeStates.map((st) => (
            <Mastery key={st} state={st} showLabel={false} size="sm" />
          ))
        ) : (
          <Mastery state="New" showLabel={false} size="sm" />
        )}
      </div>
      {showTotalCount && (
        <span className={styles.countText}>
          {totalConcepts} {totalConcepts === 1 ? "concept" : "concepts"}
        </span>
      )}
    </div>
  );
}
