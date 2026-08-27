import { Sheet } from '../components/overlays/Sheet';

export interface GoalEditingSheetProps {
  open: boolean;
  workspaceId: string;
  goalId: string;
  onClose: () => void;
}

/** Dismissible goal editor that previews consequences without deleting prior work. */
export function GoalEditingSheet({ open, onClose }: GoalEditingSheetProps) {
  return <Sheet open={open} onClose={onClose}>{null}</Sheet>;
}
