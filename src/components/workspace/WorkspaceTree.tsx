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

export function WorkspaceTree(_props: WorkspaceTreeProps) {
  return null;
}
