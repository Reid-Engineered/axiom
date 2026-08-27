import styles from "./ProgressBar.module.css";

export interface ProgressBarProps {
  value: number;
  max?: number;
  className?: string;
}

/**
 * Unlabelled 3px progress indicator. Per AXIOM-HANDOFF §2, never displays percentages or scores.
 */
export function ProgressBar({
  value,
  max = 100,
  className = "",
}: ProgressBarProps) {
  const percentage = Math.min(100, Math.max(0, (value / max) * 100));

  return (
    <div
      className={`${styles.track} ${className}`}
      role="progressbar"
      aria-valuenow={value}
      aria-valuemin={0}
      aria-valuemax={max}
    >
      <div className={styles.fill} style={{ width: `${percentage}%` }} />
    </div>
  );
}
