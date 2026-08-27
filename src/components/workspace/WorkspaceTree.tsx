import styles from './WorkspaceTree.module.css';

export interface WorkspaceTreeEntry {
  id: string;
  name: string;
}

/**
 * Sidebar's workspace list. Only the open workspace expands to its four fixed sub-items
 * (Overview / Concepts / Material / Tools); the tree never exceeds two levels
 * (AXIOM-HANDOFF.md §3). Real expand/collapse and routing wiring lands in Stage 3 (017) —
 * this is the prop contract only.
 */
export interface WorkspaceTreeProps {
  workspaces: WorkspaceTreeEntry[];
  openWorkspaceId?: string;
  activeSubItem?: 'overview' | 'concepts' | 'material' | 'tools';
  onSelectWorkspace?: (id: string) => void;
  onSelectSubItem?: (subItem: 'overview' | 'concepts' | 'material' | 'tools') => void;
  className?: string;
}

const SUB_ITEMS = ['overview', 'concepts', 'material', 'tools'] as const;

export function WorkspaceTree({
  workspaces,
  openWorkspaceId,
  activeSubItem,
  onSelectWorkspace,
  onSelectSubItem,
  className = '',
}: WorkspaceTreeProps) {
  return (
    <div className={`${styles.tree} ${className}`}>
      {workspaces.map((workspace) => {
        const isOpen = workspace.id === openWorkspaceId;

        return (
          <div className={styles.workspace} key={workspace.id}>
            <button
              type="button"
              className={`${styles.workspaceButton} ${isOpen ? styles.open : ''}`}
              aria-expanded={isOpen}
              onClick={() => onSelectWorkspace?.(workspace.id)}
            >
              {workspace.name}
            </button>
            {isOpen ? (
              <div className={styles.subItems}>
                {SUB_ITEMS.map((subItem) => (
                  <button
                    type="button"
                    key={subItem}
                    className={`${styles.subItem} ${activeSubItem === subItem ? styles.active : ''}`}
                    aria-current={activeSubItem === subItem ? 'page' : undefined}
                    onClick={() => onSelectSubItem?.(subItem)}
                  >
                    {subItem[0].toUpperCase() + subItem.slice(1)}
                  </button>
                ))}
              </div>
            ) : null}
          </div>
        );
      })}
    </div>
  );
}
