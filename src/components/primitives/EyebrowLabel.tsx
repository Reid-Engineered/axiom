import type { ReactNode } from "react";
import styles from "./EyebrowLabel.module.css";

export interface EyebrowLabelProps {
  children: ReactNode;
  className?: string;
}

/**
 * Small uppercase section header label ("CONTINUE", "TOOLS").
 */
export function EyebrowLabel({
  children,
  className = "",
}: EyebrowLabelProps) {
  return (
    <div className={`${styles.label} ${className}`}>
      {children}
    </div>
  );
}
