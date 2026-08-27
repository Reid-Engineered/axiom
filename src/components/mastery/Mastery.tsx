import type { MasteryState } from "../../types";
import styles from "./Mastery.module.css";

export interface MasteryProps {
  state: MasteryState;
  showLabel?: boolean;
  size?: "sm" | "md";
  className?: string;
}

/**
 * Mastery state indicator featuring the 5-state ring glyph and mandatory reading-distance label.
 */
export function Mastery({
  state,
  showLabel = true,
  size = "md",
  className = "",
}: MasteryProps) {
  const stateClass = {
    New: styles.stateNew,
    Developing: styles.stateDeveloping,
    Familiar: styles.stateFamiliar,
    Strong: styles.stateStrong,
    Mastered: styles.stateMastered,
  }[state];

  const ringSizeClass = size === "sm" ? styles.ringSm : styles.ringMd;
  const containerSizeClass = size === "sm" ? styles.sizeSm : styles.sizeMd;

  return (
    <span
      className={`${styles.container} ${containerSizeClass} ${className}`}
      title={`Mastery: ${state}`}
    >
      <span
        className={`${styles.ring} ${ringSizeClass} ${stateClass}`}
        aria-hidden="true"
      />
      {showLabel && <span className={styles.label}>{state}</span>}
    </span>
  );
}
