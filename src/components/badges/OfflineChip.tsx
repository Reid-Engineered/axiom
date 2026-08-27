import type { OfflineStatus } from "../../types";
import styles from "./OfflineChip.module.css";

export type OfflineStatusInput =
  | OfflineStatus
  | "offline"
  | "enhanced"
  | "required";

export interface OfflineChipProps {
  status: OfflineStatusInput;
  className?: string;
}

/**
 * Indicates module or resource offline capability: Works offline, Online enhanced, or Internet required.
 */
export function OfflineChip({
  status,
  className = "",
}: OfflineChipProps) {
  const normalizedStatus: OfflineStatus =
    status === "offline"
      ? "Works offline"
      : status === "enhanced"
      ? "Online enhanced"
      : status === "required"
      ? "Internet required"
      : (status as OfflineStatus);

  const statusClass = {
    "Works offline": styles.statusOffline,
    "Online enhanced": styles.statusEnhanced,
    "Internet required": styles.statusRequired,
  }[normalizedStatus];

  return (
    <span className={`${styles.chip} ${statusClass} ${className}`}>
      {normalizedStatus === "Works offline" && (
        <span className={styles.dotMastery} aria-hidden="true" />
      )}
      {normalizedStatus === "Internet required" && (
        <span className={styles.dotAmber} aria-hidden="true" />
      )}
      <span>{normalizedStatus}</span>
    </span>
  );
}
