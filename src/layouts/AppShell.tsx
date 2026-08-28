import type { ReactNode } from "react";
import styles from "./AppShell.module.css";

export interface AppShellProps {
  children?: ReactNode;
  sidebar?: ReactNode;
}

/** Persistent chrome with a native-OS drag strip, optional sidebar, and content slot. */
export function AppShell({ children, sidebar }: AppShellProps) {
  return (
    <div className={styles.root}>
      <div className={styles.dragStrip} data-tauri-drag-region />
      <div className={styles.body}>
        {sidebar ? <aside className={styles.sidebar}>{sidebar}</aside> : null}
        <div className={styles.content}>{children}</div>
      </div>
    </div>
  );
}
