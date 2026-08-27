import type { ReactNode } from "react";
import styles from "./AppShell.module.css";

interface AppShellProps {
  children?: ReactNode;
}

/**
 * Persistent chrome: a native-OS drag strip plus a content slot. No sidebar yet —
 * that lands with navigation in Stage 3 (see ROADMAP.md).
 */
export function AppShell({ children }: AppShellProps) {
  return (
    <div className={styles.root}>
      <div className={styles.dragStrip} data-tauri-drag-region />
      <div className={styles.content}>{children}</div>
    </div>
  );
}
