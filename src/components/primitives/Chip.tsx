import type { ReactNode } from "react";
import styles from "./Chip.module.css";

export interface ChipProps {
  label: string;
  variant?: "default" | "accent" | "subtle";
  removable?: boolean;
  onRemove?: () => void;
  icon?: ReactNode;
  className?: string;
}

/**
 * Compact pill for inferred goal facets and concept tags with an optional remove button.
 */
export function Chip({
  label,
  variant = "default",
  removable = false,
  onRemove,
  icon,
  className = "",
}: ChipProps) {
  const variantClass = {
    default: styles.variantDefault,
    accent: styles.variantAccent,
    subtle: styles.variantSubtle,
  }[variant];

  return (
    <span className={`${styles.chip} ${variantClass} ${className}`}>
      {icon && <span className={styles.icon}>{icon}</span>}
      <span>{label}</span>
      {removable && (
        <button
          type="button"
          className={styles.removeButton}
          onClick={(e) => {
            e.stopPropagation();
            onRemove?.();
          }}
          aria-label={`Remove ${label}`}
        >
          ×
        </button>
      )}
    </span>
  );
}
