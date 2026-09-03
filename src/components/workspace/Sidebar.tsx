import type { ReactNode } from 'react';

import { Button } from '../primitives/Button';
import {
  WorkspaceTree,
  type WorkspaceTreeEntry,
  type WorkspaceTreeProps,
} from './WorkspaceTree';
import styles from './Sidebar.module.css';

export interface SidebarProps
  extends Pick<
    WorkspaceTreeProps,
    'openWorkspaceId' | 'activeSubItem' | 'onSelectWorkspace' | 'onSelectSubItem'
  > {
  workspaces: WorkspaceTreeEntry[];
  onSearch?: () => void;
  onHome?: () => void;
  onMarketplace?: () => void;
  onCreateWorkspace?: () => void;
  /** Opt-in only — never invoked automatically. */
  onExploreSample?: () => void;
  exploringSample?: boolean;
  exploreSampleError?: string;
  footer?: ReactNode;
  className?: string;
}

/**
 * Permanent navigation for global destinations and the two-level workspace tree. With no
 * workspaces (a fresh install, before First Launch → Create Workspace completes) the
 * workspaces section shows an intentional empty state instead of a blank tree.
 */
export function Sidebar({
  workspaces,
  openWorkspaceId,
  activeSubItem,
  onSelectWorkspace,
  onSelectSubItem,
  onSearch,
  onHome,
  onMarketplace,
  onCreateWorkspace,
  onExploreSample,
  exploringSample = false,
  exploreSampleError,
  footer,
  className = '',
}: SidebarProps) {
  return (
    <nav className={`${styles.root} ${className}`} aria-label="Primary">
      <div className={styles.primaryLinks}>
        <button type="button" className={styles.navButton} onClick={onSearch}>
          <span>Search</span>
          <span className={styles.shortcut}>⌘K</span>
        </button>
        <button type="button" className={styles.navButton} onClick={onHome}>
          Home
        </button>
        <button type="button" className={styles.navButton} onClick={onMarketplace}>
          Marketplace
        </button>
      </div>
      <div className={styles.workspaces}>
        <div className={styles.eyebrow}>Workspaces</div>
        {workspaces.length === 0 ? (
          <div className={styles.emptyState}>
            <p className={styles.emptyStateMessage}>No workspaces yet.</p>
            <Button size="sm" onClick={onCreateWorkspace}>
              Create workspace
            </Button>
            <Button
              size="sm"
              variant="tertiary"
              onClick={onExploreSample}
              disabled={exploringSample}
            >
              Explore a sample workspace
            </Button>
            {exploreSampleError ? (
              <p className={styles.emptyStateError} role="alert">
                {exploreSampleError}
              </p>
            ) : null}
          </div>
        ) : (
          <>
            <WorkspaceTree
              workspaces={workspaces}
              openWorkspaceId={openWorkspaceId}
              activeSubItem={activeSubItem}
              onSelectWorkspace={onSelectWorkspace}
              onSelectSubItem={onSelectSubItem}
            />
            <button type="button" className={styles.createButton} onClick={onCreateWorkspace}>
              + New Workspace
            </button>
          </>
        )}
      </div>
      {footer ? <div className={styles.footer}>{footer}</div> : null}
    </nav>
  );
}
