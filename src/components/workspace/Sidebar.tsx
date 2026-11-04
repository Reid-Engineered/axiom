import type { ReactNode } from 'react';

import { Button } from '../primitives/Button';
import {
  WorkspaceTree,
  type WorkspaceTreeEntry,
  type WorkspaceTreeProps,
} from './WorkspaceTree';
import styles from './Sidebar.module.css';

export type ProductArea = 'today' | 'practice' | 'errorProfile' | 'conceptMap' | 'history';

export interface SidebarProps
  extends Pick<
    WorkspaceTreeProps,
    'openWorkspaceId' | 'activeSubItem' | 'onSelectWorkspace' | 'onSelectSubItem'
  > {
  workspaces: WorkspaceTreeEntry[];
  activeArea?: ProductArea;
  onToday?: () => void;
  onPractice?: () => void;
  onErrorProfile?: () => void;
  onConceptMap?: () => void;
  onHistory?: () => void;
  errorPatternCount?: number;
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
 * Permanent navigation for global destinations, domain-scoped Stage 9 Reid product areas
 * (Today / Practice / Error profile / Concept map / History), and the two-level workspace tree.
 * With no workspaces (a fresh install, before First Launch → Create Workspace completes) the
 * workspaces section shows an intentional empty state instead of a blank tree.
 */
export function Sidebar({
  workspaces,
  openWorkspaceId,
  activeSubItem,
  activeArea,
  onSelectWorkspace,
  onSelectSubItem,
  onToday,
  onPractice,
  onErrorProfile,
  onConceptMap,
  onHistory,
  errorPatternCount,
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
  const showProductAreas =
    workspaces.length > 0 &&
    Boolean(onToday || onPractice || onErrorProfile || onConceptMap || onHistory);

  return (
    <nav className={`${styles.root} ${className}`} aria-label="Primary">
      <div className={styles.primaryLinks}>
        <button type="button" className={styles.navButton} onClick={onSearch}>
          <span>Search</span>
          <span className={styles.shortcut}>⌘K</span>
        </button>
        <button
          type="button"
          className={`${styles.navButton} ${activeArea === 'today' ? styles.activeNav : ''}`}
          aria-current={activeArea === 'today' ? 'page' : undefined}
          onClick={onToday ?? onHome}
        >
          Home
        </button>
        <button type="button" className={styles.navButton} onClick={onMarketplace}>
          Marketplace
        </button>
      </div>
      {showProductAreas ? (
        <div className={styles.productAreas} aria-label="Workspace areas">
          <div className={styles.eyebrow}>Study</div>
          {onToday ? (
            <button
              type="button"
              className={`${styles.navButton} ${activeArea === 'today' ? styles.activeNav : ''}`}
              aria-current={activeArea === 'today' ? 'page' : undefined}
              onClick={onToday}
            >
              <span>Today</span>
            </button>
          ) : null}
          {onPractice ? (
            <button
              type="button"
              className={`${styles.navButton} ${activeArea === 'practice' ? styles.activeNav : ''}`}
              aria-current={activeArea === 'practice' ? 'page' : undefined}
              onClick={onPractice}
            >
              <span>Practice</span>
            </button>
          ) : null}
          {onErrorProfile ? (
            <button
              type="button"
              className={`${styles.navButton} ${activeArea === 'errorProfile' ? styles.activeNav : ''}`}
              aria-current={activeArea === 'errorProfile' ? 'page' : undefined}
              onClick={onErrorProfile}
            >
              <span>Error profile</span>
              {typeof errorPatternCount === 'number' && errorPatternCount > 0 ? (
                <span className={styles.areaCount}>{errorPatternCount}</span>
              ) : null}
            </button>
          ) : null}
          {onConceptMap ? (
            <button
              type="button"
              className={`${styles.navButton} ${activeArea === 'conceptMap' ? styles.activeNav : ''}`}
              aria-current={activeArea === 'conceptMap' ? 'page' : undefined}
              onClick={onConceptMap}
            >
              <span>Concept map</span>
            </button>
          ) : null}
          {onHistory ? (
            <button
              type="button"
              className={`${styles.navButton} ${activeArea === 'history' ? styles.activeNav : ''}`}
              aria-current={activeArea === 'history' ? 'page' : undefined}
              onClick={onHistory}
            >
              <span>History</span>
            </button>
          ) : null}
        </div>
      ) : null}
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
