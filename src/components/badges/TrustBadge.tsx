import type { TrustLevel } from "../../types";
import styles from "./TrustBadge.module.css";

export interface TrustBadgeProps {
  type: TrustLevel;
  label?: string;
  detail?: string;
  className?: string;
}

/**
 * Indicates module provenance: Axiom Verified, Community, or Experimental.
 */
export function TrustBadge({
  type,
  label,
  detail,
  className = "",
}: TrustBadgeProps) {
  const typeClass = {
    verified: styles.typeVerified,
    community: styles.typeCommunity,
    experimental: styles.typeExperimental,
  }[type];

  const defaultLabel = {
    verified: "Axiom Verified",
    community: "Community",
    experimental: "Experimental",
  }[type];

  return (
    <span className={`${styles.badge} ${typeClass} ${className}`}>
      {type === "verified" && <span className={styles.dot} aria-hidden="true" />}
      <span>{label || defaultLabel}</span>
      {detail && <span className={styles.detail}>· {detail}</span>}
    </span>
  );
}
