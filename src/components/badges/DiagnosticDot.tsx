import styles from "./DiagnosticDot.module.css";

export interface DiagnosticDotProps {
  type: "mistake" | "positive" | "neutral";
  size?: "sm" | "md" | "lg";
  tooltip?: string;
  className?: string;
}

/**
 * Diagnostic indicator dot: amber for specific mistake, mastery accent for positive practice, neutral for metadata.
 */
export function DiagnosticDot({
  type,
  size = "md",
  tooltip,
  className = "",
}: DiagnosticDotProps) {
  const typeClass = {
    mistake: styles.typeMistake,
    positive: styles.typePositive,
    neutral: styles.typeNeutral,
  }[type];

  const sizeClass = {
    sm: styles.sizeSm,
    md: styles.sizeMd,
    lg: styles.sizeLg,
  }[size];

  return (
    <span
      className={`${styles.dot} ${typeClass} ${sizeClass} ${className}`}
      title={tooltip}
      aria-label={tooltip || `${type} indicator`}
    />
  );
}
